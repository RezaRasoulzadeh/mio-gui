// clipboard.rs

use std::error::Error;
use std::fmt::{Display, Formatter};

pub trait TextClipboard {
    fn read_text(&mut self) -> Result<String, ClipboardError>;
    fn write_text(&mut self, text: &str) -> Result<(), ClipboardError>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct ClipboardError(String);

impl ClipboardError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for ClipboardError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for ClipboardError {}

pub struct SystemClipboard {
    clipboard: arboard::Clipboard,
}

impl SystemClipboard {
    pub fn new() -> Result<Self, ClipboardError> {
        arboard::Clipboard::new()
            .map(|clipboard| Self { clipboard })
            .map_err(map_error)
    }
}

impl TextClipboard for SystemClipboard {
    fn read_text(&mut self) -> Result<String, ClipboardError> {
        self.clipboard.get_text().map_err(map_error)
    }

    fn write_text(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.clipboard.set_text(text).map_err(map_error)
    }
}

fn map_error(error: arboard::Error) -> ClipboardError {
    ClipboardError::new(error.to_string())
}
