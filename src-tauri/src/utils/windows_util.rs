//! Windows 窗口探测与按键发送工具。
//!
//! 服务器进程都是控制台窗口，本模块负责：按标题找到窗口、把 `c_shutdown()` 输入进去。
//! 既被「停止服务器」接口使用，也被运行期崩档监视任务使用。

use crate::utils::app_component;
use crate::{ConstantComponent, SimpleAppWebError};
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    SendInput, VIRTUAL_KEY, VK_RETURN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, IsIconic, SW_RESTORE, SetForegroundWindow, ShowWindow,
};
use windows::core::{HSTRING, PCWSTR};

/// 通过窗口标题查找窗口（辅助函数）
pub fn find_window_by_title(title: &str) -> bool {
    let wide_title = HSTRING::from(title);
    match unsafe { FindWindowW(None, PCWSTR::from_raw(wide_title.as_ptr())) } {
        Ok(hwnd) => !hwnd.is_invalid(),
        Err(_) => false,
    }
}

/// 查找窗口并发送关闭命令（`c_shutdown()` + 回车）
///
/// 返回是否找到了窗口。窗口最小化时会先恢复并激活（否则收不到键盘输入）。
/// 注意：内部有阻塞等待与 SendInput，异步上下文里调用需用 `spawn_blocking` 包一层。
pub fn find_and_send_shutdown_command(window_title: &str) -> Result<bool, SimpleAppWebError> {
    unsafe {
        let wide_title = HSTRING::from(window_title);
        let hwnd: HWND = FindWindowW(None, PCWSTR::from_raw(wide_title.as_ptr()))?;

        if !hwnd.is_invalid() {
            // 如果窗口已最小化，先恢复窗口
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
                thread::sleep(Duration::from_millis(300));
            }

            // 激活窗口（必须激活才能接收输入）
            let _ = SetForegroundWindow(hwnd);

            // 等待窗口激活
            thread::sleep(Duration::from_millis(200));

            // 使用 SendInput 发送 Unicode 字符（模拟真实键盘输入）
            send_unicode_string(&app_component::<ConstantComponent>()?.dst_end_command);

            // 发送回车键
            send_key_press(VK_RETURN);

            return Ok(true);
        }
    }

    Ok(false)
}

/// 使用 SendInput 发送 Unicode 字符串
fn send_unicode_string(text: &str) {
    unsafe {
        for ch in text.chars() {
            let input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            let _ = SendInput(&[input], size_of::<INPUT>() as i32);

            // 发送按键释放
            let input_up = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch as u16,
                        dwFlags: KEYBD_EVENT_FLAGS(KEYEVENTF_UNICODE.0 | KEYEVENTF_KEYUP.0),
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };

            let _ = SendInput(&[input_up], size_of::<INPUT>() as i32);

            // 小延迟确保按键顺序
            thread::sleep(Duration::from_millis(10));
        }
    }
}

/// 发送虚拟键码按键（用于回车等功能键）
fn send_key_press(vk: VIRTUAL_KEY) {
    unsafe {
        // 按下
        let input_down = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let _ = SendInput(&[input_down], size_of::<INPUT>() as i32);

        // 释放
        let input_up = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let _ = SendInput(&[input_up], size_of::<INPUT>() as i32);
    }
}
