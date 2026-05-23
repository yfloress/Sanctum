#!/usr/bin/env bash
set -euo pipefail

# Sanctum install script
# Builds from source (frontend + release binary) and installs Sanctum.
#
# Usage:
#   ./install.sh --user               # user-local (~/.local), no sudo
#   sudo ./install.sh                 # system-wide (/usr/local)
#   ./install.sh --user --uninstall   # remove user-local install
#   sudo ./install.sh --uninstall     # remove system-wide install
#   ./install.sh --user --rebuild     # force a rebuild even if up to date
#   ./install.sh --user --no-build    # install an already-built binary
#   PREFIX=/usr sudo ./install.sh     # custom prefix
#
# Platforms:
#   Linux  — installs the binary, desktop entry, and icon.
#   macOS  — installs the binary only (no XDG desktop entry / icon theme).

# Run from the repo root regardless of where the script is invoked.
# Resolved without realpath so it stays portable on macOS (no coreutils).
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "${SCRIPT_DIR}"

PREFIX="${PREFIX:-/usr/local}"
DESTDIR="${DESTDIR:-}"

APP_NAME="sanctum"                              # installed command / icon / .desktop name
CRATE_BIN="sanctum-tauri"                       # cargo crate binary name
BINARY="target/release/${CRATE_BIN}"            # workspace-shared target dir
DESKTOP_FILE="packaging/linux/${APP_NAME}.desktop"
ICON_FILE="packaging/linux/${APP_NAME}.png"     # committed 512x512 app icon
ICON_SIZE="512x512"

# --- pretty output ---

log()  { printf '\n\033[1;32m[+] %s\033[0m\n' "$*"; }
warn() { printf '\n\033[1;33m[!] %s\033[0m\n' "$*"; }
err()  { printf '\n\033[1;31m[✗] %s\033[0m\n' "$*"; }
info() { printf '     \033[1;37m%s\033[0m\n\n' "$*"; }

# --- helpers ---

die() {
    err "$*"
    exit 1
}

need_root() {
    if [ "$(id -u)" -ne 0 ]; then
        die "This operation needs root. Use sudo, or run ./install.sh --user"
    fi
}

# Detect the platform. Linux installs everything (binary + desktop entry +
# icon); macOS installs the binary only, since it has no XDG desktop entries
# or hicolor icon theme. Windows and anything else are unsupported.
IS_MACOS=0
detect_os() {
    local os
    os="$(uname -s 2>/dev/null || echo unknown)"
    case "$os" in
        Linux) ;;
        Darwin) IS_MACOS=1 ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT)
            die "Windows is not supported — this installer only targets Linux and macOS. Build manually with: cargo tauri build"
            ;;
        *)
            die "Unsupported OS '${os}' — this installer only targets Linux and macOS."
            ;;
    esac
}

# --- build ---

build_app() {
    if [ "$NO_BUILD" -eq 1 ]; then
        [ -f "${BINARY}" ] || die "--no-build set but ${BINARY} does not exist."
        return
    fi

    local need="$FORCE_BUILD"
    if [ ! -f "${BINARY}" ]; then
        need=1
    elif [ -n "$(find src-tauri/src ui-svelte/src \
                      Cargo.toml Cargo.lock src-tauri/Cargo.toml ui-svelte/package.json \
                      -newer "${BINARY}" 2>/dev/null)" ]; then
        need=1
    fi

    if [ "$need" -eq 0 ]; then
        info "Binary up to date (${BINARY}); skipping build (use --rebuild to force)."
        return
    fi

    log "Building Sanctum (frontend + release binary)..."
    # cargo tauri build runs the frontend build (pnpm) via beforeBuildCommand,
    # then compiles the Rust binary with the frontend embedded. --no-bundle
    # skips the platform packaging (.deb/.rpm/AppImage on Linux, .app/.dmg on
    # macOS) we don't need for a local install.
    if command -v nix >/dev/null 2>&1 && [ -f flake.nix ]; then
        nix develop -c cargo tauri build --no-bundle
    elif command -v cargo-tauri >/dev/null 2>&1; then
        cargo tauri build --no-bundle
    else
        die "Build needs Nix (with flake.nix) or 'cargo tauri' (tauri-cli) in PATH."
    fi

    [ -f "${BINARY}" ] || die "Build finished but ${BINARY} was not produced."
}

# --- install ---

