use std::sync::Mutex;

use tauri::{
    AppHandle, Manager, State, WebviewWindowBuilder,
    ipc::{InvokeBody, Request, Response},
};

use crate::{
    dashboard,
    geometry::Rect,
    native::{clipboard, screen::Frame},
    session::{self, Sessions},
};

pub struct Image {
    width: i32,
    height: i32,
    packet: Vec<u8>,
}

impl Image {
    pub fn from_frame(frame: &Frame, rect: Rect) -> Result<Self, String> {
        let rect = rect.intersection(frame.image_rect());
        if !rect.has_area() {
            return Err("The selection is empty.".into());
        }
        Ok(crop(
            frame.dib.pixels(),
            frame.bounds.width() as usize,
            rect,
        ))
    }
}

fn crop(source: &[u8], source_width: usize, rect: Rect) -> Image {
    let width = rect.width();
    let height = rect.height();
    // Two little-endian u32 dimensions, then top-down opaque RGBA.
    let mut packet = Vec::with_capacity(8 + width as usize * height as usize * 4);
    packet.extend_from_slice(&width.to_le_bytes());
    packet.extend_from_slice(&height.to_le_bytes());
    for y in rect.top..rect.bottom {
        let offset = (y as usize * source_width + rect.left as usize) * 4;
        for bgra in source[offset..offset + width as usize * 4].chunks_exact(4) {
            packet.extend_from_slice(&[bgra[2], bgra[1], bgra[0], 255]);
        }
    }
    Image {
        width,
        height,
        packet,
    }
}

struct EditSession {
    image: Image,
    return_to_dashboard: bool,
}

#[derive(Default)]
pub struct Editor(Mutex<Option<EditSession>>);

pub fn open(app: &AppHandle, image: Image, return_to_dashboard: bool) -> Result<(), String> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "editor")
        .ok_or("The editor window is not configured.")?;
    let editor = app.state::<Editor>();
    *editor.0.lock().unwrap() = Some(EditSession {
        image,
        return_to_dashboard,
    });
    let result = WebviewWindowBuilder::from_config(app, config).and_then(|builder| builder.build());
    match result {
        Ok(window) => {
            let _ = window.set_focus();
            Ok(())
        }
        Err(error) => {
            editor.0.lock().unwrap().take();
            Err(format!("Could not open the editor. {error}"))
        }
    }
}

pub fn closed(app: &AppHandle) {
    let session = app.state::<Editor>().0.lock().unwrap().take();
    if let Some(edit) = session {
        app.state::<Sessions>().finish_editing();
        session::notify(app);
        if edit.return_to_dashboard {
            dashboard::request(app);
        }
    }
}

#[tauri::command]
pub fn load_edit_image(editor: State<'_, Editor>) -> Result<Response, String> {
    let session = editor.0.lock().unwrap();
    let edit = session.as_ref().ok_or("There is no capture to edit.")?;
    Ok(Response::new(edit.image.packet.clone()))
}

#[tauri::command]
pub async fn copy_edited_image(
    editor: State<'_, Editor>,
    request: Request<'_>,
) -> Result<(), String> {
    let InvokeBody::Raw(pixels) = request.body() else {
        return Err("The edited image was not sent as pixels.".into());
    };
    let session = editor.0.lock().unwrap();
    let edit = session.as_ref().ok_or("There is no capture to copy.")?;
    clipboard::copy_rgba(edit.image.width, edit.image.height, pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_packet_contains_only_the_selected_pixels_in_rgba_order() {
        let source = [1, 2, 3, 0, 10, 20, 30, 0, 4, 5, 6, 0, 40, 50, 60, 0];
        let image = crop(
            &source,
            2,
            Rect {
                left: 1,
                top: 0,
                right: 2,
                bottom: 2,
            },
        );
        assert_eq!(&image.packet[..8], &[1, 0, 0, 0, 2, 0, 0, 0]);
        assert_eq!(&image.packet[8..], &[30, 20, 10, 255, 60, 50, 40, 255]);
    }
}
