# Building Sanctum

## Table of Contents
- [Linux (recommended)](#linux-recommended)
  - [Option A: Nix (preferred)](#option-a-nix-preferred)
  - [Option B: Manual dependencies (no Nix)](#option-b-manual-dependencies-no-nix)
- [macOS](#macos)
- [Windows](#windows)

Sanctum ships with a reproducible Nix environment, but you can also build manually on Linux, macOS, and Windows. Each section below lists the required system dependencies and how to validate the workspace.

## Linux (recommended)

### Option A: Nix (preferred)
1. Install Nix and (optionally) Direnv.
2. Clone the repo:
   ```bash
   git clone https://codeberg.org/Kyronix/Sanctum.git
   cd Sanctum
   ```
3. Enter the shell:
   - With direnv: `direnv allow`
   - Without direnv: `nix develop`
4. Validate inside the shell:
   ```bash
   # Or without --release
   nix develop -c cargo run --release 
   ```
   Note: run all cargo commands inside the Nix shell unless you installed the manual dependencies.

### Option B: Manual dependencies (no Nix)
General requirements: Rust (via `rustup`), `pkg-config`, `cmake`, `ninja`, OpenSSL dev headers, SQLCipher dev headers, SQLite dev headers, fontconfig, X11/Wayland + GL/EGL headers.

Install per distro:

- **Debian/Ubuntu**
  ```bash
  sudo apt update
  sudo apt install -y build-essential pkg-config clang cmake ninja-build \
      libssl-dev libsqlcipher-dev libsqlite3-dev fontconfig libfontconfig1-dev \
      libx11-dev libxext-dev libxi-dev libxrandr-dev libxcursor-dev \
      libxkbcommon-dev libwayland-dev libgl1-mesa-dev libegl1-mesa-dev
  ```

- **Fedora/RHEL**
  ```bash
  sudo dnf groupinstall -y "Development Tools"
  sudo dnf install -y pkg-config clang cmake ninja-build openssl-devel \
      sqlcipher-devel sqlite-devel fontconfig-devel libX11-devel libXext-devel \
      libXi-devel libXrandr-devel libXcursor-devel libxkbcommon-devel \
      wayland-devel mesa-libGL-devel mesa-libEGL-devel
  ```

- **Arch Linux**
  ```bash
  sudo pacman -S --needed base-devel pkgconf clang cmake ninja \
      openssl sqlcipher sqlite fontconfig libx11 libxext libxi libxrandr \
      libxcursor libxkbcommon wayland mesa
  ```

Then install Rust and validate:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable
rustup component add clippy
git clone https://codeberg.org/Kyronix/Sanctum.git
cd Sanctum

# Or without --release
cargo run --release
```

## macOS
Requirements: Xcode Command Line Tools, Homebrew.

Install dependencies:
```bash
xcode-select --install         # if not already installed
brew install rustup-init pkg-config cmake ninja openssl@3 sqlcipher sqlite fontconfig freetype
rustup-init -y
source $HOME/.cargo/env
rustup default stable
```

Hint: expose Homebrew OpenSSL/SQLCipher to pkg-config if needed:
```bash
export PKG_CONFIG_PATH="$(brew --prefix openssl@3)/lib/pkgconfig:$(brew --prefix sqlcipher)/lib/pkgconfig:$PKG_CONFIG_PATH"
```

Validate:
```bash
git clone https://codeberg.org/Kyronix/Sanctum.git
cd Sanctum

# Or without --release
cargo run --release
```

## Windows

### Option A: WSL (recommended)
Use an Ubuntu/Fedora/Arch WSL distro and follow the Linux manual steps above (apt/dnf/pacman). Enter `wsl` and validate with `cargo check`, `cargo clippy`, and `cargo test`.

### Option B: Native Windows (MSVC toolchain)
1. Install **Visual Studio Build Tools** with the “Desktop development with C++” workload.
2. Install Git and Rust (MSVC target) via [rustup](https://rustup.rs/).
3. Install build tooling and pkg-config (example with Chocolatey):
   ```powershell
   choco install -y git cmake ninja pkgconfiglite nasm
   ```
4. Install OpenSSL and SQLCipher (via vcpkg, for example):
   ```powershell
   git clone https://github.com/microsoft/vcpkg.git C:\vcpkg
   cd C:\vcpkg
   .\bootstrap-vcpkg.bat
   .\vcpkg install openssl-windows sqlcipher
   setx VCPKG_ROOT C:\vcpkg
   setx PKG_CONFIG_PATH C:\vcpkg\installed\x64-windows\lib\pkgconfig
   ```
5. Open an “x64 Native Tools Command Prompt for VS”, then:
   ```powershell
   git clone https://codeberg.org/Kyronix/Sanctum.git
   cd Sanctum

   # Or without --release
   cargo run --release
   ```

If pkg-config cannot find OpenSSL/SQLCipher, ensure `PKG_CONFIG_PATH` points to their `lib/pkgconfig` directory and that you are using the MSVC developer prompt.
