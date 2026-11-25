pub mod clipboard;
pub mod overlay;
pub mod screen;

pub fn os_error(operation: &str) -> String {
    format!("{operation} failed ({})", std::io::Error::last_os_error())
}

pub fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
