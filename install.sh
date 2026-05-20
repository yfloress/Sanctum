#!/usr/bin/env bash
set -euo pipefail

# Sanctum install script
# Builds from source (frontend + release binary) and installs the binary,
# desktop entry, and icons.
#
# Usage:
#   ./install.sh --user               # user-local (~/.local), no sudo
#   sudo ./install.sh                 # system-wide (/usr/local)
#   ./install.sh --user --uninstall   # remove user-local install
#   sudo ./install.sh --uninstall     # remove system-wide install
#   ./install.sh --user --rebuild     # force a rebuild even if up to date
#   ./install.sh --user --no-build    # install an already-built binary
#   PREFIX=/usr sudo ./install.sh     # custom prefix

# Run from the repo root regardless of where the script is invoked.
cd "$(dirname "$(realpath "$0")")"

PREFIX="${PREFIX:-/usr/local}"
DESTDIR="${DESTDIR:-}"

APP_NAME="sanctum"                              # installed command / icon / .desktop name
CRATE_BIN="sanctum-tauri"                       # cargo crate binary name
BINARY="target/release/${CRATE_BIN}"            # workspace-shared target dir
DESKTOP_FILE="packaging/linux/${APP_NAME}.desktop"
ICON_FILE="packaging/linux/${APP_NAME}.png"     # committed 512x512 app icon
ICON_SIZE="512x512"

# --- helpers ---

die() {
    echo "ERROR: $*" >&2
    exit 1
}

need_root() {
    if [ "$(id -u)" -ne 0 ]; then
        die "This operation needs root. Use sudo, or run ./install.sh --user"
    fi
}

# Refuse to run on non-Linux: this installer relies on Linux-only conventions
# (XDG desktop entry, hicolor icon theme, update-desktop-database).
require_linux() {
    local os
    os="$(uname -s 2>/dev/null || echo unknown)"
    case "$os" in
        Linux) ;;
        Darwin)
            die "macOS is not supported yet — this installer only targets Linux. To run Sanctum on macOS, build it manually with: cargo tauri build"
            ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT)
            die "Windows is not supported yet — this installer only targets Linux. To run Sanctum on Windows, build it manually with: cargo tauri build"
            ;;
        *)
            die "Unsupported OS '${os}' — this installer only targets Linux."
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
        echo "Binary up to date (${BINARY}); skipping build (use --rebuild to force)."
        return
    fi

    echo "Building Sanctum (frontend + release binary)..."
    # cargo tauri build runs the frontend build (pnpm) via beforeBuildCommand,
    # then compiles the Rust binary with the frontend embedded. --no-bundle
    # skips the .deb/.rpm/AppImage packaging we don't need for a local install.
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

install_binary() {
    local bindir="${DESTDIR}${PREFIX}/bin"
    install -Dm755 "${BINARY}" "${bindir}/${APP_NAME}"
    echo "  -> ${bindir}/${APP_NAME}"
}

install_desktop() {
    local appdir="${DESTDIR}${PREFIX}/share/applications"
    mkdir -p "${appdir}"
    # Point Exec at the absolute install path so the launcher always finds it.
    sed "s|^Exec=.*|Exec=${PREFIX}/bin/${APP_NAME}|" "${DESKTOP_FILE}" \
        > "${appdir}/${APP_NAME}.desktop"
    chmod 644 "${appdir}/${APP_NAME}.desktop"
    echo "  -> ${appdir}/${APP_NAME}.desktop"
}

install_icon() {
    [ -f "${ICON_FILE}" ] || die "Icon not found: ${ICON_FILE}"
    local dest="${DESTDIR}${PREFIX}/share/icons/hicolor/${ICON_SIZE}/apps/${APP_NAME}.png"
    install -Dm644 "${ICON_FILE}" "${dest}"
    echo "  -> ${dest}"
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
    local appdir="${DESTDIR}${PREFIX}/share/applications"
    local icondir="${DESTDIR}${PREFIX}/share/icons/hicolor"

    rm -f "${bindir}/${APP_NAME}"
    rm -f "${appdir}/${APP_NAME}.desktop"
    rm -f "${icondir}/${ICON_SIZE}/apps/${APP_NAME}.png"
    echo "  Removed ${bindir}/${APP_NAME}"
    echo "  Removed ${appdir}/${APP_NAME}.desktop"
    echo "  Removed ${icondir}/${ICON_SIZE}/apps/${APP_NAME}.png"
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
            sed -n '3,16p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *) die "Unknown option: $arg (try --help)" ;;
    esac
done

# Bail out early on unsupported platforms (after --help so usage still prints).
require_linux

# Resolve target prefix.
if [ "$IS_USER" -eq 1 ]; then
    PREFIX="${HOME}/.local"
    DESTDIR=""
elif [ "$(id -u)" -ne 0 ]; then
    need_root
fi

if [ "$DO_UNINSTALL" -eq 1 ]; then
    echo "Uninstalling Sanctum from ${PREFIX}..."
    uninstall_files
    update_caches
    echo "Done. User data (vault DB in your config dir) was left untouched."
    exit 0
fi

if [ "$IS_USER" -eq 1 ]; then
    echo "Installing Sanctum for current user (${PREFIX})..."
else
    echo "Installing Sanctum system-wide (${PREFIX})..."
fi

build_app
install_binary
install_desktop
install_icon
update_caches

echo ""
echo "Sanctum installed successfully."
echo "Run 'sanctum' from your terminal, or find it in your application menu."
if [ "$IS_USER" -eq 1 ]; then
    case ":${PATH}:" in
        *":${HOME}/.local/bin:"*) ;;
        *) echo "NOTE: ${HOME}/.local/bin is not on your PATH; add it to use the 'sanctum' command." ;;
    esac
    echo "Uninstall with: ./install.sh --user --uninstall"
else
    echo "Uninstall with: sudo $(realpath "$0") --uninstall"
fi
