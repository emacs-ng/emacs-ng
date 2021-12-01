# Using Nix Develop Environment

## Install nix from shell

    sh <(curl -L https://nixos.org/nix/install) --daemon

then `exec bash -l` reload your bash to load the nix-env

## Enable Flake Feature Of Nix

    echo "experimental-features = nix-command flakes" | sudo tee -a /etc/nix/nix.conf
    sudo pkill nix-daemon

NOTE: `reload bash` and check your nix-version first

    nix --version
    nix (Nix) 2.4

## Nix flake feature already in emacsNg

    nix run github:emacs-ng/emacs-ng (launch emacs locally )
    nix build github:emacs-ng/emacs-ng (build emacs locally)
    nix develop github:emacs-ng/emacs-ng (enter develop emacs environment locally)

Also, you can run native nix commands (Nix Stable Version) under `emacs-ng` repo, such as

    nix-shell
    nix-build

#### Using `cachix` to (download binary cache) speed up build

1. Without installing cachix

```bash
nix build github:emacs-ng/emacs-ng --option substituters "https://emacsng.cachix.org" --option trusted-public-keys "emacsng.cachix.org-1:i7wOr4YpdRpWWtShI8bT6V7lOTnPeI7Ho6HaZegFWMI=" -o emacsNg
ls -il emacs
#or check ./result/bin/emacs
nix-build --option substituters "https://emacsng.cachix.org" --option trusted-public-keys "emacsng.cachix.org-1:i7wOr4YpdRpWWtShI8bT6V7lOTnPeI7Ho6HaZegFWMI="
```

2. Installing cachix

```bash
nix-env -iA cachix -f https://cachix.org/api/v1/install #install cachix
exec bash -l
cachix use emacsng # make sure you have saw the output like: Configured https://emacsng.cachix.org binary cache in /home/test/.config/nix/nix.conf
#then
nix-build
#or
nix build github:emacs-ng/emacs-ng -o emacsNG
ls -il emacsNG
```

## Clone Emacs Ng By Nix-Shell And Enable Emacs Cachix

```bash
nix-env -iA cachix -f https://cachix.org/api/v1/install
exec bash -l
cachix use emacsng # make sure you have saw the output like: Configured https://emacsng.cachix.org binary cache in /home/test/.config/nix/nix.conf
nix-shell -p git --command "git clone https://github.com/emacs-ng/emacs-ng.git && cd emacs-ng && nix-shell"
```

## Setting Up Rust Development Environment

### Change Rust Version
1. Nightly Version
   - located `nix/rust.nix` modify the `2021-01-14`to which your want.([rustOverlay-NightlyCheck](https://github.com/oxalica/rust-overlay/tree/master/manifests/nightly/default.nix))
     Example: `default = pkgs.rust-bin.nightly."2021-03-23";`

2. Stable version

 - example : `default = pkgs.rust-bin.stable."1.50.0";`
   ([rustOverlay-StableCheck](https://github.com/oxalica/rust-overlay/tree/master/manifests/stable/default.nix))

3. Beta version

- example : `default = pkgs.rust-bin.beta."2021-03-06";`
   ([rustOverlay-StableCheck](https://github.com/oxalica/rust-overlay/tree/master/manifests/beta/default.nix))

### Add Package from RustOverly

The RustOverly supported Packages list that you can find here ([packages-list](https://github.com/oxalica/rust-overlay/blob/master/manifests/profiles.nix))

- located `nix/rust.nix`
```nix
    rustOverlaySet = mkOption {
      type = types.listOf types.str;
      default = [
        "rustc"
        "cargo"
        "rustfmt-preview"
        "<add other package which you can found in package-list>"
      ];
      description = "Which rust tools to pull from the rust overlay
      https://github.com/oxalica/rust-overlay/blob/master/manifests/profiles.nix";
    };
```
### Add rustPackages from nixpkgs channel

- located `nix/rust.nix` and the default nix expression like this  `rustPackages.clippy`. In this cause, most of rust toolchain installed from `rustOverly`

```nix
    rustPackagesSet = mkOption {
      type = types.listOf types.str;
      default = [
        "clippy"
        "<add custom Rust toolchain package>"
      ];
      description = "Which rust tools to pull from the nixpkgs channel package set
      check valid package from https://search.nixos.org/packages?channel=unstable";
      ";
    };
```

### Add nixpkgs package to current Environment

- located `nix/rust.nix`

first, search for package in [search-package](https://search.nixos.org/packages?channel=unstable&) then add the name of package to:

```nix
devshell.packages = map (tool: cfg.rustPackages.${tool}) cfg.rustPackagesSet
      ++ map (tool: cfg.rustOverlay.${tool}) cfg.rustOverlaySet ++ (with pkgs;[
      #custom nixpkgs packages
      rustracer
      <add package here>
    ]);
 ```

## Reload all of environments when you changed something

- normally, we can re-enter `nix-shell` to reload environments. But for this project, we are using direnv to load and unload environment variables in an convenient way.

### Recommended Way -> Direnv

 Install direnv by Nix

    nix-env -i direnv

and hook direnv to your bash [direnv-hook](https://direnv.net/docs/hook.html)

- using `direnv allow`  or `direnv deny` to enable or disable load `nix-shell` in current path

- using `direnv reload` to reload `nix-shell`

- Also, you can put as follows to `~/.config/direnv/direnvrc` watching the envrs every time changes that could be reload automatically.


```bash
use_flake() {
  watch_file flake.nix
  watch_file flake.lock
  eval "$(nix print-dev-env)"
}
```

### Add Direnv Support With Emacs

- <https://github.com/purcell/envrc>

Install direnv to Doom Emacs, Example

```elisp
(package! envrc :recipe (:host github :repo "purcell/envrc"))
(use-package! envrc
:hook (after-init . envrc-global-mode)
)
```
 then `M-x` typing `envrv-<**>` check related commands similar to native direnv-commands

### Write A Custom Command Or Environment Variable

- locaed `nix/commands.toml`

- Add custom command as following:
 Example

```toml
[[commands]]
name = ""
command = "cargo build --manifest-path=./rust_src/ng-bindgen/Cargo.toml"
help = "cargo build ng-bindgen"
category = "rust-build"
```

- Add custom env variable as following:

```toml
[[env]]
name = "TEST"
vale = "/bin/test"
#prefix = "$( cd "$(dirname "$\{\BASH_SOURCE [ 0 ]}")"; pwd )" can be prefix
```

## Building Emacs-ng in development mode

- located `flake.nix` commented `emacsNG-src` to `"./."`

  Example:

```nix
        let
          #emacsNgSource = emacsNG-src;
          emacsNgSource = "./.";
        in
```

then `nix-build` it;

### Add custom configFlags to emacsNg build process

- located `flake.nix`


```nix
configureFlags = (old.configureFlags or [ ]) ++ [
        "--with-json"
        "--with-threads"
        "--with-included-regex"
        "--with-harfbuzz"
        "--with-compress-install"
        "--with-zlib"
        "<custom flags>"
];
```

NOTICE:  `nix-build` action in sandbox mode. If you want to modify something or patch it, please put it this way to the corresponding step.

For example:

```nix
preConfigure = (old.preConfigure or "") + ''
  <modify shell>
            '';

patches = (old.patches or [ ]) ++ [
./nix/<your-pathc-file.patch>
];
```

if everything looks good, you can run `nix-build` right now.
