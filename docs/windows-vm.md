# Windows VM Test Setup

This repo includes small QEMU/KVM helpers for testing the Tauri desktop app on a
real Windows runtime without committing VM images or ISO paths.

## Files

```text
scripts/windows-vm.env.example   Local VM configuration template
scripts/windows-vm-create.sh     One-time disk and UEFI vars setup
scripts/windows-vm-run.sh        Daily VM launcher
vm/                              Local VM images, ignored by git
```

Copy the example config before editing local paths:

```bash
cp scripts/windows-vm.env.example scripts/windows-vm.env
```

The run script auto-detects common Windows 11 ISO names in `$HOME/Downloads`.
You can also pin the ISO explicitly:

```bash
WINDOWS_ISO="$HOME/Downloads/Win11_25H2_German_x64.iso"
```

## Host Packages

Install QEMU, OVMF, and optional TPM/SMB helpers on the Linux host.

Ubuntu/Debian:

```bash
sudo apt install qemu-system-x86 qemu-utils ovmf swtpm samba
```

Arch:

```bash
sudo pacman -S qemu-full edk2-ovmf swtpm samba
```

Fedora:

```bash
sudo dnf install qemu-kvm qemu-img edk2-ovmf swtpm samba
```

On some Arch-based systems the OVMF files live under `/usr/share/edk2/x64/`;
the scripts detect that path automatically.

Make sure your user can access KVM. Most distros use the `kvm` group:

```bash
sudo usermod -aG kvm "$USER"
```

Log out and back in after changing group membership.

The default launcher uses `ACCEL=kvm`. You can set `ACCEL=tcg` in
`scripts/windows-vm.env` for hosts without KVM, but Windows will be much slower.

## Create the VM Storage

```bash
./scripts/windows-vm-create.sh
```

This creates:

```text
vm/windows/g1r-win11.qcow2
vm/windows/g1r-win11-OVMF_VARS.fd
```

Existing files are left untouched.

## Install Windows

Start the VM in install mode:

```bash
./scripts/windows-vm-run.sh --install
```

The default disk bus is `sata` so the Windows installer can see the disk without
extra drivers. If you want VirtIO storage, download `virtio-win.iso`, set
`VIRTIO_ISO` and `DISK_BUS=virtio` in `scripts/windows-vm.env`, then load the
storage driver from the VirtIO CD during Windows setup.

The script enables a TPM automatically when `swtpm` is installed. If Windows 11
setup rejects the VM, install `swtpm` or set `ENABLE_TPM=1` to make missing TPM
support a hard error before QEMU starts.

Secure Boot is enabled by default for Windows 11. On this host the script uses
the Secure-Boot-capable OVMF firmware when available, such as:

```text
/usr/share/edk2/x64/OVMF_CODE.secboot.4m.fd
```

If you are testing a Windows 10 image or intentionally want legacy behavior, set
`SECURE_BOOT=0` in `scripts/windows-vm.env`.

## Start the Installed VM

After Windows is installed, start it without setup media:

```bash
./scripts/windows-vm-run.sh
```

To inspect the resolved configuration without starting QEMU:

```bash
./scripts/windows-vm-run.sh --print-config
```

## Access This Repo from Windows

When `samba` is installed on the host and `SHARE_REPO=1`, QEMU exposes the repo
through user-mode SMB:

```text
\\10.0.2.4\qemu
```

Open that path in Windows Explorer to copy build artifacts or work from the
checked-out source. If `smbd` is missing, the VM still starts but the share is
skipped.

## Build and Test the App in Windows

The preferred test flow is to build the Windows installer in GitHub Actions and
use this VM only for installation and runtime testing.

1. Push this repo to GitHub.
2. Open the `Windows App Build` workflow.
3. Run it manually with `Run workflow`.
4. Download the `g1r-optimizer-windows-nsis` artifact from the completed run.
5. Copy the installer into the VM and run it there.

For a quick local transfer into the VM, serve the downloaded artifact directory
from the host:

```bash
python3 -m http.server 8000
```

Then open this URL in the Windows VM:

```text
http://10.0.2.2:8000/
```

If you intentionally want to build inside the Windows VM instead, install:

- Git
- Node.js LTS
- Rust stable with the MSVC toolchain
- Microsoft Visual Studio Build Tools with Desktop development for C++

Then open PowerShell in the repo and run:

```powershell
cd app
npm install
npm run dev
```

For a distributable Windows bundle:

```powershell
cd app
npm run build
```

The app should exercise the Windows-specific paths in `optimizer-core`, including
DXGI GPU detection and `%LOCALAPPDATA%\G1R\Saved\Config\Windows`.

## Notes

- VM images, ISOs, and `scripts/windows-vm.env` are ignored by git.
- Re-run `--install` only when you intentionally want to boot from the Windows
  ISO again.
- Keep a copy of `vm/windows/g1r-win11.qcow2` after a clean Windows setup if you
  want a quick restore point.
