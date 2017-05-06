use std::env;
use std::iter::FromIterator;
use std::path::PathBuf;

use ansi_term::Color;
use config::{Config, Value};
use clap::Shell;

use utils::{Error, ErrorKind, FormatResult};

use modules;

/// Formats the current working directory using whatever options are
/// present in the config file provided.
///
/// Returns an `Error` if it encounters any errors while parsing the
/// config file.
pub fn format_cwd(c: &Config, next_bg: Option<Color>, shell: Shell) -> Result<FormatResult, Error> {
    let options = modules::read_options("cwd", c)?;

    let mut cwd = if let Ok(pwd) = env::var("PWD") {
        // We prioritize using $PWD because the user doesn't expect to
        // see the absolute path, but rather the symlinks. This is
        // consistent with other powerline-like implementations.
        PathBuf::from(pwd)
    } else {
        // `pwd -L` must not be supported by this shell. Return the
        // absolute path instead.

        // Fail silently if the "current directory" turns out to not
        // exist. We don't want to spew a bunch of error messages in
        // this circumstance, and this should be opaque to the user.
        env::current_dir().unwrap_or_default()
    };

    // Truncate leading instance of $HOME to just "~/"
    if let Ok(home) = env::var("HOME") {
        if let Ok(stripped_cwd) = cwd.clone().strip_prefix(&home) {
            cwd = PathBuf::from("~").join(stripped_cwd);
        }
    }

    // Truncate extra long paths to a certain depth
    let depth = cwd.components().count();
    let max_depth: usize = if let Some(val) = c.get("modules.cwd.max_depth") {
        match val {
            Value::Integer(n) => {
                // Value must be a valid usize
                if n < 0 {
                    return Err(Error::new(ErrorKind::InvalidTypeInConfig,
                                          &format!("expected usize, got: {:?}", n)));
                } else {
                    n as usize
                }
            }
            _ => {
                // Passing in anything other than an integer is an error
                return Err(Error::new(ErrorKind::InvalidTypeInConfig,
                                      &format!("expected usize, got: {:?}", val)));
            }
        }
    } else {
        // Default maximum depth is 4
        4
    };

    if depth > max_depth {
        let iter = cwd.clone();
        let iter = iter.iter();

        cwd = PathBuf::from("...");
        cwd.push(PathBuf::from_iter(iter.skip(depth - max_depth)));
    }

    let format_result = FormatResult {
        output: Some(modules::format_for_module(format!("{}", cwd.display()),
                                                &options,
                                                next_bg,
                                                shell)),
        next_bg: options.style.background,
    };

    Ok(format_result)
}
