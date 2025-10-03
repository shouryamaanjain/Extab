// macOS implementation of computer control using Core Graphics

use super::{ComputerControl, MouseButton, ScreenSize};
use std::ffi::CString;
use std::os::raw::c_char;
use std::thread;
use std::time::Duration;

pub struct MacOSControl;

// External C functions from Core Graphics
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventCreateMouseEvent(
        source: *mut std::ffi::c_void,
        mouseType: u32,
        mouseCursorPosition: CGPoint,
        mouseButton: u32,
    ) -> *mut std::ffi::c_void;

    fn CGEventPost(tapLocation: u32, event: *mut std::ffi::c_void);
    fn CFRelease(cf: *mut std::ffi::c_void);

    fn CGEventCreateKeyboardEvent(
        source: *mut std::ffi::c_void,
        virtualKey: u16,
        keyDown: bool,
    ) -> *mut std::ffi::c_void;

    fn CGEventKeyboardSetUnicodeString(
        event: *mut std::ffi::c_void,
        stringLength: usize,
        unicodeString: *const u16,
    );

    fn CGEventCreateScrollWheelEvent(
        source: *mut std::ffi::c_void,
        units: u32,
        wheelCount: u32,
        wheel1: i32,
        wheel2: i32,
    ) -> *mut std::ffi::c_void;

    fn CGDisplayBounds(display: u32) -> CGRect;
    fn CGMainDisplayID() -> u32;
}

#[repr(C)]
#[derive(Copy, Clone)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct CGSize {
    width: f64,
    height: f64,
}

// CGEventType constants
const K_CG_EVENT_LEFT_MOUSE_DOWN: u32 = 1;
const K_CG_EVENT_LEFT_MOUSE_UP: u32 = 2;
const K_CG_EVENT_RIGHT_MOUSE_DOWN: u32 = 3;
const K_CG_EVENT_RIGHT_MOUSE_UP: u32 = 4;
const K_CG_EVENT_MOUSE_MOVED: u32 = 5;
const K_CG_EVENT_OTHER_MOUSE_DOWN: u32 = 25;
const K_CG_EVENT_OTHER_MOUSE_UP: u32 = 26;

// CGMouseButton constants
const K_CG_MOUSE_BUTTON_LEFT: u32 = 0;
const K_CG_MOUSE_BUTTON_RIGHT: u32 = 1;
const K_CG_MOUSE_BUTTON_CENTER: u32 = 2;

// HID Session Constants
const K_CG_HID_EVENT_TAP: u32 = 0;

// Scroll wheel units
const K_CG_SCROLL_EVENT_UNIT_PIXEL: u32 = 0;

impl ComputerControl for MacOSControl {
    fn mouse_move(&self, x: i32, y: i32) -> Result<(), String> {
        unsafe {
            let point = CGPoint {
                x: x as f64,
                y: y as f64,
            };

            let event = CGEventCreateMouseEvent(
                std::ptr::null_mut(),
                K_CG_EVENT_MOUSE_MOVED,
                point,
                K_CG_MOUSE_BUTTON_LEFT,
            );

            if event.is_null() {
                return Err("Failed to create mouse move event".to_string());
            }

            CGEventPost(K_CG_HID_EVENT_TAP, event);
            CFRelease(event);
        }
        Ok(())
    }

