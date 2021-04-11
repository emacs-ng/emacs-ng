{ lib, config, pkgs, ... }:
let
  cfg = config.language.rust;
in
with lib;
{
  options.language.rust = {
    overlaySet = mkOption {
      type = types.attrs;
      default = pkgs.rust-bin.nightly."2021-01-14";
    };

    rustSet = mkOption {
      type = types.attrs;
      default = pkgs.rustPackages;
      description = "Which rust package set to use";
    };

    rustPackagesSet = mkOption {
      type = types.listOf types.str;
      default = [
        "clippy"
      ];
      description = "Which rust tools to pull from the platform package set";
    };

    toolchain = mkOption {
      type = types.listOf types.str;
      default = [
        "rustc"
        "cargo"
        "rustfmt"
      ];
      description = "Which rust tools to pull from the rust overlay";
    };
  };

  config = {
    env = [{
      # Used by tools like rust-analyzer
      name = "RUST_SRC_PATH";
      value = (toString cfg.overlaySet.rust-src) + "/lib/rustlib/src/rust/library";
    }];

    devshell.packages = map (tool: cfg.rustSet.${tool}) cfg.rustPackagesSet
      ++ map (tool: cfg.overlaySet.${tool}) cfg.toolchain;
  };
}
