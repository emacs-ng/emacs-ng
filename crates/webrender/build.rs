extern crate codegen;
use cfg_aliases::cfg_aliases;

use codegen::generate_crate_exports;
use codegen::BuildError;

fn main() -> Result<(), BuildError> {
    generate_crate_exports()?;
    // Setup cfg aliases
    cfg_aliases! {
        android_platform: { target_os = "android" },
        wasm_platform: { target_arch = "wasm32" },
        macos_platform: { target_os = "macos" },
        ios_platform: { target_os = "ios" },
        windows_platform: { target_os = "windows" },
        apple: { any(target_os = "ios", target_os = "macos") },
        free_unix: { all(unix, not(apple), not(android_platform)) },

        x11_platform: { all(feature = "x11", free_unix, not(wasm), use_winit)},
        wayland_platform: { all(feature = "wayland", free_unix, not(wasm), use_winit) },
    }
    Ok(())
}
