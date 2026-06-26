use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::ffi::CString;

#[cfg(target_os = "linux")]
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Manager};

// Linux (PulseAudio) things
#[cfg(target_os = "linux")]
use crate::painputswapper::PAInputSwapper;
#[cfg(target_os = "linux")]
type PAState<'a> = tauri::State<'a, Mutex<PAInputSwapper>>;

#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn get_device_list(state: PAState<'_>) -> Result<HashMap<String, String>, String> {
    let state = state.lock().await;

    state.get_sources().await.map_err(|err| err.to_string())
}

#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn get_swap_device(state: PAState<'_>) -> Result<CString, ()> {
    let state = state.lock().await;
    Ok(state.swap_to.clone())
}

#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn set_swap_device(state: PAState<'_>, swap_to: CString) -> Result<(), ()> {
    let mut state = state.lock().await;
    state.swap_to = swap_to;
    println!("{}", state.swap_to.to_str().unwrap());
    Ok(())
}

#[cfg(target_os = "linux")]
// #[tauri::command]
async fn swap_on(state: PAState<'_>) -> Result<(), String> {
    let mut state = state.lock().await;
    state.swap_on().await.map_err(|err| err.to_string())
}

#[cfg(target_os = "linux")]
// #[tauri::command]
async fn swap_off(state: PAState<'_>) -> Result<(), String> {
    let mut state = state.lock().await;
    state.swap_off().await.map_err(|err| err.to_string())
}

#[cfg(target_os = "linux")]
pub fn setup_state(app: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    let client =
        pulseaudio::Client::from_env(c"funnymic").expect("failed to connect to PulseAudio");
    app.manage(Mutex::new(PAInputSwapper::new(client)))
}

// Windows things
#[cfg(target_os = "windows")]
pub fn setup_state(app: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    todo!()
}

pub fn key_callback(app: &AppHandle, on: bool) {
    let app_clone = app.clone();
    match on {
        true => {
            tauri::async_runtime::spawn(async move {
                let state = app_clone.state();
                if let Err(err) = swap_on(state).await {
                    eprintln!("error occured while swapping on: {}", err);
                }
            });
        }
        false => {
            tauri::async_runtime::spawn(async move {
                let state = app_clone.state();
                if let Err(err) = swap_off(state).await {
                    eprintln!("error occured while swapping off: {}", err);
                }
            });
        }
    };
}
