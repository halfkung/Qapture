use tauri::{
    App, Manager,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::{
    dashboard,
    session::{self, Mode, Sessions},
};

pub fn install(app: &mut App) -> tauri::Result<()> {
    let region = MenuItem::with_id(app, "region", "Capture region", true, None::<&str>)?;
    let full = MenuItem::with_id(app, "full", "Copy full desktop", true, None::<&str>)?;
    let window = MenuItem::with_id(app, "window", "Capture a window", true, None::<&str>)?;
    let history = MenuItem::with_id(app, "history", "Capture history", true, None::<&str>)?;
    let cancel = MenuItem::with_id(app, "cancel", "Cancel capture", true, None::<&str>)?;
    let dashboard = MenuItem::with_id(app, "dashboard", "Open Qapture", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Qapture", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &region, &full, &window, &cancel, &separator, &dashboard, &history, &quit,
        ],
    )?;

    let builder = TrayIconBuilder::with_id("qapture")
        .tooltip("Qapture")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "region" => session::start_background(app, Mode::Region, true),
            "full" => session::start_background(app, Mode::Full, true),
            "window" => session::start_background(app, Mode::Window, true),
            "cancel" => app.state::<Sessions>().cancel(),
            "dashboard" => dashboard::request(app),
            "history" => dashboard::request_history(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                dashboard::request(tray.app_handle());
            }
        });
    let builder = if let Some(icon) = app.default_window_icon() {
        builder.icon(icon.clone())
    } else {
        builder
    };
    builder.build(app)?;
    Ok(())
}
