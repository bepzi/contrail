use std::default::Default;

use ansi_term::{ANSIString, Color, Style};
use config::{Config, Value};
use clap::Shell;

use utils::{Error, ErrorKind};

mod cwd;
mod exit_code;
mod generic;
mod git;
mod prompt;

pub use self::cwd::*;
pub use self::exit_code::*;
pub use self::generic::*;
pub use self::git::*;
pub use self::prompt::*;

/// Representation of config options that all modules have
#[derive(Debug, PartialEq)]
pub struct ModuleOptions {
    /// String that, if present, overrides the output of the module
    pub output: Option<String>,
    /// String to display to the left of the content
    pub padding_left: String,
    /// String to display to the right of the content
    pub padding_right: String,
    /// String to print out after the content and right padding
    pub separator: String,
    /// Background color, foreground color, etc.
    pub style: ModuleStyle,
}

impl Default for ModuleOptions {
    fn default() -> ModuleOptions {
        // These defaults should be kept in sync with the
        // read_options() method (not very hard to do)
        ModuleOptions {
            output: None,
            padding_left: String::from(" "),
            padding_right: String::from(" "),
            separator: String::from(""),
            style: ModuleStyle::default(),
        }
    }
}

/// Representation of how to style a module
#[derive(Debug, Default, PartialEq)]
pub struct ModuleStyle {
    /// Color behind the text
    pub background: Option<Color>,
    /// Color of the text
    pub foreground: Option<Color>,
    /// Combination of attributes like bold, italicized, etc.
    pub text_properties: Option<Style>,
}

/// Turns a `Value` into a `String` or returns an `Error` if the
/// `Value` wasn't a `String` to begin with.
///
// Crate `config` has an `into_str()` method which always succeeds,
// but for this program the config values should be strongly
// typed. Putting an `i64` where a `String` was expected should be an
// error.
fn unwrap_value_if_string(v: Value) -> Result<String, Error> {
    match v {
        Value::String(s) => Ok(s),
        _ => {
            Err(Error::new(ErrorKind::InvalidTypeInConfig,
                           &format!("expected string, got: {:?}", v)))
        }
    }
}

// NOTE: This is the only config-parsing method from this file that's
// meant to be called explicitly from other parts of the code. The
// other methods are helper methods.
/// Gets a module's options from a config file.
///
/// `key` refers to the name of the module, for example, "prompt". The
/// padding, separator, and style will be fetched using
/// "modules.<key>.<padding/etc>".
///
/// Returns an `Error` if any of the options in the config file fail
/// to be parsed.
pub fn read_options(key: &str, config: &Config) -> Result<ModuleOptions, Error> {
    let padding_left = if let Some(val) = config.get(&format!("modules.{}.padding_left", key)) {
        unwrap_value_if_string(val)?
    } else {
        String::from(" ")
    };

    let padding_right = if let Some(val) = config.get(&format!("modules.{}.padding_right", key)) {
        unwrap_value_if_string(val)?
    } else {
        String::from(" ")
    };

    let separator = if let Some(val) = config.get(&format!("modules.{}.separator", key)) {
        unwrap_value_if_string(val)?
    } else {
        String::from("")
    };

    let overridden_output = if let Some(val) = config.get(&format!("modules.{}.output", key)) {
        Some(unwrap_value_if_string(val)?)
    } else {
        None
    };

    let style = read_style(&format!("modules.{}.style", key), config)?;

    Ok(ModuleOptions {
           output: overridden_output,
           padding_left: padding_left,
           padding_right: padding_right,
           separator: separator,
           style: style,
       })
}

