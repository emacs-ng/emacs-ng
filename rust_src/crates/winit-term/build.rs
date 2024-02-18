extern crate ng_bindgen;
use cfg_aliases::cfg_aliases;

use ng_bindgen::{generate_crate_exports, BuildError};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/wrterm.rs");
    println!("cargo:rerun-if-changed=src/term.rs");
    println!("cargo:rerun-if-changed=src/event_loop.rs");
    // TODO watch relevent files to re rerun, rs files under src?

    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Err(e) = generate_crate_exports(&path) {
        match e {
            BuildError::IOError(msg) => {
                eprintln!("{}", msg);
                std::process::exit(3);
            }
            BuildError::Lint(msg) => {
                msg.fail(1);
            }
        }
    }

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

        window_system_winit: { any(feature = "winit", feature = "tao") },
        window_system_pgtk: { all(feature = "pgtk", not(window_system_winit)) },
        have_window_system: { any(window_system_winit, window_system_pgtk) },
        use_winit: { feature = "winit" },
        use_tao: { all(feature = "tao", not(use_winit)) },

        // X11/wayland are winit specific
        x11_platform: { all(feature = "x11", free_unix, not(wasm), use_winit)},
        wayland_platform: { all(feature = "wayland", free_unix, not(wasm), use_winit) },
        use_tokio_select: { all(window_system_winit, feature = "tokio") },
        use_pselect: { all(window_system_winit, feature = "pselect", not(use_tokio_select)) },
        use_surfman: { feature = "surfman" },
        use_glutin: { all(feature = "glutin", not(use_surfman)) },
        use_gtk3: { all(feature = "gtk3", free_unix, not(use_surfman), not(use_glutin)) },
    }
}
