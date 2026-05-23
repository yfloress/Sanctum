# Installing Sanctum

Sanctum is a **Tauri 2 + Svelte 5** desktop app (Rust core + Svelte frontend).
The repository ships an [`install.sh`](../install.sh) that **builds from source**
and installs Sanctum on your system.

> **All commands are run from the repository root**, not from `src-tauri/`.

## Table of Contents
- [Quick Install](#quick-install)
- [What the Installer Does](#what-the-installer-does)
- [Prerequisites](#prerequisites)
  - [Option A: Nix (Linux, recommended)](#option-a-nix-linux-recommended)
  - [Option B: Manual Toolchain](#option-b-manual-toolchain)
- [Using the Installer](#using-the-installer)
- [Uninstalling](#uninstalling)
- [Manual Build (Development / Windows)](#manual-build-development--windows)
- [Where Your Data Lives](#where-your-data-lives)

---

## Quick Install

```bash
git clone https://codeberg.org/Kyronix/Sanctum.git
cd Sanctum
./install.sh --user      # builds + installs to ~/.local, no sudo
```

The installer builds Sanctum (frontend + release binary) from source, so it needs
a working build toolchain — see [Prerequisites](#prerequisites). If the binary is
already up to date, the build step is skipped automatically.

---

## What the Installer Does

The installer adapts to your platform:

| Platform | What gets installed |
|----------|---------------------|
| **Linux** | Binary + desktop entry + icon (and refreshes the desktop/icon caches) |
| **macOS** | **Binary only** — macOS has no XDG desktop entry or hicolor icon theme |
| **Windows** | Not supported by the installer — [build manually](#manual-build-development--windows) or use WSL |

On macOS the script prints a notice that only the binary will be installed.

---

## Prerequisites

The installer compiles from source via `cargo tauri build`, so you need **either**
Nix **or** the manual Rust + Tauri toolchain.

### Option A: Nix (Linux, recommended)

Nix handles all system libraries automatically. The installer auto-detects a
`flake.nix` and runs the build inside `nix develop`.

1. [Install Nix](https://nixos.org/download) with flakes enabled.
2. Install the frontend dependencies once:
   ```bash
   cd ui-svelte && pnpm install && cd ..
   ```
3. Run the installer (see [Using the Installer](#using-the-installer)).

### Option B: Manual Toolchain

| Tool | Version | Notes |
|------|---------|-------|
| Rust | stable ≥ 1.77 | via [rustup](https://rustup.rs/) |
| Node.js | ≥ 20 LTS | |
| pnpm | ≥ 9 | `npm install -g pnpm` |
| Tauri CLI | 2.x | `cargo install tauri-cli --version "^2"` |

Install the frontend dependencies once (required before the first build):
```bash
cd ui-svelte && pnpm install && cd ..
```

**Linux system libraries** (not needed if you use Nix):

<details>
<summary>Debian / Ubuntu</summary>

```bash
sudo apt update
sudo apt install -y build-essential pkg-config \
    libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev librsvg2-dev \
    libdbus-1-dev libxdo-dev
```
</details>

<details>
<summary>Fedora</summary>

```bash
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y pkg-config openssl-devel \
    gtk3-devel webkit2gtk4.1-devel \
    libappindicator-gtk3-devel librsvg2-devel \
    dbus-devel xdotool-devel
```
</details>

<details>
<summary>Arch Linux</summary>

```bash
sudo pacman -S --needed base-devel pkgconf openssl \
    gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg \
    dbus xdotool
```
</details>

**macOS:**
```bash
xcode-select --install
brew install rustup-init node
npm install -g pnpm
rustup-init -y && source "$HOME/.cargo/env"
cargo install tauri-cli --version "^2"
```

---

## Using the Installer

```bash
./install.sh --user      # user-local (~/.local), no sudo  ← recommended
sudo ./install.sh        # system-wide (/usr/local)
```

**Options:**

| Flag | Effect |
|------|--------|
| `--user` | Install under `~/.local` instead of system-wide (no sudo) |
| `--rebuild` | Force a rebuild even if the binary looks up to date |
| `--no-build` | Install an already-built binary (skip the build step) |
| `--uninstall` | Remove an existing install (see [Uninstalling](#uninstalling)) |
| `-h`, `--help` | Print usage |

**Environment variables:**

| Variable | Purpose |
|----------|---------|
| `PREFIX` | Install prefix (default `/usr/local`, or `~/.local` with `--user`) |
| `DESTDIR` | Staging root for packaging (prepended to `PREFIX`) |

```bash
PREFIX=/usr sudo ./install.sh      # custom prefix
```

**Install locations** (Linux, relative to `PREFIX`):

| File | Path |
|------|------|
| Binary | `bin/sanctum` |
| Desktop entry | `share/applications/sanctum.desktop` |
| Icon | `share/icons/hicolor/512x512/apps/sanctum.png` |

> **PATH note:** with `--user`, make sure `~/.local/bin` is on your `PATH` so the
> `sanctum` command is found. The installer warns you if it isn't.

---

## Uninstalling

```bash
./install.sh --user --uninstall      # remove a user-local install
sudo ./install.sh --uninstall        # remove a system-wide install
```

This removes the binary (and, on Linux, the desktop entry and icon). **Your vault
data is left untouched** — see [Where Your Data Lives](#where-your-data-lives).

---

## Manual Build (Development / Windows)

If you prefer not to use the installer — for development, or on Windows — build
directly with the Tauri CLI:

```bash
cd ui-svelte && pnpm install && cd ..   # first time only
cargo tauri dev      # development, with hot-reload
cargo tauri build    # production binary → target/release/sanctum-tauri
```

### Windows

**Option A — WSL (recommended).** Install Ubuntu or Fedora from the Microsoft
Store (Windows 10/11 with WSLg), then follow the Linux
[manual toolchain](#option-b-manual-toolchain) steps inside the WSL terminal.

**Option B — Native Windows.**
1. Install **Visual Studio Build Tools** with the "Desktop development with C++" workload.
2. Install **Rust (MSVC target)** via [rustup.rs](https://rustup.rs/).
3. Install **Node.js** (LTS) and **pnpm** (`npm install -g pnpm`).
4. Install **WebView2** (pre-installed on Windows 11; download from Microsoft for Windows 10).
5. Install the Tauri CLI: `cargo install tauri-cli --version "^2"`.
6. From the repository root:
   ```powershell
   cd ui-svelte; pnpm install; cd ..
   cargo tauri dev      # development
   cargo tauri build    # production installer
   ```

> The `install.sh` script targets Linux and macOS only. On Windows, use the
> manual build above (or WSL) to produce and run the binary.

---

## Where Your Data Lives

Your encrypted vault database is stored in your operating system's per-user
configuration directory, **separate from the installed binary**. Uninstalling
Sanctum never touches it — to fully remove Sanctum you must delete that data
yourself.
