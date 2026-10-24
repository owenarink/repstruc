//! Experimental, fail-closed handoff to an existing ChatGPT desktop composer.
//!
//! No activation, window raising, focus setters, pointer events, or system-wide
//! keyboard events are used. Apple deprecated the targeted keyboard API, and
//! desktop app accessibility trees vary: unsupported UIs return an error.

use std::sync::atomic::AtomicBool;

#[derive(Clone, Debug)]
pub struct ChatTarget {
    pub pid: i32,
    pub title: String,
    pub window_index: usize,
    /// Used only when the app exposes a nonempty identifier unique among its windows.
    pub window_identifier: Option<String>,
}

pub fn accessibility_granted() -> bool {
    platform::accessibility_granted()
}

/// Only call in response to an explicit permission button in the app.
pub fn request_accessibility() -> bool {
    platform::request_accessibility()
}

pub fn foreground_pid() -> Result<i32, String> {
    platform::foreground_pid()
}

pub fn clipboard_generation() -> Result<i64, String> {
    platform::clipboard_generation()
}

pub fn targets() -> Result<Vec<ChatTarget>, String> {
    platform::targets()
}

/// The caller must put the intended image on the clipboard immediately before
/// calling. Existing ChatGPT drafts/attachments are never overwritten on purpose.
pub fn send(
    target: &ChatTarget,
    prompt: &str,
    cancel: &AtomicBool,
    expected_foreground: i32,
    expected_clipboard: i64,
) -> Result<String, String> {
    platform::send(
        target,
        prompt,
        cancel,
        expected_foreground,
        expected_clipboard,
    )
}

