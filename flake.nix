/*
  This file is **only** for users of the Nix package manager
  (https://nixos.org/).  If you’re on:

    - macOS without nix‑darwin
    - Windows without Nix/WSL

  ...you can safely ignore it.

  For Nix users it defines:
    - all required packages
    - environment variables
    - handy dev‑shell aliases (e.g. `fmt`)

  Quick start: see “Getting Started with Nix” in README.md or CONTRIBUTING.md.
*/
{
  description = "Tessera UI dev env (desktop default, Android optional)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    (flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];

          config.allowUnfree = true; # bcs android unfree license
          config.android_sdk.accept_license = true; # silence license prompt
        };

        rust = pkgs.rust-bin.stable.latest.default;

        gfx = with pkgs; [
          wayland
          libxkbcommon
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          vulkan-loader
          vulkan-headers
        ];

        # Android payload (only for android shell)
        android = pkgs.androidenv.composeAndroidPackages {
          platformVersions = [ "34" ];
          buildToolsVersions = [ "34.0.0" ];
          abiVersions = [ "arm64-v8a" ];
          includeNDK = true;
          includeEmulator = false;
        };
        sdkRoot = "${android.androidsdk}/libexec/android-sdk";
      in
      {
        devShells = {
          # desktop‑only shell
          default = pkgs.mkShell {
            buildInputs = [
              rust
              pkgs.pkg-config
            ]
            ++ gfx;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath gfx;
            shellHook = ''
              echo "Setting up fmt command..."
              ${sharedShellHook}
              echo "Desktop shell ready."
            '';
          };

          # full blown android shell
          android = pkgs.mkShell {
            buildInputs = [
              pkgs.rustup
              pkgs.pkg-config
              pkgs.openssl.dev
              pkgs.openssl
              android.androidsdk
              pkgs.cargo-ndk
            ]
            ++ gfx;

            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath gfx;
            PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [ pkgs.openssl.dev ];

            shellHook = ''
              echo "Setting up fmt command..."
              ${sharedShellHook}

              echo "Setting up env vars..."
              export ANDROID_HOME=${sdkRoot}
              export ANDROID_NDK_HOME=$ANDROID_HOME/ndk-bundle
              export PATH="$PATH:$ANDROID_HOME/platform-tools:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"

              echo "Setting up rust toolchain..."
              # ensure a minimal stable toolchain + std‑lib for Android
              rustup --quiet toolchain install stable || true
              rustup --quiet target add aarch64-linux-android || true

              echo "Installing xbuild..."
              command -v x >/dev/null || cargo install xbuild --features vendored  # vendor OpenSSL
              echo "Android shell ready (adb / NDK / xbuild / cross‑std)."
            '';
          };
        };
      }
    ));
}
