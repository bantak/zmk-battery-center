use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconEvent},
    AppHandle, Emitter,
};
use crate::icon_generator::generate_battery_icon;

pub fn init_tray(app_handle: AppHandle) {
    let tray = app_handle.tray_by_id("tray_icon").unwrap();

    #[cfg(target_os = "macos")]
    {
        use tauri::Manager;
        if let Ok(icon_path) = app_handle
            .path()
            .resolve("icons/icon_template.png", tauri::path::BaseDirectory::Resource)
        {
            if let Ok(icon) = tauri::image::Image::from_path(&icon_path) {
                let _ = tray.set_icon(Some(icon));
            }
        }
        let _ = tray.set_icon_as_template(true);
    }

    tray.on_tray_icon_event(|tray_handle, event| {
        let app = tray_handle.app_handle();

        // Let positioner know about the event
        tauri_plugin_positioner::on_tray_event(app, &event);

        // Let frontend know about the event
        let _ = app.emit("tray_event", event.clone());

        // Handle click event
        match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                let _ = app.emit("tray_left_click", event.clone());
            }
            TrayIconEvent::Click {
                button: MouseButton::Right,
                button_state: MouseButtonState::Up,
                ..
            } => {}
            _ => {}
        }
    });
}

#[tauri::command]
pub fn update_tray_icon(app_handle: AppHandle, percentage: u8) -> Result<(), String> {
    log::info!("Updating tray icon with battery percentage: {}%", percentage);

    let png_bytes = generate_battery_icon(percentage)
        .map_err(|e| format!("Failed to generate icon: {}", e))?;

    let image = tauri::image::Image::from_bytes(&png_bytes)
        .map_err(|e| format!("Failed to create image from bytes: {}", e))?;

    let tray = app_handle.tray_by_id("tray_icon")
        .ok_or_else(|| "Tray icon not found".to_string())?;

    tray.set_icon(Some(image))
        .map_err(|e| format!("Failed to set tray icon: {}", e))?;

    #[cfg(target_os = "macos")]
    {
        // Use template mode when battery is above 50%, color mode otherwise
        let use_template = percentage > 50;
        let _ = tray.set_icon_as_template(use_template);
    }

    Ok(())
}
