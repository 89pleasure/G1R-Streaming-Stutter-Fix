#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="${WINDOWS_VM_ENV:-$SCRIPT_DIR/windows-vm.env}"

if [[ -f "$ENV_FILE" ]]; then
  # shellcheck source=/dev/null
  source "$ENV_FILE"
fi

VM_NAME="${VM_NAME:-g1r-win11}"
VM_DIR="${VM_DIR:-vm/windows}"
DISK_SIZE="${DISK_SIZE:-80G}"
SECURE_BOOT="${SECURE_BOOT:-1}"
OVMF_CODE="${OVMF_CODE:-}"
OVMF_SECURE_CODE="${OVMF_SECURE_CODE:-}"
OVMF_VARS_TEMPLATE="${OVMF_VARS_TEMPLATE:-}"

usage() {
  cat <<'EOF'
Usage: scripts/windows-vm-create.sh [--dry-run] [--help]

Creates the local QCOW2 disk and per-VM UEFI vars file used by
scripts/windows-vm-run.sh. Existing files are not overwritten.

Configuration:
  Copy scripts/windows-vm.env.example to scripts/windows-vm.env, then adjust
  VM_NAME, VM_DIR, DISK_SIZE, OVMF_CODE, or OVMF_VARS_TEMPLATE if needed.
EOF
}

dry_run=0
while (($#)); do
  case "$1" in
    --dry-run)
      dry_run=1
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

die() {
  echo "error: $*" >&2
  exit 1
}

warn() {
  echo "warning: $*" >&2
}

require_command() {
  local command_name="$1"
  command -v "$command_name" >/dev/null 2>&1 || die "missing required command: $command_name"
}

resolve_path() {
  local value="$1"
  case "$value" in
    /*) printf '%s\n' "$value" ;;
    *) printf '%s/%s\n' "$REPO_ROOT" "$value" ;;
  esac
}

first_existing_file() {
  local path
  for path in "$@"; do
    if [[ -f "$path" ]]; then
      printf '%s\n' "$path"
      return 0
    fi
  done
  return 1
}

is_enabled() {
  case "$1" in
    1 | true | yes | on) return 0 ;;
    0 | false | no | off) return 1 ;;
    *) die "expected boolean value for SECURE_BOOT, got: $1" ;;
  esac
}

detect_ovmf_code() {
  if [[ -n "$OVMF_CODE" ]]; then
    [[ -f "$OVMF_CODE" ]] || die "OVMF_CODE does not exist: $OVMF_CODE"
    printf '%s\n' "$OVMF_CODE"
    return 0
  fi

  if is_enabled "$SECURE_BOOT"; then
    if [[ -n "$OVMF_SECURE_CODE" ]]; then
      [[ -f "$OVMF_SECURE_CODE" ]] || die "OVMF_SECURE_CODE does not exist: $OVMF_SECURE_CODE"
      printf '%s\n' "$OVMF_SECURE_CODE"
      return 0
    fi

    first_existing_file \
      /usr/share/edk2/x64/OVMF_CODE.secboot.4m.fd \
      /usr/share/edk2/x64/OVMF_CODE.secboot.fd \
      /usr/share/OVMF/OVMF_CODE_4M.secboot.fd \
      /usr/share/OVMF/OVMF_CODE.secboot.fd \
      /usr/share/edk2-ovmf/x64/OVMF_CODE.secboot.fd \
      /usr/share/edk2/ovmf/OVMF_CODE.secboot.fd \
      /usr/share/qemu/OVMF_CODE.secboot.fd
    return $?
  fi

  first_existing_file \
    /usr/share/edk2/x64/OVMF_CODE.4m.fd \
    /usr/share/edk2/x64/OVMF_CODE.fd \
    /usr/share/OVMF/OVMF_CODE_4M.fd \
    /usr/share/OVMF/OVMF_CODE.fd \
    /usr/share/edk2-ovmf/x64/OVMF_CODE.fd \
    /usr/share/edk2/ovmf/OVMF_CODE.fd \
    /usr/share/qemu/OVMF_CODE.fd
}

detect_ovmf_vars_template() {
  if [[ -n "$OVMF_VARS_TEMPLATE" ]]; then
    [[ -f "$OVMF_VARS_TEMPLATE" ]] || die "OVMF_VARS_TEMPLATE does not exist: $OVMF_VARS_TEMPLATE"
    printf '%s\n' "$OVMF_VARS_TEMPLATE"
    return 0
  fi

  first_existing_file \
    /usr/share/edk2/x64/OVMF_VARS.4m.fd \
    /usr/share/edk2/x64/OVMF_VARS.fd \
    /usr/share/OVMF/OVMF_VARS_4M.fd \
    /usr/share/OVMF/OVMF_VARS.fd \
    /usr/share/edk2-ovmf/x64/OVMF_VARS.fd \
    /usr/share/edk2/ovmf/OVMF_VARS.fd \
    /usr/share/qemu/OVMF_VARS.fd
}

VM_DIR_ABS="$(resolve_path "$VM_DIR")"
DISK_PATH="$VM_DIR_ABS/$VM_NAME.qcow2"
UEFI_VARS_PATH="$VM_DIR_ABS/${VM_NAME}-OVMF_VARS.fd"

ovmf_code="$(detect_ovmf_code)" || die "could not find OVMF_CODE. Install OVMF/edk2-ovmf or set OVMF_CODE in $ENV_FILE"
ovmf_vars_template="$(detect_ovmf_vars_template)" || die "could not find OVMF_VARS. Install OVMF/edk2-ovmf or set OVMF_VARS_TEMPLATE in $ENV_FILE"

echo "VM name:       $VM_NAME"
echo "VM directory:  $VM_DIR_ABS"
echo "Disk path:     $DISK_PATH"
echo "Disk size:     $DISK_SIZE"
if is_enabled "$SECURE_BOOT"; then
  echo "Secure Boot:  enabled"
else
  echo "Secure Boot:  disabled"
fi
echo "OVMF code:     $ovmf_code"
echo "OVMF vars:     $UEFI_VARS_PATH"

if ((dry_run)); then
  echo "Dry run only; no files changed."
  exit 0
fi

require_command qemu-img
require_command qemu-system-x86_64

mkdir -p "$VM_DIR_ABS"

if [[ -e "$DISK_PATH" ]]; then
  warn "disk already exists, leaving it unchanged: $DISK_PATH"
else
  qemu-img create -f qcow2 "$DISK_PATH" "$DISK_SIZE"
fi

if [[ -e "$UEFI_VARS_PATH" ]]; then
  warn "UEFI vars file already exists, leaving it unchanged: $UEFI_VARS_PATH"
else
  cp "$ovmf_vars_template" "$UEFI_VARS_PATH"
fi

cat <<EOF

VM storage is ready.

Install Windows:
  ./scripts/windows-vm-run.sh --install

Start the installed VM:
  ./scripts/windows-vm-run.sh
EOF