/// Gets a module's style from a config file.
///
/// `key` refers to the style of the module, for example,
/// `modules.prompt.style_success`.
///
/// Returns an `Error` if any of the options in the config file fail
/// to be parsed.
pub fn read_style(key: &str, config: &Config) -> Result<ModuleStyle, Error> {
    // The layout of a config file looks something like this:
    // [modules.<module_name>]
    // separator = "something"
    // # etc.. more options
    //
    // And to *style* a module, we expect something like this:
    // [modules.<module_name>.style]
    // foreground = "white"
    // background = "(255, 255, 255)"
    // text_properties = ["bold", "italicized"]

    // If nothing is specified for foreground, background, or
    // text_properties, we should assume `None` for the `Style` we
    // will return
    let bg = try_color_from_config(&format!("{}.background", key), config)?;
    let fg = try_color_from_config(&format!("{}.foreground", key), config)?;
    let text = try_text_props_from_config(&format!("{}.text_properties", key), config)?;

    Ok(ModuleStyle {
           background: bg,
           foreground: fg,
           text_properties: text,
       })
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
pub fn format_for_module<S: Into<String>>(s: S,
                                          options: &ModuleOptions,
                                          next_bg: Option<Color>,
                                          shell: Shell)
                                          -> ANSIString<'static> {
    let s = if let Some(ref output) = options.output {
        // Override output if present
        output.to_owned()
    } else {
        // Allow usage of String or &str
        s.into()
    };
    let style = style_from_modulestyle(&options.style);

    // Each shell keeps track of the number of characters that make up
    // the prompt. The ANSI escape-sequences that color the text will
    // be accidentally included in this length *unless* we prefix and
    // suffix them with these shell-specific escape-sequences. We
    // don't want the shell to mistakenly think there's fewer
    // characters remaining on the current line than there actually
    // are.
    let (len_esc_prefix, len_esc_suffix) = if options.style.background.is_none() &&
                                              options.style.foreground.is_none() {
        // But if there aren't any color codes that we need to
        // escape, don't set the length escape codes because we
        // don't want the shell to have to deal with them if
        // they're unnecessary
        ("", "")
    } else {
        match shell {
            Shell::Bash => ("\\[", "\\]"),
            Shell::Zsh => ("%{", "%}"),
            _ => panic!("Your shell is not supported yet!"),
        }
    };

    // Every time there is a color escape-sequence, it must be
    // surrounded by the length escape-codes. We also include the
    // padding before and after the content.
    let content = format!("{}{}{}{}{}{}{}{}{}",
                    len_esc_prefix,
                    style.prefix(),
                    len_esc_suffix,
                    options.padding_left,
                    s,
                    options.padding_right,
                    len_esc_prefix,
                    style.suffix(),
                    len_esc_suffix,
    );

    // We must format the separator differently depending on whether
    // there exists a visible module after this one or not. Length
    // escape sequences should only be present if they're really
    // necessary
    let (len_esc_prefix, len_esc_suffix) = if next_bg.is_none() &&
                                              options.style.background.is_none() {
        ("", "")
    } else {
        match shell {
            Shell::Bash => ("\\[", "\\]"),
            Shell::Zsh => ("%{", "%}"),
            _ => panic!("Your shell is not supported yet!"),
        }
    };

    let separator_style = ModuleStyle {
        foreground: options.style.background,
        background: next_bg,
        text_properties: options.style.text_properties,
    };
    let separator_style = style_from_modulestyle(&separator_style);
    let separator = format!("{}{}{}{}{}{}{}",
                            len_esc_prefix,
                            separator_style.prefix(),
                            len_esc_suffix,
                            options.separator,
                            len_esc_prefix,
                            separator_style.suffix(),
                            len_esc_suffix);

    ANSIString::from(format!("{}{}", content, separator))
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

/// Attempts to create an `ansi_term::Color` from the provided
/// `Config`.
///
/// Returns an `Error` if none of the conversions succeed, or if an
/// invalid type is provided. Will return `Ok(None)` if the provided
/// `key` has no value within the `config`.
///
/// # Examples
///
/// ```
/// let mut c = Config::new();
/// c.set("foreground", "(255, 255, 255)").unwrap();
/// c.set("background", "blue").unwrap();
/// assert_eq!(try_color_from_config("foreground", &c),
///     Ok(Some(Color::RGB(255, 255, 255))));
/// assert_eq!(try_color_from_config("background", &c),
///     Ok(Some(Color::Blue)));
/// ```
fn try_color_from_config(key: &str, config: &Config) -> Result<Option<Color>, Error> {
    if let Some(val) = config.get(key) {
        match val {
            Value::Integer(i) => {
                // First, check whether it would be a valid u8
                if i < 0 || i > 255 {
                    Err(Error::new(ErrorKind::InvalidTypeInConfig,
                                   &format!("expected u8, got: {:?}", i)))
                } else {
                    Ok(Some(Color::Fixed(i as u8)))
                }
            }
            Value::String(ref s) => {
                // It *may* coerce into a `Color`, `Color::Fixed` or
                // `Color::RGB`.
                if let Ok(color) = try_color_from_str(s) {
                    Ok(Some(color))
                } else if let Ok(color) = try_fixed_from_str(s) {
                    Ok(Some(color))
                } else if let Ok(color) = try_rgb_from_str(s) {
                    Ok(Some(color))
                } else {
                    Err(Error::new(ErrorKind::ConfigParseFailure,
                                   &format!("expected valid color, u8, or rgb tuple, got: {:?}",
                                            s)))
                }
            }
            _ => {
                // Invalid type
                Err(Error::new(ErrorKind::InvalidTypeInConfig,
                               &format!("expected u8 or string, got: {:?}", val)))
            }
        }
    } else {
        // The key didn't correspond to anything within the config
        Ok(None)
    }
}

/// Attempts to create a `Style` representing text style properties
/// from a `Config`.
///
/// Returns an `Error` if the input cannot be parsed into a `Style`.
fn try_text_props_from_config(key: &str, config: &Config) -> Result<Option<Style>, Error> {
    if let Some(val) = config.get(key) {
        // The only two valid types for this option are an array of
        // strings or a single string
        match val {
            Value::String(ref s) => Ok(Some(try_text_prop_from_str(Some(s))?)),
            Value::Array(arr) => {
                let arr = arr.into_iter().map(|s| s.into_str().unwrap()).collect();
                Ok(Some(try_text_props_from_vec(arr)?))
            }
            _ => {
                Err(Error::new(ErrorKind::InvalidTypeInConfig,
                               &format!("expected string or array of strings, got: {:?}", val)))
            }
        }
    } else {
        Ok(None)
    }
}

/// Attempts to convert a string into an `ansi_term::Style`
/// representing a single text property.
///
/// Returns an `Error` if the provided string doesn't match any of the
/// text properties defined in crate `ansi_term`.
///
/// # Examples
/// ```
/// assert_eq!(try_text_prop_from_str(Some("bold")),
///     Ok(Style::new().bold()));
/// ```
fn try_text_prop_from_str(s: Option<&str>) -> Result<Style, Error> {
    if let Some(s) = s {
        match s.to_lowercase().as_ref() {
            "bold" => Ok(Style::new().bold()),
            "blink" => Ok(Style::new().blink()),
            "dimmed" => Ok(Style::new().dimmed()),
            "hidden" => Ok(Style::new().hidden()),
            "italic" => Ok(Style::new().italic()),
            "reverse" => Ok(Style::new().reverse()),
            "strikethrough" => Ok(Style::new().strikethrough()),
            "underline" => Ok(Style::new().underline()),
            _ => {
                Err(Error::new(ErrorKind::NoSuchMatchInConfig,
                               &format!("unknown text property: {:?}", s)))
            }
        }
    } else {
        Ok(Style::new())
    }
}

/// Attempts to convert a `Vec` of strings into a single
/// `ansi_term::Style` representing how text should be styled.
///
/// Returns an `Error` if any of the strings in the `Vec` can't be
/// parsed into a text style property.
///
/// # Examples
/// ```
/// let text_properties = vec!["bold", "underline"];
/// let style = try_text_props_from_vec(text_properties).unwrap();
/// assert_eq!(style, Style::new().bold().underline());
/// ```
fn try_text_props_from_vec<S: Into<String>>(props: Vec<S>) -> Result<Style, Error> {
    // This way, we can pass a Vec<String> OR a Vec<&str> (like we do
    // in the tests)
    let props: Vec<String> = props.into_iter().map(|i| i.into()).collect();

    let mut style = Style::new();
    for s in &props {
        style = match s.to_lowercase().as_ref() {
            "bold" => style.bold(),
            "blink" => style.blink(),
            "dimmed" => style.dimmed(),
            "hidden" => style.hidden(),
            "italic" => style.italic(),
            "reverse" => style.reverse(),
            "strikethrough" => style.strikethrough(),
            "underline" => style.underline(),
            _ => {
                return Err(Error::new(ErrorKind::NoSuchMatchInConfig,
                                      &format!("unknown text property: {:?}", s)));
            }
        }
    }

    Ok(style)
}

/// Attempts to convert a string into an `ansi_term::Color`.
///
/// Returns an `Error` if the provided string doesn't match any of the
/// colors defined in crate `ansi_term`.
///
/// # Examples
///
/// ```
/// assert_eq!(try_color_from_str("black"), Ok(Color::Black));
/// assert_eq!(try_color_from_str("green"), Ok(Color::Green));
/// assert!(try_color_from_str("turquoise").is_err());
/// ```
fn try_color_from_str(s: &str) -> Result<Color, Error> {
    match s.to_lowercase().as_ref() {
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "purple" => Ok(Color::Purple),
        "cyan" => Ok(Color::Cyan),
        "white" => Ok(Color::White),
        _ => {
            Err(Error::new(ErrorKind::NoSuchMatchInConfig,
                           &format!("unknown color: {:?}", s)))
        }
    }
}

/// Attempts to convert a string into an `ansi_term::Color::RGB`.
///
/// Returns an `Error` if the provided string is not a sequence of 3
/// `u8`s, separated by commas.
///
/// # Examples
///
/// ```
/// assert_eq!(try_rgb_from_str("(14, 76, 1)"), Ok(Color::RGB(14, 76, 1)));
/// assert_eq!(try_rgb_from_str("0, 100, 0"), Ok(Color::RGB(0, 100, 0)));
/// assert!(try_rgb_from_str("1000, b, c, -1").is_err());
/// ```
fn try_rgb_from_str(s: &str) -> Result<Color, Error> {
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
        Err(Error::new(ErrorKind::ConfigParseFailure,
                       &format!("expected 3 comma-separated u8's, got: {:?}", s)))
    } else {
        Ok(Color::RGB(ints[0], ints[1], ints[2]))
    }
}

