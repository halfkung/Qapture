fn main() {
    let manifest = tauri_build::AppManifest::new().commands(&[
        "dashboard_info",
        "dashboard_ready",
        "start_capture",
        "cancel_capture",
        "set_shortcut",
    ]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("Could not configure Qapture");
}
