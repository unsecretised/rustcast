//! This has all the logic regarding the cliboard history
use std::borrow::Cow;

use arboard::ImageData;

use crate::{app::apps::App, commands::Function};

/// The kinds of clipboard content that rustcast can handle and their contents
#[derive(Debug, Clone)]
pub enum ClipBoardContentType {
    Text(Cow<'static, str>),
    Image(ImageData<'static>),
}

impl ClipBoardContentType {
    /// Returns the iced element for rendering the clipboard item, and the entire content since the
    /// display name is only the first line
    pub fn to_app(&self) -> App {
        let name = match self {
            ClipBoardContentType::Image(_) => Cow::Borrowed("<img>"),
            ClipBoardContentType::Text(text) => text.clone(),
        };

        let this = self.clone();

        // only get the first line from the contents
        let name = name
            .lines()
            .next()
            .map_or_else(|| Cow::Borrowed(""), |line| Cow::Owned(line.to_owned()));

        App {
            open_command: crate::app::apps::AppCommand::Function(Function::CopyToClipboard(this)),
            desc: Cow::Borrowed("Clipboard Item"),
            icons: None,
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
