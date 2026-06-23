use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
