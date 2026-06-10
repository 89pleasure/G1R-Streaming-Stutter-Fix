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

test_missing_optional_virtio_iso_is_not_attached
test_windows_11_secure_boot_firmware_is_preferred
