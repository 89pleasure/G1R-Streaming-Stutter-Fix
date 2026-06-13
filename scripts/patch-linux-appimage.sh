#!/usr/bin/env bash
set -euo pipefail

readonly APPIMAGETOOL_URL="${APPIMAGETOOL_URL:-https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage}"

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <appimage-directory>" >&2
  exit 64
fi

appimage_dir="$1"
if [[ ! -d "$appimage_dir" ]]; then
  echo "AppImage directory does not exist: $appimage_dir" >&2
  exit 1
fi

shopt -s nullglob
appimages=("$appimage_dir"/*.AppImage)
if [[ ${#appimages[@]} -eq 0 ]]; then
  echo "No AppImage files found in: $appimage_dir" >&2
  exit 1
fi

appimagetool="${APPIMAGETOOL:-}"
if [[ -z "$appimagetool" ]]; then
  appimagetool="${RUNNER_TEMP:-/tmp}/appimagetool-x86_64.AppImage"
  if [[ ! -x "$appimagetool" ]]; then
    curl --fail --location --show-error --output "$appimagetool" "$APPIMAGETOOL_URL"
    chmod +x "$appimagetool"
  fi
fi

if [[ ! -x "$appimagetool" ]]; then
  echo "appimagetool is not executable: $appimagetool" >&2
  exit 1
fi

cleanup_dirs=()
cleanup() {
  if [[ ${#cleanup_dirs[@]} -gt 0 ]]; then
    rm -rf "${cleanup_dirs[@]}"
  fi
}
trap cleanup EXIT

remove_gtk_wayland_cache_entries() {
  local cache_file="$1"

  if [[ ! -f "$cache_file" ]]; then
    return
  fi

  awk '
    /^"im-wayland(gtk)?\.so"/ { skip = 2; next }
    skip > 0 { skip--; next }
    { print }
  ' "$cache_file" > "${cache_file}.tmp"
  mv "${cache_file}.tmp" "$cache_file"
}

for appimage in "${appimages[@]}"; do
  echo "Patching AppImage for rolling-release Linux compatibility: $appimage"

  tmpdir="$(mktemp -d)"
  cleanup_dirs+=("$tmpdir")

  cp "$appimage" "$tmpdir/source.AppImage"
  chmod +x "$tmpdir/source.AppImage"

  (
    cd "$tmpdir"
    ./source.AppImage --appimage-extract > /dev/null
  )

  appdir="$tmpdir/squashfs-root"
  if [[ ! -d "$appdir" ]]; then
    echo "Failed to extract AppImage: $appimage" >&2
    exit 1
  fi

  remove_gtk_wayland_cache_entries \
    "$appdir/usr/lib/x86_64-linux-gnu/gtk-3.0/3.0.0/immodules.cache"

  find "$appdir/usr/lib" -maxdepth 1 \
    \( -name 'libwayland-*.so*' -o -name 'im-wayland*.so' \) \
    -print -delete

  if [[ -d "$appdir/usr/lib/x86_64-linux-gnu/gtk-3.0/3.0.0/immodules" ]]; then
    find "$appdir/usr/lib/x86_64-linux-gnu/gtk-3.0/3.0.0/immodules" -maxdepth 1 \
      -name 'im-wayland*.so' \
      -print -delete
  fi

  if find "$appdir/usr/lib" -maxdepth 1 -name 'libwayland-*.so*' | grep -q .; then
    echo "Bundled Wayland libraries remain in AppImage after patching." >&2
    exit 1
  fi

  rm -f "$appimage"
  ARCH=x86_64 "$appimagetool" --appimage-extract-and-run "$appdir" "$appimage"
  chmod +x "$appimage"
done
