use std::process::Command;

use arboard::Clipboard;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::NSURL;

use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Function {
    OpenApp(String),
    RunShellCommand(String),
    RandomVar(i32),
    GoogleSearch(String),
    OpenPrefPane,
    Quit,
}

impl Function {
    pub fn execute(&self, config: &Config, query: &str) {
        match self {
            Function::OpenApp(path) => {
                NSWorkspace::new().openURL(&NSURL::fileURLWithPath(
                    &objc2_foundation::NSString::from_str(path),
                ));
            }
            Function::RunShellCommand(shell_command) => {
                let mut final_command = shell_command.to_owned();

                for (argument_num, argument) in query.split_whitespace().enumerate() {
                    final_command =
                        final_command.replace(&format!("$var{}", argument_num), argument);
                }
                Command::new("sh")
                    .arg("-c")
                    .arg(final_command)
                    .status()
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
                let query = query.strip_suffix("?").unwrap_or(&query);
                NSWorkspace::new().openURL(
                    &NSURL::URLWithString_relativeToURL(
                        &objc2_foundation::NSString::from_str(query),
                        None,
                    )
                    .unwrap(),
                );
            }

            Function::OpenPrefPane => {
                Command::new("open")
                    .arg(
                        std::env::var("HOME").unwrap_or("".to_string())
                            + "/.config/rustcast/config.toml",
                    )
                    .spawn()
                    .ok();
            }
            Function::Quit => std::process::exit(0),
        }
    }
}
