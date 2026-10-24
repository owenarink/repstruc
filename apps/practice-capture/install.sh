#!/bin/sh
# Pinned public installer. Keep outside the ZIP to avoid a checksum cycle.
# The publisher must replace the checksum after creating the final ZIP.
set -eu

release_tag='practice-capture-v0.1.0-preview.1'
archive_name='Practice-Capture-macOS-arm64.zip'
expected_sha256='5c11bc73678380e3f6d7c9efed2e77372ff74fc746fcbd13ed60790d5def116b'
download_url="https://github.com/owenarink/repstruc/releases/download/$release_tag/$archive_name"

if [ "$(uname -s)" != Darwin ]; then
    printf '%s\n' 'Practice Capture is currently available for Apple Silicon Macs only.' >&2
    exit 1
fi
if [ "$(uname -m)" != arm64 ] && \
    [ "$(/usr/sbin/sysctl -n hw.optional.arm64 2>/dev/null || true)" != 1 ]; then
    printf '%s\n' 'This release requires an Apple Silicon Mac.' >&2
    exit 1
fi
macos_version=$(/usr/bin/sw_vers -productVersion)
macos_major=${macos_version%%.*}
if [ "$macos_major" -lt 13 ]; then
    printf 'This release requires macOS 13 or later; this Mac runs %s.\n' "$macos_version" >&2
    exit 1
fi

case "$expected_sha256" in
    ''|*[!0123456789abcdef]*)
        printf '%s\n' 'The publisher has not finalized this installer checksum. No download or installation was attempted.' >&2
        exit 1
        ;;
esac
if [ "${#expected_sha256}" -ne 64 ]; then
    printf '%s\n' 'The pinned release checksum is invalid. No download or installation was attempted.' >&2
    exit 1
fi

capture_install_dir=$(mktemp -d "${TMPDIR:-/tmp}/practice-capture.XXXXXX")
trap 'rm -rf "$capture_install_dir"' 0
trap 'exit 1' 1 2 15
archive_path="$capture_install_dir/$archive_name"

printf 'Downloading Practice Capture %s…\n' "$release_tag"
/usr/bin/curl --fail --location --proto '=https' --proto-redir '=https' \
    --tlsv1.2 --connect-timeout 15 --max-time 180 --retry 2 \
    --output "$archive_path" "$download_url"

checksum_line=$(/usr/bin/shasum -a 256 "$archive_path")
actual_sha256=${checksum_line%% *}
if [ "$actual_sha256" != "$expected_sha256" ]; then
    printf '%s\n' 'The downloaded ZIP does not match the pinned SHA-256 checksum. Nothing was installed.' >&2
    exit 1
fi

/usr/bin/ditto -x -k "$archive_path" "$capture_install_dir/payload"
payload_installer="$capture_install_dir/payload/Practice-Capture/install.command"
if [ ! -f "$payload_installer" ]; then
    printf '%s\n' 'The verified archive does not contain the expected installer. Nothing was installed.' >&2
    exit 1
fi

sh "$payload_installer"
