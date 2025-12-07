{
  description = "Sanctum Dev Shell - Rust + Deno + Tauri v2";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        libraries = with pkgs; [
          webkitgtk_4_1
          gtk3
          cairo
          gdk-pixbuf
          glib
          dbus
          librsvg
          libsoup_3
        ];

        packages = with pkgs; [
          curl
          wget
          pkg-config
          dbus
          openssl_3
          glib
          gtk3
          libsoup_3
          webkitgtk_4_1
          librsvg
          deno
          # Security auditing tools
          cargo-audit
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = packages ++ [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rust-analyzer"
              ];
            })
            pkgs.cargo-tauri
          ];

          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
            export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS

            export PKG_CONFIG_PATH=${pkgs.openssl_3.dev}/lib/pkgconfig:$PKG_CONFIG_PATH

            echo "🛡️ SANCTUM DEV SHELL ACTIVO"
            echo "   Compilador: Rust $(rustc --version)"
            echo "   Runtime:    $(deno --version)"
            echo "   Tauri CLI:  $(cargo tauri --version)"
            echo "   Security:   cargo-audit $(cargo audit --version 2>/dev/null | head -n1)"
          '';
        };
      }
    );
}
