use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_store::StoreExt;

// mod keys;
mod swapper;

#[cfg(target_os = "linux")]
mod painputswapper;

#[cfg(target_os = "windows")]
mod wininputswapper;

use crate::swapper::{
    get_device_list, get_swap_device, key_callback, set_swap_device, setup_state,
};

const STORE_PATH: &str = "config.json";
const STORE_HOTKEY: &str = "hotkey";
const STORE_SWAPTO: &str = "swapTo";

#[tauri::command]
fn set_hotkey(app: AppHandle, shortcut: &str) -> Result<(), String> {
    // crate::keys::register_hotkey(&app, &shortcut)
    let gs = app.global_shortcut();
    gs.unregister_all().map_err(|err| err.to_string())?;
    gs.register(shortcut).map_err(|err| err.to_string())?;

    app.store(STORE_PATH)
        .map_err(|err| err.to_string())?
        .set(STORE_HOTKEY, shortcut);

    Ok(())
}

#[tauri::command]
fn get_hotkey(app: AppHandle) -> Result<Option<String>, String> {
    if let serde_json::Value::String(val) = app
        .store(STORE_PATH)
        .map_err(|err| err.to_string())?
        .get(STORE_HOTKEY)
        .unwrap_or_default()
    {
        Ok(Some(val))
    } else {
        Ok(None)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            let window = app
                .get_webview_window("main")
                .expect("main window should exist");
            let _ = window.show();
            let _ = window.set_focus();
        }))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_device_list,
            get_swap_device,
            set_swap_device,
            set_hotkey,
            get_hotkey
        ])
        .setup(|app| {
            let store = app.store(STORE_PATH)?;
            let swap_to = if let serde_json::Value::String(val) =
                store.get(STORE_SWAPTO).unwrap_or_default()
            {
                val
            } else {
                String::new()
            };
            setup_state(app, swap_to);

            let name_i = MenuItem::new(app, "FUNNY MIC", false, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "&Show", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "&Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&name_i, &show_i, &quit_i])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        let _ = app
                            .get_webview_window("main")
                            .expect("main window should exist")
                            .show();
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => println!("menu item {:?} not handled", event.id),
                })
                .build(app)?;

            // shortcuts
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, _, event| match event.state() {
                        ShortcutState::Pressed => key_callback(app, true),
                        ShortcutState::Released => key_callback(app, false),
                    })
                    .build(),
            )?;

            if let serde_json::Value::String(hotkey) = store.get(STORE_HOTKEY).unwrap_or_default() {
                app.global_shortcut().register(hotkey.as_str())?;
            };

            Ok(())
        });

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
