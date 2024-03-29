extern crate codegen;
use cfg_aliases::cfg_aliases;

use codegen::generate_crate_exports;
use codegen::BuildError;

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
}
