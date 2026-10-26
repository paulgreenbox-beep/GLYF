//! Windows-native window identity, geometry and custom-chrome operations.

use super::*;
#[cfg(all(windows, feature = "toolkit-uia"))]
use toolkit_windows_automation::AccessibilityRect;
use windows_uia::{
    Win32::{
        Foundation::{HWND, LPARAM, POINT, WPARAM},
        UI::{
            Input::KeyboardAndMouse::ReleaseCapture,
            WindowsAndMessaging::{
                FindWindowW, GetCursorPos, GetWindowThreadProcessId, HTCAPTION,
                IsZoomed, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, SendMessageW,
                ShowWindow, WM_NCLBUTTONDOWN,
            },
        },
    },
    core::PCWSTR,
};
#[cfg(all(windows, feature = "toolkit-uia"))]
use windows_uia::Win32::{Graphics::Gdi::ClientToScreen, UI::HiDpi::GetDpiForWindow};

#[cfg(all(windows, feature = "toolkit-uia"))]
pub(crate) struct ScreenTransform {
    origin: POINT,
    dpi_factor: f64,
}

#[cfg(all(windows, feature = "toolkit-uia"))]
impl ScreenTransform {
    pub(crate) fn for_window(hwnd: HWND) -> Result<Self, String> {
        let mut origin = POINT::default();
        unsafe { ClientToScreen(hwnd, &mut origin) }
            .ok()
            .map_err(|error| error.to_string())?;
        let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
        Ok(Self {
            origin,
            dpi_factor: dpi as f64 / 96.0,
        })
    }

    pub(crate) fn rect(&self, rect: Rect) -> AccessibilityRect {
        AccessibilityRect {
            x: self.origin.x as f64 + rect.pos.x * self.dpi_factor,
            y: self.origin.y as f64 + rect.pos.y * self.dpi_factor,
            width: rect.size.x * self.dpi_factor,
            height: rect.size.y * self.dpi_factor,
        }
    }
}

#[cfg(windows)]
pub(crate) fn makepad_hwnd() -> Result<HWND, String> {
    let wide = WINDOW_TITLE
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let hwnd =
        unsafe { FindWindowW(None, PCWSTR(wide.as_ptr())) }.map_err(|error| error.to_string())?;
    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut process_id)) };
    if process_id != std::process::id() {
        return Err("matched window belongs to another process".into());
    }
    Ok(hwnd)
}

#[cfg(windows)]
pub(crate) fn minimize_editor_window() {
    if let Ok(hwnd) = makepad_hwnd() {
        unsafe {
            let _ = ShowWindow(hwnd, SW_MINIMIZE);
        }
    }
}

#[cfg(windows)]
pub(crate) fn toggle_editor_window_maximized() {
    if let Ok(hwnd) = makepad_hwnd() {
        unsafe {
            let command = if IsZoomed(hwnd).as_bool() {
                SW_RESTORE
            } else {
                SW_MAXIMIZE
            };
            let _ = ShowWindow(hwnd, command);
        }
    }
}

#[cfg(windows)]
pub(crate) fn begin_editor_window_drag() {
    if let Ok(hwnd) = makepad_hwnd() {
        let mut cursor = POINT::default();
        if unsafe { GetCursorPos(&mut cursor) }.is_err() {
            return;
        }
        // WM_NCLBUTTONDOWN expects signed screen coordinates packed as POINTS.
        // Zero is not the click position, and breaks the native move loop.
        let x = cursor.x as i16 as u16 as usize;
        let y = cursor.y as i16 as u16 as usize;
        let screen_point = ((y << 16) | x) as isize;
        unsafe {
            let _ = ReleaseCapture();
            let _ = SendMessageW(
                hwnd,
                WM_NCLBUTTONDOWN,
                Some(WPARAM(HTCAPTION as usize)),
                Some(LPARAM(screen_point)),
            );
        }
    }
}