fn label(value: &str) -> String {
    value
        .trim()
        .trim_end_matches(['.', '…'])
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn composer_label(role: &str, identifier: &str, placeholder: &str, description: &str) -> bool {
    if !matches!(role, "AXTextArea" | "AXTextField") {
        return false;
    }
    let identifier = label(identifier);
    if matches!(identifier.as_str(), "prompt-textarea" | "chat-input") {
        return true;
    }
    [placeholder, description].iter().any(|value| {
        matches!(
            label(value).as_str(),
            "ask anything"
                | "message"
                | "message chatgpt"
                | "ask for follow-up"
                | "ask for followup"
                | "ask a follow-up"
                | "ask follow-up"
        )
    })
}

fn send_label(value: &str) -> bool {
    matches!(label(value).as_str(), "send" | "send message")
}

fn busy_label(value: &str) -> bool {
    matches!(
        label(value).as_str(),
        "stop" | "stop generating" | "stop response" | "stop responding" | "cancel response"
    )
}

fn attachment_label(role: &str, value: &str) -> bool {
    let value = label(value);
    (role == "AXImage" && matches!(value.as_str(), "image" | "screenshot"))
        || value.contains("image attachment")
        || value.contains("attached image")
        || value == "attachment"
        || value == "attachments"
        || (role == "AXButton"
            && ["remove attachment", "remove image", "remove file"]
                .iter()
                .any(|prefix| value == *prefix || value.starts_with(&format!("{prefix} "))))
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{ChatTarget, attachment_label, busy_label, composer_label, send_label};
    use std::ffi::{CStr, CString, c_char, c_void};
    use std::ptr;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    type CfRef = *const c_void;
    type Id = *mut c_void;
    type Sel = *mut c_void;
    const UTF8: u32 = 0x0800_0100;
    const MAX_NODES: usize = 5_000;
    const MAX_DEPTH: usize = 30;
    const TIMEOUT: Duration = Duration::from_secs(15);
    const MESSAGE_TIMEOUT: f32 = 0.15;
    const ALLOWED_BUNDLES: [&str; 2] = ["com.openai.codex", "com.openai.chat"];

    // These signatures follow the installed Apple SDK headers. AX Boolean and
    // Core Foundation Boolean are UInt8, while ObjC processIdentifier is pid_t.
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFRetain(value: CfRef) -> CfRef;
        fn CFRelease(value: CfRef);
        fn CFEqual(left: CfRef, right: CfRef) -> u8;
        fn CFGetTypeID(value: CfRef) -> usize;
        fn CFStringGetTypeID() -> usize;
        fn CFArrayGetTypeID() -> usize;
        fn CFBooleanGetTypeID() -> usize;
        fn CFBooleanGetValue(value: CfRef) -> u8;
        fn CFStringCreateWithCString(
            allocator: CfRef,
            value: *const c_char,
            encoding: u32,
        ) -> CfRef;
        fn CFStringGetLength(value: CfRef) -> isize;
        fn CFStringGetMaximumSizeForEncoding(length: isize, encoding: u32) -> isize;
        fn CFStringGetCString(value: CfRef, buffer: *mut c_char, size: isize, encoding: u32) -> u8;
        fn CFArrayGetCount(value: CfRef) -> isize;
        fn CFArrayGetValueAtIndex(value: CfRef, index: isize) -> CfRef;
        fn CFDictionaryCreate(
            allocator: CfRef,
            keys: *const CfRef,
            values: *const CfRef,
            count: isize,
            key_callbacks: CfRef,
            value_callbacks: CfRef,
        ) -> CfRef;
        static kCFBooleanTrue: CfRef;
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn AXIsProcessTrusted() -> u8;
        fn AXIsProcessTrustedWithOptions(options: CfRef) -> u8;
        static kAXTrustedCheckOptionPrompt: CfRef;
        fn AXUIElementCreateApplication(pid: i32) -> CfRef;
        fn AXUIElementGetTypeID() -> usize;
        fn AXUIElementSetMessagingTimeout(element: CfRef, seconds: f32) -> i32;
        fn AXUIElementCopyAttributeValue(element: CfRef, name: CfRef, value: *mut CfRef) -> i32;
        fn AXUIElementCopyActionNames(element: CfRef, names: *mut CfRef) -> i32;
        fn AXUIElementIsAttributeSettable(element: CfRef, name: CfRef, settable: *mut u8) -> i32;
        fn AXUIElementSetAttributeValue(element: CfRef, name: CfRef, value: CfRef) -> i32;
        fn AXUIElementPostKeyboardEvent(
            application: CfRef,
            character: u16,
            virtual_key: u16,
            down: u8,
        ) -> i32;
        fn AXUIElementPerformAction(element: CfRef, action: CfRef) -> i32;
    }

    #[link(name = "AppKit", kind = "framework")]
    unsafe extern "C" {}

    #[link(name = "objc")]
    unsafe extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        fn objc_msgSend();
    }

    /// Owns one +1 Core Foundation reference, including AXUIElement references.
    struct Cf(CfRef);

    impl Cf {
        fn owned(raw: CfRef) -> Result<Self, String> {
            if raw.is_null() {
                Err("macOS returned an empty accessibility object.".to_owned())
            } else {
                Ok(Self(raw))
            }
        }

        fn retained(raw: CfRef) -> Result<Self, String> {
            if raw.is_null() {
                return Err("macOS returned an empty accessibility object.".to_owned());
            }
            // SAFETY: Caller obtained a live borrowed value from a retained CF container.
            Self::owned(unsafe { CFRetain(raw) })
        }

        fn same(&self, other: &Self) -> bool {
            // SAFETY: Both values are live, owned CF references.
            unsafe { CFEqual(self.0, other.0) != 0 }
        }

        fn text(&self) -> Result<String, String> {
            // SAFETY: The type is checked before calling CFString operations.
            unsafe {
                if CFGetTypeID(self.0) != CFStringGetTypeID() {
                    return Err(
                        "The app exposed an unsupported accessibility text value.".to_owned()
                    );
                }
                let length = CFStringGetLength(self.0);
                let capacity = CFStringGetMaximumSizeForEncoding(length, UTF8)
                    .checked_add(1)
                    .filter(|size| *size > 0 && *size <= 4_000_000)
                    .ok_or_else(|| "The accessibility text value is too large.".to_owned())?;
                let mut bytes = vec![0_u8; capacity as usize];
                if CFStringGetCString(self.0, bytes.as_mut_ptr().cast(), capacity, UTF8) == 0 {
                    return Err("Could not decode accessibility text.".to_owned());
                }
                let length = bytes
                    .iter()
                    .position(|byte| *byte == 0)
                    .unwrap_or(bytes.len());
                String::from_utf8(bytes[..length].to_vec())
                    .map_err(|_| "Could not decode accessibility text.".to_owned())
            }
        }

        fn array(&self) -> Result<Vec<Self>, String> {
            // SAFETY: Type and bounds are checked before reading the CF array.
            unsafe {
                if CFGetTypeID(self.0) != CFArrayGetTypeID() {
                    return Err(
                        "The app exposed an unsupported accessibility collection.".to_owned()
                    );
                }
                let count = CFArrayGetCount(self.0);
                if !(0..=MAX_NODES as isize).contains(&count) {
                    return Err("The accessibility tree exceeds the inspection limit.".to_owned());
                }
                (0..count)
                    .map(|index| Self::retained(CFArrayGetValueAtIndex(self.0, index)))
                    .collect()
            }
        }
    }

    impl Clone for Cf {
        fn clone(&self) -> Self {
            // SAFETY: self owns a live reference that is retained for the clone.
            Self(unsafe { CFRetain(self.0) })
        }
    }

    impl Drop for Cf {
        fn drop(&mut self) {
            // SAFETY: Each Cf owns exactly one non-null +1 reference.
            unsafe { CFRelease(self.0) };
        }
    }

    fn cf_string(value: &str) -> Result<Cf, String> {
        let value = CString::new(value)
            .map_err(|_| "Text contains an unsupported NUL character.".to_owned())?;
        // SAFETY: The C string lives through the call; the create rule transfers ownership.
        Cf::owned(unsafe { CFStringCreateWithCString(ptr::null(), value.as_ptr(), UTF8) })
    }

    // All messages below target documented Foundation/AppKit methods with their
    // exact argument and return ABI. No activation or UI mutation selectors occur.
    unsafe fn msg_id(receiver: Id, selector: &CStr) -> Id {
        let call: unsafe extern "C" fn(Id, Sel) -> Id =
            unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        unsafe { call(receiver, sel_registerName(selector.as_ptr())) }
    }

    unsafe fn msg_id_arg(receiver: Id, selector: &CStr, argument: Id) -> Id {
        let call: unsafe extern "C" fn(Id, Sel, Id) -> Id =
            unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        unsafe { call(receiver, sel_registerName(selector.as_ptr()), argument) }
    }

    unsafe fn msg_id_index(receiver: Id, selector: &CStr, index: usize) -> Id {
        let call: unsafe extern "C" fn(Id, Sel, usize) -> Id =
            unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        unsafe { call(receiver, sel_registerName(selector.as_ptr()), index) }
    }

    unsafe fn msg_count(receiver: Id) -> usize {
        let call: unsafe extern "C" fn(Id, Sel) -> usize =
            unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        unsafe { call(receiver, sel_registerName(c"count".as_ptr())) }
    }

    unsafe fn msg_pid(receiver: Id) -> i32 {
        let call: unsafe extern "C" fn(Id, Sel) -> i32 =
            unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        unsafe { call(receiver, sel_registerName(c"processIdentifier".as_ptr())) }
    }

    unsafe fn msg_integer(receiver: Id, selector: &CStr) -> isize {
        let call: unsafe extern "C" fn(Id, Sel) -> isize =
            unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        unsafe { call(receiver, sel_registerName(selector.as_ptr())) }
    }

    unsafe fn msg_void(receiver: Id, selector: &CStr) {
        let call: unsafe extern "C" fn(Id, Sel) =
            unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        unsafe { call(receiver, sel_registerName(selector.as_ptr())) };
    }

    unsafe fn objc_text(value: Id) -> String {
        if value.is_null() {
            return String::new();
        }
        let bytes = unsafe { msg_id(value, c"UTF8String") }.cast::<c_char>();
        if bytes.is_null() {
            String::new()
        } else {
            unsafe { CStr::from_ptr(bytes) }
                .to_string_lossy()
                .into_owned()
        }
    }

    struct Pool(Id);

    impl Pool {
        fn new() -> Result<Self, String> {
            // SAFETY: NSAutoreleasePool is used on the current thread and drained
            // before the scope exits. All borrowed ObjC values stay within it.
            let pool = unsafe { msg_id(objc_getClass(c"NSAutoreleasePool".as_ptr()), c"new") };
            if pool.is_null() {
                Err("Could not initialize the macOS application query.".to_owned())
            } else {
                Ok(Self(pool))
            }
        }
    }

    impl Drop for Pool {
        fn drop(&mut self) {
            // SAFETY: drain consumes this thread's still-live autorelease pool.
            unsafe { msg_void(self.0, c"drain") };
        }
    }

    fn front_pid() -> Result<i32, String> {
        // SAFETY: These are read-only documented NSWorkspace/NSRunningApplication queries.
        let application = unsafe {
            let workspace = msg_id(objc_getClass(c"NSWorkspace".as_ptr()), c"sharedWorkspace");
            msg_id(workspace, c"frontmostApplication")
        };
        if application.is_null() {
            return Err("Could not identify the foreground app; handoff stopped.".to_owned());
        }
        let pid = unsafe { msg_pid(application) };
        if pid <= 0 {
            Err("Could not identify the foreground app; handoff stopped.".to_owned())
        } else {
            Ok(pid)
        }
    }

    pub fn foreground_pid() -> Result<i32, String> {
        let _pool = Pool::new()?;
        front_pid()
    }

    fn pasteboard_generation() -> Result<i64, String> {
        // SAFETY: These are read-only NSPasteboard queries. changeCount returns
        // NSInteger (pointer-sized signed integer), not pid_t or an ObjC object.
        let pasteboard = unsafe {
            msg_id(
                objc_getClass(c"NSPasteboard".as_ptr()),
                c"generalPasteboard",
            )
        };
        if pasteboard.is_null() {
            return Err("Could not identify the system clipboard; handoff stopped.".to_owned());
        }
        let generation = unsafe { msg_integer(pasteboard, c"changeCount") };
        if generation < 0 {
            return Err(
                "Could not verify the system clipboard generation; handoff stopped.".to_owned(),
            );
        }
        Ok(generation as i64)
    }

    pub fn clipboard_generation() -> Result<i64, String> {
        let _pool = Pool::new()?;
        pasteboard_generation()
    }

    pub fn accessibility_granted() -> bool {
        // SAFETY: This reads authorization state and never prompts.
        unsafe { AXIsProcessTrusted() != 0 }
    }

    pub fn request_accessibility() -> bool {
        // SAFETY: Dictionary entries are process-lifetime CF constants. Null
        // callbacks do not transfer ownership; the dictionary is released here.
        unsafe {
            let keys = [kAXTrustedCheckOptionPrompt];
            let values = [kCFBooleanTrue];
            let options = CFDictionaryCreate(
                ptr::null(),
                keys.as_ptr(),
                values.as_ptr(),
                1,
                ptr::null(),
                ptr::null(),
            );
            let Ok(options) = Cf::owned(options) else {
                return false;
            };
            AXIsProcessTrustedWithOptions(options.0) != 0
        }
    }

    fn running_apps() -> Result<Vec<(i32, String)>, String> {
        let mut found = Vec::new();
        for bundle in ALLOWED_BUNDLES {
            let bundle_c = CString::new(bundle).expect("static bundle ID");
            // SAFETY: NSString copies the C string; NSArray is kept alive by the
            // caller's autorelease pool. Bundle IDs are verified before use.
            unsafe {
                let ns_string = msg_id_arg(
                    objc_getClass(c"NSString".as_ptr()),
                    c"stringWithUTF8String:",
                    bundle_c.as_ptr().cast_mut().cast(),
                );
                let apps = msg_id_arg(
                    objc_getClass(c"NSRunningApplication".as_ptr()),
                    c"runningApplicationsWithBundleIdentifier:",
                    ns_string,
                );
                if apps.is_null() {
                    continue;
                }
                let count = msg_count(apps);
                if count > 64 {
                    return Err("Too many matching ChatGPT application processes.".to_owned());
                }
                for index in 0..count {
                    let app = msg_id_index(apps, c"objectAtIndex:", index);
                    let actual_bundle = objc_text(msg_id(app, c"bundleIdentifier"));
                    if actual_bundle != bundle {
                        continue;
                    }
                    let pid = msg_pid(app);
                    if pid > 0 && !found.iter().any(|(existing, _)| *existing == pid) {
                        let name = objc_text(msg_id(app, c"localizedName"));
                        found.push((
                            pid,
                            if name.is_empty() {
                                "ChatGPT".to_owned()
                            } else {
                                name
                            },
                        ));
                    }
                }
            }
        }
        Ok(found)
    }

    struct Guard<'a> {
        deadline: Instant,
        cancel: &'a AtomicBool,
        foreground: i32,
        clipboard: Option<i64>,
    }

    impl Guard<'_> {
        fn check(&self) -> Result<(), String> {
            if self.cancel.load(Ordering::Acquire) {
                return Err("Handoff cancelled.".to_owned());
            }
            if Instant::now() >= self.deadline {
                return Err("Handoff reached its 15-second limit.".to_owned());
            }
            if front_pid()? != self.foreground {
                return Err(
                    "The foreground app changed. Handoff stopped without restoring focus."
                        .to_owned(),
                );
            }
            if let Some(expected) = self.clipboard
                && pasteboard_generation()? != expected
            {
                return Err("The clipboard changed after the screenshot was prepared. Handoff stopped to avoid sending a different image.".to_owned());
            }
            Ok(())
        }
    }

    fn ax_error(action: &str, code: i32) -> String {
        let explanation = match code {
            -25202 => "the selected app or control is no longer available",
            -25204 => "the app did not respond within the accessibility timeout",
            -25205 => "the app does not expose the required attribute",
            -25206 => "the app does not support the required action",
            -25208 => "the app does not implement this accessibility operation",
            -25211 => "Accessibility permission is unavailable",
            -25212 => "the required accessibility value is absent",
            _ => "macOS rejected the accessibility operation",
        };
        format!("Could not {action}: {explanation} (AX {code}).")
    }

    fn set_timeout(element: &Cf) -> Result<(), String> {
        // SAFETY: Live AX element; this changes only its IPC timeout, not UI state.
        let code = unsafe { AXUIElementSetMessagingTimeout(element.0, MESSAGE_TIMEOUT) };
        if code == 0 {
            Ok(())
        } else {
            Err(ax_error("set an accessibility timeout", code))
        }
    }

    fn attr(element: &Cf, name: &str, guard: &Guard<'_>) -> Result<Option<Cf>, String> {
        guard.check()?;
        set_timeout(element)?;
        let name = cf_string(name)?;
        let mut value = ptr::null();
        // SAFETY: AX takes a live element/name and writes an owned CF reference.
        let code = unsafe { AXUIElementCopyAttributeValue(element.0, name.0, &mut value) };
        match code {
            0 => Cf::owned(value).map(Some),
            -25205 | -25212 => Ok(None),
            _ => Err(ax_error("inspect the ChatGPT control", code)),
        }
    }

    fn text_attr(element: &Cf, name: &str, guard: &Guard<'_>) -> Result<String, String> {
        attr(element, name, guard)?.map_or_else(|| Ok(String::new()), |value| value.text())
    }

    fn bool_attr(element: &Cf, name: &str, guard: &Guard<'_>) -> Result<Option<bool>, String> {
        let Some(value) = attr(element, name, guard)? else {
            return Ok(None);
        };
        // SAFETY: CFBooleanGetValue is called only after checking its type.
        unsafe {
            if CFGetTypeID(value.0) != CFBooleanGetTypeID() {
                return Err("The app exposed an unsupported accessibility state.".to_owned());
            }
            Ok(Some(CFBooleanGetValue(value.0) != 0))
        }
    }

    fn required_attr(element: &Cf, name: &str, guard: &Guard<'_>) -> Result<Cf, String> {
        attr(element, name, guard)?.ok_or_else(|| {
            format!("ChatGPT does not expose {name}; background handoff is unavailable.")
        })
    }

    fn app_element(pid: i32) -> Result<Cf, String> {
        // SAFETY: pid comes from an approved running application.
        let app = Cf::owned(unsafe { AXUIElementCreateApplication(pid) })?;
        set_timeout(&app)?;
        Ok(app)
    }

    fn window_title(name: &str, window: &Cf, guard: &Guard<'_>) -> Result<String, String> {
        let title = text_attr(window, "AXTitle", guard)?;
        Ok(format!(
            "{name} — {}",
            if title.trim().is_empty() {
                "Untitled window"
            } else {
                title.trim()
            }
        ))
    }

    pub fn targets() -> Result<Vec<ChatTarget>, String> {
        if !accessibility_granted() {
            return Err("Grant Accessibility access using the permission button, then refresh ChatGPT windows.".to_owned());
        }
        let _pool = Pool::new()?;
        let cancel = AtomicBool::new(false);
        let guard = Guard {
            deadline: Instant::now() + TIMEOUT,
            cancel: &cancel,
            foreground: front_pid()?,
            clipboard: None,
        };
        let mut targets = Vec::new();
        for (pid, name) in running_apps()? {
            guard.check()?;
            let app = app_element(pid)?;
            let Some(windows) = attr(&app, "AXWindows", &guard)? else {
                continue;
            };
            let windows = windows.array()?;
            let identifiers: Vec<String> = windows
                .iter()
                .map(|window| text_attr(window, "AXIdentifier", &guard))
                .collect::<Result<_, _>>()?;
            for (window_index, window) in windows.into_iter().enumerate() {
                let identifier = &identifiers[window_index];
                let window_identifier = (!identifier.is_empty()
                    && identifiers
                        .iter()
                        .filter(|value| *value == identifier)
                        .count()
                        == 1)
                    .then(|| identifier.clone());
                targets.push(ChatTarget {
                    pid,
                    title: window_title(&name, &window, &guard)?,
                    window_index,
                    window_identifier,
                });
            }
        }
        guard.check()?;
        Ok(targets)
    }

    fn same_window_identity(
        target: &ChatTarget,
        app_name: &str,
        window: &Cf,
        guard: &Guard<'_>,
    ) -> Result<bool, String> {
        if window_title(app_name, window, guard)? != target.title {
            return Ok(false);
        }
        match &target.window_identifier {
            Some(identifier) => Ok(text_attr(window, "AXIdentifier", guard)? == *identifier),
            None => Ok(true),
        }
    }

    struct Node {
        element: Cf,
        parent: Option<usize>,
        role: String,
        identifier: String,
        placeholder: String,
        description: String,
        labels: Vec<String>,
        enabled: Option<bool>,
    }

    impl Node {
        fn composer(&self) -> bool {
            composer_label(
                &self.role,
                &self.identifier,
                &self.placeholder,
                &self.description,
            )
        }
        fn send(&self) -> bool {
            self.role == "AXButton" && self.labels.iter().any(|value| send_label(value))
        }
        fn busy(&self) -> bool {
            self.role == "AXButton" && self.labels.iter().any(|value| busy_label(value))
        }
        fn attachment(&self) -> bool {
            self.labels
                .iter()
                .any(|value| attachment_label(&self.role, value))
        }
    }

    fn tree(root: &Cf, guard: &Guard<'_>) -> Result<Vec<Node>, String> {
        let mut stack = vec![(root.clone(), None, 0_usize)];
        let mut nodes = Vec::new();
        while let Some((element, parent, depth)) = stack.pop() {
            guard.check()?;
            if nodes.len() >= MAX_NODES || depth > MAX_DEPTH {
                return Err(
                    "The ChatGPT accessibility tree exceeds the inspection limit.".to_owned(),
                );
            }
            let role = text_attr(&element, "AXRole", guard)?;
            let mut identifier = String::new();
            let mut placeholder = String::new();
            let mut description = String::new();
            let mut labels = Vec::new();
            let mut enabled = None;
            if matches!(
                role.as_str(),
                "AXTextArea" | "AXTextField" | "AXButton" | "AXImage" | "AXGroup"
            ) {
                description = text_attr(&element, "AXDescription", guard)?;
                labels.push(description.clone());
                labels.push(text_attr(&element, "AXTitle", guard)?);
            }
            if matches!(role.as_str(), "AXTextArea" | "AXTextField") {
                identifier = text_attr(&element, "AXIdentifier", guard)?;
                placeholder = text_attr(&element, "AXPlaceholderValue", guard)?;
            }
            if matches!(role.as_str(), "AXButton" | "AXTextArea" | "AXTextField") {
                enabled = bool_attr(&element, "AXEnabled", guard)?;
            }
            let children = attr(&element, "AXChildren", guard)?
                .map_or_else(|| Ok(Vec::new()), |value| value.array())?;
            let index = nodes.len();
            nodes.push(Node {
                element,
                parent,
                role,
                identifier,
                placeholder,
                description,
                labels,
                enabled,
            });
            if nodes.len() + stack.len() + children.len() > MAX_NODES {
                return Err(
                    "The ChatGPT accessibility tree exceeds the inspection limit.".to_owned(),
                );
            }
            for child in children.into_iter().rev() {
                // SAFETY: Only AXUIElement children are traversed.
                if unsafe { CFGetTypeID(child.0) == AXUIElementGetTypeID() } {
                    stack.push((child, Some(index), depth + 1));
                }
            }
        }
        Ok(nodes)
    }

    fn ensure_focus(app: &Cf, window: &Cf, composer: &Cf, guard: &Guard<'_>) -> Result<(), String> {
        guard.check()?;
        if !required_attr(app, "AXFocusedWindow", guard)?.same(window)
            || !required_attr(app, "AXFocusedUIElement", guard)?.same(composer)
        {
            return Err("The selected ChatGPT composer is not its existing focused control. Click that composer manually before returning to the website; no focus was changed.".to_owned());
        }
        guard.check()
    }

    fn draft(composer: &Cf, guard: &Guard<'_>) -> Result<String, String> {
        required_attr(composer, "AXValue", guard)?.text()
    }

    fn write_draft(composer: &Cf, prompt: &str, guard: &Guard<'_>) -> Result<(), String> {
        let name = cf_string("AXValue")?;
        let text = cf_string(prompt)?;
        guard.check()?;
        let mut settable = 0_u8;
        // SAFETY: Live AX element/name and a writable Boolean out-parameter.
        let code = unsafe { AXUIElementIsAttributeSettable(composer.0, name.0, &mut settable) };
        if code != 0 {
            return Err(ax_error("check whether the composer is editable", code));
        }
        if settable == 0 {
            return Err(
                "ChatGPT does not expose an editable composer; background handoff is unavailable."
                    .to_owned(),
            );
        }
        guard.check()?;
        // SAFETY: This sets only the verified empty composer's text value.
        let code = unsafe { AXUIElementSetAttributeValue(composer.0, name.0, text.0) };
        if code != 0 {
            return Err(ax_error("set the composer text", code));
        }
        guard.check()?;
        if draft(composer, guard)? != prompt {
            return Err(
                "ChatGPT did not confirm the intended composer text; handoff stopped.".to_owned(),
            );
        }
        Ok(())
    }

    /// Release targeted keys even on cancellation so the target cannot be left
    /// with a synthetic modifier down. No event is posted system-wide.
    struct Keys<'a> {
        app: &'a Cf,
        command: bool,
        paste: bool,
    }

    impl Drop for Keys<'_> {
        fn drop(&mut self) {
            // SAFETY: app remains live; these are cleanup key-up events directed
            // only at the same approved application, never the system-wide object.
            unsafe {
                if self.paste {
                    AXUIElementPostKeyboardEvent(self.app.0, 0x76, 0x09, 0);
                }
                if self.command {
                    AXUIElementPostKeyboardEvent(self.app.0, 0, 0x37, 0);
                }
            }
        }
    }

    fn paste(app: &Cf, guard: &Guard<'_>) -> Result<(), String> {
        let mut keys = Keys {
            app,
            command: false,
            paste: false,
        };
        for (character, virtual_key, down) in
            [(0, 0x37, 1), (0x76, 0x09, 1), (0x76, 0x09, 0), (0, 0x37, 0)]
        {
            guard.check()?;
            // A timeout does not prove a key-down was ignored. Arrange targeted
            // key-up cleanup before issuing each possible key-down.
            if down != 0 {
                if virtual_key == 0x37 {
                    keys.command = true;
                } else {
                    keys.paste = true;
                }
            }
            // SAFETY: The application-specific AX object is always used. Carbon
            // key codes 0x37/0x09 are Command/V; Boolean is UInt8 in this API.
            let code = unsafe { AXUIElementPostKeyboardEvent(app.0, character, virtual_key, down) };
            if code != 0 {
                return Err(ax_error("paste into the existing ChatGPT composer", code));
            }
            if virtual_key == 0x37 {
                keys.command = down != 0;
            } else {
                keys.paste = down != 0;
            }
        }
        guard.check()
    }

    fn press(button: &Cf, guard: &Guard<'_>) -> Result<(), String> {
        guard.check()?;
        let title = text_attr(button, "AXTitle", guard)?;
        let description = text_attr(button, "AXDescription", guard)?;
        if text_attr(button, "AXRole", guard)? != "AXButton"
            || ![&title, &description].iter().any(|value| send_label(value))
            || [&title, &description].iter().any(|value| busy_label(value))
            || bool_attr(button, "AXEnabled", guard)? != Some(true)
        {
            return Err(
                "The button is no longer a confirmed enabled Send control; handoff stopped."
                    .to_owned(),
            );
        }
        let mut actions = ptr::null();
        // SAFETY: The live AX element writes a retained CFArray of actions.
        let code = unsafe { AXUIElementCopyActionNames(button.0, &mut actions) };
        if code != 0 {
            return Err(ax_error("inspect the Send action", code));
        }
        let actions = Cf::owned(actions)?.array()?;
        if !actions
            .iter()
            .any(|action| action.text().is_ok_and(|value| value == "AXPress"))
        {
            return Err("The identified Send button does not support AXPress.".to_owned());
        }
        let action = cf_string("AXPress")?;
        guard.check()?;
        // SAFETY: The only performed action is AXPress on a verified Send button.
        let code = unsafe { AXUIElementPerformAction(button.0, action.0) };
        if code == 0 {
            Ok(())
        } else {
            Err(ax_error("press Send", code))
        }
    }

    pub fn send(
        target: &ChatTarget,
        prompt: &str,
        cancel: &AtomicBool,
        expected_foreground: i32,
        expected_clipboard: i64,
    ) -> Result<String, String> {
        if !accessibility_granted() {
            return Err(
                "Accessibility access is required for the experimental ChatGPT handoff.".to_owned(),
            );
        }
        if prompt.trim().is_empty() {
            return Err("The handoff prompt is empty.".to_owned());
        }
        if expected_foreground <= 0 || expected_clipboard < 0 {
            return Err(
                "The original foreground app or prepared clipboard could not be verified."
                    .to_owned(),
            );
        }
        let _pool = Pool::new()?;
        let guard = Guard {
            deadline: Instant::now() + TIMEOUT,
            cancel,
            foreground: expected_foreground,
            clipboard: Some(expected_clipboard),
        };
        guard.check()?;
        if guard.foreground == target.pid {
            return Err("Return to the website before triggering background handoff.".to_owned());
        }
        let app_name = running_apps()?
            .into_iter()
            .find(|(pid, _)| *pid == target.pid)
            .map(|(_, name)| name)
            .ok_or_else(|| "The selected approved ChatGPT app is no longer running.".to_owned())?;
        let app = app_element(target.pid)?;
        let windows = required_attr(&app, "AXWindows", &guard)?.array()?;
        let window = if let Some(identifier) = &target.window_identifier {
            let mut matching = Vec::new();
            for window in &windows {
                if text_attr(window, "AXIdentifier", &guard)? == *identifier {
                    matching.push(window);
                }
            }
            if matching.len() != 1 {
                return Err("The selected ChatGPT window identifier is no longer unique or available; refresh the window list.".to_owned());
            }
            matching[0]
        } else {
            windows.get(target.window_index).ok_or_else(|| {
                "The selected ChatGPT window is no longer available; refresh the window list."
                    .to_owned()
            })?
        };
        if !same_window_identity(target, &app_name, window, &guard)? {
            return Err(
                "The selected ChatGPT window changed; refresh and select it again.".to_owned(),
            );
        }
        let focused = required_attr(&app, "AXFocusedUIElement", &guard)?;
        let nodes = tree(window, &guard)?;
        if nodes.iter().any(Node::busy) {
            return Err("ChatGPT is already generating a response. Handoff stopped.".to_owned());
        }
        let composer_index = nodes.iter().position(|node| node.composer() && node.element.same(&focused))
            .ok_or_else(|| "The existing focused ChatGPT control is not a recognized message composer. Focus the intended composer manually, then return to the website.".to_owned())?;
        let composer = nodes[composer_index].element.clone();
        if nodes[composer_index].enabled != Some(true) {
            return Err("The selected ChatGPT composer is not confirmed enabled.".to_owned());
        }
        ensure_focus(&app, window, &composer, &guard)?;
        if !draft(&composer, &guard)?.is_empty() {
            return Err("The selected ChatGPT composer already contains text. Clear or send that draft manually first.".to_owned());
        }

        // Inspect a bounded nearby composer group, rather than treating historical
        // images elsewhere in the conversation as new attachments.
        let mut ancestor = nodes[composer_index].parent;
        let mut scope = None;
        for _ in 0..3 {
            let Some(index) = ancestor else { break };
            if matches!(nodes[index].role.as_str(), "AXGroup" | "AXToolbar") {
                scope = Some(nodes[index].element.clone());
            }
            if matches!(nodes[index].role.as_str(), "AXWindow" | "AXWebArea") {
                break;
            }
            ancestor = nodes[index].parent;
        }
        let scope = scope.ok_or_else(|| "ChatGPT does not expose a bounded composer group; background attachment verification is unavailable.".to_owned())?;
        if tree(&scope, &guard)?.iter().any(Node::attachment) {
            return Err("The composer group already contains an image or attachment. Remove it manually before handoff.".to_owned());
        }

        let mut draft_changed = false;
        let mut send_attempted = false;
        let result = (|| {
            ensure_focus(&app, window, &composer, &guard)?;
            // An AX call can take effect even if its reply times out, so record
            // possible draft mutation before issuing it.
            draft_changed = true;
            write_draft(&composer, prompt, &guard)?;
            ensure_focus(&app, window, &composer, &guard)?;
            paste(&app, &guard)?;

            let button = loop {
                guard.check()?;
                ensure_focus(&app, window, &composer, &guard)?;
                if draft(&composer, &guard)? != prompt {
                    return Err(
                        "The composer text changed after paste; handoff stopped.".to_owned()
                    );
                }
                let current = tree(&scope, &guard)?;
                if current.iter().any(Node::busy) {
                    return Err("ChatGPT became busy before Send; handoff stopped.".to_owned());
                }
                let buttons: Vec<_> = current
                    .iter()
                    .filter(|node| node.send() && node.enabled == Some(true))
                    .collect();
                if current.iter().any(Node::attachment) && buttons.len() == 1 {
                    break buttons[0].element.clone();
                }
                if buttons.len() > 1 {
                    return Err(
                        "Multiple enabled Send buttons were exposed; handoff stopped.".to_owned(),
                    );
                }
                std::thread::sleep(Duration::from_millis(80));
            };

            ensure_focus(&app, window, &composer, &guard)?;
            if !same_window_identity(target, &app_name, window, &guard)? {
                return Err(
                    "The selected ChatGPT chat or window changed; handoff stopped.".to_owned(),
                );
            }
            if draft(&composer, &guard)? != prompt
                || bool_attr(&button, "AXEnabled", &guard)? != Some(true)
            {
                return Err("The composer or Send state changed; handoff stopped.".to_owned());
            }
            guard.check()?;
            send_attempted = true;
            press(&button, &guard)?;
            guard.check()?;

            loop {
                guard.check()?;
                let current = tree(&scope, &guard)?;
                let empty_composer = current
                    .iter()
                    .filter(|node| node.composer())
                    .any(|node| draft(&node.element, &guard).is_ok_and(|value| value.is_empty()));
                if empty_composer && !current.iter().any(Node::attachment) {
                    guard.check()?;
                    return Ok("ChatGPT cleared the submitted composer. Foreground app remained unchanged during the checks.".to_owned());
                }
                std::thread::sleep(Duration::from_millis(80));
            }
        })();
        result.map_err(|error| {
            if send_attempted {
                format!("{error} Send may already have taken effect. Inspect ChatGPT before retrying; no retry was attempted.")
            } else if draft_changed {
                format!("{error} A draft or attachment may remain in ChatGPT. Send was not invoked; nothing was cleared automatically.")
            } else { error }
        })
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::{AtomicBool, ChatTarget};
    pub fn accessibility_granted() -> bool {
        false
    }
    pub fn request_accessibility() -> bool {
        false
    }
    pub fn foreground_pid() -> Result<i32, String> {
        Err("Background ChatGPT handoff currently supports macOS only.".to_owned())
    }
    pub fn clipboard_generation() -> Result<i64, String> {
        Err("Background ChatGPT handoff currently supports macOS only.".to_owned())
    }
    pub fn targets() -> Result<Vec<ChatTarget>, String> {
        Err("Background ChatGPT handoff currently supports macOS only.".to_owned())
    }
    pub fn send(_: &ChatTarget, _: &str, _: &AtomicBool, _: i32, _: i64) -> Result<String, String> {
        Err("Background ChatGPT handoff currently supports macOS only.".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composer_must_have_a_text_role_and_known_identity() {
        assert!(composer_label("AXTextArea", "prompt-textarea", "", ""));
        assert!(composer_label("AXTextField", "", "Ask anything…", ""));
        assert!(composer_label("AXTextArea", "", "Ask for follow-up", ""));
        assert!(!composer_label("AXButton", "chat-input", "Message", ""));
        assert!(!composer_label("AXTextField", "search", "Search chats", ""));
    }

    #[test]
    fn send_and_busy_labels_are_exact() {
        assert!(send_label(" Send message "));
        assert!(send_label("Send"));
        assert!(!send_label("Send feedback"));
        assert!(!send_label("Resend message"));
        assert!(busy_label("Stop generating"));
        assert!(!busy_label("Stopwatch"));
    }

    #[test]
    fn attachment_requires_a_recognized_indicator() {
        assert!(attachment_label(
            "AXButton",
            "Remove attachment screenshot.png"
        ));
        assert!(attachment_label("AXImage", "Image"));
        assert!(!attachment_label("AXButton", "Create image"));
        assert!(!attachment_label("AXStaticText", "Explain this image"));
    }
}
