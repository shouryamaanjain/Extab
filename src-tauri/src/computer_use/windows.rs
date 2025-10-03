// Windows implementation of computer control using Windows API

use super::{ComputerControl, MouseButton, ScreenSize};
use std::thread;
use std::time::Duration;

pub struct WindowsControl;

#[cfg(target_os = "windows")]
mod win_impl {
    use super::*;
    use std::mem;

    #[link(name = "user32")]
    extern "system" {
        fn SetCursorPos(x: i32, y: i32) -> i32;
        fn mouse_event(dwFlags: u32, dx: u32, dy: u32, dwData: u32, dwExtraInfo: usize);
        fn SendInput(nInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32;
        fn GetSystemMetrics(nIndex: i32) -> i32;
    }

    const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
    const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
    const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
    const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;
    const MOUSEEVENTF_MIDDLEDOWN: u32 = 0x0020;
    const MOUSEEVENTF_MIDDLEUP: u32 = 0x0040;
    const MOUSEEVENTF_WHEEL: u32 = 0x0800;
    const MOUSEEVENTF_HWHEEL: u32 = 0x01000;
    const MOUSEEVENTF_MOVE: u32 = 0x0001;
    const MOUSEEVENTF_ABSOLUTE: u32 = 0x8000;

    const INPUT_MOUSE: u32 = 0;
    const INPUT_KEYBOARD: u32 = 1;

    const KEYEVENTF_KEYUP: u32 = 0x0002;
    const KEYEVENTF_UNICODE: u32 = 0x0004;

    const SM_CXSCREEN: i32 = 0;
    const SM_CYSCREEN: i32 = 1;

    #[repr(C)]
    struct INPUT {
        type_: u32,
        u: InputUnion,
    }

    #[repr(C)]
    union InputUnion {
        mi: MOUSEINPUT,
        ki: KEYBDINPUT,
    }

    #[repr(C)]
    struct MOUSEINPUT {
        dx: i32,
        dy: i32,
        mouseData: u32,
        dwFlags: u32,
        time: u32,
        dwExtraInfo: usize,
    }

    #[repr(C)]
    struct KEYBDINPUT {
        wVk: u16,
        wScan: u16,
        dwFlags: u32,
        time: u32,
        dwExtraInfo: usize,
    }

    pub fn mouse_move_impl(x: i32, y: i32) -> Result<(), String> {
        unsafe {
            if SetCursorPos(x, y) == 0 {
                return Err("Failed to move mouse cursor".to_string());
            }
        }
        Ok(())
    }

    pub fn mouse_click_impl(x: i32, y: i32, button: MouseButton) -> Result<(), String> {
        mouse_move_impl(x, y)?;
        thread::sleep(Duration::from_millis(10));

        unsafe {
            let (down_flag, up_flag) = match button {
                MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
                MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
                MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
            };

            mouse_event(down_flag, 0, 0, 0, 0);
            thread::sleep(Duration::from_millis(10));
            mouse_event(up_flag, 0, 0, 0, 0);
        }
        Ok(())
    }

    pub fn mouse_double_click_impl(x: i32, y: i32) -> Result<(), String> {
        mouse_click_impl(x, y, MouseButton::Left)?;
        thread::sleep(Duration::from_millis(50));
        mouse_click_impl(x, y, MouseButton::Left)?;
        Ok(())
    }

    pub fn mouse_drag_impl(from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Result<(), String> {
        mouse_move_impl(from_x, from_y)?;
        thread::sleep(Duration::from_millis(50));

        unsafe {
            mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
            thread::sleep(Duration::from_millis(100));
        }

        mouse_move_impl(to_x, to_y)?;
        thread::sleep(Duration::from_millis(50));

        unsafe {
            mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
        }

        Ok(())
    }

    pub fn mouse_scroll_impl(_x: i32, _y: i32, _scroll_x: i32, scroll_y: i32) -> Result<(), String> {
        unsafe {
            // Windows scroll amount is in multiples of WHEEL_DELTA (120)
            let wheel_delta = 120;
            let scroll_amount = (scroll_y * wheel_delta) as u32;

            mouse_event(MOUSEEVENTF_WHEEL, 0, 0, scroll_amount, 0);
        }
        Ok(())
    }

    pub fn keyboard_type_impl(text: &str) -> Result<(), String> {
        unsafe {
            for ch in text.chars() {
                let mut inputs = [
                    INPUT {
                        type_: INPUT_KEYBOARD,
                        u: InputUnion {
                            ki: KEYBDINPUT {
                                wVk: 0,
                                wScan: ch as u16,
                                dwFlags: KEYEVENTF_UNICODE,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                    INPUT {
                        type_: INPUT_KEYBOARD,
                        u: InputUnion {
                            ki: KEYBDINPUT {
                                wVk: 0,
                                wScan: ch as u16,
                                dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                ];

                SendInput(2, inputs.as_ptr(), mem::size_of::<INPUT>() as i32);
                thread::sleep(Duration::from_millis(10));
            }
        }
        Ok(())
    }

    pub fn keyboard_key_impl(key: &str) -> Result<(), String> {
        let vk_code = parse_key(key)?;

        unsafe {
            let mut inputs = [
                INPUT {
                    type_: INPUT_KEYBOARD,
                    u: InputUnion {
                        ki: KEYBDINPUT {
                            wVk: vk_code,
                            wScan: 0,
                            dwFlags: 0,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
                INPUT {
                    type_: INPUT_KEYBOARD,
                    u: InputUnion {
                        ki: KEYBDINPUT {
                            wVk: vk_code,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
            ];

            SendInput(2, inputs.as_ptr(), mem::size_of::<INPUT>() as i32);
        }

        Ok(())
    }

    pub fn get_screen_size_impl() -> Result<super::ScreenSize, String> {
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);

            Ok(super::ScreenSize {
                width: width as u32,
                height: height as u32,
            })
        }
    }

    fn parse_key(key: &str) -> Result<u16, String> {
        let lower_key = key.to_lowercase();
        match lower_key.as_str() {
            "return" | "enter" => Ok(0x0D),
            "tab" => Ok(0x09),
            "space" => Ok(0x20),
            "backspace" => Ok(0x08),
            "escape" | "esc" => Ok(0x1B),
            "left" => Ok(0x25),
            "up" => Ok(0x26),
            "right" => Ok(0x27),
            "down" => Ok(0x28),
            _ => Err(format!("Unknown key: {}", key)),
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod win_impl {
    use super::*;

    pub fn mouse_move_impl(_x: i32, _y: i32) -> Result<(), String> {
        Err("Windows control is only available on Windows".to_string())
    }

    pub fn mouse_click_impl(_x: i32, _y: i32, _button: MouseButton) -> Result<(), String> {
        Err("Windows control is only available on Windows".to_string())
    }

    pub fn mouse_double_click_impl(_x: i32, _y: i32) -> Result<(), String> {
        Err("Windows control is only available on Windows".to_string())
    }

    pub fn mouse_drag_impl(_from_x: i32, _from_y: i32, _to_x: i32, _to_y: i32) -> Result<(), String> {
        Err("Windows control is only available on Windows".to_string())
    }

    pub fn mouse_scroll_impl(_x: i32, _y: i32, _scroll_x: i32, _scroll_y: i32) -> Result<(), String> {
        Err("Windows control is only available on Windows".to_string())
    }

    pub fn keyboard_type_impl(_text: &str) -> Result<(), String> {
        Err("Windows control is only available on Windows".to_string())
    }

    pub fn keyboard_key_impl(_key: &str) -> Result<(), String> {
        Err("Windows control is only available on Windows".to_string())
    }

    pub fn get_screen_size_impl() -> Result<super::ScreenSize, String> {
        Err("Windows control is only available on Windows".to_string())
    }
}

impl ComputerControl for WindowsControl {
    fn mouse_move(&self, x: i32, y: i32) -> Result<(), String> {
        win_impl::mouse_move_impl(x, y)
    }

    fn mouse_click(&self, x: i32, y: i32, button: MouseButton) -> Result<(), String> {
        win_impl::mouse_click_impl(x, y, button)
    }

    fn mouse_double_click(&self, x: i32, y: i32) -> Result<(), String> {
        win_impl::mouse_double_click_impl(x, y)
    }

    fn mouse_drag(&self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Result<(), String> {
        win_impl::mouse_drag_impl(from_x, from_y, to_x, to_y)
    }

    fn mouse_scroll(&self, x: i32, y: i32, scroll_x: i32, scroll_y: i32) -> Result<(), String> {
        win_impl::mouse_scroll_impl(x, y, scroll_x, scroll_y)
    }

    fn keyboard_type(&self, text: &str) -> Result<(), String> {
        win_impl::keyboard_type_impl(text)
    }

    fn keyboard_key(&self, key: &str) -> Result<(), String> {
        win_impl::keyboard_key_impl(key)
    }

    fn get_screen_size(&self) -> Result<ScreenSize, String> {
        win_impl::get_screen_size_impl()
    }
}
