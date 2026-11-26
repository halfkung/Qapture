use std::sync::Mutex;

use tauri::{
    AppHandle, State, WebviewWindowBuilder,
    ipc::{InvokeBody, Request, Response},
};

use crate::editor::Image;

#[derive(Default)]
pub struct Pin(Mutex<Option<Vec<u8>>>);

impl Pin {
    pub fn clear(&self) {
        self.0.lock().unwrap().take();
    }
}

#[tauri::command]
pub async fn pin_image(
    app: AppHandle,
    state: State<'_, Pin>,
    request: Request<'_>,
) -> Result<(), String> {
    let InvokeBody::Raw(packet) = request.body() else {
        return Err("The image to pin is missing.".into());
    };
    let (width, height, bytes) = Image::from_packet(packet.clone())?.into_packet();
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "pin")
        .ok_or("Pinned window configuration is missing.")?;
    {
        let mut content = state.0.lock().unwrap();
        if content.is_some() {
            return Err("Close the current pinned image first.".into());
        }
        *content = Some(bytes);
    }
    let scale = (540.0 / f64::from(width))
        .min(380.0 / f64::from(height))
        .min(1.0);
    let result = WebviewWindowBuilder::from_config(&app, config).and_then(|window| {
        window
            .inner_size(
                (f64::from(width) * scale).max(160.0),
                (f64::from(height) * scale).max(100.0),
            )
            .build()
    });
    if let Err(error) = result {
        state.clear();
        return Err(format!("Could not pin the image. {error}"));
    }
    Ok(())
}

#[tauri::command]
pub fn load_pin_image(state: State<'_, Pin>) -> Result<Response, String> {
    state
        .0
        .lock()
        .unwrap()
        .as_ref()
        .cloned()
        .map(Response::new)
        .ok_or("The pinned image was closed.".into())
}
