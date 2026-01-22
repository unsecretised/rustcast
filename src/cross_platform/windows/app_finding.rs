use {
    crate::app::apps::App,
    rayon::prelude::*,
    std::path::PathBuf,
    windows::{
        Win32::{
            System::Com::CoTaskMemFree,
            UI::{
                Shell::{
                    FOLDERID_LocalAppData, FOLDERID_ProgramFiles, FOLDERID_ProgramFilesX86,
                    KF_FLAG_DEFAULT, SHGetKnownFolderPath,
                },
                WindowsAndMessaging::GetCursorPos,
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
fn get_apps_from_registry(apps: &mut Vec<App>) {
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
            let exe_path = exe_path.to_string_lossy().to_string();
            let exe = exe_path.split(",").next().unwrap().to_string();

            // make sure it ends with .exe
            if !exe.ends_with(".exe") {
                return;
            }

            if !display_name.is_empty() {
                use crate::{app::apps::AppCommand, commands::Function};

                apps.push(App {
                    open_command: AppCommand::Function(Function::OpenApp(exe)),
                    name: display_name.clone().into_string().unwrap(),
                    name_lc: display_name.clone().into_string().unwrap().to_lowercase(),
                    icons: None,
                    desc: "Application".to_string(),
                })
            }
        });
    });
}

/// Recursively loads apps from a set of folders.
///
/// [`exclude_patterns`] is a set of glob patterns to include, while [`include_patterns`] is a set of
/// patterns to include ignoring [`exclude_patterns`].
fn get_apps_from_known_folder(
    exclude_patterns: &[glob::Pattern],
    include_patterns: &[glob::Pattern],
) -> impl ParallelIterator<Item = App> {
    let paths = get_known_paths();
    use crate::{app::apps::AppCommand, commands::Function};
    use walkdir::WalkDir;

    paths.into_par_iter().flat_map(|path| {
        WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .par_bridge()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "exe"))
            .filter_map(|entry| {
                let path = entry.path();

                if exclude_patterns.iter().any(|x| x.matches_path(path))
                    && !include_patterns.iter().any(|x| x.matches_path(path))
                {
                    #[cfg(debug_assertions)]
                    tracing::trace!("Executable skipped [kfolder]: {:?}", path.to_str());

                    return None;
                }

                let file_name = path.file_name().unwrap().to_string_lossy();
                let name = file_name.replace(".exe", "");

                #[cfg(debug_assertions)]
                tracing::trace!("Executable loaded  [kfolder]: {:?}", path.to_str());

                Some(App {
                    open_command: AppCommand::Function(Function::OpenApp(
                        path.to_string_lossy().to_string(),
                    )),
                    name: name.clone(),
                    name_lc: name.to_lowercase(),
                    icons: None,
                    desc: "Application".to_string(),
                })
            })
    })
}

/** Returns the set of known paths */
fn get_known_paths() -> Vec<PathBuf> {
    let paths = vec![
        get_windows_path(&FOLDERID_ProgramFiles).unwrap_or_default(),
        get_windows_path(&FOLDERID_ProgramFilesX86).unwrap_or_default(),
        (get_windows_path(&FOLDERID_LocalAppData)
            .unwrap_or_default()
            .join("Programs/")),
    ];
    paths
}

/** Wrapper around `SHGetKnownFolderPath` to get paths to known folders */
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

/// Gets windows apps
///
/// When searching known folders, [`exclude_patterns`] is a set of glob patterns to include, while
/// [`include_patterns`] is a set of patterns to include ignoring [`exclude_patterns`].
pub fn get_installed_windows_apps(
    exclude_patterns: &[glob::Pattern],
    include_patterns: &[glob::Pattern],
) -> Vec<App> {
    use crate::utils::index_dirs_from_config;

    let mut apps = Vec::new();

    tracing::debug!("Getting apps from registry");
    get_apps_from_registry(&mut apps);

    tracing::debug!("Getting apps from known folder");
    apps.par_extend(get_apps_from_known_folder(
        exclude_patterns,
        include_patterns,
    ));

    tracing::debug!("Getting apps from config");
    index_dirs_from_config(&mut apps);

    tracing::debug!("Apps loaded ({} total count)", apps.len());

    apps
}