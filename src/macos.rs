#![allow(deprecated)]

use crate::app::App;
use crate::commands::Function;
use crate::config::Config;
use crate::utils::index_dirs_from_config;
use crate::utils::{handle_from_icns, log_error, log_error_and_exit};
use std::ptr;
#[cfg(target_os = "macos")]
use {
    iced::wgpu::rwh::RawWindowHandle,
    iced::wgpu::rwh::WindowHandle,
    objc2::MainThreadMarker,
    objc2::rc::Retained,
    objc2_app_kit::NSView,
    objc2_app_kit::{NSApp, NSApplicationActivationPolicy},
    objc2_app_kit::{NSFloatingWindowLevel, NSWindowCollectionBehavior},
    objc2_application_services::{
        TransformProcessType, kCurrentProcess, kProcessTransformToUIElementApplication,
    },
};

use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

#[cfg(target_os = "macos")]
pub fn set_activation_policy_accessory() {
    let mtm = MainThreadMarker::new().expect("must be on main thread");
    let app = NSApp(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
}

#[cfg(target_os = "macos")]
pub fn macos_window_config(handle: &WindowHandle) {
    match handle.as_raw() {
        RawWindowHandle::AppKit(handle) => {
            let ns_view = handle.ns_view.as_ptr();
            let ns_view: Retained<NSView> = unsafe { Retained::retain(ns_view.cast()) }.unwrap();
            let ns_window = ns_view
                .window()
                .expect("view was not installed in a window");

            ns_window.setLevel(NSFloatingWindowLevel);

            ns_window.setCollectionBehavior(NSWindowCollectionBehavior::CanJoinAllSpaces);
        }
        _ => {
            panic!(
                "Why are you running this as a non-appkit window? this is a macos only app as of now"
            );
        }
    }
}

#[cfg(target_os = "macos")]
pub fn focus_this_app() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApp;

    let mtm = MainThreadMarker::new().expect("must be on main thread");
    let app = NSApp(mtm);

    app.activateIgnoringOtherApps(true);
}

// because objc2_application_services is mean and this type isn't public
#[repr(C)]
struct ProcessSerialNumber {
    low: u32,
    hi: u32,
}

/// see mostly https://github.com/electron/electron/blob/e181fd040f72becd135db1fa977622b81da21643/shell/browser/browser_mac.mm#L512C1-L532C2.
///
/// returns ApplicationServices OSStatus (u32)
///
/// doesn't seem to do anything if you haven't opened a window yet, so wait to call it until after that.
#[cfg(target_os = "macos")]
pub fn transform_process_to_ui_element() -> u32 {
    let psn = ProcessSerialNumber {
        low: 0,
        hi: kCurrentProcess,
    };
    unsafe {
        TransformProcessType(
            ptr::from_ref(&psn).cast(),
            kProcessTransformToUIElementApplication,
        )
    }
}

pub(crate) fn get_installed_apps(dir: impl AsRef<Path>, store_icons: bool) -> Vec<App> {
    let entries: Vec<_> = fs::read_dir(dir.as_ref())
        .unwrap_or_else(|x| {
            log_error_and_exit(&x.to_string());
            exit(-1)
        })
        .filter_map(|x| x.ok())
        .collect();

    entries
        .into_par_iter()
        .filter_map(|x| {
            let file_type = x.file_type().unwrap_or_else(|e| {
                log_error(&e.to_string());
                exit(-1)
            });
            if !file_type.is_dir() {
                return None;
            }

            let file_name_os = x.file_name();
            let file_name = file_name_os.into_string().unwrap_or_else(|e| {
                log_error(e.to_str().unwrap_or(""));
                exit(-1)
            });
            if !file_name.ends_with(".app") {
                return None;
            }

            let path = x.path();
            let path_str = path.to_str().map(|x| x.to_string()).unwrap_or_else(|| {
                log_error("Unable to get file_name");
                exit(-1)
            });

            let icons = if store_icons {
                match fs::read_to_string(format!("{}/Contents/Info.plist", path_str)).map(
                    |content| {
                        let icon_line = content
                            .lines()
                            .scan(false, |expect_next, line| {
                                if *expect_next {
                                    *expect_next = false;
                                    // Return this line to the iterator
                                    return Some(Some(line));
                                }

                                if line.trim() == "<key>CFBundleIconFile</key>" {
                                    *expect_next = true;
                                }

                                // For lines that are not the one after the key, return None to skip
                                Some(None)
                            })
                            .flatten() // remove the Nones
                            .next()
                            .map(|x| {
                                x.trim()
                                    .strip_prefix("<string>")
                                    .unwrap_or("")
                                    .strip_suffix("</string>")
                                    .unwrap_or("")
                            });

                        handle_from_icns(Path::new(&format!(
                            "{}/Contents/Resources/{}",
                            path_str,
                            icon_line.unwrap_or("AppIcon.icns")
                        )))
                    },
                ) {
                    Ok(Some(a)) => Some(a),
                    _ => {
                        // Fallback method
                        let direntry = fs::read_dir(format!("{}/Contents/Resources", path_str))
                            .into_iter()
                            .flatten()
                            .filter_map(|x| {
                                let file = x.ok()?;
                                let name = file.file_name();
                                let file_name = name.to_str()?;
                                if file_name.ends_with(".icns") {
                                    Some(file.path())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<PathBuf>>();

                        if direntry.len() > 1 {
                            let icns_vec = direntry
                                .iter()
                                .filter(|x| x.ends_with("AppIcon.icns"))
                                .collect::<Vec<&PathBuf>>();
                            handle_from_icns(icns_vec.first().unwrap_or(&&PathBuf::new()))
                        } else if !direntry.is_empty() {
                            handle_from_icns(direntry.first().unwrap_or(&PathBuf::new()))
                        } else {
                            None
                        }
                    }
                }
            } else {
                None
            };

            let name = file_name.strip_suffix(".app").unwrap().to_string();
            Some(App {
                open_command: Function::OpenApp(path_str),
                icons,
                name_lc: name.to_lowercase(),
                name,
            })
        })
        .collect()
}

pub fn get_installed_macos_apps(config: &Config) -> Vec<App> {
    let store_icons = config.theme.show_icons;
    let user_local_path = std::env::var("HOME").unwrap() + "/Applications/";
    let paths: Vec<String> = vec![
        "/Applications/".to_string(),
        user_local_path.to_string(),
        "/System/Applications/".to_string(),
        "/System/Applications/Utilities/".to_string(),
    ];

    let mut apps = paths
        .par_iter()
        .map(|path| get_installed_apps(path, store_icons))
        .flatten()
        .collect();
    index_dirs_from_config(&mut apps);

    apps
}