    fn mouse_click(&self, x: i32, y: i32, button: MouseButton) -> Result<(), String> {
        unsafe {
            let point = CGPoint {
                x: x as f64,
                y: y as f64,
            };

            let (down_type, up_type, button_num) = match button {
                MouseButton::Left => (K_CG_EVENT_LEFT_MOUSE_DOWN, K_CG_EVENT_LEFT_MOUSE_UP, K_CG_MOUSE_BUTTON_LEFT),
                MouseButton::Right => (K_CG_EVENT_RIGHT_MOUSE_DOWN, K_CG_EVENT_RIGHT_MOUSE_UP, K_CG_MOUSE_BUTTON_RIGHT),
                MouseButton::Middle => (K_CG_EVENT_OTHER_MOUSE_DOWN, K_CG_EVENT_OTHER_MOUSE_UP, K_CG_MOUSE_BUTTON_CENTER),
            };

            // Mouse down
            let down_event = CGEventCreateMouseEvent(
                std::ptr::null_mut(),
                down_type,
                point,
                button_num,
            );

            if down_event.is_null() {
                return Err("Failed to create mouse down event".to_string());
            }

            CGEventPost(K_CG_HID_EVENT_TAP, down_event);
            CFRelease(down_event);

            // Small delay
            thread::sleep(Duration::from_millis(10));

            // Mouse up
            let up_event = CGEventCreateMouseEvent(
                std::ptr::null_mut(),
                up_type,
                point,
                button_num,
            );

            if up_event.is_null() {
                return Err("Failed to create mouse up event".to_string());
            }

            CGEventPost(K_CG_HID_EVENT_TAP, up_event);
            CFRelease(up_event);
        }
        Ok(())
    }

    fn mouse_double_click(&self, x: i32, y: i32) -> Result<(), String> {
        self.mouse_click(x, y, MouseButton::Left)?;
        thread::sleep(Duration::from_millis(50));
        self.mouse_click(x, y, MouseButton::Left)?;
        Ok(())
    }