# Portable install: BSD install (macOS) has no -D, so create dirs separately.
install_binary() {
    local bindir="${DESTDIR}${PREFIX}/bin"
    mkdir -p "${bindir}"
    install -m755 "${BINARY}" "${bindir}/${APP_NAME}"
    info "${bindir}/${APP_NAME}"
}

install_desktop() {
    local appdir="${DESTDIR}${PREFIX}/share/applications"
    mkdir -p "${appdir}"
    # Point Exec at the absolute install path so the launcher always finds it.
    sed "s|^Exec=.*|Exec=${PREFIX}/bin/${APP_NAME}|" "${DESKTOP_FILE}" \
        > "${appdir}/${APP_NAME}.desktop"
    chmod 644 "${appdir}/${APP_NAME}.desktop"
    info "${appdir}/${APP_NAME}.desktop"
}

install_icon() {
    [ -f "${ICON_FILE}" ] || die "Icon not found: ${ICON_FILE}"
    local icondir="${DESTDIR}${PREFIX}/share/icons/hicolor/${ICON_SIZE}/apps"
    mkdir -p "${icondir}"
    install -m644 "${ICON_FILE}" "${icondir}/${APP_NAME}.png"
    info "${icondir}/${APP_NAME}.png"
}

update_caches() {
    local appdir="${DESTDIR}${PREFIX}/share/applications"
    local icondir="${DESTDIR}${PREFIX}/share/icons/hicolor"
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "${appdir}" 2>/dev/null || true
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t "${icondir}" 2>/dev/null || true
    fi
}

# --- uninstall ---

uninstall_files() {
    local bindir="${DESTDIR}${PREFIX}/bin"
    rm -f "${bindir}/${APP_NAME}"
    info "Removed ${bindir}/${APP_NAME}"

    # Desktop entry and icon only exist on Linux installs.
    if [ "$IS_MACOS" -eq 0 ]; then
        local appdir="${DESTDIR}${PREFIX}/share/applications"
        local icondir="${DESTDIR}${PREFIX}/share/icons/hicolor"
        rm -f "${appdir}/${APP_NAME}.desktop"
        rm -f "${icondir}/${ICON_SIZE}/apps/${APP_NAME}.png"
        info "Removed ${appdir}/${APP_NAME}.desktop"
        info "Removed ${icondir}/${ICON_SIZE}/apps/${APP_NAME}.png"
    fi
}

# --- main ---

IS_USER=0
DO_UNINSTALL=0
FORCE_BUILD=0
NO_BUILD=0

for arg in "$@"; do
    case "$arg" in
        --user)      IS_USER=1 ;;
        --uninstall) DO_UNINSTALL=1 ;;
        --rebuild)   FORCE_BUILD=1 ;;
        --no-build)  NO_BUILD=1 ;;
        -h|--help)
            sed -n '4,18p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *) die "Unknown option: $arg (try --help)" ;;
    esac
done

# Bail out early on unsupported platforms (after --help so usage still prints).
detect_os

# Resolve target prefix.
if [ "$IS_USER" -eq 1 ]; then
    PREFIX="${HOME}/.local"
    DESTDIR=""
elif [ "$(id -u)" -ne 0 ]; then
    need_root
fi

if [ "$DO_UNINSTALL" -eq 1 ]; then
    log "Uninstalling Sanctum from ${PREFIX}..."
    uninstall_files
    [ "$IS_MACOS" -eq 0 ] && update_caches
    info "User data (vault DB in your config dir) was left untouched."
    exit 0
fi

if [ "$IS_USER" -eq 1 ]; then
    log "Installing Sanctum for current user (${PREFIX})..."
else
    log "Installing Sanctum system-wide (${PREFIX})..."
fi

if [ "$IS_MACOS" -eq 1 ]; then
    warn "macOS detected: only the binary will be installed (no desktop entry or icon)."
fi

build_app
install_binary
if [ "$IS_MACOS" -eq 0 ]; then
    install_desktop
    install_icon
    update_caches
fi

log "Sanctum installed successfully."
if [ "$IS_MACOS" -eq 1 ]; then
    info "Run 'sanctum' from your terminal."
else
    info "Run 'sanctum' from your terminal, or find it in your application menu."
fi

if [ "$IS_USER" -eq 1 ]; then
    case ":${PATH}:" in
        *":${HOME}/.local/bin:"*) ;;
        *) warn "${HOME}/.local/bin is not on your PATH; add it to use the 'sanctum' command." ;;
    esac
    info "Uninstall with: ./install.sh --user --uninstall"
else
    info "Uninstall with: sudo ${SCRIPT_DIR}/$(basename "$0") --uninstall"
fi
