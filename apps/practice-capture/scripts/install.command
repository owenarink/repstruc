#!/bin/sh
# Install the adjacent app in the current user's Applications directory.
set -eu

if [ "$(uname -s)" != Darwin ]; then
    printf '%s\n' 'This download is for Apple Silicon Macs.' >&2
    exit 1
fi
if [ "$(uname -m)" != arm64 ] && \
    [ "$(/usr/sbin/sysctl -n hw.optional.arm64 2>/dev/null || true)" != 1 ]; then
    printf '%s\n' 'This download is for Apple Silicon Macs.' >&2
    exit 1
fi
macos_version=$(/usr/bin/sw_vers -productVersion)
macos_major=${macos_version%%.*}
if [ "$macos_major" -lt 13 ]; then
    printf '%s\n' 'This release requires macOS 13 or later.' >&2
    exit 1
fi

payload_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
source_app="$payload_dir/Practice Capture.app"
applications_dir="${PRACTICE_CAPTURE_INSTALL_DIR:-$HOME/Applications}"
destination_app="$applications_dir/Practice Capture.app"
bundle_id=com.practicecapture.app

read_bundle_id() {
    /usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$1/Contents/Info.plist" 2>/dev/null
}

if [ ! -x "$source_app/Contents/MacOS/practice-capture" ] || \
    [ "$(read_bundle_id "$source_app" || true)" != "$bundle_id" ]; then
    printf '%s\n' 'The adjacent Practice Capture.app is missing or invalid. Extract the complete ZIP and try again.' >&2
    exit 1
fi

if [ -L "$destination_app" ]; then
    printf 'Refusing to replace a symbolic link: %s\n' "$destination_app" >&2
    exit 1
fi
if [ -e "$destination_app" ] && \
    [ "$(read_bundle_id "$destination_app" || true)" != "$bundle_id" ]; then
    printf 'Another application already exists at %s. Move it before installing.\n' "$destination_app" >&2
    exit 1
fi

mkdir -p "$applications_dir"
staging_dir=$(mktemp -d "$applications_dir/.practice-capture-install.XXXXXX")
trap 'rm -rf "$staging_dir"' EXIT HUP INT TERM
staged_app="$staging_dir/Practice Capture.app"
ditto "$source_app" "$staged_app"
codesign --verify --strict "$staged_app"

backup_app=''
if [ -e "$destination_app" ]; then
    backup_app="$applications_dir/Practice Capture.backup.$(date -u +%Y%m%dT%H%M%SZ).$$.app"
    mv "$destination_app" "$backup_app"
    printf 'Previous installation saved at: %s\n' "$backup_app"
fi

if ! mv "$staged_app" "$destination_app"; then
    if [ -n "$backup_app" ] && [ ! -e "$destination_app" ]; then
        mv "$backup_app" "$destination_app"
    fi
    printf '%s\n' 'Installation failed.' >&2
    exit 1
fi

printf 'Installed: %s\n' "$destination_app"
printf '%s\n' 'macOS may require first-launch approval, Screen Recording permission, and Accessibility permission.'
if [ "${PRACTICE_CAPTURE_NO_LAUNCH:-0}" != 1 ]; then
    open "$destination_app"
fi
