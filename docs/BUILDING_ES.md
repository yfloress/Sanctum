# Compilación de Sanctum

## Índice
- [1) Linux (recomendado)](#1-linux-recomendado)
  - [Opción A: Nix (preferida)](#opción-a-nix-preferida)
  - [Opción B: Dependencias manuales (sin Nix)](#opción-b-dependencias-manuales-sin-nix)
- [2) macOS](#2-macos)
- [3) Windows](#3-windows)

Sanctum incluye un entorno reproducible con Nix, pero también puedes compilarlo manualmente en Linux, macOS y Windows. A continuación se listan dependencias y pasos por plataforma.

## 1) Linux (recomendado)

### Opción A: Nix (preferida)
1. Instala Nix y (opcionalmente) Direnv.
2. Clona el repo:
   ```bash
   git clone https://codeberg.org/Kyronix/Sanctum.git
   cd Sanctum
   ```
3. Entra al shell:
   - Con direnv: `direnv allow`
   - Sin direnv: `nix develop`
4. Compila/ejecuta dentro del shell:
   ```bash
   cargo build --release   # o: cargo run
   ```

### Opción B: Dependencias manuales (sin Nix)
Requisitos generales: Rust (`rustup`), `pkg-config`, `cmake`, `ninja`, headers de OpenSSL, SQLCipher y SQLite, fontconfig, X11/Wayland + GL/EGL.

Instala por distro:

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

Luego instala Rust y compila:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable
rustup component add clippy
git clone https://codeberg.org/Kyronix/Sanctum.git
cd Sanctum
cargo build --release   # o: cargo run
```

## 2) macOS
Requisitos: Xcode Command Line Tools y Homebrew.

Dependencias:
```bash
xcode-select --install         # si no está instalado
brew install rustup-init pkg-config cmake ninja openssl@3 sqlcipher sqlite fontconfig freetype
rustup-init -y
source $HOME/.cargo/env
rustup default stable
```

Sugerencia: expón OpenSSL/SQLCipher de Homebrew a pkg-config si hace falta:
```bash
export PKG_CONFIG_PATH="$(brew --prefix openssl@3)/lib/pkgconfig:$(brew --prefix sqlcipher)/lib/pkgconfig:$PKG_CONFIG_PATH"
```

Compila:
```bash
git clone https://codeberg.org/Kyronix/Sanctum.git
cd Sanctum
cargo build --release   # o: cargo run
```

## 3) Windows

### Opción A: WSL (recomendada)
Usa una distro Ubuntu/Fedora/Arch en WSL y sigue los pasos manuales de Linux (apt/dnf/pacman). Entra a `wsl` y compila con `cargo build --release`.

### Opción B: Windows nativo (toolchain MSVC)
1. Instala **Visual Studio Build Tools** con el workload “Desktop development with C++”.
2. Instala Git y Rust (objetivo MSVC) vía [rustup](https://rustup.rs/).
3. Instala herramientas de build y pkg-config (ejemplo con Chocolatey):
   ```powershell
   choco install -y git cmake ninja pkgconfiglite nasm
   ```
4. Instala OpenSSL y SQLCipher (por ejemplo con vcpkg):
   ```powershell
   git clone https://github.com/microsoft/vcpkg.git C:\vcpkg
   cd C:\vcpkg
   .\bootstrap-vcpkg.bat
   .\vcpkg install openssl-windows sqlcipher
   setx VCPKG_ROOT C:\vcpkg
   setx PKG_CONFIG_PATH C:\vcpkg\installed\x64-windows\lib\pkgconfig
   ```
5. Abre un “x64 Native Tools Command Prompt for VS” y compila:
   ```powershell
   git clone https://codeberg.org/Kyronix/Sanctum.git
   cd Sanctum
   cargo build --release   # o: cargo run
   ```

Si pkg-config no encuentra OpenSSL/SQLCipher, verifica que `PKG_CONFIG_PATH` apunta a `lib/pkgconfig` y que usas la consola de desarrollador de MSVC.
