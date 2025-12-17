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
          fontconfig
          freetype
          harfbuzz
          openssl
          # X11 support (uncomment if running on Xorg)
          # xorg.libX11
          # xorg.libXcursor
          # xorg.libXi
          # xorg.libXrandr
          # xorg.libXrender
          # xorg.libXfixes
          # xorg.libxcb
          # xorg.xcbutil
          # xorg.xcbutilkeysyms
          # xorg.xcbutilwm
          # xorg.xcbutilimage
        ];

        packages = with pkgs; [
          curl
          wget
          pkg-config
          sqlite
          cargo-audit
          slint-lsp
          bacon
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
            echo "   Slint LSP: $(slint-lsp --version 2>/dev/null | head -n1)"
            echo "   Security:  cargo-audit $(cargo audit --version 2>/dev/null | head -n1)"
          '';
        };
      }
    );
}
