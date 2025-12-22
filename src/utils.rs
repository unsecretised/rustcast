use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    process::exit,
};

use global_hotkey::hotkey::Code;
use iced::widget::image::Handle;
use icns::IconFamily;
use image::RgbaImage;

use crate::{app::App, commands::Function};
#[cfg(target_os = "macos")]
use {
    crate::macos::get_installed_macos_apps, objc2_app_kit::NSWorkspace, objc2_foundation::NSURL,
    std::os::unix::fs::PermissionsExt,
};
#[cfg(target_os = "windows")]
use {crate::windows::get_installed_windows_apps, std::process::Command};

const ERR_LOG_PATH: &str = "/tmp/rustscan-err.log";

pub(crate) fn log_error(msg: &str) {
    if let Ok(mut file) = File::options().create(true).append(true).open(ERR_LOG_PATH) {
        let _ = file.write_all(msg.as_bytes()).ok();
    }
}

pub(crate) fn log_error_and_exit(msg: &str) {
    log_error(msg);
    exit(-1)
}

pub(crate) fn handle_from_icns(path: &Path) -> Option<Handle> {
    let data = std::fs::read(path).ok()?;
    let family = IconFamily::read(std::io::Cursor::new(&data)).ok()?;

    let icon_type = family.available_icons();

    let icon = family.get_icon_with_type(*icon_type.first()?).ok()?;
    let image = RgbaImage::from_raw(
        icon.width() as u32,
        icon.height() as u32,
        icon.data().to_vec(),
    )?;
    Some(Handle::from_rgba(
        image.width(),
        image.height(),
        image.into_raw(),
    ))
}

pub fn to_key_code(key_str: &str) -> Option<Code> {
    match key_str.to_lowercase().as_str() {
        // Letters
        "a" => Some(Code::KeyA),
        "b" => Some(Code::KeyB),
        "c" => Some(Code::KeyC),
        "d" => Some(Code::KeyD),
        "e" => Some(Code::KeyE),
        "f" => Some(Code::KeyF),
        "g" => Some(Code::KeyG),
        "h" => Some(Code::KeyH),
        "i" => Some(Code::KeyI),
        "j" => Some(Code::KeyJ),
        "k" => Some(Code::KeyK),
        "l" => Some(Code::KeyL),
        "m" => Some(Code::KeyM),
        "n" => Some(Code::KeyN),
        "o" => Some(Code::KeyO),
        "p" => Some(Code::KeyP),
        "q" => Some(Code::KeyQ),
        "r" => Some(Code::KeyR),
        "s" => Some(Code::KeyS),
        "t" => Some(Code::KeyT),
        "u" => Some(Code::KeyU),
        "v" => Some(Code::KeyV),
        "w" => Some(Code::KeyW),
        "x" => Some(Code::KeyX),
        "y" => Some(Code::KeyY),
        "z" => Some(Code::KeyZ),

        // Digits (main row)
        "0" => Some(Code::Digit0),
        "1" => Some(Code::Digit1),
        "2" => Some(Code::Digit2),
        "3" => Some(Code::Digit3),
        "4" => Some(Code::Digit4),
        "5" => Some(Code::Digit5),
        "6" => Some(Code::Digit6),
        "7" => Some(Code::Digit7),
        "8" => Some(Code::Digit8),
        "9" => Some(Code::Digit9),

        // Function keys
        "f1" => Some(Code::F1),
        "f2" => Some(Code::F2),
        "f3" => Some(Code::F3),
        "f4" => Some(Code::F4),
        "f5" => Some(Code::F5),
        "f6" => Some(Code::F6),
        "f7" => Some(Code::F7),
        "f8" => Some(Code::F8),
        "f9" => Some(Code::F9),
        "f10" => Some(Code::F10),
        "f11" => Some(Code::F11),
        "f12" => Some(Code::F12),

        // Arrows
        "up" | "arrowup" => Some(Code::ArrowUp),
        "down" | "arrowdown" => Some(Code::ArrowDown),
        "left" | "arrowleft" => Some(Code::ArrowLeft),
        "right" | "arrowright" => Some(Code::ArrowRight),

        // Modifiers
        "shift" | "lshift" => Some(Code::ShiftLeft),
        "rshift" => Some(Code::ShiftRight),
        "ctrl" | "control" | "lctrl" => Some(Code::ControlLeft),
        "rctrl" => Some(Code::ControlRight),
        "alt" | "lalt" => Some(Code::AltLeft),
        "ralt" => Some(Code::AltRight),
        "meta" | "super" | "win" | "lmeta" => Some(Code::MetaLeft),
        "rmeta" => Some(Code::MetaRight),

        // Whitespace / editing
        "space" => Some(Code::Space),
        "enter" => Some(Code::Enter),
        "tab" => Some(Code::Tab),
        "backspace" => Some(Code::Backspace),
        "delete" => Some(Code::Delete),
        "insert" => Some(Code::Insert),
        "escape" | "esc" => Some(Code::Escape),

        // Punctuation (US layout-style names)
        "-" | "minus" => Some(Code::Minus),
        "=" | "equal" => Some(Code::Equal),
        "[" | "bracketleft" => Some(Code::BracketLeft),
        "]" | "bracketright" => Some(Code::BracketRight),
        "\\" | "backslash" => Some(Code::Backslash),
        ";" | "semicolon" => Some(Code::Semicolon),
        "'" | "quote" => Some(Code::Quote),
        "," | "comma" => Some(Code::Comma),
        "." | "period" => Some(Code::Period),
        "/" | "slash" => Some(Code::Slash),
        "`" | "backquote" | "grave" => Some(Code::Backquote),

        // Numpad
        "numpad0" => Some(Code::Numpad0),
        "numpad1" => Some(Code::Numpad1),
        "numpad2" => Some(Code::Numpad2),
        "numpad3" => Some(Code::Numpad3),
        "numpad4" => Some(Code::Numpad4),
        "numpad5" => Some(Code::Numpad5),
        "numpad6" => Some(Code::Numpad6),
        "numpad7" => Some(Code::Numpad7),
        "numpad8" => Some(Code::Numpad8),
        "numpad9" => Some(Code::Numpad9),
        "numpadadd" | "numadd" | "kp+" => Some(Code::NumpadAdd),
        "numpadsubtract" | "numsub" | "kp-" => Some(Code::NumpadSubtract),
        "numpadmultiply" | "nummul" | "kp*" => Some(Code::NumpadMultiply),
        "numpaddivide" | "numdiv" | "kp/" => Some(Code::NumpadDivide),
        "numpaddecimal" | "numdecimal" | "kp." => Some(Code::NumpadDecimal),
        "numpadenter" | "numenter" => Some(Code::NumpadEnter),

        // Navigation / misc
        "home" => Some(Code::Home),
        "end" => Some(Code::End),
        "pageup" => Some(Code::PageUp),
        "pagedown" => Some(Code::PageDown),
        "capslock" => Some(Code::CapsLock),
        "scrolllock" => Some(Code::ScrollLock),
        "numlock" => Some(Code::NumLock),
        "pause" => Some(Code::Pause),

        _ => None,
    }
}

