use std::process::Command;

use arboard::Clipboard;
#[cfg(target_os = "macos")]
use objc2_app_kit::NSWorkspace;
#[cfg(target_os = "macos")]
use objc2_foundation::NSURL;

use crate::config::Config;
use crate::utils::{get_config_file_path, open_application};

#[derive(Debug, Clone)]
pub enum Function {
    OpenApp(String),
    RunShellCommand,
    RandomVar(i32),
    GoogleSearch(String),
    OpenPrefPane,
    Quit,
}

impl Function {
    pub fn execute(&self, config: &Config, query: &str) {
        match self {
            Function::OpenApp(path) => {
                open_application(path);
            }
            Function::RunShellCommand => {
                Command::new("sh").arg("-c").arg(query).status().ok();
            }
            Function::RandomVar(var) => {
                Clipboard::new()
                    .unwrap()
                    .set_text(var.to_string())
                    .unwrap_or(());
            }

            Function::GoogleSearch(query_string) => {
                let query_args = query_string.replace(" ", "+");
                let query = config.search_url.replace("%s", &query_args);
                let query = query.strip_suffix("?").unwrap_or(&query);

                #[cfg(target_os = "windows")]
                {
                    Command::new("powershell")
                        .args(["-Command", &format!("Start-Process {}", query)])
                        .status()
                        .ok();
                }

                #[cfg(target_os = "macos")]
                {
                    NSWorkspace::new().openURL(
                        &NSURL::URLWithString_relativeToURL(
                            &objc2_foundation::NSString::from_str(query),
                            None,
                        )
                        .unwrap(),
                    );
                }
            }

            Function::OpenPrefPane => {
                Command::new("open")
                    .arg(get_config_file_path())
                    .spawn()
                    .ok();
            }
            Function::Quit => std::process::exit(0),
        }
    }
}
