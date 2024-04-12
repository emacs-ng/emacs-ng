extern crate codegen;
use cfg_aliases::cfg_aliases;

use codegen::generate_crate_exports;
use codegen::BuildError;

fn main() -> Result<(), BuildError> {
    // Setup cfg aliases
    cfg_aliases! {
        // Systems.
        android_platform: { target_os = "android" },
        wasm_platform: { target_arch = "wasm32" },
        macos_platform: { target_os = "macos" },
        ios_platform: { target_os = "ios" },
        windows_platform: { target_os = "windows" },
        apple: { any(target_os = "ios", target_os = "macos") },
        free_unix: { all(unix, not(apple), not(android_platform)) },
    }

    generate_crate_exports()?;
    Ok(())
}
