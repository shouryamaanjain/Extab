// Linux implementation of computer control using X11

use super::{ComputerControl, MouseButton, ScreenSize};
use std::thread;
use std::time::Duration;

pub struct LinuxControl;

#[cfg(target_os = "linux")]
mod x11_impl {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    #[link(name = "X11")]
    extern "C" {
        fn XOpenDisplay(display_name: *const i8) -> *mut std::ffi::c_void;
        fn XCloseDisplay(display: *mut std::ffi::c_void) -> i32;
        fn XDefaultRootWindow(display: *mut std::ffi::c_void) -> u64;
        fn XDisplayWidth(display: *mut std::ffi::c_void, screen_number: i32) -> i32;
        fn XDisplayHeight(display: *mut std::ffi::c_void, screen_number: i32) -> i32;
        fn XFlush(display: *mut std::ffi::c_void) -> i32;
    }

    #[link(name = "Xtst")]
    extern "C" {
        fn XTestFakeMotionEvent(display: *mut std::ffi::c_void, screen: i32, x: i32, y: i32, delay: u64) -> i32;
        fn XTestFakeButtonEvent(display: *mut std::ffi::c_void, button: u32, is_press: i32, delay: u64) -> i32;
        fn XTestFakeKeyEvent(display: *mut std::ffi::c_void, keycode: u32, is_press: i32, delay: u64) -> i32;
    }

    const BUTTON1: u32 = 1; // Left mouse button
    const BUTTON2: u32 = 2; // Middle mouse button
    const BUTTON3: u32 = 3; // Right mouse button
    const BUTTON4: u32 = 4; // Scroll up
    const BUTTON5: u32 = 5; // Scroll down

    struct Display {
        ptr: *mut std::ffi::c_void,
    }

    impl Display {
        fn open() -> Result<Self, String> {
            unsafe {
                let display = XOpenDisplay(ptr::null());
                if display.is_null() {
                    return Err("Failed to open X display".to_string());
                }
                Ok(Display { ptr: display })
            }
        }
    }

    impl Drop for Display {
        fn drop(&mut self) {
            unsafe {
                if !self.ptr.is_null() {
                    XCloseDisplay(self.ptr);
                }
            }
        }
    }

    pub fn mouse_move_impl(x: i32, y: i32) -> Result<(), String> {
        let display = Display::open()?;
        unsafe {
            XTestFakeMotionEvent(display.ptr, -1, x, y, 0);
            XFlush(display.ptr);
        }
        Ok(())
    }