/// Attempts to convert a string into an `ansi_term::Color::Fixed`.
///
/// Returns an `Error` if the provided string fails to coerce into a
/// `u8`.
///
/// # Examples
///
/// ```
/// assert_eq!(try_fixed_from_str("63"), Ok(Color::Fixed(63)));
/// assert!(try_fixed_from_str("257").is_err());
/// ```
fn try_fixed_from_str(s: &str) -> Result<Color, Error> {
    Ok(Color::Fixed(s.parse::<u8>()?))
}

// NOTE: We do not need a try_color_from_u8 OR a try_rgb_from_vec.
// The implementation would just look like:
// pub fn try_color_from_u8(i: u8) -> Color { Color::u8(i) }
// It would always succeed due to Rust's type system.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_options_from_config() {
        let mut c = Config::new();

        // No options set
        assert_eq!(read_options("prompt", &c), Ok(ModuleOptions::default()));

        // Only a separator set
        let options = ModuleOptions {
            output: None,
            padding_left: String::from(" "),
            padding_right: String::from(" "),
            separator: String::from(">"),
            style: ModuleStyle::default(),
        };
        c.set("modules.prompt.separator", ">").unwrap();
        assert_eq!(read_options("prompt", &c), Ok(options));

        // All options set
        let options = ModuleOptions {
            output: Some(String::from("Hello")),
            padding_left: String::from("|"),
            padding_right: String::from("/"),
            separator: String::from(" "),
            style: ModuleStyle {
                foreground: Some(Color::White),
                background: Some(Color::RGB(6, 47, 200)),
                text_properties: Some(Style::new().bold()),
            },
        };
        c.set("modules.prompt.output", "Hello").unwrap();
        c.set("modules.prompt.padding_left", "|").unwrap();
        c.set("modules.prompt.padding_right", "/").unwrap();
        c.set("modules.prompt.separator", " ").unwrap();
        c.set("modules.prompt.style.foreground", "white")
            .unwrap();
        c.set("modules.prompt.style.background", "(6, 47, 200)")
            .unwrap();
        c.set("modules.prompt.style.text_properties", "bold")
            .unwrap();
        assert_eq!(read_options("prompt", &c), Ok(options));

        // Error in one of the options
        c.set("modules.prompt.padding_left", true).unwrap();
        assert!(read_options("prompt", &c).is_err());
    }

    #[test]
    fn test_read_style_from_config() {
        let mut c = Config::new();

        // No options set
        assert_eq!(read_style("prompt", &c), Ok(ModuleStyle::default()));

        // Only a background set
        c.set("modules.prompt.style.background", "blue").unwrap();
        let style = ModuleStyle {
            foreground: None,
            background: Some(Color::Blue),
            text_properties: None,
        };
        assert_eq!(read_style("modules.prompt.style", &c), Ok(style));

        // All properties set
        c.set("modules.prompt.style.foreground", "white")
            .unwrap();
        c.set("modules.prompt.style.background", "50").unwrap();
        c.set("modules.prompt.style.text_properties",
                 vec!["bold", "italic"])
            .unwrap();
        let style = ModuleStyle {
            foreground: Some(Color::White),
            background: Some(Color::Fixed(50)),
            text_properties: Some(Style::new().bold().italic()),
        };
        assert_eq!(read_style("modules.prompt.style", &c), Ok(style));

        // Erroneous property set (malformed Color::RGB, too many inputs)
        c.set("modules.prompt.style.foreground", "(0, 0, 0, 0)")
            .unwrap();
        assert!(read_style("modules.prompt.style", &c).is_err());
    }

    #[test]
    fn test_try_text_props_from_config() {
        let mut c = Config::new();

        // No text properties set
        assert_eq!(try_text_props_from_config("text_properties", &c), Ok(None));

        // Text properties are a single string
        c.set("text_properties", "bold").unwrap();
        assert_eq!(try_text_props_from_config("text_properties", &c),
                   Ok(Some(Style::new().bold())));

        // Text properties are a vec of strings
        c.set("text_properties", vec!["bold", "underline", "italic"])
            .unwrap();
        assert_eq!(try_text_props_from_config("text_properties", &c),
                   Ok(Some(Style::new().bold().underline().italic())));

        // Text properties are an invalid type
        c.set("text_properties", true).unwrap();
        assert!(try_text_props_from_config("text_properties", &c).is_err());
    }

    #[test]
    fn test_try_text_props_from_vec() {
        // A single valid text property
        let props = vec!["bold"];
        assert_eq!(try_text_props_from_vec(props), Ok(Style::new().bold()));

        // Multiple valid text properties
        let props = vec!["bold", "hidden", "underline"];
        assert_eq!(try_text_props_from_vec(props),
                   Ok(Style::new().bold().hidden().underline()));

        // No text properties
        let props: Vec<&str> = Vec::new();
        assert_eq!(try_text_props_from_vec(props), Ok(Style::new()));

        // Invalid text property
        let props = vec!["hidden", "invalid"];
        assert!(try_text_props_from_vec(props).is_err());
    }

    #[test]
    fn test_try_text_prop_from_str() {
        // Valid text property
        assert_eq!(try_text_prop_from_str(Some("bold")),
                   Ok(Style::new().bold()));

        // No string passed
        assert_eq!(try_text_prop_from_str(None), Ok(Style::new()));

        // Invalid text property
        assert!(try_text_prop_from_str(Some("invalid")).is_err());
    }

    #[test]
    fn test_try_color_from_config() {
        let mut c = Config::new();

        // No fg or bg set
        assert_eq!(try_color_from_config("foreground", &c), Ok(None));
        assert_eq!(try_color_from_config("background", &c), Ok(None));

        // Only a fg set
        c.set("foreground", "white").unwrap();
        assert_eq!(try_color_from_config("foreground", &c),
                   Ok(Some(Color::White)));
        assert_eq!(try_color_from_config("background", &c), Ok(None));

        // Both fg and bg set
        c.set("foreground", "green").unwrap();
        c.set("background", "black").unwrap();
        assert_eq!(try_color_from_config("foreground", &c),
                   Ok(Some(Color::Green)));
        assert_eq!(try_color_from_config("background", &c),
                   Ok(Some(Color::Black)));

        // fg set to a valid u8
        c.set("foreground", 10).unwrap();
        assert_eq!(try_color_from_config("foreground", &c),
                   Ok(Some(Color::Fixed(10))));

        // fg set to an invalid u8
        c.set("foreground", -1).unwrap();
        assert!(try_color_from_config("foreground", &c).is_err());

        // bg set to an invalid type
        c.set("background", true).unwrap();
        assert!(try_color_from_config("background", &c).is_err());

        c.set("background", vec!["a", "b", "c"]).unwrap();
        assert!(try_color_from_config("background", &c).is_err());
    }

    #[test]
    #[should_panic]
    fn test_panic_on_unsupported_shell() {
        // We must include at least one color, because we only want to
        // panic if we're using the length escape sequences AND the
        // shell is unsupported.
        let options = ModuleOptions {
            output: None,
            padding_left: String::new(),
            padding_right: String::new(),
            separator: String::new(),
            style: ModuleStyle {
                background: Some(Color::Blue),
                foreground: None,
                text_properties: None,
            },
        };

        let _ = format_for_module("", &options, None, Shell::Fish);
    }

    #[test]
    fn test_format_for_module() {
        const CONTENT: &'static str = "Hello";
        const PADDING: &'static str = " ";
        const SEPARATOR: &'static str = ">";

        let mut options = ModuleOptions {
            output: None,
            padding_left: PADDING.to_string(),
            padding_right: PADDING.to_string(),
            separator: SEPARATOR.to_string(),
            style: ModuleStyle {
                background: Some(Color::Blue),
                foreground: Some(Color::White),
                text_properties: Some(Style::default().bold()),
            },
        };

        let formatted_string = format_for_module(CONTENT.to_string(), &options, None, Shell::Bash);
        assert_eq!(format!("\\[\x1B[1;44;37m\\]{}{}{}\\[\x1B[0m\\]\\[\x1B[1;34m\\]{}\\[\x1B[0m\\]",
                           PADDING,
                           CONTENT,
                           PADDING,
                           SEPARATOR),
                   format!("{}", formatted_string));

        // Override the output, use ZSH
        options.output = Some(String::from("modified"));
        let formatted_string = format_for_module(CONTENT.to_string(), &options, None, Shell::Bash);
        assert_eq!(format!("\\[\x1B[1;44;37m\\]{}{}{}\\[\x1B[0m\\]\\[\x1B[1;34m\\]{}\\[\x1B[0m\\]",
                           PADDING,
                           "modified",
                           PADDING,
                           SEPARATOR),
                   format!("{}", formatted_string));
    }

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
    fn test_try_color_from_str() {
        // Corresponds to one of the colors defined in `ansi_term`
        assert_eq!(try_color_from_str("blue"), Ok(Color::Blue));

        // Not part of the `ansi_term::Color` enum
        assert!(try_color_from_str("teal").is_err());
    }

    #[test]
    fn test_try_fixed_from_str() {
        // Valid inputs
        assert_eq!(try_fixed_from_str("0"), Ok(Color::Fixed(0)));
        assert_eq!(try_fixed_from_str("100"), Ok(Color::Fixed(100)));

        // Inputs that can't be parsed to u8
        assert!(try_fixed_from_str("256").is_err());
        assert!(try_fixed_from_str("-1").is_err());
    }

    #[test]
    fn test_try_rgb_from_str() {
        // Valid inputs
        assert_eq!(try_rgb_from_str("(0, 0, 0)"), Ok(Color::RGB(0, 0, 0)));
        assert_eq!(try_rgb_from_str("(255, 255, 255)"),
                   Ok(Color::RGB(255, 255, 255)));

        // Questionable inputs (should still work)
        assert_eq!(try_rgb_from_str("(0, 0, 0))"), Ok(Color::RGB(0, 0, 0)));

        // Improperly formed ("too many" inputs)
        assert!(try_rgb_from_str("(0, 0, 0,)").is_err());

        // Too few inputs
        assert!(try_rgb_from_str("(0, 0)").is_err());

        // Inputs aren't u8's
        assert!(try_rgb_from_str("(1000, 0, 0)").is_err());
        assert!(try_rgb_from_str("(0, 0, -1)").is_err());
    }
}
