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
          config = {
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
        };

        # Toolchain Android (SDK + NDK) para cross-compilar el núcleo nativo
        # (Rust + SQLCipher/OpenSSL) a Android. Sirve para la validación F0 y
        # luego para construir el cliente Android.
        androidComposition = pkgs.androidenv.composeAndroidPackages {
          # El proyecto generado por Tauri usa compileSdk/targetSdk 36 y AGP pide
          # build-tools 35.0.0. Deben estar provistos por el flake porque el SDK
          # vive en el Nix store (solo lectura) y Gradle no puede auto-instalarlos.
          platformVersions = [ "36" ];
          buildToolsVersions = [ "35.0.0" ];
          includeNDK = true;
          ndkVersions = [ "26.3.11579264" ];
        };
        androidSdk = androidComposition.androidsdk;

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
          pnpm
          nodejs
          cargo-audit
          cargo-edit
          cargo-modules
          cargo-tauri
          cargo-ndk
          android-tools
          jdk17
          perl
          python315
          nix-output-monitor
        ];
      in
      {
        apps.default = {
          type = "app";
          program = let
            script = pkgs.writeShellScriptBin "sanctum-dev" ''
              exec ${pkgs.cargo-tauri}/bin/cargo-tauri dev "$@"
            '';
          in
            "${script}/bin/sanctum-dev";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = packages ++ libraries ++ [
            androidSdk
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rust-analyzer"
              ];
              targets = [
                "aarch64-linux-android"
                "armv7-linux-androideabi"
                "i686-linux-android"
                "x86_64-linux-android"
              ];
            })
          ];

          shellHook = ''
            export LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LIBRARY_PATH
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
            export PKG_CONFIG_PATH=${pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" libraries}:$PKG_CONFIG_PATH
            export WEBKIT_DISABLE_DMABUF_RENDERER=1

            # Android SDK/NDK (para cross-compile a Android via cargo-ndk)
            export ANDROID_HOME="${androidSdk}/libexec/android-sdk"
            export ANDROID_SDK_ROOT="$ANDROID_HOME"
            export ANDROID_NDK_ROOT="$(ls -d "$ANDROID_HOME"/ndk/* 2>/dev/null | head -1)"
            export ANDROID_NDK_HOME="$ANDROID_NDK_ROOT"
            # JDK para el Gradle de Android (tauri android init/build)
            export JAVA_HOME="${pkgs.jdk17.home}"
            # Herramientas del SDK en PATH (emulator, avdmanager, sdkmanager)
            export PATH="$ANDROID_HOME/emulator:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"

            echo "> SANCTUM DEV SHELL ACTIVE"
            echo "   Compiler:  Rust $(rustc --version)"
            echo "   Runtime:   pnpm $(pnpm --version)"
            echo "   Android:   NDK $ANDROID_NDK_ROOT"
            echo "   Java:      $(java -version 2>&1 | head -1)"
          '';
        };
      }
    );
}
