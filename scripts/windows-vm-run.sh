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
MEMORY_MB="${MEMORY_MB:-8192}"
CPUS="${CPUS:-4}"
ACCEL="${ACCEL:-kvm}"
CPU_MODEL="${CPU_MODEL:-}"
SECURE_BOOT="${SECURE_BOOT:-1}"
WINDOWS_ISO="${WINDOWS_ISO:-}"
VIRTIO_ISO="${VIRTIO_ISO:-}"
DISK_BUS="${DISK_BUS:-sata}"
NETWORK_DEVICE="${NETWORK_DEVICE:-e1000e}"
ENABLE_TPM="${ENABLE_TPM:-auto}"
SHARE_REPO="${SHARE_REPO:-1}"
SHARE_DIR="${SHARE_DIR:-}"
DISPLAY_BACKEND="${DISPLAY_BACKEND:-gtk}"
OVMF_CODE="${OVMF_CODE:-}"
OVMF_SECURE_CODE="${OVMF_SECURE_CODE:-}"
OVMF_VARS_TEMPLATE="${OVMF_VARS_TEMPLATE:-}"

usage() {
  cat <<'EOF'
Usage: scripts/windows-vm-run.sh [--install] [--print-config] [--help]

Starts the local Windows QEMU VM.

Modes:
  --install       Attach the Windows ISO and boot the installer once.
  --print-config  Print resolved VM settings without starting QEMU.

Configuration:
  Copy scripts/windows-vm.env.example to scripts/windows-vm.env, then adjust
  local paths and runtime settings.
EOF
}

install_mode=0
print_config=0
while (($#)); do
  case "$1" in
    --install)
      install_mode=1
      ;;
    --print-config)
      print_config=1
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

