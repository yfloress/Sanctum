# Building Sanctum

## Table of Contents
- [Prerequisites](#prerequisites)
- [Getting the Source Code](#getting-the-source-code)
- [Linux](#linux)
  - [Option A: Nix (Recommended)](#option-a-nix-recommended)
  - [Option B: Manual Setup](#option-b-manual-setup)
- [macOS](#macos)
- [Windows](#windows)

> **All commands are run from the repository root**, not from `src-tauri/`.

Sanctum is a **Tauri 2 + Svelte 5** desktop app. The build requires both
a Rust toolchain (for the core + Tauri shell) and a Node.js/pnpm setup
(for the Svelte frontend).

---

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | stable ≥ 1.77 | via [rustup](https://rustup.rs/) |
| Node.js | ≥ 20 LTS | |
| pnpm | ≥ 9 | `npm install -g pnpm` |
| Tauri CLI | 2.x | installed via cargo (see below) |

Install the Tauri CLI once:
```bash
cargo install tauri-cli --version "^2"
```

---

## Getting the Source Code

```bash
git clone https://codeberg.org/Kyronix/Sanctum.git
cd Sanctum
```

Install frontend dependencies (first time only):
```bash
cd ui-svelte && pnpm install && cd ..
```

---

## Linux

### Option A: Nix (Recommended)

Nix handles all system libraries automatically.

1. [Install Nix](https://nixos.org/download) with flakes enabled.
2. From the repository root:
   ```bash
   direnv allow   # or: nix develop
   cd ui-svelte && pnpm install && cd ..
   cargo tauri dev      # development
   cargo tauri build    # production binary
   ```

### Option B: Manual Setup

Install the WebKit/GTK system libraries required by Tauri:

**Debian/Ubuntu**
```bash
sudo apt update
sudo apt install -y build-essential pkg-config \
    libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev librsvg2-dev \
    libdbus-1-dev libxdo-dev
```

**Fedora**
```bash
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y pkg-config openssl-devel \
    gtk3-devel webkit2gtk4.1-devel \
    libappindicator-gtk3-devel librsvg2-devel \
    dbus-devel xdotool-devel
```

**Arch Linux**
```bash
sudo pacman -S --needed base-devel pkgconf openssl \
    gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg \
    dbus xdotool
```

Then run:
```bash
cargo tauri dev      # development (hot-reload)
cargo tauri build    # production binary → src-tauri/target/release/bundle/
```

---

## macOS

**Requirements:** Xcode Command Line Tools, Homebrew, Node.js, pnpm.

```bash
xcode-select --install
brew install rustup-init node
npm install -g pnpm
rustup-init -y && source $HOME/.cargo/env
cargo install tauri-cli --version "^2"
```

Then from the repository root:
```bash
cd ui-svelte && pnpm install && cd ..
cargo tauri dev      # development
cargo tauri build    # production .app bundle
```

---

## Windows

### Option A: WSL (Recommended)

Requires Windows 10/11 with WSLg enabled.

1. Install Ubuntu or Fedora from the Microsoft Store.
2. Follow the **Linux (Option B)** instructions inside the WSL terminal.

### Option B: Native Windows

1. Install **Visual Studio Build Tools** with the
   "Desktop development with C++" workload.
2. Install **Rust (MSVC target)** via [rustup.rs](https://rustup.rs/).
3. Install **Node.js** (LTS) and **pnpm**:
   ```powershell
   npm install -g pnpm
   ```
4. Install **WebView2** (pre-installed on Windows 11; download from
   Microsoft for Windows 10).
5. Install the Tauri CLI:
   ```powershell
   cargo install tauri-cli --version "^2"
   ```
6. From the repository root:
   ```powershell
   cd ui-svelte; pnpm install; cd ..
   cargo tauri dev      # development
   cargo tauri build    # production installer
   ```
