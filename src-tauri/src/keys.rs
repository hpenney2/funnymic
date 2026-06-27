// This currently is unused; the X11 APIs seem to work on KDE Plasma 6 with Wayland.
// This might be a problem on other DEs (like Hyprland).

#[cfg(target_os = "linux")]
use std::collections::HashSet;
use std::{
    env,
    sync::{Arc, LazyLock, OnceLock, RwLock},
};

use rdev::{
    listen,
    EventType::{KeyPress, KeyRelease},
    Key,
};
use tauri::AppHandle;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::swapper::key_callback;

#[cfg(target_os = "linux")]
static WL_STARTED: OnceLock<()> = OnceLock::new();
static WL_KEY_STATE: LazyLock<Arc<RwLock<HashSet<Key>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashSet::new())));
// #[cfg(target_os = "linux")]
// type KeySet = RwLock<HashSet<String>>;
// #[cfg(target_os = "linux")]
// type KeyState<'a> = tauri::State<'a, KeySet>;

#[derive(Debug, PartialEq, Eq)]
pub enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

pub fn detect_server() -> DisplayServer {
    if let Ok(session) = env::var("XDG_SESSION_TYPE") {
        match session.to_lowercase().as_str() {
            "wayland" => return DisplayServer::Wayland,
            "x11" => return DisplayServer::X11,
            _ => (),
        }
    };

    DisplayServer::Unknown
}

pub fn register_hotkey(app: &AppHandle, shortcut: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        if detect_server() == DisplayServer::Wayland {
            println!("Detected Wayland, registering with alternative method");
            return register_wayland(app, shortcut);
        }
    }

    register_with_plugin(app, shortcut).map_err(|err| err.to_string())
}

fn register_wayland(app: &AppHandle, shortcut: &str) -> Result<(), String> {
    let combo = parse_combo(shortcut).unwrap();
    println!("combo is {:?}", combo);
    *WL_KEY_STATE
        .write()
        .expect("failed to write to state lock (rdev); poisoned?") =
        parse_combo(shortcut).map_err(|err| err.to_string())?;

    if WL_STARTED.get().is_none() {
        let app_clone = app.clone();
        std::thread::spawn(move || run_rdev_listener(app_clone));
        let _ = WL_STARTED.set(());
    };

    Ok(())
}

fn run_rdev_listener(app: AppHandle) {
    let mut held: HashSet<Key> = HashSet::new();
    let mut active = false;

    // listen() on rdev v0.5 seems to be X11 specific, but works on KDE with Wayland?
    // grab() (unstable feature due to potentially changing API) explicitly works with Wayland,
    // but uses evdev (and needs extra permissions as a result).
    if let Err(error) = listen(move |event| {
        let pressed: bool;
        let _key = match event.event_type {
            KeyPress(key) => {
                pressed = true;
                key
            }
            KeyRelease(key) => {
                pressed = false;
                key
            }
            _ => return, // we don't care about other events!
        };

        // we do this so we can ignore sidedness
        let key = match _key {
            Key::ControlRight => Key::ControlLeft,
            Key::MetaRight => Key::MetaLeft,
            Key::ShiftRight => Key::ShiftLeft,
            other => other,
        };

        if pressed {
            held.insert(key);
        } else {
            held.remove(&key);
        }

        let listen_for = WL_KEY_STATE
            .read()
            .expect("failed to read key state lock (rdev); poisoned?");

        if *listen_for == held {
            if !active {
                active = true;
                key_callback(&app, true);
            }
        } else if active {
            active = false;
            key_callback(&app, false);
        }
    }) {
        eprintln!("rdev listening error (channel closing): {:?}", error)
    };
}

