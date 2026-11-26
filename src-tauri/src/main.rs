#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod dashboard;
mod diagnostics;
mod editor;
mod export;
mod geometry;
mod native;
mod pin;
mod session;
mod settings;
mod shortcuts;
mod startup;
mod storage;
mod tray;

use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Manager, RunEvent, State};

use diagnostics::{Diagnostics, Report};
use session::{Mode, Phase, Sessions};
use shortcuts::{ShortcutStatus, Shortcuts};

#[derive(Serialize)]
struct DashboardInfo {
    shortcuts: ShortcutStatus,
    phase: Phase,
    last: Option<Report>,
    history_requested: bool,
}

#[tauri::command]
async fn dashboard_info(
    shortcuts: State<'_, Shortcuts>,
    sessions: State<'_, Sessions>,
    diagnostics: State<'_, Diagnostics>,
    history_request: State<'_, dashboard::HistoryRequest>,
) -> Result<DashboardInfo, String> {
    Ok(DashboardInfo {
        shortcuts: shortcuts.status(),
        phase: sessions.phase(),
        last: diagnostics.latest(),
        history_requested: history_request.take(),
    })
}

#[tauri::command]
fn dashboard_ready(diagnostics: State<'_, Diagnostics>) {
    diagnostics.dashboard_ready();
}

#[tauri::command]
fn start_capture(
    app: AppHandle,
    mode: Mode,
    options: Option<session::CaptureOptions>,
) -> Result<(), String> {
    session::start(&app, mode, true, true, options.unwrap_or_default())
}

#[tauri::command]
fn cancel_capture(sessions: State<'_, Sessions>) {
    sessions.cancel();
}

#[tauri::command]
async fn set_shortcut(
    app: AppHandle,
    mode: Mode,
    binding: String,
) -> Result<ShortcutStatus, String> {
    app.state::<Shortcuts>().update(&app, mode, binding)
}

#[tauri::command]
async fn recover_shortcut(app: AppHandle, mode: Mode) -> Result<ShortcutStatus, String> {
    app.state::<Shortcuts>().recover(&app, mode)
}

#[tauri::command]
fn set_shortcut_recording(shortcuts: State<'_, Shortcuts>, recording: bool) {
    shortcuts.set_recording(recording);
}

#[tauri::command]
async fn set_editor_text_input(app: AppHandle, active: bool) {
    let active = active
        && app
            .get_webview_window("editor")
            .is_some_and(|window| window.is_focused().unwrap_or(false));
    app.state::<Shortcuts>().pause_for_text(&app, active);
}

fn main() {
    let started = Instant::now();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            dashboard::request(app)
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(Sessions::new())
        .manage(Diagnostics::new(started))
        .manage(editor::Editor::default())
        .manage(pin::Pin::default())
        .manage(dashboard::HistoryRequest::default())
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                match window.label() {
                    "editor" => {
                        let app = window.app_handle().clone();
                        tauri::async_runtime::spawn(async move {
                            app.state::<Shortcuts>().pause_for_text(&app, false);
                            editor::closed(&app);
                        });
                    }
                    "pin" => window.app_handle().state::<pin::Pin>().clear(),
                    "main" => window
                        .app_handle()
                        .state::<Shortcuts>()
                        .set_recording(false),
                    _ => {}
                }
            }
        })
        .setup(|app| {
            let store = settings::Store::new(&app.path().app_config_dir()?);
            app.manage(Shortcuts::new(store));
            app.manage(storage::Storage::new(app.path().app_local_data_dir()?));
            app.state::<Shortcuts>().initialize(app.handle());
            tray::install(app)?;
            if !std::env::args_os().any(|arg| arg == "--tray") {
                dashboard::show(app.handle())?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            dashboard_info,
            dashboard_ready,
            start_capture,
            cancel_capture,
            set_shortcut,
            recover_shortcut,
            set_shortcut_recording,
            set_editor_text_input,
            editor::load_edit_image,
            editor::copy_edited_image,
            export::save_edited_image,
            export::save_to_export_folder,
            export::auto_save_image,
            editor::load_edit_document,
            storage::output_preferences,
            storage::set_output_preferences,
            storage::list_projects,
            storage::keep_project,
            storage::delete_project,
            storage::clear_projects,
            storage::reopen_project,
            storage::import_image,
            startup::startup_enabled,
            startup::set_startup,
            pin::pin_image,
            pin::load_pin_image,
        ])
        .build(tauri::generate_context!())
        .unwrap_or_else(|error| {
            eprintln!("Qapture could not start: {error}");
            std::process::exit(1);
        });
    app.run(|_, event| {
        if let RunEvent::ExitRequested {
            code: None, api, ..
        } = event
        {
            api.prevent_exit();
        }
    });
}
