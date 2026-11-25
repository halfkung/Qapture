use tauri::{AppHandle, Manager, WebviewWindowBuilder};

use crate::session::{self, Phase, Sessions};

pub fn show(app: &AppHandle) -> tauri::Result<()> {
    if app.state::<Sessions>().phase() != Phase::Idle {
        return Ok(());
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