pub fn get_config_installation_dir() -> String {
    if cfg!(target_os = "windows") {
        std::env::var("LOCALAPPDATA").unwrap()
    } else {
        std::env::var("HOME").unwrap()
    }
}

pub fn get_config_file_path() -> String {
    let home = get_config_installation_dir();

    if cfg!(target_os = "windows") {
        home + "\\rustcast\\config.toml"
    } else {
        home + "/.config/rustcast/config.toml"
    }
}
use crate::config::Config;

pub fn read_config_file(file_path: &str) -> Result<Config, std::io::Error> {
    let config: Config = match std::fs::read_to_string(file_path) {
        Ok(a) => toml::from_str(&a).unwrap(),
        Err(_) => Config::default(),
    };

    Ok(config)
}

pub fn create_config_file_if_not_exists(
    file_path: &str,
    config: &Config,
) -> Result<(), std::io::Error> {
    // check if file exists
    if let Ok(exists) = std::fs::metadata(file_path)
        && exists.is_file()
    {
        return Ok(());
    }

    let path = Path::new(&file_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }

    std::fs::write(
        file_path,
        toml::to_string(&config).unwrap_or_else(|x| x.to_string()),
    )
    .unwrap();

    Ok(())
}

pub fn open_application(path: &str) {
    #[cfg(target_os = "windows")]
    {
        println!("Opening application: {}", path);

        Command::new("powershell")
            .arg(format!("Start-Process '{}'", path))
            .status()
            .ok();
    }

    #[cfg(target_os = "macos")]
    {
        NSWorkspace::new().openURL(&NSURL::fileURLWithPath(
            &objc2_foundation::NSString::from_str(path),
        ));
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path).status().ok();
    }
}

pub fn index_dirs_from_config(apps: &mut Vec<App>) -> bool {
    let path = get_config_file_path();
    let config = read_config_file(&path);

    // if config is not valid return false otherwise unwrap config so it is usable
    let config = match config {
        Ok(config) => config,
        Err(err) => {
            println!("Error reading config file: {}", err);
            return false;
        }
    };

    if config.index_dirs.is_empty() {
        return false;
    }

    config.index_dirs.clone().iter().for_each(|dir| {
        // check if dir exists
        if !Path::new(dir).exists() {
            println!("Directory {} does not exist", dir);
            return;
        }

        let paths = fs::read_dir(dir).unwrap();

        for path in paths {
            let path = path.unwrap().path();
            let metadata = fs::metadata(&path).unwrap();

            #[cfg(target_os = "windows")]
            let is_executable =
                metadata.is_file() && path.extension().and_then(|s| s.to_str()) == Some("exe");

            #[cfg(target_os = "macos")]
            let is_executable = {
                (metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0))
                    || path.extension().and_then(|s| s.to_str()) == Some("app")
            };

            if is_executable {
                let display_name = path.file_name().unwrap().to_string_lossy().to_string();
                apps.push(App {
                    open_command: Function::OpenApp(path.to_string_lossy().to_string()),
                    name: display_name.clone(),
                    name_lc: display_name.clone().to_lowercase(),
                    icons: None,
                });
            }
        }
    });

    true
}

pub fn get_installed_apps(config: &Config) -> Vec<App> {
    #[cfg(target_os = "macos")]
    {
        get_installed_macos_apps(config)
    }

    #[cfg(target_os = "windows")]
    {
        get_installed_windows_apps()
    }
}
