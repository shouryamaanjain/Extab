// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod window;
mod shortcuts;
mod activate;
mod api;
mod computer_use;

#[cfg(target_os = "macos")]
use tauri_plugin_macos_permissions;
use xcap::Monitor;
use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageBuffer, Rgba};
use tauri_plugin_http;

use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

mod speaker;

#[derive(Default)]
pub struct AudioState {
    stream_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn set_window_height(window: tauri::WebviewWindow, height: u32) -> Result<(), String> {
    use tauri::{LogicalSize, Size};
    
    let new_size = LogicalSize::new(700.0, height as f64);
    
    match window.set_size(Size::Logical(new_size)) {
        Ok(_) => {
            if let Err(e) = window::position_window_top_center(&window, 54) {
                eprintln!("Failed to reposition window: {}", e);
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to resize window: {}", e))
    }
}

#[tauri::command]
fn capture_to_base64() -> Result<String, String> {
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;
    let primary_monitor = monitors
        .into_iter()
        .find(|m| m.is_primary())
        .ok_or("No primary monitor found".to_string())?;

    let captured_image = primary_monitor.capture_image().map_err(|e| format!("Failed to capture image: {}", e))?;

    // Convert to DynamicImage
    let img_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
        captured_image.width(),
        captured_image.height(),
        captured_image.as_raw().to_vec(),
    ).ok_or("Failed to create image buffer".to_string())?;

    let dynamic_image = DynamicImage::ImageRgba8(img_buffer);

    // Convert to RGB (JPEG doesn't support alpha)
    let rgb_image = dynamic_image.to_rgb8();

    // Try different quality levels to stay under 5MB
    // Start with quality 85, then reduce if needed
    for quality in [85, 75, 65, 55, 45, 35].iter() {
        let mut jpeg_buffer = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_buffer, *quality);
        encoder.encode(
            rgb_image.as_raw(),
            rgb_image.width(),
            rgb_image.height(),
            image::ExtendedColorType::Rgb8,
        ).map_err(|e| format!("Failed to encode to JPEG: {}", e))?;

        // Check if under 5MB (leave some margin: 4.8MB)
        if jpeg_buffer.len() <= 5_000_000 {
            let base64_str = base64::engine::general_purpose::STANDARD.encode(jpeg_buffer);
            return Ok(base64_str);
        }
    }

    // If still too large even at quality 35, resize down and try again
    let scale_factor = 0.8; // 80% of original size
    let new_width = (rgb_image.width() as f64 * scale_factor) as u32;
    let new_height = (rgb_image.height() as f64 * scale_factor) as u32;

    let resized = DynamicImage::ImageRgb8(rgb_image).resize_exact(
        new_width,
        new_height,
        image::imageops::FilterType::Lanczos3
    ).to_rgb8();

    let mut jpeg_buffer = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_buffer, 60);
    encoder.encode(
        resized.as_raw(),
        resized.width(),
        resized.height(),
        image::ExtendedColorType::Rgb8,
    ).map_err(|e| format!("Failed to encode to JPEG: {}", e))?;

    let base64_str = base64::engine::general_purpose::STANDARD.encode(jpeg_buffer);
    Ok(base64_str)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .manage(AudioState::default())
        .manage(shortcuts::WindowVisibility(Mutex::new(false)))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_keychain::init())
        .plugin(tauri_plugin_shell::init())  // Add shell plugin
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_version,
            set_window_height,
            capture_to_base64,
            shortcuts::get_shortcuts,
            shortcuts::check_shortcuts_registered,
            shortcuts::set_app_icon_visibility,
            shortcuts::set_always_on_top,
            activate::activate_license_api,
            activate::mask_license_key_cmd,
            activate::get_checkout_url,
            activate::secure_storage_save,
            activate::secure_storage_get,
            activate::secure_storage_remove,
            api::transcribe_audio,
            api::chat_stream,
            api::fetch_models,
            api::check_license_status,
            speaker::start_system_audio_capture,
            speaker::stop_system_audio_capture,
            speaker::check_system_audio_access,
            speaker::request_system_audio_access,
            computer_use::computer_mouse_move,
            computer_use::computer_mouse_click,
            computer_use::computer_mouse_double_click,
            computer_use::computer_mouse_drag,
            computer_use::computer_mouse_scroll,
            computer_use::computer_keyboard_type,
            computer_use::computer_keyboard_key,
            computer_use::computer_get_screen_size
        ])
        .setup(|app| {
            // Setup main window positioning
            window::setup_main_window(app).expect("Failed to setup main window");
            
            // Setup global shortcuts
            if let Err(e) = shortcuts::setup_global_shortcuts(app.handle()) {
                eprintln!("Failed to setup global shortcuts: {}", e);
            }
            
            Ok(())
        });

    // Add macOS-specific permissions plugin
    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_plugin_macos_permissions::init());
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}