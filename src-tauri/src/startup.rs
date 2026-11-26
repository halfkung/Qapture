use std::os::windows::ffi::OsStrExt;

use windows_sys::Win32::{
    Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
    System::Registry::{
        HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
    },
};

use crate::native::wide;
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

#[tauri::command]
pub fn startup_enabled() -> Result<bool, String> {
    let mut bytes = 0u32;
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            wide(RUN_KEY).as_ptr(),
            wide("Qapture").as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut bytes,
        )
    };
    match result {
        ERROR_SUCCESS => Ok(true),
        ERROR_FILE_NOT_FOUND => Ok(false),
        code => Err(format!("Could not read Start with Windows ({code}).")),
    }
}

#[tauri::command]
pub fn set_startup(enabled: bool) -> Result<(), String> {
    let directory = wide(RUN_KEY);
    let name = wide("Qapture");
    let result = if enabled {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        let command: Vec<_> = [b'"' as u16]
            .into_iter()
            .chain(executable.as_os_str().encode_wide())
            .chain("\" --tray".encode_utf16())
            .chain([0])
            .collect();
        unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                directory.as_ptr(),
                name.as_ptr(),
                REG_SZ,
                command.as_ptr().cast(),
                (command.len() * 2) as u32,
            )
        }
    } else {
        unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, directory.as_ptr(), name.as_ptr()) }
    };
    if result == ERROR_SUCCESS || (!enabled && result == ERROR_FILE_NOT_FOUND) {
        Ok(())
    } else {
        Err(format!("Could not update Start with Windows ({result})."))
    }
}
