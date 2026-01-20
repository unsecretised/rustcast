//! This has all the logic regarding the cliboard history
use arboard::ImageData;

use crate::{app::apps::App, commands::Function};

/// The kinds of clipboard content that rustcast can handle and their contents
#[derive(Debug, Clone)]
pub enum ClipBoardContentType {
    Text(String),
    Image(ImageData<'static>),
}

impl ClipBoardContentType {
    /// Returns the iced element for rendering the clipboard item, and the entire content since the
    /// display name is only the first line
    pub fn to_app(&self) -> App {
        let name = match self {
            ClipBoardContentType::Image(_) => "<img>".to_string(),
            ClipBoardContentType::Text(a) => {
                a.lines().next().unwrap_or(a.as_str()).to_string()
            }
        };

        let self_clone = self.clone();

        App {
            open_command: crate::app::apps::AppCommand::Function(Function::CopyToClipboard(
                self_clone.to_owned(),
            )),
            desc: "Clipboard Item".to_string(),
            icons: None,
            name_lc: name.clone(),
            name,
        }
    }
}

impl PartialEq for ClipBoardContentType {
    /// Let cliboard items be comparable
    fn eq(&self, other: &Self) -> bool {
        if let Self::Text(a) = self
            && let Self::Text(b) = other
        {
            return a == b;
        } else if let Self::Image(image_data) = self
            && let Self::Image(other_image_data) = other
        {
            return image_data.bytes == other_image_data.bytes;
        }
        false
    }
}
