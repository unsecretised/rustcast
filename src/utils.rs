//! This has all the utility functions that rustcast uses
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::exit,
};

use iced::widget::image::Handle;
use icns::IconFamily;
use image::RgbaImage;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{app::apps::App, commands::Function};

/// The default error log path (works only on unix systems, and must be changed for windows
/// support)
const ERR_LOG_PATH: &str = "/tmp/rustscan-err.log";

/// This logs an error to the error log file
pub(crate) fn log_error(msg: &str) {
    if let Ok(mut file) = File::options().create(true).append(true).open(ERR_LOG_PATH) {
        let _ = file.write_all(msg.as_bytes()).ok();
    }
}

/// This logs an error to the error log file, and exits the program
pub(crate) fn log_error_and_exit(msg: &str) {
    log_error(msg);
    exit(-1)
}

/// This converts an icns file to an iced image handle
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

/// This gets all the installed apps in the given directory
///
/// the directories are defined in [`crate::app::tile::Tile::new`]
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
                desc: "Application".to_string(),
                icons,
                name_lc: name.to_lowercase(),
                name,
            })
        })
        .collect()
}
