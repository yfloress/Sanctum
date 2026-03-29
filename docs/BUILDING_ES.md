# Compilación de Sanctum

## Índice
- [Prerequisitos](#prerequisitos)
- [Obtener el Código Fuente](#obtener-el-código-fuente)
- [Linux](#linux)
  - [Opción A: Nix (Recomendada)](#opción-a-nix-recomendada)
  - [Opción B: Instalación Manual](#opción-b-instalación-manual)
- [macOS](#macos)
- [Windows](#windows)

> **Todos los comandos se ejecutan desde la raíz del repositorio**, no desde `src-tauri/`.

Sanctum es una app de escritorio **Tauri 2 + Svelte 5**. La compilación requiere
tanto un toolchain de Rust (para el núcleo + shell Tauri) como Node.js/pnpm
(para el frontend Svelte).

---

## Prerequisitos

| Herramienta | Versión | Notas |
|-------------|---------|-------|
| Rust | stable ≥ 1.77 | via [rustup](https://rustup.rs/) |
| Node.js | ≥ 20 LTS | |
| pnpm | ≥ 9 | `npm install -g pnpm` |
| Tauri CLI | 2.x | instalado via cargo (ver abajo) |

Instala el CLI de Tauri una vez:
```bash
cargo install tauri-cli --version "^2"
```

---

## Obtener el Código Fuente

```bash
git clone https://codeberg.org/Kyronix/Sanctum.git
cd Sanctum
```

Instala las dependencias del frontend (solo la primera vez):
```bash
cd ui-svelte && pnpm install && cd ..
```

---

## Linux

### Opción A: Nix (Recomendada)

Nix gestiona todas las librerías del sistema automáticamente.

1. [Instala Nix](https://nixos.org/download) con flakes habilitados.
2. Desde la raíz del repositorio:
   ```bash
   direnv allow   # o: nix develop
   cd ui-svelte && pnpm install && cd ..
   cargo tauri dev      # modo desarrollo
   cargo tauri build    # binario de producción
   ```

### Opción B: Instalación Manual

Instala las librerías de sistema WebKit/GTK que requiere Tauri:

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

Luego ejecuta:
```bash
cargo tauri dev      # desarrollo (hot-reload)
cargo tauri build    # binario → src-tauri/target/release/bundle/
```

---

## macOS

**Requisitos:** Xcode Command Line Tools, Homebrew, Node.js, pnpm.

```bash
xcode-select --install
brew install rustup-init node
npm install -g pnpm
rustup-init -y && source $HOME/.cargo/env
cargo install tauri-cli --version "^2"
```

Desde la raíz del repositorio:
```bash
cd ui-svelte && pnpm install && cd ..
cargo tauri dev      # desarrollo
cargo tauri build    # bundle .app de producción
```

---

## Windows

### Opción A: WSL (Recomendada)

Requiere Windows 10/11 con WSLg habilitado.

1. Instala Ubuntu o Fedora desde la Microsoft Store.
2. Sigue las instrucciones de **Linux (Opción B)** dentro de la terminal WSL.

### Opción B: Windows Nativo

1. Instala **Visual Studio Build Tools** con el módulo
   "Desarrollo para escritorio con C++".
2. Instala **Rust (target MSVC)** vía [rustup.rs](https://rustup.rs/).
3. Instala **Node.js** (LTS) y **pnpm**:
   ```powershell
   npm install -g pnpm
   ```
4. Instala **WebView2** (preinstalado en Windows 11; descárgalo desde
   Microsoft para Windows 10).
5. Instala el CLI de Tauri:
   ```powershell
   cargo install tauri-cli --version "^2"
   ```
6. Desde la raíz del repositorio:
   ```powershell
   cd ui-svelte; pnpm install; cd ..
   cargo tauri dev      # desarrollo
   cargo tauri build    # instalador de producción
   ```
