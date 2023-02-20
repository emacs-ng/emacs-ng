{
  description = "Emacs NG Nix flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    emacs-overlay = {
      type = "github";
      owner = "nix-community";
      repo = "emacs-overlay";
    };
    devshell.url = "github:numtide/devshell";

    flake-compat.url = "github:edolstra/flake-compat";
    flake-compat.flake = false;

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    emacs-overlay,
    flake-compat,
    rust-overlay,
    flake-utils,
    devshell,
  }:
    {}
    // (
      flake-utils.lib.eachSystem ["x86_64-linux" "x86_64-darwin"]
      (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              self.overlays.default
              emacs-overlay.overlay
              (import rust-overlay)
              devshell.overlay
            ];
            config = {};
          };
        in rec {
          devShells.default = with pkgs; let
            custom-llvmPackages = llvmPackages_latest;
          in
            pkgs.devshell.mkShell {
              imports = [
                ./nix/rust.nix
                (pkgs.devshell.importTOML ./nix/commands.toml)
              ];

              packages = [
                custom-llvmPackages.clang
              ];
              env = [
                {
                  name = "LIBCLANG_PATH";
                  value = "${custom-llvmPackages.libclang}/lib";
                }
                {
                  name = "CACHIX_AUTH_TOKEN";
                  value = let
                    pwd = builtins.getEnv "PWD";
                    key = pwd + "/nix/cachix-key.secrets";
                  in
                    if lib.pathExists key
                    then lib.removeSuffix "\n" (builtins.readFile key)
                    else "";
                }
              ];
            };

          apps = {
            emacsng = flake-utils.lib.mkApp {
              drv = packages.emacsng;
              exePath = "/bin/emacs";
            };
            emacsclient = flake-utils.lib.mkApp {
              drv = packages.emacsng;
              exePath = "/bin/emacsclient";
            };
          };

          defaultApp = apps.emacsng;

          packages =
            flake-utils.lib.flattenTree
            {
              inherit
                (pkgs)
                emacsng
                ;
              default = pkgs.emacsng;
            };

          hydraJobs = {
            inherit packages;
          };
        }
      )
    )
    // {
      overlays.default = final: prev: let
        #rust nightly date
        locked-date = prev.lib.removePrefix "nightly-" (prev.lib.removeSuffix "\n" (builtins.readFile ./rust-toolchain));
      in {
        emacsng = with prev; let
          withWebrender = true;
        in
          (
            final.emacsGit.override
            {
              withImageMagick = true;
              withNS = false;
              inherit (prev) imagemagick;
            }
          )
          .overrideAttrs
          (old: let
            custom-llvmPackages = prev.llvmPackages_latest;
            #withGLX
            rpathLibs = with xorg;
              lib.optionals (stdenv.isLinux && withWebrender) [
                libX11
                libXrandr
                libXi
                libGLU
                libGL
                libXpm
                libXext
                libXxf86vm
                alsaLib
                libxkbcommon
                wayland
                libxcb
              ];
          in rec {
            name = "emacs-ng-" + version;
            src = ./.;

            # FIXME (@declantsien) Read it directly from configure.ac AC_INIT
            emacsVersion = "30.0.50";
            version = emacsVersion;

            # Cargo build requires this, see:
            # https://github.com/NixOS/nixpkgs/blob/22.11/pkgs/applications/networking/browsers/firefox/common.nix#L574
            dontFixLibtool = true;
            cargoDeps = prev.rustPlatform.importCargoLock {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            preConfigure =
              (old.preConfigure or "")
              + ''

              ''
              + lib.optionalString withWebrender ''
                export NIX_CFLAGS_LINK="$NIX_CFLAGS_LINK -lxcb-render -lxcb-xfixes -lxcb-shape"
              '';

            patches =
              (old.patches or [])
              ++ [
              ];

            makeFlags =
              (old.makeFlags or [])
              ++ [
                "CARGO_HOME=source/cargo-vendor-dir/.cargo/" # nightly channel
                "CARGO_FLAGS=--offline" # nightly channel
              ];

            #custom configure Flags Setting
            configureFlags =
              (
                if withWebrender
                then
                  lib.subtractLists [
                    "--with-x-toolkit=gtk3"
                    "--with-xft"
                    "--with-harfbuzz"
                    "--with-cairo"
                    "--with-imagemagick"
                  ]
                  old.configureFlags
                else old.configureFlags
              )
              ++ [
                "--with-json"
                "--with-threads"
                "--with-included-regex"
                "--with-compress-install"
                "--with-zlib"
                "--with-dumping=pdumper"
              ]
              ++ lib.optionals withWebrender [
                "--with-webrender"
                "--enable-webrender-x11"
              ]
              ++ lib.optionals (stdenv.isDarwin && withWebrender) [
                "--disable-webrender-self-contained"
              ]
              ++ lib.optionals (! withWebrender) [
                "--with-harfbuzz"
              ]
              ++ lib.optionals stdenv.isLinux [
                "--with-dbus"
              ];

            postPatch =
              (old.postPatch or "")
              + ''
                pwd="$(type -P pwd)"
                substituteInPlace Makefile.in --replace "/bin/pwd" "$pwd"
                substituteInPlace lib-src/Makefile.in --replace "/bin/pwd" "$pwd"
              '';

            LIBCLANG_PATH = "${custom-llvmPackages.libclang.lib}/lib";
            RUST_BACKTRACE = "full";

            buildInputs =
              (old.buildInputs or [])
              ++ [
                custom-llvmPackages.clang
                custom-llvmPackages.libclang
                final.rust-bin.nightly."${locked-date}".default
                git
              ]
              ++ lib.optionals withWebrender ([
                  python3
                ]
                ++ rpathLibs)
              ++ (with rustPlatform; [
                cargoSetupHook
                rust.cargo
                rust.rustc
              ])
              ++ lib.optionals
              stdenv.isDarwin
              (with darwin.apple_sdk.frameworks;
                with darwin;
                  [
                    libobjc
                    Security
                    CoreServices
                    Metal
                    Foundation
                    libiconv
                  ]
                  ++ lib.optionals (withWebrender && stdenv.isDarwin) [
                    AppKit
                    CoreGraphics
                    CoreServices
                    CoreText
                    Foundation
                    OpenGL
                  ]);

            dontPatchShebangs = true; # straight_watch_callback.py: unsupported interpreter directive "#!/usr/bin/env -S python3 -u"

            postFixup =
              (old.postFixup or "")
              + (
                if withWebrender
                then
                  lib.concatStringsSep "\n" [
                    (lib.optionalString stdenv.isLinux ''
                      patchelf --set-rpath \
                        "$(patchelf --print-rpath "$out/bin/emacs-$emacsVersion"):${lib.makeLibraryPath rpathLibs}" \
                        "$out/bin/emacs-$emacsVersion"
                        patchelf --add-needed "libfontconfig.so" "$out/bin/emacs-$emacsVersion"
                    '')
                  ]
                else ""
              );
          });
      };
    };
}
