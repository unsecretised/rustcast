use {
    crate::app::apps::App,
    rayon::prelude::*,
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

fn get_apps_from_registry(apps: &mut Vec<App>) {
    use std::ffi::OsString;
    let hkey = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);

    let registers = [
        hkey.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
            .unwrap(),
        hkey.open_subkey("SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
            .unwrap(),
    ];

    // where we can find installed applications
    // src: https://stackoverflow.com/questions/2864984/how-to-programatically-get-the-list-of-installed-programs/2892848#2892848
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
fn get_apps_from_known_folder(apps: &mut Vec<App>) {
    let paths = get_known_paths();
    use crate::{app::apps::AppCommand, commands::Function};
    use walkdir::WalkDir;

    let found_apps: Vec<App> = paths
        .par_iter()
        .flat_map(|path| {
            WalkDir::new(path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "exe"))
                .map(|entry| {
                    let path = entry.path();
                    let file_name = path.file_name().unwrap().to_string_lossy();
                    let name = file_name.replace(".exe", "");

                    #[cfg(debug_assertions)]
                    tracing::trace!("Executable loaded [kfolder]: {:?}", path.to_str());

                    App {
                        open_command: AppCommand::Function(Function::OpenApp(
                            path.to_string_lossy().to_string(),
                        )),
                        name: name.clone(),
                        name_lc: name.to_lowercase(),
                        icons: None,
                        desc: "Application".to_string(),
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();

    apps.extend(found_apps);
}
fn get_known_paths() -> Vec<String> {
    let paths = vec![
        get_windows_path(&FOLDERID_ProgramFiles).unwrap_or_default(),
        get_windows_path(&FOLDERID_ProgramFilesX86).unwrap_or_default(),
        (get_windows_path(&FOLDERID_LocalAppData).unwrap_or_default() + "\\Programs\\"),
    ];
    paths
}
fn get_windows_path(folder_id: &GUID) -> Option<String> {
    unsafe {
        let folder = SHGetKnownFolderPath(folder_id, KF_FLAG_DEFAULT, None);
        if let Ok(folder) = folder {
            let path = folder.to_string().ok();
            CoTaskMemFree(Some(folder.0 as *mut _));
            path
        } else {
            None
        }
    }
}
pub fn get_installed_windows_apps() -> Vec<App> {
    use crate::utils::index_dirs_from_config;

    let mut apps = Vec::new();

    tracing::debug!("Getting apps from registry");
    get_apps_from_registry(&mut apps);

    tracing::debug!("Getting apps from known folder");
    get_apps_from_known_folder(&mut apps);

    tracing::debug!("Getting apps from config");
    index_dirs_from_config(&mut apps);

    tracing::debug!("Apps loaded ({} total count)", apps.len());

    apps
}

pub fn open_on_focused_monitor() -> iced::Point {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
    };

    use crate::app::{DEFAULT_WINDOW_HEIGHT, WINDOW_WIDTH};
    let mut point = POINT { x: 0, y: 0 };
    let mut monitor_info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    let _cursor = unsafe { GetCursorPos(&mut point) };
    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
    let _monitor_infos = unsafe { GetMonitorInfoW(monitor, &mut monitor_info) };

    let monitor_width = monitor_info.rcMonitor.right - monitor_info.rcMonitor.left;
    let monitor_height = monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top;
    let window_width = WINDOW_WIDTH;
    let window_height = DEFAULT_WINDOW_HEIGHT;

    let x = monitor_info.rcMonitor.left as f32 + (monitor_width as f32 - window_width) / 2.0;
    let y = monitor_info.rcMonitor.top as f32 + (monitor_height as f32 - window_height) / 2.0;

    iced::Point { x, y }
}
