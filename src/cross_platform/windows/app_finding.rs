use {
    crate::{
        app::apps::App,
        cross_platform::windows::{appicon::get_first_icon, get_acp},
    },
    lnk::ShellLink,
    std::path::{Path, PathBuf},
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
            tracing::trace!(
                target: "reg_app_search",
                "App added: {:?}",
                key
            );

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
            let exe = PathBuf::from(exe_path.split(",").next().unwrap());

            // make sure it ends with .exe
            if exe.extension() != Some(&OsString::from("exe")) {
                return;
            }

            if !display_name.is_empty() {
                let icon = get_first_icon(&exe)
                    .inspect_err(|e| tracing::error!("Error getting icons: {e}"))
                    .ok()
                    .flatten();

                apps.push(App::new_executable(
                    &display_name.clone().to_string_lossy(),
                    &display_name.clone().to_string_lossy().to_lowercase(),
                    "Application",
                    exe,
                    icon,
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

fn parse_link(lnk: ShellLink, link_path: impl AsRef<Path>) -> Option<App> {
    let link_path = link_path.as_ref();

    let Some(target) = lnk.link_target() else {
        tracing::trace!(
            target: "smenu_app_search",
            "Link at {} has no target, skipped",
            link_path.display()
        );
        return None;
    };
    let target = PathBuf::from(target);

    tracing::trace!(
        "Link at {} loaded (target: {:?})",
        link_path.display(),
        &target
    );

    let Some(file_name) = target.file_name() else {
        tracing::trace!(
            target: "smenu_app_search",
            "Link at {} skipped (not pointing to a directory)",
            link_path.display()
        );
        return None;
    };

    tracing::trace!(
        target: "smenu_app_search",
        "Link at {} added",
        link_path.display()
    );

    Some(App::new_executable(
        &file_name.to_string_lossy(),
        &file_name.to_string_lossy().to_lowercase(),
        "Shortcut",
        target.clone(),
        None,
    ))
}

pub fn index_start_menu() -> Vec<App> {
    WalkDir::new(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs")
        .into_iter()
        .filter_map(|x| x.ok())
        .filter_map(|entry| {
            let ext = entry.path().extension();
            let path = entry.path();

            if ext.is_none() {
                tracing::trace!("{} has no extension (maybe a dir)", path.display());
                return None;
            }

            if let Some(ext) = ext
                && ext != "lnk"
            {
                tracing::trace!("{} not a .lnk file, skipping", path.display());
                return None;
            }

            let lnk = lnk::ShellLink::open(path, get_acp());

            match lnk {
                Ok(x) => parse_link(x, path),
                Err(e) => {
                    tracing::trace!(
                        target: "smenu_app_search",
                        "Error opening link {} ({e}), skipped",
                        entry.path().to_string_lossy()
                    );
                    None
                }
            }
        })
        .collect()
}
