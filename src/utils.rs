use std::convert::From;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

use ansi_term::{ANSIString, Color};
use config::{Config, Value};
use git2;

/// Type that will be returned when a module is formatted
#[derive(Debug, Default)]
pub struct FormatResult {
    pub output: Option<ANSIString<'static>>,
    pub next_bg: Option<Color>,
}

#[derive(Debug, PartialEq)]
/// Error type for when parsing a config file to another type fails
pub enum ModuleError {
    /// Input doesn't correspond to a valid result
    NoSuchMatch,
    /// Input is malformed and cannot be parsed
    InvalidForm,
    /// An error was encountered while creating the output
    FormatFailure,
}

impl fmt::Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ModuleError {
    fn description(&self) -> &str {
        match *self {
            ModuleError::NoSuchMatch => "no match was found for the provided input",
            ModuleError::InvalidForm => "provided input was malformed and could not be parsed",
            ModuleError::FormatFailure => "there was an error while trying to format the output",
        }
    }
}

// So that we can use try!() and ? to return early if we encounter a
// ParseIntError
impl From<ParseIntError> for ModuleError {
    fn from(_: ParseIntError) -> ModuleError {
        ModuleError::InvalidForm
    }
}

// For when we encounter an error while trying to fetch data about a
// git repo
impl From<git2::Error> for ModuleError {
    fn from(_: git2::Error) -> ModuleError {
        ModuleError::FormatFailure
    }
}


/// Gets an array from a config file using a key.
///
/// Returns `None` if the key wasn't present or couldn't be coerced
/// into a `Vec<Value>`.
///
/// `Config::get_array()` in version `0.4.1` has a bug where it
/// consumes `self` instead of taking `self` by reference. This
/// function is a workaround for the time being. The bug has been
/// fixed but will not be available until the release of version `0.5`
/// of `config-rs`.
pub fn ref_get_array(key: &str, config: &Config) -> Option<Vec<Value>> {
    config.get(key).and_then(Value::into_array)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // The real test is that this compiles successfully without giving
    // a warning about the config file being moved.
    fn test_ref_get_array() {
        let mut c: Config = Config::new();
        c.set("numbers", vec![1, 2, 3]).unwrap();
        c.set("boolean", true).unwrap();

        // Uncomment and the compiler should complain
        // assert_eq!(c.get_array("numbers"),
        //            Some(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]));
        // assert_eq!(c.get_array("boolean"), None); // Use of moved value: c

        assert_eq!(ref_get_array("numbers", &c),
                   Some(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]));
        assert_eq!(ref_get_array("boolean", &c), None);
    }
}
