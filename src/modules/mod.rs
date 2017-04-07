use std::convert::From;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

use ansi_term::{ANSIString, Color, Style};

use util::Shell;

/// Representation of config options that all modules have
#[derive(Clone)]
pub struct ModuleOptions {
    /// String to display to the left of the content
    pub padding_left: String,
    /// String to display to the right of the content
    pub padding_right: String,
    /// String to print out after the content and right padding
    pub separator: String,
    /// Background color, foreground color, etc.
    pub style: ModuleStyle,
}

/// Representation of how to style a module
#[derive(Clone)]
pub struct ModuleStyle {
    /// Color behind the text
    pub background: Option<Color>,
    /// Color of the text
    pub foreground: Option<Color>,
    /// Optional combination of attributes like bold, italicized, etc.
    pub text_properties: Option<Style>,
}

/// Formats a string with the given `ModuleOptions` for a specific
/// `Shell`.
///
/// # Parameters
///
/// - `s` - the contents of the module to be formatted
/// - `options` - the background, foreground, padding, etc. to apply
/// - `next_bg` - the background color, if any, of the next visible module
/// - `shell` - the type of shell to format the string for
pub fn format_for_module(s: &str,
                         options: ModuleOptions,
                         next_bg: Option<Color>,
                         shell: Shell)
                         -> ANSIString<'static> {

    let style = style_from_modulestyle(&options.style);

    // Each shell keeps track of the number of characters that make up
    // the prompt. The ANSI escape-sequences that color the text will
    // be accidentally included in this length *unless* we prefix and
    // suffix them with these shell-specific escape-sequences. We
    // don't want the shell to mistakenly think there's fewer
    // characters remaining on the current line than there actually
    // are.
    let (len_esc_prefix, len_esc_suffix) = match shell {
        Shell::Bash => ("\\[", "\\]"),
        Shell::Zsh => ("%{", "%}"),
    };

    unimplemented!();
}

/// Converts a `ModuleStyle` into an `ansi_term::Style`.
fn style_from_modulestyle(s: &ModuleStyle) -> Style {
    let mut style = s.text_properties.unwrap_or_default();
    if let Some(bg) = s.background {
        style = style.on(bg);
    }
    if let Some(fg) = s.foreground {
        style = style.fg(fg);
    }
    style
}

// We use methods of the form "try_*_from_*" because the user should
// just be able to specify (in their config) what color/style they
// want, and we have to be able to parse what they say into a
// meaningful type.

/// Attempts to convert a string into an `ansi_term::Color`.
///
/// Returns a `ConvertError::NoSuchMatch` if the provided string
/// doesn't match any of the colors defined in crate `ansi_term`.
///
/// # Examples
/// ```
/// assert_eq!(try_color_from_str("black"), Ok(Color::Black));
/// assert_eq!(try_color_from_str("green"), Ok(Color::Green));
/// assert_eq!(try_color_from_str("turquoise"), Err(ConvertError::NoSuchMatch));
/// ```
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

/// Attempts to convert a string into an `ansi_term::Color::RGB`.
///
/// Returns a `ConvertError::InvalidForm` if the provided string is
/// not a sequence of 3 `u8`s, separated by commas.
///
/// # Examples
/// ```
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

// NOTE: We do not need a try_color_from_u8 OR a try_rgb_from_vec.
// The implementation would just look like:
// pub fn try_color_from_u8(i: u8) -> Color { Color::u8(i) }
// It would always succeed due to Rust's type system.

#[derive(Debug, PartialEq)]
/// Error type for when a conversion due to parsing fails
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
    fn from(_: ParseIntError) -> ConvertError {
        ConvertError::InvalidForm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_from_modulestyle() {
        const CONTENT: &'static str = "Hello";

        struct Test {
            style: ModuleStyle,
            expected: String,
        }

        let tests = [// Normal text, no bg, custom fg
                     Test {
                         style: ModuleStyle {
                             background: None,
                             foreground: Some(Color::White),
                             text_properties: None,
                         },
                         expected: format!("\x1B[37m{}\x1B[0m", CONTENT),
                     },
                     // Bold text, no fg, custom bg
                     Test {
                         style: ModuleStyle {
                             background: Some(Color::Blue),
                             foreground: None,
                             text_properties: Some(Style::default().bold()),
                         },
                         expected: format!("\x1B[1;44m{}\x1B[0m", CONTENT),
                     },
                     // Underlined text, custom bg and fg
                     Test {
                         style: ModuleStyle {
                             background: Some(Color::Blue),
                             foreground: Some(Color::White),
                             text_properties: Some(Style::default().underline()),
                         },
                         expected: format!("\x1B[4;44;37m{}\x1B[0m", CONTENT),
                     },
                     // Normal text, no bg nor fg
                     Test {
                         style: ModuleStyle {
                             background: None,
                             foreground: None,
                             text_properties: None,
                         },
                         expected: format!("{}", CONTENT),
                     }];

        for test in &tests {
            let result = format!("{}", style_from_modulestyle(&test.style).paint(CONTENT));
            assert_eq!(result, test.expected);
        }
    }

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

        // Improperly formed ("too many" inputs)
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
