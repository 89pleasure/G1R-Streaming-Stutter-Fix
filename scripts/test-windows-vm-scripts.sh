#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

test_missing_optional_virtio_iso_is_not_attached() {
  if [[ -e /usr/share/virtio-win/virtio-win.iso ]]; then
    echo "SKIP: /usr/share/virtio-win/virtio-win.iso exists on this host"
    return 0
  fi

  local tmp_dir
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' RETURN

  mkdir -p "$tmp_dir/home/Downloads" "$tmp_dir/vm"
  touch "$tmp_dir/home/Downloads/Win11_test.iso"
  touch "$tmp_dir/OVMF_CODE.fd" "$tmp_dir/OVMF_VARS.fd"

  local env_file="$tmp_dir/windows-vm.env"
  cat >"$env_file" <<EOF
VM_NAME="test-win11"
VM_DIR="$tmp_dir/vm"
SECURE_BOOT="0"
OVMF_CODE="$tmp_dir/OVMF_CODE.fd"
OVMF_VARS_TEMPLATE="$tmp_dir/OVMF_VARS.fd"
WINDOWS_ISO="$tmp_dir/home/Downloads/Win11_test.iso"
EOF

  local output
  output="$(
    HOME="$tmp_dir/home" \
    WINDOWS_VM_ENV="$env_file" \
    "$REPO_ROOT/scripts/windows-vm-run.sh" --install --print-config
  )"

  grep -Fq "VirtIO ISO:     not attached" <<<"$output" \
    || fail "missing optional VirtIO ISO should not be attached; output was: $output"
}

test_windows_11_secure_boot_firmware_is_preferred() {
  local tmp_dir
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' RETURN

  mkdir -p "$tmp_dir/home/Downloads" "$tmp_dir/vm"
  touch "$tmp_dir/home/Downloads/Win11_test.iso"
  touch "$tmp_dir/OVMF_CODE.secboot.fd" "$tmp_dir/OVMF_VARS.fd"

  local env_file="$tmp_dir/windows-vm.env"
  cat >"$env_file" <<EOF
VM_NAME="test-win11"
VM_DIR="$tmp_dir/vm"
OVMF_SECURE_CODE="$tmp_dir/OVMF_CODE.secboot.fd"
OVMF_VARS_TEMPLATE="$tmp_dir/OVMF_VARS.fd"
WINDOWS_ISO="$tmp_dir/home/Downloads/Win11_test.iso"
EOF

  local output
  output="$(
    HOME="$tmp_dir/home" \
    WINDOWS_VM_ENV="$env_file" \
    "$REPO_ROOT/scripts/windows-vm-run.sh" --install --print-config
  )"

  grep -Fq "Secure Boot:    enabled" <<<"$output" \
    || fail "Windows 11 VM should enable Secure Boot by default; output was: $output"
  grep -Fq "OVMF code:      $tmp_dir/OVMF_CODE.secboot.fd" <<<"$output" \
    || fail "Windows 11 VM should use configured secure OVMF code; output was: $output"
}

test_default_display_resolution_is_1080p() {
  local tmp_dir
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' RETURN

  mkdir -p "$tmp_dir/home/Downloads" "$tmp_dir/vm"
  touch "$tmp_dir/OVMF_CODE.fd" "$tmp_dir/OVMF_VARS.fd"

  local env_file="$tmp_dir/windows-vm.env"
  cat >"$env_file" <<EOF
VM_NAME="test-win11"
VM_DIR="$tmp_dir/vm"
SECURE_BOOT="0"
OVMF_CODE="$tmp_dir/OVMF_CODE.fd"
OVMF_VARS_TEMPLATE="$tmp_dir/OVMF_VARS.fd"
EOF

  local output
  output="$(
    HOME="$tmp_dir/home" \
    WINDOWS_VM_ENV="$env_file" \
    "$REPO_ROOT/scripts/windows-vm-run.sh" --print-config
  )"

  grep -Fq "Resolution:     1920x1080" <<<"$output" \
    || fail "Windows VM should default to 1920x1080 display resolution; output was: $output"
  grep -Fq "Video device:   VGA" <<<"$output" \
    || fail "Windows VM should use an explicit VGA device for display mode hints; output was: $output"
  grep -Fq "Display:        gtk,zoom-to-fit=on" <<<"$output" \
    || fail "Windows VM should scale the GTK window to fit the guest display; output was: $output"
}

test_windows_icon_file_exists_for_tauri_build() {
  local icon_path="$REPO_ROOT/app/src-tauri/icons/icon.ico"

  [[ -s "$icon_path" ]] \
    || fail "Tauri Windows builds require a non-empty icon file at $icon_path"
}

test_tauri_windows_release_uses_gui_subsystem() {
  local main_rs="$REPO_ROOT/app/src-tauri/src/main.rs"

  grep -Fq 'windows_subsystem = "windows"' "$main_rs" \
    || fail "Tauri Windows release builds should use the GUI subsystem to avoid a console window"
}

test_missing_optional_virtio_iso_is_not_attached
test_windows_11_secure_boot_firmware_is_preferred
test_default_display_resolution_is_1080p
test_windows_icon_file_exists_for_tauri_build
test_tauri_windows_release_uses_gui_subsystem
