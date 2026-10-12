//! Build script for super_player.
//! Automatically coordinates native libmpv linking across Linux, Windows, and Android.

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    println!("cargo:rerun-if-env-changed=LIBMPV_SOURCE");
    println!("cargo:rerun-if-env-changed=MPV_ANDROID_LIB_DIR");

    match target_os.as_str() {
        "windows" => {
            // Windows x64: link against mpv.lib generated from mpv.def or in C:\mpv-dev
            let mpv_dir = std::env::var("LIBMPV_SOURCE").unwrap_or_else(|_| "C:\\mpv-dev".to_string());
            println!("cargo:rustc-link-search=native={}", mpv_dir);
            println!("cargo:rustc-link-lib=mpv");
        }
        "android" => {
            // Android ARM64: link against libmpv.so located in NDK or custom path
            if let Ok(dir) = std::env::var("MPV_ANDROID_LIB_DIR") {
                println!("cargo:rustc-link-search=native={}", dir);
            }
            println!("cargo:rustc-link-lib=mpv");
        }
        "linux" => {
            // Linux x64: probe via pkg-config
            if let Err(err) = pkg_config::probe_library("mpv") {
                println!("cargo:warning=pkg-config failed to locate mpv: {}. Using fallback link.", err);
                println!("cargo:rustc-link-lib=mpv");
            }
        }
        _ => {
            println!("cargo:rustc-link-lib=mpv");
        }
    }
}
