# Building Sanctum

## Table of Contents
- [Getting the Source Code](#getting-the-source-code)
- [Linux](#linux)
  - [Option A: Nix (Recommended)](#option-a-nix-recommended)
  - [Option B: Manual Setup](#option-b-manual-setup)
- [macOS](#macos)
- [Windows](#windows)

Sanctum includes a reproducible Nix environment, but you can also build it manually on your preferred operating system.

## Getting the Source Code
First, download the repository to your machine:
```bash
git clone https://codeberg.org/Kyronix/Sanctum.git
cd Sanctum
```

## Linux

### Option A: Nix (Recommended)
This is the easiest way to run Sanctum without worrying about system libraries.
1. Install Nix.
2. From the project folder, simply run:
   ```bash
   nix develop -c cargo run --release
   ```

### Option B: Manual Setup
If you prefer not to use Nix, you must install the required system libraries first.

**Debian/Ubuntu**
```bash
sudo apt update
sudo apt install -y build-essential pkg-config cmake ninja-build \
    libssl-dev libfontconfig1-dev libfreetype-dev libharfbuzz-dev \
    libx11-dev libxext-dev libxi-dev libxrandr-dev libxcursor-dev \
    libxkbcommon-dev libwayland-dev libgl1-mesa-dev libegl1-mesa-dev
```

**Fedora**
```bash
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y pkg-config cmake ninja-build openssl-devel \
    fontconfig-devel freetype-devel harfbuzz-devel libX11-devel libXext-devel \
    libXi-devel libXrandr-devel libXcursor-devel libxkbcommon-devel \
    wayland-devel mesa-libGL-devel mesa-libEGL-devel
```

**Arch Linux**
```bash
sudo pacman -S --needed base-devel pkgconf cmake ninja \
    openssl fontconfig freetype2 harfbuzz libx11 libxext libxi libxrandr \
    libxcursor libxkbcommon wayland mesa
```

**Run the App:**
Ensure you have [Rust installed](https://rustup.rs/), then run:
```bash
cargo run --release
```

## macOS
**Requirements:** Xcode Command Line Tools and Homebrew.

1. Install system dependencies:
   ```bash
   xcode-select --install
   brew install rustup-init pkg-config cmake ninja openssl@3 fontconfig freetype harfbuzz
   ```

2. Initialize Rust (if not already installed):
   ```bash
   rustup-init -y
   source $HOME/.cargo/env
   ```

3. Run the App:
   ```bash
   cargo run --release
   ```

## Windows

### Option A: WSL (Windows Subsystem for Linux)
Requires Windows 10 or 11 with WSLg (GUI support) enabled.
1. Install a distro like Ubuntu or Fedora from the Microsoft Store.
2. Follow the **Linux (Option B: Manual Setup)** instructions above for your chosen distro.
3. Run `cargo run --release` inside the WSL terminal.

### Option B: Native Windows
1. Install **Visual Studio Build Tools** (select the "Desktop development with C++" workload).
2. Install **Rust** (MSVC) via [rustup.rs](https://rustup.rs/).
3. Install **Git**, **CMake**, and **Ninja** (using a package manager like Chocolatey is recommended):
   ```powershell
   choco install -y git cmake ninja
   ```
4. **Run the App:**
   Open the "x64 Native Tools Command Prompt for VS", navigate to the Sanctum folder, and run:
   ```powershell
   cargo run --release
   ```
