fn main() {
    if cfg!(target_os = "macos") {
        std::env::set_var("MACOSX_DEPLOYMENT_TARGET", "10.15");
    }
    tauri_build::build()
}
