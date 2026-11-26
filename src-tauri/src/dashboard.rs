use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, WebviewWindowBuilder};

use crate::session::{self, Phase, Sessions};

#[derive(Default)]
pub struct HistoryRequest(AtomicBool);

impl HistoryRequest {
    pub fn take(&self) -> bool {
        self.0.swap(false, Ordering::AcqRel)
    }
}

pub fn request_history(app: &AppHandle) {
    app.state::<HistoryRequest>()
        .0
        .store(true, Ordering::Release);
    request(app);
}

pub fn show(app: &AppHandle) -> tauri::Result<()> {
    match app.state::<Sessions>().phase() {
        Phase::Idle => {}
        Phase::Editing => {
            if let Some(window) = app.get_webview_window("editor") {
                window.unminimize()?;
                window.show()?;
                let _ = window.set_focus();
            }
            return Ok(());
        }
        _ => return Ok(()),
    }
    let window = if let Some(window) = app.get_webview_window("main") {
        window
    } else {
        let config = &app.config().app.windows[0];
        WebviewWindowBuilder::from_config(app, config)?.build()?
    };
    window.unminimize()?;
    window.show()?;
    let _ = window.set_focus();
    session::notify(app);
    Ok(())
}

pub fn request(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = show(&app) {
            eprintln!("Could not open the Qapture dashboard: {error}");
        }
    });
}
