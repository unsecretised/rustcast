use std::{process::Command, thread};

use arboard::Clipboard;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::NSURL;

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Function {
    OpenApp(String),
    RunShellCommand(String, String),
    RandomVar(i32),
    GoogleSearch(String),
    OpenPrefPane,
    Quit,
}

impl Function {
    pub fn execute(&self, config: &Config, query: &str) {
        match self {
            Function::OpenApp(path) => {
                let path = path.to_owned();
                thread::spawn(move || {
                    NSWorkspace::new().openURL(&NSURL::fileURLWithPath(
                        &objc2_foundation::NSString::from_str(&path),
                    ));
                });
            }
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
                thread::spawn(move || {
                    NSWorkspace::new().openURL(
                        &NSURL::URLWithString_relativeToURL(
                            &objc2_foundation::NSString::from_str(&query),
                            None,
                        )
                        .unwrap(),
                    );
                });
            }

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
        }
    }
}
