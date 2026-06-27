use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::ffi::CString;

use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

use crate::STORE_PATH;
use crate::STORE_SWAPTO;

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
pub async fn set_swap_device(
    app: AppHandle,
    state: PAState<'_>,
    swap_to: CString,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state.swap_to = swap_to;
    app.store(STORE_PATH).map_err(|err| err.to_string())?.set(
        STORE_SWAPTO,
        swap_to.to_str().expect("failed to convert CString"),
    );
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
pub fn setup_state(app: &tauri::App, swap_to: String) -> bool {
    let client =
        pulseaudio::Client::from_env(c"funnymic").expect("failed to connect to PulseAudio");
    app.manage(Mutex::new(PAInputSwapper::new(client, swap_to)))
}

// Windows things
#[cfg(target_os = "windows")]
use crate::wininputswapper::WinInputSwapper;
#[cfg(target_os = "windows")]
type WinState<'a> = tauri::State<'a, Mutex<WinInputSwapper>>;

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn get_device_list(state: WinState<'_>) -> Result<HashMap<String, String>, String> {
    let state = state.lock().await;

    state.get_sources().map_err(|err| err.to_string())
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn get_swap_device(state: WinState<'_>) -> Result<String, ()> {
    let state = state.lock().await;
    Ok(state.swap_to.clone())
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn set_swap_device(
    app: AppHandle,
    state: WinState<'_>,
    swap_to: String,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state.swap_to = swap_to.clone();
    app.store(STORE_PATH)
        .map_err(|err| err.to_string())?
        .set(STORE_SWAPTO, swap_to);
    Ok(())
}

#[cfg(target_os = "windows")]
async fn swap_on(state: WinState<'_>) -> Result<(), String> {
    let mut state = state.lock().await;
    state.swap_on().map_err(|err| err.to_string())
}

#[cfg(target_os = "windows")]
async fn swap_off(state: WinState<'_>) -> Result<(), String> {
    let mut state = state.lock().await;
    state.swap_off().map_err(|err| err.to_string())
}

#[cfg(target_os = "windows")]
pub fn setup_state(app: &tauri::App, swap_to: String) -> bool {
    app.manage(Mutex::new(WinInputSwapper::new(swap_to)))
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
