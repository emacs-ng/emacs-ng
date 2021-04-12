# Using Nix Develop Environment

## Install nix from shell

    sh <(curl -L https://nixos.org/nix/install) --daemon

then `exec bash -l` reload your bash to load the nix-env

## Install flake feature of nix

    nix-env -iA nixpkgs.nixUnstable
    echo "experimental-features = nix-command flakes" | sudo tee -a /etc/nix/nix.conf
    sudo pkill nix-daemon

 NOTE: `reload bash` and check your nix-version first

    nix --version
    nix (Nix) 2.4pre20210326_dd77f71
  
## Clone Emacs-ng by nix-shell and enable emacs cachix 

```bash
nix-env -iA cachix -f https://cachix.org/api/v1/install
exec bash -l
cachix use emacsng # make sure you have saw the output like: Configured https://emacsng.cachix.org binary cache in /home/test/.config/nix/nix.conf
nix-shell -p git --command "git clone https://github.com/emacs-ng/emacs-ng.git && cd emacs-ng && nix-shell"
```

## Setting up Rust develop environment

### change rust version
1. Night version
   - located `nix/rust.nix` modify the `2021-01-14`to which your want.([rustOverlay-NightlyCheck](https://github.com/oxalica/rust-overlay/tree/master/manifests/nightly/default.nix)) 
     Example: `default = pkgs.rust-bin.nightly."2021-03-23";`

2. stable version

 - example : `default = pkgs.rust-bin.stable."1.50.0";`
   ([rustOverlay-StableCheck](https://github.com/oxalica/rust-overlay/tree/master/manifests/stable/default.nix)) 

3. beta version

- example : `default = pkgs.rust-bin.beta."2021-03-06";`
   ([rustOverlay-StableCheck](https://github.com/oxalica/rust-overlay/tree/master/manifests/beta/default.nix)) 

### Add Package from RustOverly

the RustOverly supported Packages list that you can find here ([packages-list](https://github.com/oxalica/rust-overlay/blob/master/manifests/profiles.nix))

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

first search any package in [search-package](https://search.nixos.org/packages?channel=unstable&) then add the name to 

```nix
     devshell.packages = map (tool: cfg.rustPackages.${tool}) cfg.rustPackagesSet
      ++ map (tool: cfg.rustOverlay.${tool}) cfg.rustOverlaySet ++ (with pkgs;[
      #custom nixpkgs packages
      rustracer
      <add package here>
    ]);
 ```

## Reload all of envs when you changed something

- normally, we can re-enter `nix-shell` to reload envrs. But for this project, we are using direnv to load and unload environment variables in an convenient way.

### Recommended way -> Direnv

 Install direnv by Nix

    nix-env -i direnv

and hook direnv to your bash [direnv-hook](https://direnv.net/docs/hook.html)

- using `direnv allow`  or `direnv deny` to enable or disable load `nix-shell` in current path

- using `direnv reload` to reload `nix-shell`

- Also, you can put as follows to `~/.config/direnv/direnvrc` watching the envrs every time changes that could be reload automatically.

```conf
use_flake() {
  watch_file flake.nix
  watch_file flake.lock
  eval "$(nix print-dev-env)"
}
```

### Add direnv support with Emacs

- <https://github.com/purcell/envrc>

Install direnv to Doom Emacs, Example

```elisp
(package! envrc :recipe (:host github :repo "purcell/envrc"))
(use-package! envrc
:hook (after-init . envrc-global-mode)
)
```
 then `M-x` typing `envrv-<**>` check related commands similar to native direnv-commands

### write a custom command or environment variable

- locaed `nix/commands.toml`

- Add custom command as following: 
 Example

```toml
[[commands]]
name = ""
command = "cargo build --manifest-path=./rust_src/remacs-bindings/Cargo.toml"
help = "cargo build remacs-bindings"
category = "rust-build"
```

- Add custom env variable as following: 

```toml
[[env]]
name = "TEST"
vale = "/bin/test"
#prefix = "$( cd "$(dirname "$\{\BASH_SOURCE [ 0 ]}")"; pwd )" can be prefix 
```

## Building Emacs-ng in develop mode

- located `flake.nix` commented `emacsNg-src` to `"./."` 
  
  Example:

```nix
        let
          #emacsNgSource = emacsNg-src;
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

NOTICE :  `nix-build` action in sandbox mode. If you want to modify something or patch it, please put it this way to the corresponding step. For example:

```nix
preConfigure = (old.preConfigure or "") + ''
  <modify shell>
            '';
            
patches = (old.patches or [ ]) ++ [ 
./nix/<your-pathc-file.patch>
];
```

if everything looks like good, run `nix-build` right now.
