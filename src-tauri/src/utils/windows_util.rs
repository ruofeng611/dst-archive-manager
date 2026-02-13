use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
use windows::core::{HSTRING, PCWSTR};

/// 通过窗口标题查找窗口（辅助函数）
pub fn find_window_by_title(title: &str) -> bool {
    let wide_title = HSTRING::from(title);
    match unsafe { FindWindowW(None, PCWSTR::from_raw(wide_title.as_ptr())) } {
        Ok(hwnd) => !hwnd.is_invalid(),
        Err(_) => false,
    }
}
