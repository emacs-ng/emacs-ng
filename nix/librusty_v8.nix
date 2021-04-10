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
  version = "0.20.0";
  shas = {
    x86_64-linux = "sha256-pTWNYQzChyYJh+afn1AMw/MxUE+Cv4k2FnM3+KDYCvg=";
    x86_64-darwin = "sha256-k0kS5NiITqW/WEFWe/Bnt7Z9HZp2YN19L7DvVlptrj4=";
    # aarch64-linux = "sha256-yeDcrxEp3qeE6/NWEc1v7VoHjlgppIOkcHTNVksXNsM=";
    # aarch64-darwin = "sha256-aq2Kjn8QSDMhNg8pEbXkJCHUKmDTNnitq42SDDVyRd4=";
  };
}