    pub fn mouse_click_impl(x: i32, y: i32, button: MouseButton) -> Result<(), String> {
        let display = Display::open()?;

        let button_num = match button {
            MouseButton::Left => BUTTON1,
            MouseButton::Middle => BUTTON2,
            MouseButton::Right => BUTTON3,
        };

        unsafe {
            XTestFakeMotionEvent(display.ptr, -1, x, y, 0);
            XFlush(display.ptr);
            thread::sleep(Duration::from_millis(10));

            XTestFakeButtonEvent(display.ptr, button_num, 1, 0); // Press
            XFlush(display.ptr);
            thread::sleep(Duration::from_millis(10));

            XTestFakeButtonEvent(display.ptr, button_num, 0, 0); // Release
            XFlush(display.ptr);
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
        let display = Display::open()?;

        unsafe {
            // Move to start
            XTestFakeMotionEvent(display.ptr, -1, from_x, from_y, 0);
            XFlush(display.ptr);
            thread::sleep(Duration::from_millis(50));

            // Press
            XTestFakeButtonEvent(display.ptr, BUTTON1, 1, 0);
            XFlush(display.ptr);
            thread::sleep(Duration::from_millis(100));

            // Drag to end
            XTestFakeMotionEvent(display.ptr, -1, to_x, to_y, 0);
            XFlush(display.ptr);
            thread::sleep(Duration::from_millis(50));

            // Release
            XTestFakeButtonEvent(display.ptr, BUTTON1, 0, 0);
            XFlush(display.ptr);
        }
        Ok(())
    }

    pub fn mouse_scroll_impl(_x: i32, _y: i32, _scroll_x: i32, scroll_y: i32) -> Result<(), String> {
        let display = Display::open()?;

        unsafe {
            let (button, count) = if scroll_y > 0 {
                (BUTTON4, scroll_y.abs())
            } else {
                (BUTTON5, scroll_y.abs())
            };

            for _ in 0..count {
                XTestFakeButtonEvent(display.ptr, button, 1, 0);
                XTestFakeButtonEvent(display.ptr, button, 0, 0);
                XFlush(display.ptr);
                thread::sleep(Duration::from_millis(10));
            }
        }
        Ok(())
    }

    pub fn keyboard_type_impl(text: &str) -> Result<(), String> {
        // Note: This is a simplified implementation
        // A full implementation would need to use XStringToKeysym and XKeysymToKeycode
        let display = Display::open()?;

        for ch in text.chars() {
            let keycode = char_to_keycode(ch);
            if let Some(kc) = keycode {
                unsafe {
                    XTestFakeKeyEvent(display.ptr, kc, 1, 0);
                    XTestFakeKeyEvent(display.ptr, kc, 0, 0);
                    XFlush(display.ptr);
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        Ok(())
    }

    pub fn keyboard_key_impl(key: &str) -> Result<(), String> {
        let display = Display::open()?;
        let keycode = parse_key(key)?;

        unsafe {
            XTestFakeKeyEvent(display.ptr, keycode, 1, 0);
            XFlush(display.ptr);
            thread::sleep(Duration::from_millis(10));

            XTestFakeKeyEvent(display.ptr, keycode, 0, 0);
            XFlush(display.ptr);
        }

        Ok(())
    }

    pub fn get_screen_size_impl() -> Result<super::ScreenSize, String> {
        let display = Display::open()?;

        unsafe {
            let width = XDisplayWidth(display.ptr, 0);
            let height = XDisplayHeight(display.ptr, 0);

            Ok(super::ScreenSize {
                width: width as u32,
                height: height as u32,
            })
        }
    }

    fn char_to_keycode(ch: char) -> Option<u32> {
        // This is a simplified mapping - a full implementation would use X11 keysym functions
        match ch {
            'a'..='z' => Some(38 + (ch as u32 - 'a' as u32)),
            'A'..='Z' => Some(38 + (ch as u32 - 'A' as u32)),
            ' ' => Some(65),
            _ => None,
        }
    }

    fn parse_key(key: &str) -> Result<u32, String> {
        let lower_key = key.to_lowercase();
        match lower_key.as_str() {
            "return" | "enter" => Ok(36),
            "tab" => Ok(23),
            "space" => Ok(65),
            "backspace" => Ok(22),
            "escape" | "esc" => Ok(9),
            "left" => Ok(113),
            "up" => Ok(111),
            "right" => Ok(114),
            "down" => Ok(116),
            _ => Err(format!("Unknown key: {}", key)),
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod x11_impl {
    use super::*;

    pub fn mouse_move_impl(_x: i32, _y: i32) -> Result<(), String> {
        Err("Linux control is only available on Linux".to_string())
    }

    pub fn mouse_click_impl(_x: i32, _y: i32, _button: MouseButton) -> Result<(), String> {
        Err("Linux control is only available on Linux".to_string())
    }

    pub fn mouse_double_click_impl(_x: i32, _y: i32) -> Result<(), String> {
        Err("Linux control is only available on Linux".to_string())
    }

    pub fn mouse_drag_impl(_from_x: i32, _from_y: i32, _to_x: i32, _to_y: i32) -> Result<(), String> {
        Err("Linux control is only available on Linux".to_string())
    }

    pub fn mouse_scroll_impl(_x: i32, _y: i32, _scroll_x: i32, _scroll_y: i32) -> Result<(), String> {
        Err("Linux control is only available on Linux".to_string())
    }

    pub fn keyboard_type_impl(_text: &str) -> Result<(), String> {
        Err("Linux control is only available on Linux".to_string())
    }

    pub fn keyboard_key_impl(_key: &str) -> Result<(), String> {
        Err("Linux control is only available on Linux".to_string())
    }

    pub fn get_screen_size_impl() -> Result<super::ScreenSize, String> {
        Err("Linux control is only available on Linux".to_string())
    }
}

impl ComputerControl for LinuxControl {
    fn mouse_move(&self, x: i32, y: i32) -> Result<(), String> {
        x11_impl::mouse_move_impl(x, y)
    }

    fn mouse_click(&self, x: i32, y: i32, button: MouseButton) -> Result<(), String> {
        x11_impl::mouse_click_impl(x, y, button)
    }

    fn mouse_double_click(&self, x: i32, y: i32) -> Result<(), String> {
        x11_impl::mouse_double_click_impl(x, y)
    }

    fn mouse_drag(&self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Result<(), String> {
        x11_impl::mouse_drag_impl(from_x, from_y, to_x, to_y)
    }

    fn mouse_scroll(&self, x: i32, y: i32, scroll_x: i32, scroll_y: i32) -> Result<(), String> {
        x11_impl::mouse_scroll_impl(x, y, scroll_x, scroll_y)
    }

    fn keyboard_type(&self, text: &str) -> Result<(), String> {
        x11_impl::keyboard_type_impl(text)
    }

    fn keyboard_key(&self, key: &str) -> Result<(), String> {
        x11_impl::keyboard_key_impl(key)
    }

    fn get_screen_size(&self) -> Result<ScreenSize, String> {
        x11_impl::get_screen_size_impl()
    }
}
