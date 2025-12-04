# INSTALL.md

> Instrucciones de instalación y compilación de Sanctum **sin Nix**.

---

## Requisitos Previos

| Dependencia | Versión Mínima | Propósito |
|:------------|:---------------|:----------|
| Rust | 1.70+ | Core del backend |
| Deno | 1.40+ | Build del frontend |
| Tauri CLI | 2.x | Compilación y empaquetado |

---

## Linux

### Ubuntu / Debian

```bash
# 1. Dependencias del sistema
sudo apt update
sudo apt install -y \
  build-essential curl wget git \
  libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
  librsvg2-dev libssl-dev pkg-config

# 2. Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 3. Deno
curl -fsSL https://deno.land/install.sh | sh
# Agregar a PATH según las instrucciones del instalador

# 4. Tauri CLI
cargo install tauri-cli
```

### Fedora

```bash
# 1. Dependencias del sistema
sudo dnf install -y \
  gcc gcc-c++ make curl wget git \
  webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel \
  librsvg2-devel openssl-devel pkg-config

# 2. Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 3. Deno
curl -fsSL https://deno.land/install.sh | sh

# 4. Tauri CLI
cargo install tauri-cli
```

### Arch Linux

```bash
# 1. Dependencias del sistema
sudo pacman -S --needed \
  base-devel curl wget git \
  webkit2gtk-4.1 gtk3 libappindicator-gtk3 \
  librsvg openssl pkg-config

# 2. Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 3. Deno
curl -fsSL https://deno.land/install.sh | sh

# 4. Tauri CLI
cargo install tauri-cli
```

---

## macOS

macOS incluye WebKit de forma nativa, por lo que requiere menos dependencias.

**Nota:** `xcode-select --install` instala solo las Command Line Tools (~1.5GB), no el IDE completo de Xcode (~12GB). Estas herramientas incluyen el compilador C/C++ (clang) y el linker, necesarios para compilar dependencias nativas de Rust como SQLCipher.

```bash
# 1. Xcode Command Line Tools (obligatorio)
xcode-select --install

# 2. Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 3. Deno (opción A: Homebrew)
brew install deno

# 3. Deno (opción B: script directo)
curl -fsSL https://deno.land/install.sh | sh

# 4. Tauri CLI
cargo install tauri-cli
```

---

## Compilación

Una vez instaladas las dependencias:

```bash
# Clonar el repositorio
git clone https://codeberg.org/Kyronix/Sanctum.git
cd Sanctum

# Modo desarrollo (con hot-reload)
cargo tauri dev

# Build de producción
cargo tauri build
```

---

## Artefactos de Build

Los binarios compilados se encuentran en `src-tauri/target/release/bundle/`:

| Sistema | Formatos Generados |
|:--------|:-------------------|
| Linux | `.deb`, `.AppImage`, `.rpm` |
| macOS | `.app`, `.dmg` |

---

## Verificación de Instalación

Antes de compilar, verifica que todo esté correctamente instalado:

```bash
rustc --version       # Debe mostrar 1.70+
cargo --version       # Debe coincidir con rustc
deno --version        # Debe mostrar 1.40+
cargo tauri --version # Debe mostrar 2.x
```

---

## Solución de Problemas

### Error: `pkg-config` no encuentra librerías

```bash
# Linux: asegúrate de tener instaladas las versiones -dev
sudo apt install libwebkit2gtk-4.1-dev  # Ubuntu/Debian
```

### Error: WebKit no encontrado en Linux

Algunas distribuciones empaquetan WebKit 4.0 en lugar de 4.1. Tauri v2 requiere la versión 4.1.

```bash
# Verificar versión disponible
apt search libwebkit2gtk
```

### Error de permisos en macOS

Si `cargo tauri build` falla con errores de firma:

```bash
# Compilar sin firmar (para desarrollo local)
cargo tauri build --no-bundle
```

---

## Entorno Alternativo: Nix

Si prefieres un entorno reproducible, el proyecto incluye un `flake.nix` para Linux:

```bash
# Requiere Nix con flakes habilitado
cd Sanctum
nix develop
cargo tauri dev
```

---

## Recursos Adicionales

- [Documentación de Tauri](https://tauri.app/v2/guides/)
- [Instalación de Rust](https://www.rust-lang.org/tools/install)
- [Instalación de Deno](https://deno.land/manual/getting_started/installation)