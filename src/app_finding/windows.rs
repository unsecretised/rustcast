use {
    crate::{app::apps::App, cross_platform::windows::get_acp},
    std::path::PathBuf,
    walkdir::WalkDir,
    windows::{
        Win32::{
            System::Com::CoTaskMemFree,
            UI::Shell::{
                FOLDERID_LocalAppData, FOLDERID_ProgramFiles, FOLDERID_ProgramFilesX86,
                KF_FLAG_DEFAULT, SHGetKnownFolderPath,
            },
        },
        core::GUID,
    },
};

/// Loads apps from the registry keys `SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall` and
/// `SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall`. `apps` has the relvant items
/// appended to it.
///
/// Based on https://stackoverflow.com/questions/2864984
pub fn get_apps_from_registry(apps: &mut Vec<App>) {
    use std::ffi::OsString;
    let hkey = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);

    let registers = [
        hkey.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
            .unwrap(),
        hkey.open_subkey("SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
            .unwrap(),
    ];

    registers.iter().for_each(|reg| {
        reg.enum_keys().for_each(|key| {
            // Not debug only just because it doesn't run too often
            tracing::trace!("App added [reg]: {:?}", key);

            // https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key
            let name = key.unwrap();
            let key = reg.open_subkey(&name).unwrap();
            let display_name: OsString = key.get_value("DisplayName").unwrap_or_default();

            // they might be useful one day ?
            // let publisher = key.get_value("Publisher").unwrap_or(OsString::new());
            // let version = key.get_value("DisplayVersion").unwrap_or(OsString::new());

            // Trick, I saw on internet to point to the exe location..
            let exe_path: OsString = key.get_value("DisplayIcon").unwrap_or_default();
            if exe_path.is_empty() {
                return;
            }
            // if there is something, it will be in the form of
            // "C:\Program Files\Microsoft Office\Office16\WINWORD.EXE",0
            let exe_string = exe_path.to_string_lossy();
            let exe_string = exe_string.split(",").next().unwrap();

            // make sure it ends with .exe
            if !exe_string.ends_with(".exe") {
                return;
            }

            if !display_name.is_empty() {
                apps.push(App::new_executable(
                    &display_name.clone().to_string_lossy(),
                    &display_name.clone().to_string_lossy().to_lowercase(),
                    "Application",
                    exe_path,
                    None,
                ))
            }
        });
    });
}

/// Returns the set of known paths
pub fn get_known_paths() -> Vec<PathBuf> {
    let paths = vec![
        get_windows_path(&FOLDERID_ProgramFiles).unwrap_or_default(),
        get_windows_path(&FOLDERID_ProgramFilesX86).unwrap_or_default(),
        (get_windows_path(&FOLDERID_LocalAppData)
            .unwrap_or_default()
            .join("Programs")),
    ];
    paths
}

/// Wrapper around `SHGetKnownFolderPath` to get paths to known folders
fn get_windows_path(folder_id: &GUID) -> Option<PathBuf> {
    unsafe {
        let folder = SHGetKnownFolderPath(folder_id, KF_FLAG_DEFAULT, None);
        if let Ok(folder) = folder {
            let path = folder.to_string().ok()?;
            CoTaskMemFree(Some(folder.0 as *mut _));
            Some(path.into())
        } else {
            None
        }
    }
}

pub fn index_start_menu() -> Vec<App> {
    WalkDir::new(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs")
        .into_iter()
        .filter_map(|x| x.ok())
        .filter_map(|path| {
            let lnk = lnk::ShellLink::open(path.path(), get_acp());

            match lnk {
                Ok(x) => {
                    let target = x.link_target();
                    let file_name = path.file_name().to_string_lossy().to_string();

                    match target {
                        Some(target) => Some(App::new_executable(
                            &file_name,
                            &file_name,
                            "",
                            PathBuf::from(target.clone()),
                            None,
                        )),
                        None => {
                            tracing::debug!(
                                "Link at {} has no target, skipped",
                                path.path().display()
                            );
                            None
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!(
                        "Error opening link {} ({e}), skipped",
                        path.path().to_string_lossy()
                    );
                    None
                }
            }
        })
        .collect()
}
