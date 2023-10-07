// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io::{self, Write}, fs};
use reqwest::blocking::get;
use tauri::{CustomMenuItem,SystemTrayMenu,SystemTrayMenuItem,SystemTray,Manager, AppHandle};
use tauri_plugin_positioner::{WindowExt, Position};

pub fn start_logging(address: &str) -> io::Result<i32> {
    let response = get(format!("http://{}/co2", address));
    match response{
        Ok(response) => {
            match response.text(){
                Ok(response) => {
                    return Ok(response.trim().parse().expect("Failed to parse response as i32"));
                },
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::Other, e));
                }
            }
        },
        Err(e) => {
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }
    };
}

#[tauri::command]
fn read_file(path: &str) -> String {
    // pathのファイルの存在を確認する
    if !fs::metadata(path).is_ok() {
        return format!("");
    }
    let contents = std::fs::read_to_string(path)
        .expect("Something went wrong reading the file");
    format!("{}", contents)
}

#[tauri::command]
fn write_file(path: &str, contents: &str) {
    // pathのフォルダが存在しない場合は作成する
    let dir = std::path::Path::new(path).parent().unwrap();
    if !dir.exists() {
        fs::create_dir_all(dir).expect("Failed to create directory");
    }
    let mut file = fs::File::create(path).expect("Failed to create file");
    file.write_all(contents.as_bytes())
        .expect("Failed to write to file");
}

#[tauri::command]
fn ppm(address: &str) -> String {
    let log = start_logging(address);
    match log{
        Ok(log) => {
            format!("{}", log)
        },
        Err(e) => {
            format!("{}", e)
        }
    }
}

fn show_window(window: &AppHandle, label: &str) {
    let label_window = match window.get_window(label) {
        Some(label_window) => label_window,
        None => {
            let label_window = tauri::WindowBuilder::new(
                window,
                label,
                tauri::WindowUrl::App(format!("{}.html", label).into())
            ).build().unwrap();
            let _ = label_window.set_title(label);
            label_window
        }
    };
    label_window.set_focus().unwrap();
}

fn show_or_hide_main_window(window: &AppHandle) {
    let label = "main";
    match window.get_window(label) {
        Some(label_window) => {
            match label_window.is_visible() {
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
            }
        },
        None => {
            let label_window = tauri::WindowBuilder::new(
                window,
                label,
                tauri::WindowUrl::App("index.html".into())
            ).build().unwrap();
            let _ = label_window.set_title("ud_co2s_viewer");
            label_window.set_focus().unwrap();
        }
    };
}

fn main() {
    let hide = CustomMenuItem::new("show or hide".to_string(), "Show or Hide");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let license = CustomMenuItem::new("license".to_string(), "License");
    let tray_menu = SystemTrayMenu::new()
        .add_item(hide)
        .add_item(license)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .system_tray(system_tray)
        .setup(|api|{
            let window = api.get_window("main").unwrap();
            window.hide().unwrap();
            window.move_window(Position::RightCenter).unwrap();
            Ok(())
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if event.window().label() == "main"{
                    event.window().hide().unwrap();
                    api.prevent_close();
                }
                else{
                    event.window().close().unwrap();
                };
            }
            _ => {}
        })
        .on_system_tray_event(|app, event| match event {
            tauri::SystemTrayEvent::LeftClick { .. } => {
                show_or_hide_main_window(app);
            }
            tauri::SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show or hide" => {
                        show_or_hide_main_window(app);
                    }
                    "license" => {
                        show_window(app, "license");
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![read_file, write_file, ppm])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}