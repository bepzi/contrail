use std::convert::From;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

use ansi_term::{Color, Style};

/// Holds information about how to style a module
pub struct ModuleStyle {
    pub background: Color,
    pub foreground: Color,
    /// Attributes like bold, italicized, underlined, etc...
    pub text_properties: Vec<Style>,
}

// We use methods of the form "try_*_from_*" because the user should
// just be able to specify (in their config) what color/style they
// want, and we have to be able to parse what they say into a
// meaningful type.

/// Attempts to convert a string into an `ansi_term::Color`.
///
/// Returns a `ConvertError` if the provided string doesn't match
/// any of the colors defined in crate `ansi_term`.
pub fn try_color_from_str(s: &str) -> Result<Color, ConvertError> {
    match s {
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "purple" => Ok(Color::Purple),
        "cyan" => Ok(Color::Cyan),
        "white" => Ok(Color::White),
        _ => Err(ConvertError::NoSuchMatch),
    }
}

// NOTE: We do not need a try_color_from_u8.
// The implementation would just look like:
// pub fn try_color_from_u8(i: u8) -> Color { Color::u8(i) }
// It would always succeed due to Rust's type system.

/// Attempts to convert a string into an `ansi_term::Color::RGB`.
///
/// Returns a `ConvertError::InvalidForm` if the provided string is
/// not a sequence of 3 `u8`s, separated by commas.
///
/// # Examples
/// ```
/// # use modules::*
/// assert_eq!(try_rgb_from_str("(14, 76, 1)"), Ok(Color::RGB(14, 76, 1)));
/// assert_eq!(try_rgb_from_str("0, 100, 0"), Ok(Color::RGB(0, 100, 0)));
/// assert_eq!(try_rgb_from_str("1000, b, c, -1"), Err(ConvertError::InvalidForm));
/// ```
pub fn try_rgb_from_str(s: &str) -> Result<Color, ConvertError> {
    // Strip out non-integer characters
    let cleaned: Vec<String> = s.split(',')
        .map(|i| i.replace(|j| j == '(' || j == ')' || j == ' ', ""))
        .collect();

    // We want to immediately return an Error if a conversion fails
    let mut ints: Vec<u8> = Vec::new();
    for i in &cleaned {
        ints.push(i.parse::<u8>()?);
    }

    // Note: somewhat-malformed input, like "()()(0, 0, 0)" can still
    // get through, but we can still make "sense" of it so it's good
    // to go.
    if ints.len() != 3 {
        Err(ConvertError::InvalidForm)
    } else {
        Ok(Color::RGB(ints[0], ints[1], ints[2]))
    }
}

#[derive(Debug, PartialEq)]
/// Error type for when a conversion fails
pub enum ConvertError {
    /// Input doesn't correspond to a valid result
    NoSuchMatch,
    /// Input is malformed and cannot be parsed
    InvalidForm,
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ConvertError {
    fn description(&self) -> &str {
        match *self {
            ConvertError::NoSuchMatch => "no match was found for the provided input",
            ConvertError::InvalidForm => "provided input was malformed and could not be parsed",
        }
    }
}

// So that we can use try!() and ? to return early if we encounter a
// ParseIntError
impl From<ParseIntError> for ConvertError {
    fn from(err: ParseIntError) -> ConvertError {
        ConvertError::InvalidForm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_try_from_str() {
        assert_eq!(try_color_from_str("blue"), Ok(Color::Blue));
        assert_eq!(try_color_from_str("teal"), Err(ConvertError::NoSuchMatch));
    }

    #[test]
    fn test_rgb_try_from_str() {
        // Valid inputs
        assert_eq!(try_rgb_from_str("(0, 0, 0)"), Ok(Color::RGB(0, 0, 0)));
        assert_eq!(try_rgb_from_str("(255, 255, 255)"),
                   Ok(Color::RGB(255, 255, 255)));

        // Questionable inputs (should still work)
        assert_eq!(try_rgb_from_str("(0, 0, 0))"), Ok(Color::RGB(0, 0, 0)));

        // Improperly formed
        assert_eq!(try_rgb_from_str("(0, 0, 0,)"),
                   Err(ConvertError::InvalidForm));

        // Too few inputs
        assert_eq!(try_rgb_from_str("(0, 0)"), Err(ConvertError::InvalidForm));

        // Inputs aren't u8's
        assert_eq!(try_rgb_from_str("(1000, 0, 0)"),
                   Err(ConvertError::InvalidForm));
        assert_eq!(try_rgb_from_str("(0, 0, -1)"),
                   Err(ConvertError::InvalidForm));
    }
}
