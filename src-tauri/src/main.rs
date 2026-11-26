#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod dashboard;
mod diagnostics;
mod editor;
mod geometry;
mod native;
mod session;
mod settings;
mod shortcuts;
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
}

#[tauri::command]
async fn dashboard_info(
    shortcuts: State<'_, Shortcuts>,
    sessions: State<'_, Sessions>,
    diagnostics: State<'_, Diagnostics>,
) -> Result<DashboardInfo, String> {
    Ok(DashboardInfo {
        shortcuts: shortcuts.status(),
        phase: sessions.phase(),
        last: diagnostics.latest(),
    })
}

#[tauri::command]
fn dashboard_ready(diagnostics: State<'_, Diagnostics>) {
    diagnostics.dashboard_ready();
}

#[tauri::command]
fn start_capture(app: AppHandle, mode: Mode) -> Result<(), String> {
    session::start(&app, mode, true, true)
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
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                match window.label() {
                    "editor" => editor::closed(window.app_handle()),
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
            editor::load_edit_image,
            editor::copy_edited_image,
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
