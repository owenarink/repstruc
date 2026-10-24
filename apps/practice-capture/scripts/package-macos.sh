#!/bin/sh
# Build the native Apple Silicon app and prepare a downloadable ZIP.
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
target=aarch64-apple-darwin
dist_dir="$project_dir/dist"
app_dir="$dist_dir/Practice Capture.app"
archive_path="$dist_dir/Practice-Capture-macOS-arm64.zip"

if [ "$(uname -s)" != Darwin ]; then
    printf '%s\n' 'Packaging requires macOS.' >&2
    exit 1
fi

for program in cargo ditto codesign; do
    if ! command -v "$program" >/dev/null 2>&1; then
        printf 'Required program is missing: %s\n' "$program" >&2
        exit 1
    fi
done

cd "$project_dir"
if [ "${SKIP_BUILD:-0}" != 1 ]; then
    MACOSX_DEPLOYMENT_TARGET=13.0 cargo build --release --locked --target "$target"
fi
release_binary="$project_dir/target/$target/release/practice-capture"
if [ ! -x "$release_binary" ]; then
    printf 'Release binary is missing: %s\n' "$release_binary" >&2
    exit 1
fi

mkdir -p "$dist_dir"
package_dir=$(mktemp -d "$dist_dir/.practice-capture-package.XXXXXX")
trap 'rm -rf "$package_dir"' EXIT HUP INT TERM
payload_dir="$package_dir/Practice-Capture"
staged_app="$payload_dir/Practice Capture.app"
mkdir -p "$staged_app/Contents/MacOS"
mkdir -p "$staged_app/Contents/Resources"

cp "$release_binary" \
    "$staged_app/Contents/MacOS/practice-capture"
chmod 755 "$staged_app/Contents/MacOS/practice-capture"
cp "$project_dir/LICENSE" "$staged_app/Contents/Resources/LICENSE"
if [ -f "$project_dir/THIRD_PARTY_NOTICES.txt" ]; then
    cp "$project_dir/THIRD_PARTY_NOTICES.txt" "$staged_app/Contents/Resources/THIRD_PARTY_NOTICES.txt"
fi

cat > "$staged_app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>Practice Capture</string>
    <key>CFBundleDisplayName</key><string>Practice Capture</string>
    <key>CFBundleIdentifier</key><string>com.practicecapture.app</string>
    <key>CFBundleExecutable</key><string>practice-capture</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>0.1.0</string>
    <key>CFBundleVersion</key><string>1</string>
    <key>LSMinimumSystemVersion</key><string>13.0</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>NSScreenCaptureUsageDescription</key>
    <string>Capture the selected display when you request practice help.</string>
    <key>NSAccessibilityUsageDescription</key>
    <string>Attach your requested screenshot and send solve to the selected ChatGPT window.</string>
</dict>
</plist>
PLIST

/usr/bin/plutil -lint "$staged_app/Contents/Info.plist"
codesign --force --sign - "$staged_app"
codesign --verify --strict "$staged_app"

cp "$project_dir/scripts/install.command" "$payload_dir/install.command"
chmod 755 "$payload_dir/install.command"
cp "$project_dir/README.md" "$payload_dir/README.md"
mkdir -p "$payload_dir/context"
cp "$project_dir/assets/rule-library.md" "$payload_dir/context/rule-library.md"

# ditto replaces the existing archive. It preserves the bundle's metadata.
ditto -c -k --sequesterRsrc --keepParent "$payload_dir" "$archive_path"

# Keep an unpacked copy beside the ZIP for local inspection.
if [ -e "$app_dir" ] || [ -L "$app_dir" ]; then
    previous_app="$dist_dir/Practice Capture.previous.$(date -u +%Y%m%dT%H%M%SZ).$$.app"
    mv "$app_dir" "$previous_app"
    printf 'Previous local build saved at: %s\n' "$previous_app"
fi
ditto "$staged_app" "$app_dir"

printf '\nApp: %s\nZIP: %s\n' "$app_dir" "$archive_path"
printf '%s\n' 'This build is ad-hoc signed. Public distribution still requires Developer ID signing and notarization for the smoothest macOS setup.'
