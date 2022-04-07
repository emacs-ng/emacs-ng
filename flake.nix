{
  description = "emacsng Nix flake";

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
              rust-overlay.overlay
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
                emacsng-rust
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
        emacsng-sources = prev.callPackages ./nix/_sources/generated.nix {};
        emacsng-source = emacsng-sources.emacsng.src;
        locked-date = prev.lib.removePrefix "nightly-" (prev.lib.removeSuffix "\n" (builtins.readFile ./rust-toolchain));
      in {
        emacsng-rust = with final; (
          let
            installPhase = ''
              tar --owner=0 --group=0 --numeric-owner --format=gnu \
                --sort=name --mtime="@$SOURCE_DATE_EPOCH" \
                -czf $out $name-versioned
            '';
            doVersionedUpdate = ''
              cargo vendor --versioned-dirs $name-versioned
            '';

            emacsngLibDeps = prev.rustPlatform.fetchCargoTarball {
              src = emacsng-source + "/rust_src/remacs-lib";
              name = "emacsngLibDeps";
              cargoUpdateHook = let
                pathDir = emacsng-source + "/rust_src/crates";
              in
                ''
                  cp -r ${pathDir} crates
                  sed -i 's|../crates/lisp_util|./crates/lisp_util|' Cargo.toml
                ''
                + doVersionedUpdate;
              sha256 = "sha256-LOzhuuucqDa1/wBqXPfPozHQL696JEmUgaTZ8y4yREA=";
              inherit installPhase;
            };

            ngBindgen = prev.rustPlatform.fetchCargoTarball {
              src = emacsng-source + "/rust_src/ng-bindgen";
              sourceRoot = null;
              cargoUpdateHook = doVersionedUpdate;
              name = "ngBindgen";
              sha256 = "sha256-MsMfcZ/Oni5dsOeuA37bSYscQLTZOJe5D4dB8KAgc5s=";
              inherit installPhase;
            };

            emacsngSrc = prev.rustPlatform.fetchCargoTarball {
              src = emacsng-source + "/rust_src";
              cargoUpdateHook =
                ''
                  sed -e 's/@CARGO_.*@//' Cargo.toml.in > Cargo.toml
                  sed -i 's|@WEBRENDER_DEFAULT_FEATURES@|"webrender"|' Cargo.toml
                ''
                + doVersionedUpdate;
              name = "emacsngSrc";
              sha256 = "sha256-nlzELzshSJWmSaWt5tewSbklnqnvdfWJeWfkdjCX1mo=";
              inherit installPhase;
            };

            emacsngHashdir = prev.rustPlatform.fetchCargoTarball {
              src = emacsng-source + "/lib-src/hashdir";
              sourceRoot = null;
              name = "emacsngHashdir";
              cargoUpdateHook = doVersionedUpdate;
              sha256 = "sha256-pcIYKdlt/qfmnrr3q0pyD0F0RMl+ZPls5YbhJdlenRA=";
              inherit installPhase;
            };
          in
            stdenv.mkDerivation {
              name = "emacsng-rust";
              srcs = [
                emacsngLibDeps
                ngBindgen
                emacsngHashdir
                emacsngSrc
              ];
              sourceRoot = ".";
              phases = ["unpackPhase" "installPhase"];
              installPhase = ''
                mkdir -p $out/.cargo/registry
                cat > $out/.cargo/config.toml << EOF
                [source.crates-io]
                registry = "https://github.com/rust-lang/crates.io-index"
                replace-with = "vendored-sources"
                [source.vendored-sources]
                directory = "$out/.cargo/registry"
                EOF
                cp -R emacsngLibDeps-vendor.tar.gz-versioned/* $out/.cargo/registry
                cp -R ngBindgen-vendor.tar.gz-versioned/* $out/.cargo/registry
                cp -R emacsngHashdir-vendor.tar.gz-versioned/* $out/.cargo/registry
                cp -R emacsngSrc-vendor.tar.gz-versioned/* $out/.cargo/registry
              '';
            }
        );

        librusty_v8 = prev.callPackage ./nix/librusty_v8.nix {};

        emacsng = with prev; let
          withWebrender = false;
        in
          (
            final.emacsGcc.override
            {
              withImageMagick = true;
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
            name = "emacsng-" + version;
            src = emacsng-source;
            version = builtins.substring 0 7 emacsng-source.rev;

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
                "CARGO_FLAGS=--offline" #nightly channel
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
              ]
              ++ lib.optionals (! withWebrender) [
                "--with-harfbuzz"
              ]
              ++ lib.optionals stdenv.isLinux [
                "--with-dbus"
              ];

            preBuild = let
              arch = rust.toRustTarget stdenv.hostPlatform;
            in
              (old.preBuild or "")
              + ''
                _librusty_v8_setup() {
                    for v in "$@"; do
                      install -D ${final.librusty_v8} "rust_src/target/$v/gn_out/obj/librusty_v8.a"
                    done
                  }
                  _librusty_v8_setup "debug" "release" "${arch}/release"
                    sed -i 's|deno = { git = "https://github.com/emacs-ng/deno", branch = "emacs-ng"|deno = { version = "1.9.2"|' rust_src/crates/js/Cargo.toml
                    sed -i 's|deno_runtime = { git = "https://github.com/emacs-ng/deno", branch = "emacs-ng"|deno_runtime = { version = "0.13.0"|' rust_src/crates/js/Cargo.toml
                    sed -i 's|deno_core = { git = "https://github.com/emacs-ng/deno", branch = "emacs-ng"|deno_core = { version = "0.86.0"|' rust_src/crates/js/Cargo.toml

                    sed -i 's|git = "https://github.com/servo/webrender.git", rev = ".*."|version = "0.61.0"|' rust_src/crates/webrender/Cargo.toml
                  export HOME=${final.emacsng-rust}
              '';

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
              ++ lib.optionals withWebrender (with xorg; [
                python3
                rpathLibs
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

            dontPatchShebangs = true; #straight_watch_callback.py: unsupported interpreter directive "#!/usr/bin/env -S python3 -u"

            postFixup =
              (old.postFixup or "")
              + (
                if withWebrender
                then
                  lib.concatStringsSep "\n" [
                    (lib.optionalString stdenv.isLinux ''
                      patchelf --set-rpath \
                        "$(patchelf --print-rpath "$out/bin/.emacs-28.0.50-wrapped"):${lib.makeLibraryPath rpathLibs}" \
                        "$out/bin/.emacs-28.0.50-wrapped"
                        patchelf --add-needed "libfontconfig.so" "$out/bin/.emacs-28.0.50-wrapped"
                    '')
                  ]
                else ""
              );
          });
      };
    };
}
