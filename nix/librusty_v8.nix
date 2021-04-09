{ rust, stdenv, fetchurl }:

let
  arch = rust.toRustTarget stdenv.hostPlatform;
  fetch_librusty_v8 = args: fetchurl {
    name = "librusty_v8-${args.version}";
    url = "https://github.com/denoland/rusty_v8/releases/download/v${args.version}/librusty_v8_release_${arch}.a";
    sha256 = args.shas.${stdenv.hostPlatform.system};
    meta = { inherit (args) version; };
  };
in
fetch_librusty_v8 {
  version = "0.16.0";
  shas = {
    x86_64-linux = "sha256-YwPtdajqWohD1g1QJUAN0pDhy6R4REKfbzM6RhS1WJw=";
    x86_64-darwin = "sha256-NmgE7/rrIQiu2GyEv0CnzbwpqtLo2wVFjt2fhfyZ5KI=";
    # aarch64-linux = "sha256-yeDcrxEp3qeE6/NWEc1v7VoHjlgppIOkcHTNVksXNsM=";
    # aarch64-darwin = "sha256-aq2Kjn8QSDMhNg8pEbXkJCHUKmDTNnitq42SDDVyRd4=";
  };
}
