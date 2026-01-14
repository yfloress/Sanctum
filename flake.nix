{
  description = "Sanctum Dev Shell - Rust + Slint (Rust-only UI)";

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
          libGL
          wayland
          libxkbcommon
          harfbuzz
          openssl
          fontconfig
          freetype
          dbus
        ];

        packages = with pkgs; [
          curl
          wget
          pkg-config
          sqlite
          cargo-audit
          cargo-edit
          slint-lsp
          cargo-modules
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

            echo "> SANCTUM DEV SHELL ACTIVE"
            echo "   Compiler:  Rust $(rustc --version)"
          '';
        };
      }
    );
}