detect_windows_iso() {
  if [[ -n "$WINDOWS_ISO" ]]; then
    [[ -f "$WINDOWS_ISO" ]] || die "WINDOWS_ISO does not exist: $WINDOWS_ISO"
    printf '%s\n' "$WINDOWS_ISO"
    return 0
  fi

  shopt -s nullglob
  local candidates=(
    "$HOME"/Downloads/*Win*11*.iso
    "$HOME"/Downloads/*Windows*11*.iso
    "$HOME"/Downloads/*win*11*.iso
    "$HOME"/Downloads/*windows*11*.iso
  )
  shopt -u nullglob

  first_existing_file "${candidates[@]}"
}

detect_virtio_iso() {
  if [[ -n "$VIRTIO_ISO" ]]; then
    [[ -f "$VIRTIO_ISO" ]] || die "VIRTIO_ISO does not exist: $VIRTIO_ISO"
    printf '%s\n' "$VIRTIO_ISO"
    return 0
  fi

  shopt -s nullglob
  local candidates=(
    "$HOME"/Downloads/virtio-win*.iso
    /var/lib/libvirt/images/virtio-win*.iso
    /usr/share/virtio-win/virtio-win.iso
  )
  shopt -u nullglob

  first_existing_file "${candidates[@]}"
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

wait_for_socket() {
  local socket_path="$1"
  local attempt
  for attempt in {1..50}; do
    [[ -S "$socket_path" ]] && return 0
    sleep 0.1
  done
  return 1
}

VM_DIR_ABS="$(resolve_path "$VM_DIR")"
DISK_PATH="$VM_DIR_ABS/$VM_NAME.qcow2"
UEFI_VARS_PATH="$VM_DIR_ABS/${VM_NAME}-OVMF_VARS.fd"
TPM_DIR="$VM_DIR_ABS/swtpm"
TPM_SOCKET="$TPM_DIR/swtpm.sock"
SHARE_DIR_ABS="$(resolve_path "${SHARE_DIR:-$REPO_ROOT}")"

windows_iso=""
if ((install_mode)); then
  windows_iso="$(detect_windows_iso)" || die "could not find a Windows 11 ISO. Set WINDOWS_ISO in $ENV_FILE"
fi

virtio_iso=""
if virtio_iso="$(detect_virtio_iso)"; then
  :
elif [[ "$DISK_BUS" == "virtio" ]]; then
  die "DISK_BUS=virtio needs a VirtIO driver ISO. Set VIRTIO_ISO in $ENV_FILE"
fi

ovmf_code="$(detect_ovmf_code)" || die "could not find OVMF_CODE. Install OVMF/edk2-ovmf or set OVMF_CODE in $ENV_FILE"
ovmf_vars_template="$(detect_ovmf_vars_template)" || die "could not find OVMF_VARS. Install OVMF/edk2-ovmf or set OVMF_VARS_TEMPLATE in $ENV_FILE"

case "$DISK_BUS" in
  sata | virtio) ;;
  *) die "DISK_BUS must be sata or virtio, got: $DISK_BUS" ;;
esac

case "$ACCEL" in
  kvm | tcg) ;;
  *) die "ACCEL must be kvm or tcg, got: $ACCEL" ;;
esac

secure_boot_enabled=0
if is_enabled "$SECURE_BOOT"; then
  secure_boot_enabled=1
fi

if [[ -z "$CPU_MODEL" ]]; then
  if [[ "$ACCEL" == "kvm" ]]; then
    CPU_MODEL="host"
  else
    CPU_MODEL="max"
  fi
fi

if ((print_config)); then
  cat <<EOF
VM name:        $VM_NAME
VM directory:   $VM_DIR_ABS
Disk path:      $DISK_PATH
Memory MB:      $MEMORY_MB
CPUs:           $CPUS
Accelerator:    $ACCEL
CPU model:      $CPU_MODEL
Secure Boot:    $([[ "$secure_boot_enabled" == "1" ]] && echo enabled || echo disabled)
Disk bus:       $DISK_BUS
Network device: $NETWORK_DEVICE
Display:        $DISPLAY_BACKEND
Install mode:   $install_mode
Windows ISO:    ${windows_iso:-not attached}
VirtIO ISO:     ${virtio_iso:-not attached}
OVMF code:      $ovmf_code
OVMF vars:      $UEFI_VARS_PATH
TPM:            $ENABLE_TPM
Share repo:     $SHARE_REPO
Share dir:      $SHARE_DIR_ABS
EOF
  exit 0
fi

command -v qemu-system-x86_64 >/dev/null 2>&1 || die "missing required command: qemu-system-x86_64"

[[ -f "$DISK_PATH" ]] || die "missing VM disk: $DISK_PATH. Run ./scripts/windows-vm-create.sh first"

mkdir -p "$VM_DIR_ABS"
if [[ ! -f "$UEFI_VARS_PATH" ]]; then
  cp "$ovmf_vars_template" "$UEFI_VARS_PATH"
fi

if [[ "$ACCEL" == "kvm" && (! -r /dev/kvm || ! -w /dev/kvm) ]]; then
  die "no read/write access to /dev/kvm. Add your user to the kvm group or set ACCEL=tcg"
fi

machine_options="q35,accel=$ACCEL"
if [[ "$secure_boot_enabled" == "1" ]]; then
  machine_options+=",smm=on"
fi

qemu_args=(
  -name "$VM_NAME"
  -machine "$machine_options"
  -cpu "$CPU_MODEL"
  -smp "$CPUS"
  -m "$MEMORY_MB"
  -rtc base=localtime,clock=host
  -display "$DISPLAY_BACKEND"
  -device qemu-xhci
  -device usb-tablet
  -device ich9-ahci,id=ahci
  -drive "if=pflash,format=raw,readonly=on,file=$ovmf_code"
  -drive "if=pflash,format=raw,file=$UEFI_VARS_PATH"
)

if [[ "$secure_boot_enabled" == "1" ]]; then
  qemu_args+=(-global driver=cfi.pflash01,property=secure,value=on)
fi

if [[ "$DISK_BUS" == "virtio" ]]; then
  qemu_args+=(
    -drive "if=none,id=system,format=qcow2,file=$DISK_PATH,cache=writeback"
    -device virtio-blk-pci,drive=system
  )
else
  qemu_args+=(
    -drive "if=none,id=system,format=qcow2,file=$DISK_PATH,cache=writeback"
    -device ide-hd,drive=system,bus=ahci.0
  )
fi

if ((install_mode)); then
  qemu_args+=(
    -drive "if=none,id=windows_iso,media=cdrom,readonly=on,file=$windows_iso"
    -device ide-cd,drive=windows_iso,bus=ahci.1
    -boot once=d,menu=on
  )
fi

if [[ -n "$virtio_iso" ]]; then
  qemu_args+=(
    -drive "if=none,id=virtio_iso,media=cdrom,readonly=on,file=$virtio_iso"
    -device ide-cd,drive=virtio_iso,bus=ahci.2
  )
fi

netdev="user,id=net0"
if [[ "$SHARE_REPO" == "1" ]]; then
  if command -v smbd >/dev/null 2>&1; then
    netdev+=",smb=$SHARE_DIR_ABS"
  else
    warn "smbd is not installed; skipping QEMU SMB share"
  fi
fi
qemu_args+=(-netdev "$netdev" -device "$NETWORK_DEVICE,netdev=net0")

swtpm_pid=""
if [[ "$ENABLE_TPM" == "1" || "$ENABLE_TPM" == "auto" ]]; then
  if command -v swtpm >/dev/null 2>&1; then
    mkdir -p "$TPM_DIR"
    rm -f "$TPM_SOCKET"
    swtpm socket \
      --tpm2 \
      --tpmstate "dir=$TPM_DIR" \
      --ctrl "type=unixio,path=$TPM_SOCKET" &
    swtpm_pid="$!"
    wait_for_socket "$TPM_SOCKET" || die "swtpm did not create its control socket"
    qemu_args+=(
      -chardev "socket,id=chrtpm,path=$TPM_SOCKET"
      -tpmdev emulator,id=tpm0,chardev=chrtpm
      -device tpm-tis,tpmdev=tpm0
    )
  elif [[ "$ENABLE_TPM" == "1" ]]; then
    die "ENABLE_TPM=1 but swtpm is not installed"
  else
    warn "swtpm is not installed; Windows 11 setup may reject this VM"
  fi
fi

cleanup() {
  if [[ -n "$swtpm_pid" ]]; then
    kill "$swtpm_pid" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

qemu-system-x86_64 "${qemu_args[@]}"
