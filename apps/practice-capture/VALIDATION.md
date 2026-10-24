# Preview validation

Release: `practice-capture-v0.1.0-preview.1` (2026-10-24)

Validation performed on an Apple Silicon Mac:

- `cargo fmt --check` passed.
- `cargo clippy --release --locked --target aarch64-apple-darwin --all-targets -- -D warnings` passed.
- `cargo test --release --locked --target aarch64-apple-darwin` passed all six tests.
- `cargo build --release --locked --target aarch64-apple-darwin` passed.
- Packaged app bundle metadata and ad-hoc code signature passed validation.
- ZIP extraction, installation into a temporary application directory without launch, and the installed executable's `--version` command passed.
- The diagnostic command ran without capture or permission prompts. It reported Screen Recording permission available and Accessibility permission unavailable in the build environment.

These checks do not validate a live screenshot-to-ChatGPT submission. The Accessibility handoff has not been exercised with permission enabled, and compatibility with the recipient's ChatGPT desktop app remains unverified. Tests cover session cancellation, duplicate-key handling, debounce, and Accessibility control classification.

The binary is ad-hoc signed and is not notarized. macOS first-launch approval, Screen Recording permission, Accessibility permission, and initial destination selection are required. Browser focus continuity and undetectability to websites or monitoring software are not guaranteed.

Published ZIP SHA-256:

```text
5c11bc73678380e3f6d7c9efed2e77372ff74fc746fcbd13ed60790d5def116b
```

The release includes general solving instructions. It excludes the original IQ screenshots, private question database, and answer key.
