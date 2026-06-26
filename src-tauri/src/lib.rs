use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

// mod keys;
mod swapper;

#[cfg(target_os = "linux")]
mod painputswapper;

use crate::swapper::{
    get_device_list, get_swap_device, key_callback, set_swap_device, setup_state,
};

#[tauri::command]
fn set_hotkey(app: AppHandle, shortcut: &str) -> Result<(), String> {
    // crate::keys::register_hotkey(&app, &shortcut)
    let gs = app.global_shortcut();
    gs.unregister_all().map_err(|err| err.to_string())?;
    gs.register(shortcut).map_err(|err| err.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_device_list,
            get_swap_device,
            set_swap_device,
            set_hotkey
        ])
        .setup(|app| {
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
                        app.webview_windows()
                            .iter()
                            .next()
                            .unwrap()
                            .1
                            .show()
                            .expect("window failed to show");
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

            Ok(())
        });

    app = setup_state(app);

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
