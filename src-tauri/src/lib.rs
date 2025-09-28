use reqwest::blocking::get;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use tauri_plugin_positioner::{Position, WindowExt};
use anyhow::{anyhow, Result};

pub fn start_logging(address: &str) -> Result<i32> {
    let response = get(format!("http://{}/co2", address));
    match response {
        Ok(response) => match response.text() {
            Ok(response) => {
                return Ok(response
                    .trim()
                    .parse()
                    .expect("Failed to parse response as i32"));
            }
            Err(e) => {
                return Err(anyhow!(e));
            }
        }
        Err(e) => {
            return Err(anyhow!(e));
        }
    };
}

#[tauri::command]
fn ppm(address: &str) -> String {
    let log = start_logging(address);
    match log {
        Ok(log) => {
            format!("{}", log)
        }
        Err(e) => {
            format!("{}", e)
        }
    }
}

fn show_window(window: &AppHandle, label: &str) {
    let label_window = match window.get_webview_window(label) {
        Some(label_window) => label_window,
        None => {
            let label_window = tauri::WebviewWindowBuilder::new(
                window,
                label,
                tauri::WebviewUrl::App(format!("{}.html", label).into()),
            )
            .build()
            .unwrap();
            let _ = label_window.set_title(label);
            label_window
        }
    };
    label_window.set_focus().unwrap();
}

fn show_or_hide_main_window(window: &AppHandle) {
    let label = "main";
    match window.get_webview_window(label) {
        Some(label_window) => match label_window.is_visible() {
            Ok(visible) => {
                if visible {
                    label_window.hide().unwrap();
                } else {
                    label_window.show().unwrap();
                }
            }
            Err(err) => {
                panic!("failed toggle visible for main window {}", err);
            }
        },
        None => {
            let label_window = tauri::WebviewWindowBuilder::new(
                window,
                label,
                tauri::WebviewUrl::App("index.html".into()),
            )
            .build()
            .unwrap();
            let _ = label_window.set_title("ud_co2s_viewer");
            label_window.set_focus().unwrap();
        }
    };
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_positioner::init())
        .setup(|api| {
            let window = api.get_webview_window("main").unwrap();
            window.hide().unwrap();
            window.move_window(Position::RightCenter).unwrap();

            let quit_i = MenuItem::with_id(api, "quit", "Quit", true, None::<&str>)?;
            let show_or_hide_i =
                MenuItem::with_id(api, "show or hide", "Show or Hide", true, None::<&str>)?;
            let license_i = MenuItem::with_id(api, "license", "License", true, None::<&str>)?;
            let menu = Menu::with_items(api, &[&quit_i, &show_or_hide_i, &license_i])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        let app = tray.app_handle();
                        show_or_hide_main_window(app);
                    }
                    _ => {}
                })
                .on_menu_event(|api, event| match event.id.as_ref() {
                    "quit" => {
                        println!("quit menu item was clicked");
                        api.exit(0);
                    }
                    "show or hide" => {
                        show_or_hide_main_window(api);
                    }
                    "license" => {
                        show_window(api, "license");
                    }
                    _ => {}
                })
                .build(api)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if window.label() == "main" {
                    window.hide().unwrap();
                    api.prevent_close();
                } else {
                    window.close().unwrap();
                };
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![ppm])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
