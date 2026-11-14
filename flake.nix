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

  Quick start: see “Getting Started with Nix” in CONTRIBUTING.md.
*/
{
  description = "Tessera UI dev env (desktop default, Android optional)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
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

        # `cargo tessera check` analyzes the standard library sources.
        rustWith =
          targets:
          pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
            targets = [ "wasm32-unknown-unknown" ] ++ targets;
          };

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

        sharedPkgs =
          with pkgs;
          [
            pkg-config
            openssl
            rust-script
            binaryen
            git-lfs
          ]
          ++ gfx;

        # Matches the Android Gradle plugin and NDK used by generated projects.
        ndkVersion = "28.2.13676358";
        android = pkgs.androidenv.composeAndroidPackages {
          platformVersions = [ "34" ];
          buildToolsVersions = [ "35.0.0" ];
          abiVersions = [
            "arm64-v8a"
            "x86_64"
          ];
          includeNDK = true;
          ndkVersions = [ ndkVersion ];
          includeEmulator = false;
        };
        sdkRoot = "${android.androidsdk}/libexec/android-sdk";

        sharedShellHook = ''
          # project root baked in at eval time
          PRJ_ROOT="${toString self}"

          fmt() {
            local root
            if root=$(git -C "$PWD" rev-parse --show-toplevel 2>/dev/null); then
              echo "Using git root"
            else
              root="$PRJ_ROOT"
              echo "Not in git repo, using PRJ_ROOT"
            fi
            ( cd "$root" && rust-script scripts/check-imports.rs "$@" )
          }
          export -f fmt
        '';
      in
      {
        devShells = {
          # desktop‑only shell
          default = pkgs.mkShell {
            buildInputs = [ (rustWith [ ]) ] ++ sharedPkgs;
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
              (rustWith [
                "aarch64-linux-android"
                "x86_64-linux-android"
              ])
              android.androidsdk
              pkgs.jdk21
            ]
            ++ sharedPkgs;

            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath gfx;
            ANDROID_HOME = sdkRoot;
            ANDROID_NDK_HOME = "${sdkRoot}/ndk/${ndkVersion}";
            NDK_HOME = "${sdkRoot}/ndk/${ndkVersion}";
            JAVA_HOME = pkgs.jdk21.home;

            shellHook = ''
              echo "Setting up fmt command..."
              ${sharedShellHook}
              export PATH="$PATH:$ANDROID_HOME/platform-tools"
              echo "Android shell ready (adb / NDK / cargo tessera android)."
            '';
          };
        };
      }
    ));
}
