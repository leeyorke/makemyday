use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    LogicalPosition,
    Manager, RunEvent, WebviewUrl, WebviewWindowBuilder,
};

const PANEL_WIDTH: f64 = 380.0;
const PANEL_HEIGHT: f64 = 660.0;

fn show_or_create_panel(app: &tauri::AppHandle, tray_x: f64, tray_y: f64, tray_w: f64, tray_h: f64) {
    // Platform-specific positioning
    #[cfg(target_os = "macos")]
    {
        // macOS: Tray is at top of screen, panel appears below tray
        let panel_x = tray_x + (tray_w / 2.0) - (PANEL_WIDTH / 2.0);
        let panel_y = tray_y + tray_h + 4.0;
        show_or_create_panel_at(app, panel_x, panel_y);
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: Tray is at bottom (taskbar), panel appears above tray
        let panel_x = tray_x + (tray_w / 2.0) - (PANEL_WIDTH / 2.0);
        // Position above the taskbar
        let panel_y = tray_y - PANEL_HEIGHT - 4.0;
        show_or_create_panel_at(app, panel_x, panel_y);
    }
}

fn show_or_create_panel_at(app: &tauri::AppHandle, panel_x: f64, panel_y: f64) {
    // Ensure panel stays within screen bounds
    let monitor = app.primary_monitor().ok().flatten();
    let (final_x, final_y) = if let Some(m) = monitor {
        let scale = m.scale_factor();
        let pos = m.position();
        let size = m.size();
        let screen_x = pos.x as f64 / scale;
        let screen_y = pos.y as f64 / scale;
        let screen_w = size.width as f64 / scale;
        let screen_h = size.height as f64 / scale;

        let x = panel_x.max(screen_x).min(screen_x + screen_w - PANEL_WIDTH);
        let y = panel_y.max(screen_y).min(screen_y + screen_h - PANEL_HEIGHT);
        (x, y)
    } else {
        (panel_x, panel_y)
    };


    // If panel already exists, reposition and toggle visibility
    if let Some(window) = app.get_webview_window("panel") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.set_position(LogicalPosition::new(final_x, final_y));
            let _ = window.show();
            let _ = window.set_focus();
        }
        return;
    }

    // Create a new panel window loading the same frontend
    let builder = WebviewWindowBuilder::new(app, "panel", WebviewUrl::App("index.html?compact=1".into()))
        .title("MakeMyDay")
        .inner_size(PANEL_WIDTH, PANEL_HEIGHT)
        .position(final_x, final_y)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(true);

    if let Ok(panel) = builder.build() {
        let _ = panel.set_focus();

        // Hide panel when it loses focus
        let panel_clone = panel.clone();
        panel.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(false) = event {
                let _ = panel_clone.hide();
            }
        });
    }
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }

    // Recreate main window if it was closed
    let mut builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("")
        .inner_size(1200.0, 900.0)
        .resizable(true)
        .fullscreen(false);

    // macOS-specific: Use overlay title bar style
    #[cfg(target_os = "macos")]
    {
        builder = builder
            .title_bar_style(tauri::TitleBarStyle::Overlay)
            .hidden_title(true);
    }

    if let Ok(win) = builder.build() {
        let _ = win.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Build tray menu (right-click)
            let show_main = MenuItemBuilder::with_id("show", "打开主窗口").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show_main, &quit]).build()?;

            // Build tray icon
            let tray_icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
                .expect("failed to load tray icon");

            let tray_builder = TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("MakeMyDay");

            // macOS-specific: Use template icon for automatic light/dark mode
            #[cfg(target_os = "macos")]
            let tray_builder = tray_builder.icon_as_template(true);

            tray_builder
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        show_main_window(app);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        rect,
                        ..
                    } = event
                    {
                        let scale = tray.app_handle()
                            .primary_monitor()
                            .ok()
                            .flatten()
                            .map(|m| m.scale_factor())
                            .unwrap_or(2.0);
                        let pos = rect.position.to_logical::<f64>(scale);
                        let size = rect.size.to_logical::<f64>(scale);
                        show_or_create_panel(tray.app_handle(), pos.x, pos.y, size.width, size.height);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| match event {
        // Keep app running when all windows are closed (tray stays)
        RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        // macOS: clicking dock icon re-shows main window
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => {
            show_main_window(app_handle);
        }
        _ => {}
    });
}
