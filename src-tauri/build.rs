fn main() {
    let manifest = tauri_build::AppManifest::new().commands(&[
        "dashboard_info",
        "dashboard_ready",
        "start_capture",
        "cancel_capture",
        "set_shortcut",
        "recover_shortcut",
        "set_shortcut_recording",
        "set_editor_text_input",
        "load_edit_image",
        "copy_edited_image",
        "save_edited_image",
        "save_to_export_folder",
        "auto_save_image",
        "load_edit_document",
        "output_preferences",
        "set_output_preferences",
        "list_projects",
        "keep_project",
        "delete_project",
        "clear_projects",
        "reopen_project",
        "import_image",
        "startup_enabled",
        "set_startup",
        "pin_image",
        "load_pin_image",
    ]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("Could not configure Qapture");
}
