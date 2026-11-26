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
    pub fn into_packet(self) -> (i32, i32, Vec<u8>) {
        (self.width, self.height, self.packet)
    }

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
    document: Option<serde_json::Value>,
}

#[derive(Default)]
pub struct Editor(Mutex<Option<EditSession>>);

pub fn open(app: &AppHandle, image: Image, return_to_dashboard: bool) -> Result<(), String> {
    open_document(app, image, return_to_dashboard, None)
}

pub fn open_document(
    app: &AppHandle,
    image: Image,
    return_to_dashboard: bool,
    document: Option<serde_json::Value>,
) -> Result<(), String> {
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
        document,
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

impl Image {
    pub fn from_packet(packet: Vec<u8>) -> Result<Self, String> {
        let Some(header) = packet.get(..8) else {
            return Err("The image is incomplete.".into());
        };
        let width = i32::from_le_bytes(header[..4].try_into().unwrap());
        let height = i32::from_le_bytes(header[4..].try_into().unwrap());
        if width < 1
            || height < 1
            || i64::from(width) * i64::from(height) > 64 * 1024 * 1024
            || packet.len() != 8 + width as usize * height as usize * 4
        {
            return Err("The image has invalid or unsupported dimensions.".into());
        }
        Ok(Self {
            width,
            height,
            packet,
        })
    }
}

impl Editor {
    pub fn with_original<T>(
        &self,
        use_image: impl FnOnce(&[u8]) -> Result<T, String>,
    ) -> Result<T, String> {
        let session = self.0.lock().unwrap();
        let edit = session.as_ref().ok_or("There is no capture open.")?;
        use_image(&edit.image.packet)
    }
}

#[tauri::command]
pub fn load_edit_document(editor: State<'_, Editor>) -> Option<serde_json::Value> {
    editor.0.lock().unwrap().as_ref()?.document.clone()
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
    let image = &edit.image;
    if !request.headers().contains_key("x-qapture-cropped") {
        return clipboard::copy_rgba(image.width, image.height, pixels);
    }
    let (width, height, rgba) = cropped_pixels(pixels, image)?;
    clipboard::copy_rgba(width, height, rgba)
}

fn cropped_pixels<'a>(packet: &'a [u8], image: &Image) -> Result<(i32, i32, &'a [u8]), String> {
    let Some(header) = packet.get(..8) else {
        return Err("The edited image has no size header.".into());
    };
    let width = i32::from_le_bytes(header[..4].try_into().unwrap());
    let height = i32::from_le_bytes(header[4..].try_into().unwrap());
    if width < 1
        || height < 1
        || width > image.width
        || height > image.height
        || packet.len() != 8 + width as usize * height as usize * 4
    {
        return Err("The edited image has invalid dimensions.".into());
    }
    Ok((width, height, &packet[8..]))
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

    #[test]
    fn crop_packet_requires_exact_dimensions_and_size() {
        let image = Image::from_packet(vec![
            2, 0, 0, 0, 1, 0, 0, 0, 10, 20, 30, 255, 40, 50, 60, 255,
        ])
        .unwrap();
        let valid = [1, 0, 0, 0, 1, 0, 0, 0, 10, 20, 30, 255];
        assert_eq!(
            cropped_pixels(&valid, &image),
            Ok((1, 1, &[10, 20, 30, 255][..]))
        );
        assert!(cropped_pixels(&valid[..11], &image).is_err());
        let mut outside = valid;
        outside[0] = 3;
        assert!(cropped_pixels(&outside, &image).is_err());
    }
}
