{
  description = "Sanctum Dev Shell - Rust + Tauri + Svelte (pnpm)";

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
          stdenv.cc.cc.lib
          # Tauri / WebKit
          webkitgtk_4_1
          gtk3
          glib
          cairo
          pango
          gdk-pixbuf
          # System
          openssl
          dbus
          wayland
          libxkbcommon
        ];

        packages = with pkgs; [
          curl
          wget
          pkg-config
          sqlite
          nodePackages.pnpm
          nodejs
          cargo-audit
          cargo-edit
          cargo-modules
          cargo-tauri
          python315
          nix-output-monitor
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = packages ++ libraries ++ [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rust-analyzer"
              ];
            })
          ];

          shellHook = ''
            export LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LIBRARY_PATH
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
            export PKG_CONFIG_PATH=${pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" libraries}:$PKG_CONFIG_PATH
            export WEBKIT_DISABLE_DMABUF_RENDERER=1

            echo "> SANCTUM DEV SHELL ACTIVE"
            echo "   Compiler:  Rust $(rustc --version)"
            echo "   Runtime:   pnpm $(pnpm --version)"
          '';
        };
      }
    );
}
