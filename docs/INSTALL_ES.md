# Instalación de Sanctum

Sanctum es una app de escritorio **Tauri 2 + Svelte 5** (núcleo en Rust +
frontend en Svelte). El repositorio incluye un [`install.sh`](../install.sh) que
**compila desde el código fuente** e instala Sanctum en tu sistema.

> **Todos los comandos se ejecutan desde la raíz del repositorio**, no desde `src-tauri/`.

## Tabla de Contenidos
- [Instalación Rápida](#instalación-rápida)
- [Qué Hace el Instalador](#qué-hace-el-instalador)
- [Requisitos Previos](#requisitos-previos)
  - [Opción A: Nix (Linux, recomendado)](#opción-a-nix-linux-recomendado)
  - [Opción B: Toolchain Manual](#opción-b-toolchain-manual)
- [Usar el Instalador](#usar-el-instalador)
- [Desinstalar](#desinstalar)
- [Compilación Manual (Desarrollo / Windows)](#compilación-manual-desarrollo--windows)
- [Dónde se Guardan tus Datos](#dónde-se-guardan-tus-datos)

---

## Instalación Rápida

```bash
git clone https://codeberg.org/Kyronix/Sanctum.git
cd Sanctum
./install.sh --user      # compila + instala en ~/.local, sin sudo
```

El instalador compila Sanctum (frontend + binario release) desde el código fuente,
así que necesita un toolchain de compilación funcional — ver
[Requisitos Previos](#requisitos-previos). Si el binario ya está al día, el paso de
compilación se omite automáticamente.

---

## Qué Hace el Instalador

El instalador se adapta a tu plataforma:

| Plataforma | Qué se instala |
|------------|----------------|
| **Linux** | Binario + entrada de escritorio + icono (y refresca las cachés de escritorio/iconos) |
| **macOS** | **Solo el binario** — macOS no tiene entrada de escritorio XDG ni tema de iconos hicolor |
| **Windows** | No soportado por el instalador — [compila manualmente](#compilación-manual-desarrollo--windows) o usa WSL |

En macOS el script muestra un aviso de que solo se instalará el binario.

---

## Requisitos Previos

El instalador compila desde el código fuente con `cargo tauri build`, así que
necesitas **o bien** Nix **o bien** el toolchain manual de Rust + Tauri.

### Opción A: Nix (Linux, recomendado)

Nix gestiona todas las librerías del sistema automáticamente. El instalador
detecta el `flake.nix` y ejecuta la compilación dentro de `nix develop`.

1. [Instala Nix](https://nixos.org/download) con flakes habilitados.
2. Instala las dependencias del frontend una vez:
   ```bash
   cd ui-svelte && pnpm install && cd ..
   ```
3. Ejecuta el instalador (ver [Usar el Instalador](#usar-el-instalador)).

### Opción B: Toolchain Manual

| Herramienta | Versión | Notas |
|-------------|---------|-------|
| Rust | stable ≥ 1.77 | vía [rustup](https://rustup.rs/) |
| Node.js | ≥ 20 LTS | |
| pnpm | ≥ 9 | `npm install -g pnpm` |
| Tauri CLI | 2.x | `cargo install tauri-cli --version "^2"` |

Instala las dependencias del frontend una vez (necesario antes de la primera compilación):
```bash
cd ui-svelte && pnpm install && cd ..
```

**Librerías del sistema en Linux** (no hacen falta si usas Nix):

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

## Usar el Instalador

```bash
./install.sh --user      # local al usuario (~/.local), sin sudo  ← recomendado
sudo ./install.sh        # a nivel de sistema (/usr/local)
```

**Opciones:**

| Flag | Efecto |
|------|--------|
| `--user` | Instala en `~/.local` en vez de a nivel de sistema (sin sudo) |
| `--rebuild` | Fuerza recompilar aunque el binario parezca al día |
| `--no-build` | Instala un binario ya compilado (omite la compilación) |
| `--uninstall` | Elimina una instalación existente (ver [Desinstalar](#desinstalar)) |
| `-h`, `--help` | Muestra el uso |

**Variables de entorno:**

| Variable | Propósito |
|----------|-----------|
| `PREFIX` | Prefijo de instalación (por defecto `/usr/local`, o `~/.local` con `--user`) |
| `DESTDIR` | Raíz de staging para empaquetado (se antepone a `PREFIX`) |

```bash
PREFIX=/usr sudo ./install.sh      # prefijo personalizado
```

**Ubicaciones de instalación** (Linux, relativas a `PREFIX`):

| Archivo | Ruta |
|---------|------|
| Binario | `bin/sanctum` |
| Entrada de escritorio | `share/applications/sanctum.desktop` |
| Icono | `share/icons/hicolor/512x512/apps/sanctum.png` |

> **Nota sobre el PATH:** con `--user`, asegúrate de que `~/.local/bin` esté en tu
> `PATH` para que se encuentre el comando `sanctum`. El instalador te avisa si no lo está.

---

## Desinstalar

```bash
./install.sh --user --uninstall      # elimina una instalación local de usuario
sudo ./install.sh --uninstall        # elimina una instalación de sistema
```

Esto elimina el binario (y, en Linux, la entrada de escritorio y el icono). **Tus
datos del vault quedan intactos** — ver [Dónde se Guardan tus Datos](#dónde-se-guardan-tus-datos).

---

## Compilación Manual (Desarrollo / Windows)

Si prefieres no usar el instalador — para desarrollo, o en Windows — compila
directamente con la CLI de Tauri:

```bash
cd ui-svelte && pnpm install && cd ..   # solo la primera vez
cargo tauri dev      # desarrollo, con recarga en caliente
cargo tauri build    # binario de producción → target/release/sanctum-tauri
```

### Windows

**Opción A — WSL (recomendado).** Instala Ubuntu o Fedora desde la Microsoft
Store (Windows 10/11 con WSLg), luego sigue los pasos del
[toolchain manual de Linux](#opción-b-toolchain-manual) dentro de la terminal WSL.

**Opción B — Windows Nativo.**
1. Instala **Visual Studio Build Tools** con la carga de trabajo "Desarrollo para escritorio con C++".
2. Instala **Rust (target MSVC)** vía [rustup.rs](https://rustup.rs/).
3. Instala **Node.js** (LTS) y **pnpm** (`npm install -g pnpm`).
4. Instala **WebView2** (preinstalado en Windows 11; descárgalo de Microsoft para Windows 10).
5. Instala la CLI de Tauri: `cargo install tauri-cli --version "^2"`.
6. Desde la raíz del repositorio:
   ```powershell
   cd ui-svelte; pnpm install; cd ..
   cargo tauri dev      # desarrollo
   cargo tauri build    # instalador de producción
   ```

> El script `install.sh` solo está pensado para Linux y macOS. En Windows, usa la
> compilación manual de arriba (o WSL) para producir y ejecutar el binario.

---

## Dónde se Guardan tus Datos

La base de datos cifrada de tu vault se guarda en el directorio de configuración
por usuario de tu sistema operativo, **separada del binario instalado**.
Desinstalar Sanctum nunca la toca — para eliminar Sanctum por completo debes
borrar esos datos tú mismo.
