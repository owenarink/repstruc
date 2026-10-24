//! Capture a selected display without generating input or changing application focus.

use xcap::Monitor;

#[derive(Clone, Debug)]
pub struct DisplayInfo {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub primary: bool,
}

#[derive(Debug)]
pub struct Captured {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn displays() -> Result<Vec<DisplayInfo>, String> {
    let monitors = Monitor::all().map_err(|error| format!("Could not list displays: {error}"))?;

    monitors
        .into_iter()
        .map(|monitor| {
            let id = monitor
                .id()
                .map_err(|error| format!("Could not identify a display: {error}"))?;
            let name = monitor
                .friendly_name()
                .or_else(|_| monitor.name())
                .unwrap_or_else(|_| format!("Display {id}"));
            let metadata_error = |error| format!("Could not read display {id}: {error}");

            Ok(DisplayInfo {
                id,
                name,
                x: monitor.x().map_err(metadata_error)?,
                y: monitor.y().map_err(metadata_error)?,
                width: monitor.width().map_err(metadata_error)?,
                height: monitor.height().map_err(metadata_error)?,
                primary: monitor.is_primary().map_err(metadata_error)?,
            })
        })
        .collect()
}

pub fn capture(display_id: u32) -> Result<Captured, String> {
    if !permission_granted() {
        return Err(
            "Screen Recording access is required. Grant it in System Settings > Privacy & Security > Screen & System Audio Recording, then restart the app if macOS asks."
                .to_owned(),
        );
    }

    let monitor = Monitor::all()
        .map_err(|error| format!("Could not list displays: {error}"))?
        .into_iter()
        .find(|monitor| monitor.id().ok() == Some(display_id))
        .ok_or_else(|| {
            "The selected display is no longer available. Refresh the display list and select it again."
                .to_owned()
        })?;

    let image = monitor.capture_image().map_err(|error| {
        format!(
            "Could not capture the selected display: {error}. Check Screen Recording access and that the display is connected."
        )
    })?;

    Ok(Captured {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    })
}

/// Read the current permission state without showing a consent dialog.
#[must_use]
pub fn permission_granted() -> bool {
    #[cfg(target_os = "macos")]
    {
        // SAFETY: This public Core Graphics function has no arguments or ownership
        // requirements and only reads the calling process's authorization state.
        unsafe { macos::CGPreflightScreenCaptureAccess() }
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Ask macOS for Screen Recording access. Call only from an explicit UI action.
#[must_use]
pub fn request_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        // SAFETY: This public Core Graphics function has no arguments or ownership
        // requirements. The caller is responsible for initiating it from the UI.
        unsafe { macos::CGRequestScreenCaptureAccess() }
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

#[cfg(target_os = "macos")]
mod macos {
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        pub(super) fn CGPreflightScreenCaptureAccess() -> bool;
        pub(super) fn CGRequestScreenCaptureAccess() -> bool;
    }
}