fn register_with_plugin(
    app: &AppHandle,
    shortcut: &str,
) -> Result<(), tauri_plugin_global_shortcut::Error> {
    let shortcut_man = app.global_shortcut();

    shortcut_man.unregister_all()?;
    shortcut_man.register(shortcut)?;

    println!("Registered shortcut {}", shortcut);

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum KeyParseError {
    #[error("could not parse key {0}")]
    UnknownKey(String),

    #[error("empty token found in key combo: {0}")]
    EmptyToken(String),
}

// key parsing lovingly borrowed/adapted from tauri-apps/global-hotkey
// this is seperate so we can parse into rdev Key enums as well
fn parse_combo(hotkey: &str) -> Result<HashSet<Key>, KeyParseError> {
    let tokens = hotkey.split('+').collect::<Vec<&str>>();

    let mut keys: HashSet<Key> = HashSet::new();

    match tokens.len() {
        // single key hotkey
        1 => {
            keys.insert(parse_key(tokens[0])?);
        }
        // modifiers and key comobo hotkey
        _ => {
            for raw in tokens {
                let token = raw.trim();

                if token.is_empty() {
                    return Err(KeyParseError::EmptyToken(hotkey.to_string()));
                }

                match token.to_uppercase().as_str() {
                    "OPTION" | "ALT" => {
                        keys.insert(Key::Alt);
                    }
                    "CONTROL" | "CTRL" => {
                        keys.insert(Key::ControlLeft);
                    }
                    "COMMAND" | "CMD" | "SUPER" => {
                        keys.insert(Key::MetaLeft);
                    }
                    "SHIFT" => {
                        keys.insert(Key::ShiftLeft);
                    }
                    #[cfg(target_os = "macos")]
                    "COMMANDORCONTROL" | "COMMANDORCTRL" | "CMDORCTRL" | "CMDORCONTROL" => {
                        keys.insert(Key::MetaLeft);
                    }
                    #[cfg(not(target_os = "macos"))]
                    "COMMANDORCONTROL" | "COMMANDORCTRL" | "CMDORCTRL" | "CMDORCONTROL" => {
                        keys.insert(Key::ControlLeft);
                    }
                    _ => {
                        keys.insert(parse_key(token)?);
                    }
                }
            }
        }
    }

    Ok(keys)
}

fn parse_key(key: &str) -> Result<Key, KeyParseError> {
    use Key::*;
    match key.to_uppercase().as_str() {
        "BACKQUOTE" | "`" => Ok(BackQuote),
        "BACKSLASH" | "\\" => Ok(BackSlash),
        "BRACKETLEFT" | "[" => Ok(LeftBracket),
        "BRACKETRIGHT" | "]" => Ok(RightBracket),
        "PAUSE" | "PAUSEBREAK" => Ok(Pause),
        "COMMA" | "," => Ok(Comma),
        "DIGIT0" | "0" => Ok(Num0),
        "DIGIT1" | "1" => Ok(Num1),
        "DIGIT2" | "2" => Ok(Num2),
        "DIGIT3" | "3" => Ok(Num3),
        "DIGIT4" | "4" => Ok(Num4),
        "DIGIT5" | "5" => Ok(Num5),
        "DIGIT6" | "6" => Ok(Num6),
        "DIGIT7" | "7" => Ok(Num7),
        "DIGIT8" | "8" => Ok(Num8),
        "DIGIT9" | "9" => Ok(Num9),
        "EQUAL" | "=" => Ok(Equal),
        "KEYA" | "A" => Ok(KeyA),
        "KEYB" | "B" => Ok(KeyB),
        "KEYC" | "C" => Ok(KeyC),
        "KEYD" | "D" => Ok(KeyD),
        "KEYE" | "E" => Ok(KeyE),
        "KEYF" | "F" => Ok(KeyF),
        "KEYG" | "G" => Ok(KeyG),
        "KEYH" | "H" => Ok(KeyH),
        "KEYI" | "I" => Ok(KeyI),
        "KEYJ" | "J" => Ok(KeyJ),
        "KEYK" | "K" => Ok(KeyK),
        "KEYL" | "L" => Ok(KeyL),
        "KEYM" | "M" => Ok(KeyM),
        "KEYN" | "N" => Ok(KeyN),
        "KEYO" | "O" => Ok(KeyO),
        "KEYP" | "P" => Ok(KeyP),
        "KEYQ" | "Q" => Ok(KeyQ),
        "KEYR" | "R" => Ok(KeyR),
        "KEYS" | "S" => Ok(KeyS),
        "KEYT" | "T" => Ok(KeyT),
        "KEYU" | "U" => Ok(KeyU),
        "KEYV" | "V" => Ok(KeyV),
        "KEYW" | "W" => Ok(KeyW),
        "KEYX" | "X" => Ok(KeyX),
        "KEYY" | "Y" => Ok(KeyY),
        "KEYZ" | "Z" => Ok(KeyZ),
        "MINUS" | "-" => Ok(Minus),
        "PERIOD" | "." => Ok(Dot),
        "QUOTE" | "'" => Ok(Quote),
        "SEMICOLON" | ";" => Ok(SemiColon),
        "SLASH" | "/" => Ok(Slash),
        "BACKSPACE" => Ok(Backspace),
        "CAPSLOCK" => Ok(CapsLock),
        "ENTER" => Ok(Return),
        "SPACE" => Ok(Space),
        "TAB" => Ok(Tab),
        "DELETE" => Ok(Delete),
        "END" => Ok(End),
        "HOME" => Ok(Home),
        "INSERT" => Ok(Insert),
        "PAGEDOWN" => Ok(PageDown),
        "PAGEUP" => Ok(PageUp),
        "PRINTSCREEN" => Ok(PrintScreen),
        "SCROLLLOCK" => Ok(ScrollLock),
        "ARROWDOWN" | "DOWN" => Ok(DownArrow),
        "ARROWLEFT" | "LEFT" => Ok(LeftArrow),
        "ARROWRIGHT" | "RIGHT" => Ok(RightArrow),
        "ARROWUP" | "UP" => Ok(UpArrow),
        "NUMLOCK" => Ok(NumLock),
        "NUMPAD0" | "NUM0" => Ok(Kp0),
        "NUMPAD1" | "NUM1" => Ok(Kp1),
        "NUMPAD2" | "NUM2" => Ok(Kp2),
        "NUMPAD3" | "NUM3" => Ok(Kp3),
        "NUMPAD4" | "NUM4" => Ok(Kp4),
        "NUMPAD5" | "NUM5" => Ok(Kp5),
        "NUMPAD6" | "NUM6" => Ok(Kp6),
        "NUMPAD7" | "NUM7" => Ok(Kp7),
        "NUMPAD8" | "NUM8" => Ok(Kp8),
        "NUMPAD9" | "NUM9" => Ok(Kp9),
        "NUMPADADD" | "NUMADD" | "NUMPADPLUS" | "NUMPLUS" => Ok(KpPlus),
        "NUMPADDECIMAL" | "NUMDECIMAL" => Ok(KpDelete),
        "NUMPADDIVIDE" | "NUMDIVIDE" => Ok(KpDivide),
        "NUMPADENTER" | "NUMENTER" => Ok(KpReturn),
        // "NUMPADEQUAL" | "NUMEQUAL" => Ok(NumpadEqual),
        "NUMPADMULTIPLY" | "NUMMULTIPLY" => Ok(KpMultiply),
        "NUMPADSUBTRACT" | "NUMSUBTRACT" => Ok(KpMinus),
        "ESCAPE" | "ESC" => Ok(Escape),
        "F1" => Ok(F1),
        "F2" => Ok(F2),
        "F3" => Ok(F3),
        "F4" => Ok(F4),
        "F5" => Ok(F5),
        "F6" => Ok(F6),
        "F7" => Ok(F7),
        "F8" => Ok(F8),
        "F9" => Ok(F9),
        "F10" => Ok(F10),
        "F11" => Ok(F11),
        "F12" => Ok(F12),
        // rdev doesn't support these!
        // "AUDIOVOLUMEDOWN" | "VOLUMEDOWN" => Ok(AudioVolumeDown),
        // "AUDIOVOLUMEUP" | "VOLUMEUP" => Ok(AudioVolumeUp),
        // "AUDIOVOLUMEMUTE" | "VOLUMEMUTE" => Ok(AudioVolumeMute),
        // "MEDIAPLAY" => Ok(MediaPlay),
        // "MEDIAPAUSE" => Ok(MediaPause),
        // "MEDIAPLAYPAUSE" => Ok(MediaPlayPause),
        // "MEDIASTOP" => Ok(MediaStop),
        // "MEDIATRACKNEXT" => Ok(MediaTrackNext),
        // "MEDIATRACKPREV" | "MEDIATRACKPREVIOUS" => Ok(MediaTrackPrevious),
        // "F13" => Ok(F13),
        // "F14" => Ok(F14),
        // "F15" => Ok(F15),
        // "F16" => Ok(F16),
        // "F17" => Ok(F17),
        // "F18" => Ok(F18),
        // "F19" => Ok(F19),
        // "F20" => Ok(F20),
        // "F21" => Ok(F21),
        // "F22" => Ok(F22),
        // "F23" => Ok(F23),
        // "F24" => Ok(F24),
        _ => Err(KeyParseError::UnknownKey(key.to_string())),
    }
}
