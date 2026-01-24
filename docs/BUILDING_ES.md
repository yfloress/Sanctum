# Compilación de Sanctum

## Índice
- [Obtener el Código Fuente](#obtener-el-código-fuente)
- [Linux](#linux)
  - [Opción A: Nix (Recomendada)](#opción-a-nix-recomendada)
  - [Opción B: Instalación Manual](#opción-b-instalación-manual)
- [macOS](#macos)
- [Windows](#windows)

Sanctum incluye un entorno reproducible con Nix, pero también puedes compilarlo manualmente en tu sistema operativo preferido.

## Obtener el Código Fuente
Primero, descarga el repositorio en tu equipo:
```bash
git clone https://codeberg.org/Kyronix/Sanctum.git
cd Sanctum
```

## Linux

### Opción A: Nix (Recomendada)
Es la forma más fácil de ejecutar Sanctum sin preocuparse por librerías del sistema.
1. Instala Nix.
2. Desde la carpeta del proyecto, ejecuta:
   ```bash
   nix develop -c cargo run --release
   ```

### Opción B: Instalación Manual
Si prefieres no usar Nix, debes instalar primero las librerías del sistema necesarias.

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

**Ejecutar la App:**
Asegúrate de tener [Rust instalado](https://rustup.rs/) y luego ejecuta:
```bash
cargo run --release
```

## macOS
**Requisitos:** Xcode Command Line Tools y Homebrew.

1. Instala las dependencias del sistema:
   ```bash
   xcode-select --install
   brew install rustup-init pkg-config cmake ninja openssl@3 fontconfig freetype harfbuzz
   ```

2. Inicializa Rust (si no está instalado):
   ```bash
   rustup-init -y
   source $HOME/.cargo/env
   ```

3. Ejecuta la App:
   ```bash
   cargo run --release
   ```

## Windows

### Opción A: WSL (Subsistema de Windows para Linux)
Requiere Windows 10 u 11 con soporte para WSLg (interfaz gráfica) habilitado.
1. Instala una distribución como Ubuntu o Fedora desde la Microsoft Store.
2. Sigue las instrucciones de **Linux (Opción B: Instalación Manual)** de arriba para tu distribución elegida.
3. Ejecuta `cargo run --release` dentro de la terminal de WSL.

### Opción B: Windows Nativo
1. Instala **Visual Studio Build Tools** (selecciona la carga de trabajo "Desarrollo para el escritorio con C++").
2. Instala **Rust** (MSVC) vía [rustup.rs](https://rustup.rs/).
3. Instala **Git**, **CMake** y **Ninja** (se recomienda usar un gestor de paquetes como Chocolatey):
   ```powershell
   choco install -y git cmake ninja
   ```
4. **Ejecutar la App:**
Abre el "x64 Native Tools Command Prompt for VS", navega a la carpeta de Sanctum y ejecuta:
   ```powershell
   cargo run --release
   ```
