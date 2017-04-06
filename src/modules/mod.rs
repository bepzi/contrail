use std::error::Error;
use std::fmt;

use ansi_term::{Color, Style};

/// Holds information about how to style a module
pub struct ModuleStyle {
    pub background: Color,
    pub foreground: Color,
    /// Attributes like bold, italicized, underlined, etc...
    pub text_properties: Vec<Style>,
}

/// Attempts to convert a string into an `ansi_term::Color`.
///
/// Returns a `FromStringError` if the provided string doesn't match
/// any of the colors defined in crate `ansi_term`.
pub fn try_color_from_str(s: &str) -> Result<Color, FromStringError> {
    match s {
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "purple" => Ok(Color::Purple),
        "cyan" => Ok(Color::Cyan),
        "white" => Ok(Color::White),
        _ => Err(FromStringError),
    }
}

#[derive(Debug, PartialEq)]
/// Error type for when converting a string fails
pub struct FromStringError;

impl fmt::Display for FromStringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for FromStringError {
    fn description(&self) -> &str {
        "unrecognized input"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_try_from_str() {
        assert_eq!(try_color_from_str("blue"), Ok(Color::Blue));
        assert_eq!(try_color_from_str("teal"), Err(FromStringError));
    }
}
