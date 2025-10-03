// Computer Use tool implementation
// Provides mouse and keyboard control for Claude's computer use feature

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

// Platform-agnostic interface
pub trait ComputerControl {
    fn mouse_move(&self, x: i32, y: i32) -> Result<(), String>;
    fn mouse_click(&self, x: i32, y: i32, button: MouseButton) -> Result<(), String>;
    fn mouse_double_click(&self, x: i32, y: i32) -> Result<(), String>;
    fn mouse_drag(&self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Result<(), String>;
    fn mouse_scroll(&self, x: i32, y: i32, scroll_x: i32, scroll_y: i32) -> Result<(), String>;
    fn keyboard_type(&self, text: &str) -> Result<(), String>;
    fn keyboard_key(&self, key: &str) -> Result<(), String>;
    fn get_screen_size(&self) -> Result<ScreenSize, String>;
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

// Get platform-specific implementation
pub fn get_computer_control() -> Box<dyn ComputerControl + Send + Sync> {
    #[cfg(target_os = "macos")]
    return Box::new(macos::MacOSControl);

    #[cfg(target_os = "windows")]
    return Box::new(windows::WindowsControl);

    #[cfg(target_os = "linux")]
    return Box::new(linux::LinuxControl);
}

// Tauri commands
#[tauri::command]
pub fn computer_mouse_move(x: i32, y: i32) -> Result<String, String> {
    let control = get_computer_control();
    control.mouse_move(x, y)?;
    Ok("Mouse moved successfully".to_string())
}

#[tauri::command]
pub fn computer_mouse_click(x: i32, y: i32, button: Option<String>) -> Result<String, String> {
    let control = get_computer_control();
    let mouse_button = match button.as_deref() {
        Some("right") => MouseButton::Right,
        Some("middle") => MouseButton::Middle,
        _ => MouseButton::Left,
    };
    control.mouse_click(x, y, mouse_button)?;
    Ok("Mouse clicked successfully".to_string())
}

#[tauri::command]
pub fn computer_mouse_double_click(x: i32, y: i32) -> Result<String, String> {
    let control = get_computer_control();
    control.mouse_double_click(x, y)?;
    Ok("Mouse double-clicked successfully".to_string())
}

#[tauri::command]
pub fn computer_mouse_drag(from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Result<String, String> {
    let control = get_computer_control();
    control.mouse_drag(from_x, from_y, to_x, to_y)?;
    Ok("Mouse drag completed successfully".to_string())
}

#[tauri::command]
pub fn computer_mouse_scroll(x: i32, y: i32, scroll_x: i32, scroll_y: i32) -> Result<String, String> {
    let control = get_computer_control();
    control.mouse_scroll(x, y, scroll_x, scroll_y)?;
    Ok("Mouse scroll completed successfully".to_string())
}

#[tauri::command]
pub fn computer_keyboard_type(text: String) -> Result<String, String> {
    let control = get_computer_control();
    control.keyboard_type(&text)?;
    Ok(format!("Typed text: {}", text))
}

#[tauri::command]
pub fn computer_keyboard_key(key: String) -> Result<String, String> {
    let control = get_computer_control();
    control.keyboard_key(&key)?;
    Ok(format!("Pressed key: {}", key))
}

#[tauri::command]
pub fn computer_get_screen_size() -> Result<ScreenSize, String> {
    let control = get_computer_control();
    control.get_screen_size()
}
