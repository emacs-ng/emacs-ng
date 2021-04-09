{
  description = "emacsNg Nix flake";

  inputs = {
    nixpkgs.url = "nixpkgs/e4650171afb53ca227d0c51f13fae87207153d69";
    emacs-overlay = {
      type = "github";
      owner = "nix-community";
      repo = "emacs-overlay";
      rev = "d9530a7048f4b1c0f65825202a0ce1d111a1d39a";
    };
    flake-compat = { url = "github:edolstra/flake-compat"; flake = false; };
    rust-overlay = { url = "github:oxalica/rust-overlay"; inputs.nixpkgs.follows = "nixpkgs"; };
    emacsNg-src = { url = "github:emacs-ng/emacs-ng"; flake = false; };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, emacs-overlay, flake-compat, emacsNg-src, rust-overlay, flake-utils }:
    { }
    //
    (flake-utils.lib.eachSystem [ "x86_64-linux" "x86_64-darwin" ]
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              self.overlay
              emacs-overlay.overlay
              rust-overlay.overlay
            ];
            config = { };
          };
        in
        rec {
          devShell = with pkgs;
            let
              llvmPackages = llvmPackages_9;
            in
            stdenv.mkDerivation rec {
              name = "emacsNg-shell";
              LIBCLANG_PATH = "${llvmPackages.libclang}/lib";
              buildInputs = [
                rust-bin.nightly."2021-01-14".rust
                llvmPackages.clang
                llvmPackages.libclang
              ];
            };

          apps = {
            emacsNg = flake-utils.lib.mkApp { drv = packages.emacsNg; exePath = "/bin/emacs"; };
          };
          defaultApp = apps.emacsNg;

          defaultPackage = pkgs.emacsNg;
          packages = flake-utils.lib.flattenTree {
            inherit (pkgs)
              emacsNg-rust
              emacsNg
              ;
          };

          hydraJobs = {
            inherit packages;
          };
        }
      )
    )
    // {
      overlay = final: prev: {
        emacsNg-rust = with final;
          (
            let
              installPhase = ''
                tar --owner=0 --group=0 --numeric-owner --format=gnu \
                    --sort=name --mtime="@$SOURCE_DATE_EPOCH" \
                    -czf $out $name-versioned
              '';
              doVersionedUpdate = ''
                cargo vendor --versioned-dirs $name-versioned
              '';

              remacsLibDeps = prev.rustPlatform.fetchCargoTarball {
                src = emacsNg-src;
                sourceRoot = "source/rust_src/remacs-lib";
                name = "remacsLibDeps";
                cargoUpdateHook = doVersionedUpdate;
                sha256 = "sha256-TtL+zfr4iaCG9I4NJ1i18c4aIgGyPfYfryHVAzBl3eI=";
                inherit installPhase;
              };

              remacsBindings = prev.rustPlatform.fetchCargoTarball {
                src = emacsNg-src;
                sourceRoot = "source/rust_src/remacs-bindings";
                cargoUpdateHook = doVersionedUpdate;
                name = "remacsBindings";
                sha256 = "sha256-uEUXWv1ybXN7B8sOsVnXxGgjDPTtsVbE++I0grwvn2E=";
                inherit installPhase;
              };

              remacsSrc = prev.rustPlatform.fetchCargoTarball {
                src = "${emacsNg-src}/rust_src";
                cargoUpdateHook = ''
                  sed -e 's/@CARGO_.*@//' Cargo.toml.in > Cargo.toml
                '' + doVersionedUpdate;
                name = "remacsSrc";
                sha256 = "sha256-5D7Q46JTtr8jfE43CnrxvpTQj6iw9Qi5aGJL/TgCc/Y=";
                inherit installPhase;
              };

              remacsHashdir = prev.rustPlatform.fetchCargoTarball {
                src = "${emacsNg-src}/lib-src/hashdir";
                sourceRoot = null;
                name = "remacsHashdir";
                cargoUpdateHook = doVersionedUpdate;
                sha256 = "sha256-yC/1uhiVJ2OOf56A+Hy8jRqhXvSMC5V/DwdSsBFgGDI=";
                inherit installPhase;
              };
            in
            stdenv.mkDerivation {
              name = "emacsNg-rust";
              srcs = [
                remacsLibDeps
                remacsBindings
                remacsHashdir
                remacsSrc
              ];
              sourceRoot = ".";
              phases = [ "unpackPhase" "installPhase" ];
              installPhase = ''
                mkdir -p $out/.cargo/registry
                cat > $out/.cargo/config.toml << EOF
                  [source.crates-io]
                  registry = "https://github.com/rust-lang/crates.io-index"
                  replace-with = "vendored-sources"
                  [source.vendored-sources]
                  directory = "$out/.cargo/registry"
                EOF
                cp -R remacsLibDeps-vendor.tar.gz-versioned/* $out/.cargo/registry
                cp -R remacsBindings-vendor.tar.gz-versioned/* $out/.cargo/registry
                cp -R remacsHashdir-vendor.tar.gz-versioned/* $out/.cargo/registry
                cp -R remacsSrc-vendor.tar.gz-versioned/* $out/.cargo/registry
              '';
            }
          );

        librusty_v8 = prev.callPackage ./nix/librusty_v8.nix { };
        emacsNg = with prev; (
          final.emacsGcc.override
            ({
              withImageMagick = true;
              imagemagick = prev.imagemagick;
            })
        ).overrideAttrs
          (old:
            let
              custom-llvmPackages = prev.llvmPackages_9;
            in
            rec {
              name = "emacsNg-" + version;
              src = emacsNg-src;
              version = builtins.substring 0 7 emacsNg-src.rev;

              preConfigure = (old.preConfigure or "") + ''
            '';

              #custom configure Flags Setting
              configureFlags = (old.configureFlags or [ ]) ++ [
                "--with-json"
                "--with-threads"
                "--with-included-regex"
              ];

              preBuild = let arch = rust.toRustTarget stdenv.hostPlatform; in
                (old.preBuild or "") + ''
                  _librusty_v8_setup() {
                      for v in "$@"; do
                        install -D ${final.librusty_v8} "rust_src/target/$v/gn_out/obj/librusty_v8.a"
                      done
                    }
                    _librusty_v8_setup "debug" "release" "${arch}/release"
                    sed -i 's|deno = { git = "https://github.com/DavidDeSimone/deno", branch = "emacs-ng"|deno = { version = "1.7.2"|' rust_src/Cargo.toml
                    sed -i 's|deno_runtime = { git = "https://github.com/DavidDeSimone/deno", branch = "emacs-ng"|deno_runtime = { version = "0.8.0"|' rust_src/Cargo.toml
                    sed -i 's|deno_core = { git = "https://github.com/DavidDeSimone/deno"|deno_core = { version = "0.78.0"|' rust_src/Cargo.toml
                    export HOME=${final.emacsNg-rust}
                '';

              postPatch = (old.postPatch or "") + ''
                pwd="$(type -P pwd)"
                substituteInPlace Makefile.in --replace "/bin/pwd" "$pwd"
                substituteInPlace lib-src/Makefile.in --replace "/bin/pwd" "$pwd"
              '';

              LIBCLANG_PATH = "${custom-llvmPackages.libclang}/lib";

              buildInputs = (old.buildInputs or [ ]) ++
              [
                custom-llvmPackages.clang
                custom-llvmPackages.libclang
                final.rust-bin.nightly."2021-01-14".rust
              ] ++ lib.optionals
                stdenv.isDarwin [
                darwin.libobjc
                darwin.apple_sdk.frameworks.Security
                darwin.apple_sdk.frameworks.CoreServices
                darwin.apple_sdk.frameworks.Metal
                darwin.apple_sdk.frameworks.Foundation
              ];

              makeFlags = (old.makeFlags or [ ]) ++ [
                "CARGO_FLAGS=--offline" #nightly channel
              ];
            });
      };
    };
}
