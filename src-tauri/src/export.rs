use std::{ffi::OsString, fs, os::windows::ffi::OsStringExt, path::PathBuf};

use crate::{
    native::wide,
    storage::{self, Preferences, Storage},
};
use tauri::{
    State, WebviewWindow,
    ipc::{InvokeBody, Request},
};
use windows::{
    Win32::{
        Foundation::{ERROR_CANCELLED, HWND},
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
            CoTaskMemFree, CoUninitialize,
        },
        UI::Shell::{
            Common::COMDLG_FILTERSPEC, FOS_FORCEFILESYSTEM, FOS_OVERWRITEPROMPT, FOS_PATHMUSTEXIST,
            FileSaveDialog, IFileSaveDialog, IShellItem, SHCreateItemFromParsingName,
            SIGDN_FILESYSPATH,
        },
    },
    core::{HRESULT, PCWSTR},
};
#[derive(Clone, Copy)]
struct Format {
    extension: &'static str,
    label: &'static str,
    pattern: &'static str,
}

fn format(bytes: &[u8]) -> Result<Format, String> {
    let (extension, label, pattern) = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        ("png", "PNG image", "*.png")
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        ("jpg", "JPEG image", "*.jpg;*.jpeg")
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        ("webp", "WebP image", "*.webp")
    } else {
        return Err("The image encoder returned an unsupported format.".into());
    };
    Ok(Format {
        extension,
        label,
        pattern,
    })
}

#[tauri::command]
pub async fn save_edited_image(
    window: WebviewWindow,
    storage: State<'_, Storage>,
    request: Request<'_>,
) -> Result<bool, String> {
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err("The encoded image is missing.".into());
    };
    let format = format(bytes)?;
    let preferences = storage.preferences();
    let owner = window.hwnd().map_err(|error| error.to_string())?.0 as isize;
    let dialog = std::thread::Builder::new()
        .name("qapture-save-dialog".into())
        .spawn(move || save_path(owner, format, &preferences))
        .map_err(|error| error.to_string())?;
    let Some(path) = dialog
        .join()
        .map_err(|_| "Windows Save dialog stopped unexpectedly.")??
    else {
        return Ok(false);
    };
    let matching_extension = path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| {
            value.eq_ignore_ascii_case(format.extension)
                || (format.extension == "jpg" && value.eq_ignore_ascii_case("jpeg"))
        });
    if !matching_extension {
        return Err(format!(
            "Use a .{} extension for this format.",
            format.extension
        ));
    }
    fs::write(path, bytes).map_err(|error| format!("Could not save the image. {error}"))?;
    Ok(true)
}

#[tauri::command]
pub async fn auto_save_image(
    storage: State<'_, Storage>,
    request: Request<'_>,
) -> Result<(), String> {
    let preferences = storage.preferences();
    if !preferences.auto_save {
        return Ok(());
    }
    write_to_folder(&preferences, request)
}

#[tauri::command]
pub async fn save_to_export_folder(
    storage: State<'_, Storage>,
    request: Request<'_>,
) -> Result<(), String> {
    write_to_folder(&storage.preferences(), request)
}

fn write_to_folder(preferences: &Preferences, request: Request<'_>) -> Result<(), String> {
    use std::io::Write;
    if preferences.directory.is_empty() {
        return Err("Set an export folder in History or use Save as.".into());
    }
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err("The encoded image is missing.".into());
    };
    let format = format(bytes)?;
    let path = PathBuf::from(&preferences.directory).join(filename(&format, preferences));
    let pending = path.with_extension("tmp");
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&pending)
        .map_err(|error| format!("Could not open the export folder. {error}"))?;
    let result = file.write_all(bytes);
    drop(file);
    let result = result.and_then(|()| fs::rename(&pending, &path));
    if result.is_err() {
        let _ = fs::remove_file(pending);
    }
    result.map_err(|error| format!("Could not save the image. {error}"))
}

fn filename(format: &Format, preferences: &Preferences) -> String {
    format!(
        "{}-{}.{}",
        preferences.prefix,
        storage::stamp().trim_start_matches('0'),
        format.extension
    )
}

fn save_path(
    owner: isize,
    format: Format,
    preferences: &Preferences,
) -> Result<Option<PathBuf>, String> {
    let pick = || -> windows::core::Result<Option<PathBuf>> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()? };
        let selected = pick_path(owner, &format, preferences);
        unsafe { CoUninitialize() };
        selected
    };
    pick().map_err(|error| format!("Windows could not select a save path. {error}"))
}

fn pick_path(
    owner: isize,
    format: &Format,
    preferences: &Preferences,
) -> windows::core::Result<Option<PathBuf>> {
    let label = wide(format.label);
    let pattern = wide(format.pattern);
    let extension = wide(format.extension);
    let suggested = wide(&filename(format, preferences));
    // A dedicated COM thread keeps the Tauri event loop available while the dialog is open.
    unsafe {
        let dialog: IFileSaveDialog =
            CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)?;
        dialog.SetOptions(
            dialog.GetOptions()? | FOS_FORCEFILESYSTEM | FOS_OVERWRITEPROMPT | FOS_PATHMUSTEXIST,
        )?;
        dialog.SetFileTypes(&[COMDLG_FILTERSPEC {
            pszName: PCWSTR(label.as_ptr()),
            pszSpec: PCWSTR(pattern.as_ptr()),
        }])?;
        dialog.SetDefaultExtension(PCWSTR(extension.as_ptr()))?;
        dialog.SetFileName(PCWSTR(suggested.as_ptr()))?;
        if !preferences.directory.is_empty() {
            let directory = wide(&preferences.directory);
            let folder: IShellItem = SHCreateItemFromParsingName(PCWSTR(directory.as_ptr()), None)?;
            dialog.SetFolder(&folder)?;
        }
        match dialog.Show(Some(HWND(owner as _))) {
            Err(error) if error.code() == HRESULT::from_win32(ERROR_CANCELLED.0) => {
                return Ok(None);
            }
            result => result?,
        }
        let item = dialog.GetResult()?;
        let location = item.GetDisplayName(SIGDN_FILESYSPATH)?;
        let path = PathBuf::from(OsString::from_wide(location.as_wide()));
        CoTaskMemFree(Some(location.0.cast()));
        Ok(Some(path))
    }
}
