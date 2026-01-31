//! This has all the utility functions that rustcast uses
use std::{
    io,
    path::{Path, PathBuf},
    thread,
    time::Instant,
};

use rayon::prelude::*;

#[cfg(target_os = "macos")]
use {objc2_app_kit::NSWorkspace, objc2_foundation::NSURL};

#[cfg(target_os = "windows")]
use std::process::Command;

use crate::app::apps::App;

pub fn get_config_installation_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        std::env::var("LOCALAPPDATA").unwrap().into()
    } else {
        std::env::var("HOME").unwrap().into()
    }
}

pub fn get_config_file_path() -> PathBuf {
    let home = get_config_installation_dir();

    if cfg!(target_os = "windows") {
        home.join("rustcast/config.toml")
    } else {
        home.join(".config/rustcast/config.toml")
    }
}

/// Recursively loads apps from a set of folders.
///
/// [`exclude_patterns`] is a set of glob patterns to include, while [`include_patterns`] is a set of
/// patterns to include ignoring [`exclude_patterns`].
fn search_dir(
    path: impl AsRef<Path>,
    exclude_patterns: &[glob::Pattern],
    include_patterns: &[glob::Pattern],
    max_depth: usize,
) -> impl ParallelIterator<Item = App> {
    use crate::{app::apps::AppCommand, commands::Function};
    use walkdir::WalkDir;

    WalkDir::new(path.as_ref())
        .follow_links(false)
        .max_depth(max_depth)
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

            #[cfg(target_os = "windows")]
            let icon = {
                use crate::cross_platform::windows::appicon::get_first_icon;

                get_first_icon(path)
                    .inspect_err(|e| {
                        tracing::error!("Error getting icon for {}: {e}", path.display())
                    })
                    .ok()
                    .flatten()
            };

            #[cfg(not(target_os = "windows"))]
            let icon = None;

            Some(App {
                open_command: AppCommand::Function(Function::OpenApp(path.to_path_buf())),
                name: name.clone(),
                name_lc: name.to_lowercase(),
                icons: icon,
                desc: "Application".to_string(),
            })
        })
}

use crate::config::Config;

pub fn read_config_file(file_path: &Path) -> anyhow::Result<Config> {
    match std::fs::read_to_string(file_path) {
        Ok(a) => Ok(toml::from_str(&a)?),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let cfg = Config::default();
            std::fs::write(
                file_path,
                toml::to_string(&cfg).unwrap_or_else(|x| x.to_string()),
            )?;
            Ok(cfg)
        }
        Err(e) => Err(e.into()),
    }
}

pub fn open_application(path: PathBuf) {
    thread::spawn(move || {
        #[cfg(target_os = "windows")]
        {
            println!("Opening application: {}", &path.display());

            Command::new("powershell")
                .arg(format!("Start-Process '{}'", &path.to_string_lossy()))
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
    });
}

pub fn index_installed_apps(config: &Config) -> anyhow::Result<Vec<App>> {
    tracing::debug!("Indexing installed apps");
    tracing::debug!("Exclude patterns: {:?}", &config.index_exclude_patterns);
    tracing::debug!("Include patterns: {:?}", &config.index_include_patterns);

    let path = get_config_file_path();
    let config = read_config_file(path.as_path())?;

    if config.index_dirs.is_empty() {
        tracing::debug!("No extra index dirs provided")
    }

    #[cfg(target_os = "windows")]
    {
        use crate::cross_platform::windows::app_finding::get_apps_from_registry;
        use crate::cross_platform::windows::app_finding::index_start_menu;

        let start = Instant::now();

        let mut other_apps = index_start_menu();
        get_apps_from_registry(&mut other_apps);

        let res = config
            .index_dirs
            .par_iter()
            .flat_map(|x| {
                search_dir(
                    &x.path,
                    &config.index_exclude_patterns,
                    &config.index_include_patterns,
                    x.max_depth,
                )
            })
            .chain(other_apps.into_par_iter())
            .collect();

        let end = Instant::now();
        tracing::info!(
            "Finished indexing apps (t = {}s)",
            (end - start).as_secs_f32()
        );

        Ok(res)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let start = Instant::now();

        let res = config
            .index_dirs
            .par_iter()
            .flat_map(|x| {
                search_dir(
                    &x.path,
                    &config.index_exclude_patterns,
                    &config.index_include_patterns,
                    x.max_depth,
                )
            })
            .collect();

        let end = Instant::now();
        tracing::info!(
            "Finished indexing apps (t = {}s)",
            (end - start).as_secs_f32()
        );

        Ok(res)
    }
}

/// Converts a slice of BGRA data to RGBA using SIMD
///
/// Stolen from https://stackoverflow.com/a/78190249/
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn bgra_to_rgba(data: &mut [u8]) {
    use std::arch::x86_64::__m128i;
    use std::arch::x86_64::_mm_loadu_si128;
    use std::arch::x86_64::_mm_setr_epi8;
    use std::arch::x86_64::_mm_storeu_si128;

    #[cfg(target_arch = "x86")]
    use std::arch::x86::_mm_shuffle_epi8;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::_mm_shuffle_epi8;
    //
    // The shuffle mask for converting BGRA -> RGBA
    let mask: __m128i = unsafe {
        _mm_setr_epi8(
            2, 1, 0, 3, // First pixel
            6, 5, 4, 7, // Second pixel
            10, 9, 8, 11, // Third pixel
            14, 13, 12, 15, // Fourth pixel
        )
    };
    // For each 16-byte chunk in your data
    for chunk in data.chunks_exact_mut(16) {
        let mut vector = unsafe { _mm_loadu_si128(chunk.as_ptr() as *const __m128i) };
        vector = unsafe { _mm_shuffle_epi8(vector, mask) };
        unsafe { _mm_storeu_si128(chunk.as_mut_ptr() as *mut __m128i, vector) };
    }
}

// Fallback for non x86/x86_64 devices (not like that'll ever be used, but why not)
/// Converts a slice of BGRA data to RGBA
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn bgra_to_rgba(data: &mut [u8]) {
    for i in (0..data.len()).step_by(4) {
        let r = data[i + 2];

        data[i + 2] = data[i];
        data[i] = r;
    }
}
