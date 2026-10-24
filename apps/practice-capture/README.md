# Practice Capture

A Rust desktop app for practice questions on macOS. **This is an experimental preview:** screenshot capture and Start/Stop controls are implemented; automatic handoff to the existing ChatGPT desktop app still needs a live compatibility check with Accessibility permission enabled.

The intended flow is: press **Start**, return to your browser, then press **O**. The app captures your selected display, puts the image on the clipboard, and attempts to attach it to your selected ChatGPT conversation and send `solve`. Answers appear in ChatGPT, which you can place on your second monitor. No API key is needed.

The handoff uses macOS Accessibility actions against the existing ChatGPT composer. It does not intentionally activate or raise ChatGPT, move the pointer, or generate global mouse clicks or keystrokes. It checks the foreground application and aborts if it changes. **It cannot guarantee uninterrupted browser focus or make activity undetectable to websites or monitoring software.**

## One-command installation

This preview targets **Apple Silicon Macs running macOS 13 or later**, with the ChatGPT desktop app installed and signed in. Install the pinned preview with:

```sh
curl -fsSL https://raw.githubusercontent.com/owenarink/repstruc/practice-capture-v0.1.0-preview.1/apps/practice-capture/install.sh | sh
```

The standalone installer downloads the [pinned ZIP](https://github.com/owenarink/repstruc/releases/download/practice-capture-v0.1.0-preview.1/Practice-Capture-macOS-arm64.zip), verifies its SHA-256 checksum, copies the app to `~/Applications`, and opens it. It does not require `sudo`. A previous Practice Capture installation is backed up beside it before replacement. A different application with the same filename is left in place, and installation stops. [Source code](https://github.com/owenarink/repstruc/tree/practice-capture-v0.1.0-preview.1/apps/practice-capture) is included at the same pinned tag.

With Homebrew already installed, the [Homebrew tap](https://github.com/owenarink/homebrew-installs) also provides:

```sh
brew install --cask owenarink/installs/practice-capture && open -a "Practice Capture"
```

macOS first-launch approval, Screen Recording permission, and Accessibility permission still require setup. The installer cannot grant these permissions. This build is ad-hoc signed and **not notarized**; macOS may require first-launch approval under **System Settings → Privacy & Security**. The installer preserves macOS security attributes. A one-command install does not remove this initial setup.

If your friend already has `Practice-Capture-macOS-arm64.zip` in Downloads, this local installation command also works:

```sh
capture_install_dir="$(mktemp -d)" && ditto -x -k "$HOME/Downloads/Practice-Capture-macOS-arm64.zip" "$capture_install_dir" && sh "$capture_install_dir/Practice-Capture/install.command"
```

Downloading the ZIP again may add a filename suffix; use its actual filename in the local command.

## Use

1. Open ChatGPT and the practice conversation on your second monitor. This preview expects the ChatGPT desktop app's **English UI**. Leave its composer empty and select the composer once before returning to the browser.
2. Open Practice Capture. Use **Request Screen Recording permission** and **Request Accessibility permission** in **First-run macOS setup**. Approve both in System Settings; restart if macOS asks.
3. Click **Refresh displays and ChatGPT windows**, choose the display containing your question, and choose the correct **ChatGPT destination window**.
4. Leave **Include solving instructions with the first screenshot** enabled to send the compact, general solving rules with the first successful submission after app launch, refreshing windows, or changing the destination. Subsequent submissions send the screenshot with the literal message `solve`.
5. Press **Start**, return to your browser, and press **O** to capture and attempt automatic submission. Check the app's status during your first practice run.
6. Press **Stop** or quit the app to release O. Stop also cancels pending work where possible; an already submitted ChatGPT message cannot be retracted.

While Start is active, plain O is a global shortcut and may interfere with normal typing. The app ignores another capture while one is in progress. Display and destination settings can be changed while stopped.

The app stops the handoff if it cannot identify a compatible empty composer, verify an attachment, or perform the targeted send action. It does not fall back to activating ChatGPT or clicking screen coordinates. A failed attempt may leave the image on your clipboard or an attachment in the ChatGPT draft; inspect and clear the draft before retrying. ChatGPT app updates may change Accessibility compatibility.

The destination checks use the ChatGPT window's identifier and title; these checks do not guarantee the identity of the conversation inside that window. Keep the selected practice conversation open and do not navigate ChatGPT while capture is running. Refresh and select the destination again after changing its window or conversation.

The app keeps captured image data in memory and the clipboard rather than deliberately saving image files. The clipboard's previous contents are replaced. Clipboard managers and your ChatGPT conversation may retain submitted content. Keep unrelated private content off the selected display.

The included rules request an answer as soon as one is supported, with 60 seconds as a maximum, no tools or subagents, and only the option number. A prompt cannot enforce an exact wall-clock deadline inside ChatGPT. Their readable copy is in `context/rule-library.md` in the ZIP. The distribution includes general instructions, without the original IQ screenshots, private question database, or answer key.

## Build and package

Install the Rust toolchain on an Apple Silicon Mac, then run:

```sh
sh scripts/package-macos.sh
```

Packaging builds the release target `aarch64-apple-darwin` with a macOS 13 deployment target, verifies its ad-hoc signature, and creates:

- `dist/Practice Capture.app`
- `dist/Practice-Capture-macOS-arm64.zip`

The ZIP contains the app, installer, this README, and the practice rules inside a `Practice-Capture` folder. After a completed release build, `SKIP_BUILD=1 sh scripts/package-macos.sh` repackages it without compiling again.

The public `install.sh` stays outside the ZIP. After final packaging, compute `shasum -a 256 dist/Practice-Capture-macOS-arm64.zip` and set its exact digest in `expected_sha256` in `install.sh` before publishing the pinned source tag and ZIP. The script refuses to download or install anything while its checksum is unset. Keeping it outside the archive avoids a checksum cycle.

For smoother public distribution, sign and notarize a release with an Apple Developer ID before hosting it. No Developer ID signing identity was available for this preview. Intel Mac and Windows packages are not included. The app declares macOS 13 as its minimum version; compatibility with a recipient's exact macOS and ChatGPT versions must be checked in practice.

The installer intentionally does not launch during packaging. Installation launches the app only when the recipient runs `install.command`.