    fn mouse_drag(&self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Result<(), String> {
        unsafe {
            // Move to start position
            self.mouse_move(from_x, from_y)?;
            thread::sleep(Duration::from_millis(50));

            // Mouse down
            let start_point = CGPoint {
                x: from_x as f64,
                y: from_y as f64,
            };

            let down_event = CGEventCreateMouseEvent(
                std::ptr::null_mut(),
                K_CG_EVENT_LEFT_MOUSE_DOWN,
                start_point,
                K_CG_MOUSE_BUTTON_LEFT,
            );

            if down_event.is_null() {
                return Err("Failed to create mouse down event".to_string());
            }

            CGEventPost(K_CG_HID_EVENT_TAP, down_event);
            CFRelease(down_event);

            thread::sleep(Duration::from_millis(100));

            // Drag to end position
            let end_point = CGPoint {
                x: to_x as f64,
                y: to_y as f64,
            };

            let drag_event = CGEventCreateMouseEvent(
                std::ptr::null_mut(),
                K_CG_EVENT_MOUSE_MOVED,
                end_point,
                K_CG_MOUSE_BUTTON_LEFT,
            );

            if drag_event.is_null() {
                return Err("Failed to create drag event".to_string());
            }

            CGEventPost(K_CG_HID_EVENT_TAP, drag_event);
            CFRelease(drag_event);

            thread::sleep(Duration::from_millis(50));

            // Mouse up
            let up_event = CGEventCreateMouseEvent(
                std::ptr::null_mut(),
                K_CG_EVENT_LEFT_MOUSE_UP,
                end_point,
                K_CG_MOUSE_BUTTON_LEFT,
            );

            if up_event.is_null() {
                return Err("Failed to create mouse up event".to_string());
            }

            CGEventPost(K_CG_HID_EVENT_TAP, up_event);
            CFRelease(up_event);
        }
        Ok(())
    }

    fn mouse_scroll(&self, _x: i32, _y: i32, scroll_x: i32, scroll_y: i32) -> Result<(), String> {
        unsafe {
            // Note: macOS scrolling is inverted by default
            let event = CGEventCreateScrollWheelEvent(
                std::ptr::null_mut(),
                K_CG_SCROLL_EVENT_UNIT_PIXEL,
                2, // wheel count (vertical and horizontal)
                -scroll_y, // Invert for natural scrolling
                -scroll_x, // Invert for natural scrolling
            );

            if event.is_null() {
                return Err("Failed to create scroll event".to_string());
            }

            CGEventPost(K_CG_HID_EVENT_TAP, event);
            CFRelease(event);
        }
        Ok(())
    }

    fn keyboard_type(&self, text: &str) -> Result<(), String> {
        unsafe {
            // Convert text to UTF-16
            let utf16_text: Vec<u16> = text.encode_utf16().collect();

            for &char_code in &utf16_text {
                let event = CGEventCreateKeyboardEvent(
                    std::ptr::null_mut(),
                    0, // virtual key (not used for unicode)
                    true,
                );

                if event.is_null() {
                    return Err("Failed to create keyboard event".to_string());
                }

                CGEventKeyboardSetUnicodeString(event, 1, &char_code);
                CGEventPost(K_CG_HID_EVENT_TAP, event);
                CFRelease(event);

                // Small delay between characters
                thread::sleep(Duration::from_millis(10));
            }
        }
        Ok(())
    }

    fn keyboard_key(&self, key: &str) -> Result<(), String> {
        let key_code = parse_key_combination(key)?;

        unsafe {
            // Key down
            let down_event = CGEventCreateKeyboardEvent(
                std::ptr::null_mut(),
                key_code.key,
                true,
            );

            if down_event.is_null() {
                return Err("Failed to create key down event".to_string());
            }

            // Apply modifiers
            if key_code.cmd {
                // Set command flag - would need CGEventSetFlags
            }

            CGEventPost(K_CG_HID_EVENT_TAP, down_event);
            CFRelease(down_event);

            thread::sleep(Duration::from_millis(10));

            // Key up
            let up_event = CGEventCreateKeyboardEvent(
                std::ptr::null_mut(),
                key_code.key,
                false,
            );

            if up_event.is_null() {
                return Err("Failed to create key up event".to_string());
            }

            CGEventPost(K_CG_HID_EVENT_TAP, up_event);
            CFRelease(up_event);
        }
        Ok(())
    }

    fn get_screen_size(&self) -> Result<ScreenSize, String> {
        unsafe {
            let display_id = CGMainDisplayID();
            let bounds = CGDisplayBounds(display_id);

            Ok(ScreenSize {
                width: bounds.size.width as u32,
                height: bounds.size.height as u32,
            })
        }
    }
}

struct KeyCode {
    key: u16,
    cmd: bool,
    shift: bool,
    ctrl: bool,
    alt: bool,
}

fn parse_key_combination(key: &str) -> Result<KeyCode, String> {
    let lower_key = key.to_lowercase();
    let parts: Vec<&str> = lower_key.split('+').collect();

    let mut key_code = KeyCode {
        key: 0,
        cmd: false,
        shift: false,
        ctrl: false,
        alt: false,
    };

    for part in parts.iter() {
        match part.trim() {
            "cmd" | "command" | "meta" => key_code.cmd = true,
            "shift" => key_code.shift = true,
            "ctrl" | "control" => key_code.ctrl = true,
            "alt" | "option" => key_code.alt = true,
            key_str => {
                key_code.key = match key_str {
                    "return" | "enter" => 36,
                    "tab" => 48,
                    "space" => 49,
                    "delete" | "backspace" => 51,
                    "escape" | "esc" => 53,
                    "left" => 123,
                    "right" => 124,
                    "down" => 125,
                    "up" => 126,
                    "a" => 0,
                    "s" => 1,
                    "d" => 2,
                    "f" => 3,
                    "h" => 4,
                    "g" => 5,
                    "z" => 6,
                    "x" => 7,
                    "c" => 8,
                    "v" => 9,
                    "b" => 11,
                    "q" => 12,
                    "w" => 13,
                    "e" => 14,
                    "r" => 15,
                    "y" => 16,
                    "t" => 17,
                    _ => return Err(format!("Unknown key: {}", key_str)),
                };
            }
        }
    }

    Ok(key_code)
}
