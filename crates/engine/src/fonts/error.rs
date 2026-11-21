//! Why a font could not be registered or found.

use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum FontError {
    NotRegistered(String),
    NoMatch {
        family: String,
        weight: u16,
        italic: bool,
    },
    UnknownStandard(String),
    Collection,
    Invalid,
}

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FontError::NotRegistered(family) => write!(
                f,
                "Font family not registered: {family}. Please register it calling Font.register() method."
            ),
            FontError::NoMatch {
                family,
                weight,
                italic,
            } => {
                let style = if *italic { "italic" } else { "normal" };
                write!(
                    f,
                    "Could not resolve font for {family}, fontWeight {weight}, fontStyle {style}"
                )
            }
            FontError::UnknownStandard(name) => write!(f, "Unknown standard font: {name}"),
            FontError::Collection => write!(f, "Font collection is not supported"),
            FontError::Invalid => write!(f, "Unknown font format"),
        }
    }
}

impl std::error::Error for FontError {}
