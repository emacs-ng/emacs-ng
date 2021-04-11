{
  description = "emacsNg Nix flake";

  inputs = {
    nixpkgs.url = "nixpkgs/d09f37cc24e4ec1a567f77e553a298158185182d";
    emacs-overlay = {
      type = "github";
      owner = "nix-community";
      repo = "emacs-overlay";
      rev = "d9530a7048f4b1c0f65825202a0ce1d111a1d39a";
    };

    master.url = "nixpkgs/7d71001b796340b219d1bfa8552c81995017544a";
    devshell-flake.url = "github:numtide/devshell";
    emacsNg-src = { url = "github:emacs-ng/emacs-ng"; flake = false; };
    flake-compat = { url = "github:edolstra/flake-compat"; flake = false; };
    rust-overlay = { url = "github:oxalica/rust-overlay"; inputs.nixpkgs.follows = "nixpkgs"; };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, master, emacs-overlay, emacsNg-src, flake-compat, rust-overlay, flake-utils, devshell-flake }:
    { }
    //
    (flake-utils.lib.eachSystem [ "x86_64-linux" "x86_64-darwin" ]
      (system:
        let
          unstable = final: prev: {
            inherit ((import master) { inherit system; })
              rustracer;
          };
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              self.overlay
              emacs-overlay.overlay
              rust-overlay.overlay
              devshell-flake.overlay
              unstable
            ];
            config = { };
          };
        in
        rec {
          devShell = with pkgs; let
            custom-llvmPackages = llvmPackages_10;
          in
          devshell.mkShell {
            imports = [ ./nix/rust.nix ];

            packages = [
              custom-llvmPackages.clang
              nixpkgs-fmt
              rustracer
            ];

            env = [
              {
                name = "LIBCLANG_PATH";
                value = "${custom-llvmPackages.libclang}/lib";
              }
              {
                name = "CACHIX_AUTH_TOKEN";
                prefix = ''
                  export CACHIX_AUTH_TOKEN="$(cat nix/cachix-key.secrets)"
                '';
              }
            ];

            commands = with pkgs; [
              {
                name = "emacsNg";
                command = ''
                  $(nix-build . --option substituters "https://emacsng.cachix.org" --option trusted-public-keys "emacsng.cachix.org-1:i7wOr4YpdRpWWtShI8bT6V7lOTnPeI7Ho6HaZegFWMI=" \
                  --no-out-link)/bin/emacs
                '';
                help = ''
                  launch emacsNg
                '';
              }
              {
                name = "emacs-bumpup";
                command = ''
                  nix flake lock --update-input emacsNg-src
                '';
                help = ''
                  Bumpup EmacsNg src
                '';
              }
              {
                name = "copy-deps";
                command = ''
                  cp -rf --no-preserve=mode,ownership ${emacsNg-rust}/.cargo/ $@
                '';
                help = ''
                  copy emacsNg rust deps path to where
                '';
              }
              {
                name = "push-cachix";
                command = ''
                  nix-build | cachix push emacsng
                '';
                help = ''
                  push emacsNg binary cache to Cachix
                '';
              }
              {
                name = "build-bindings";
                command = ''
                  cargo build --manifest-path=./rust_src/remacs-bindings/Cargo.toml
                '';
                help = ''
                  cargo build remacs-bindings
                '';
              }
            ];
          };


          apps = {
            emacsNg = flake-utils.lib.mkApp { drv = packages.emacsNg; exePath = "/bin/emacs"; };
          };

          defaultApp = apps.emacsNg;

          defaultPackage = pkgs.emacsNg;
          packages = flake-utils.lib.flattenTree
            {
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
                src = "${emacsNg-src}/rust_src/remacs-lib";
                sourceRoot = null;
                name = "remacsLibDeps";
                cargoUpdateHook = doVersionedUpdate;
                sha256 = "sha256-TtL+zfr4iaCG9I4NJ1i18c4aIgGyPfYfryHVAzBl3eI=";
                inherit installPhase;
              };

              remacsBindings = prev.rustPlatform.fetchCargoTarball {
                src = "${emacsNg-src}/rust_src/remacs-bindings";
                sourceRoot = null;
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
                sha256 = "sha256-8Es749ddZ3yxBnij8swIda6AKlHJffWaLV2yIi7oRqU=";
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
              custom-llvmPackages = prev.llvmPackages_10;
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
                "--with-harfbuzz"
                "--with-compress-install"
                "--with-zlib"
              ];

              preBuild = let arch = rust.toRustTarget stdenv.hostPlatform; in
                (old.preBuild or "") + ''
                  _librusty_v8_setup() {
                      for v in "$@"; do
                        install -D ${final.librusty_v8} "rust_src/target/$v/gn_out/obj/librusty_v8.a"
                      done
                    }
                    _librusty_v8_setup "debug" "release" "${arch}/release"
                      sed -i 's|deno = { git = "https://github.com/DavidDeSimone/deno", branch = "emacs-ng"|deno = { version = "1.8.1"|' rust_src/Cargo.toml
                      sed -i 's|deno_runtime = { git = "https://github.com/DavidDeSimone/deno", branch = "emacs-ng"|deno_runtime = { version = "0.9.3"|' rust_src/Cargo.toml
                      sed -i 's|deno_core = { git = "https://github.com/DavidDeSimone/deno"|deno_core = { version = "0.80.2"|' rust_src/Cargo.toml
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
