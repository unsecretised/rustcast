//! This handles all the different commands that rustcast can perform, such as opening apps,
//! copying to clipboard, etc.
use std::path::PathBuf;
use std::process::Command;
#[cfg(target_os = "macos")]
use std::thread;

use arboard::Clipboard;
#[cfg(target_os = "macos")]
use objc2_app_kit::NSWorkspace;
#[cfg(target_os = "macos")]
use objc2_foundation::NSURL;

use crate::utils::open_application;
use crate::{calculator::Expr, clipboard::ClipBoardContentType, config::Config};

/// The different functions that rustcast can perform
#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    OpenApp(PathBuf),
    RunShellCommand(String, String),
    OpenWebsite(String),
    RandomVar(i32), // Easter egg function
    CopyToClipboard(ClipBoardContentType),
    GoogleSearch(String),
    Calculate(Expr),
    OpenPrefPane,
    Quit,
}

impl Function {
    /// Run the command
    pub fn execute(&self, config: &Config, query: &str) {
        tracing::debug!("Executing command: {:?}", self);
        match self {
            Function::OpenApp(path) => open_application(path.clone()), // I think the clone is necessary
            Function::RunShellCommand(command, alias) => {
                let query = query.to_string();
                let final_command =
                    format!(r#"{} {}"#, command, query.strip_prefix(alias).unwrap_or(""));
                Command::new("sh")
                    .arg("-c")
                    .arg(final_command.trim())
                    .spawn()
                    .ok();
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
                let query = query.strip_suffix("?").unwrap_or(&query).to_string();

                open::that(query).unwrap();
            }

            Function::OpenWebsite(url) => {
                let open_url = if url.starts_with("http") {
                    url.to_owned()
                } else {
                    format!("https://{}", url)
                };

                // Should never get here without it being validated first
                open::that(open_url).unwrap();
            }

            Function::Calculate(expr) => {
                Clipboard::new()
                    .unwrap()
                    .set_text(expr.eval().map(|x| x.to_string()).unwrap_or("".to_string()))
                    .unwrap_or(());
            }

            Function::CopyToClipboard(clipboard_content) => match clipboard_content {
                ClipBoardContentType::Text(text) => {
                    Clipboard::new().unwrap().set_text(text).ok();
                }
                ClipBoardContentType::Image(img) => {
                    Clipboard::new().unwrap().set_image(img.to_owned_img()).ok();
                }
            },

            #[cfg(target_os = "macos")]
            Function::OpenPrefPane => {
                thread::spawn(move || {
                    NSWorkspace::new().openURL(&NSURL::fileURLWithPath(
                        &objc2_foundation::NSString::from_str(
                            &(std::env::var("HOME").unwrap_or("".to_string())
                                + "/.config/rustcast/config.toml"),
                        ),
                    ));
                });
            }

            Function::Quit => std::process::exit(0),
            f => {
                // TODO: something in the UI to show this
                tracing::error!("The function {:?} is unimplemented for this platform", f);
            }
        }
    }
}
