use ansi_term::Color;
use config::{Config, Value};
use clap::Shell;

use utils::{ConvertError, FormatResult};

use modules;

pub fn format_cwd(c: &Config,
                  next_bg: Option<Color>,
                  shell: Shell)
                  -> Result<FormatResult, ConvertError> {
    use std::env;
    use std::iter::FromIterator;
    use std::path::PathBuf;

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
        env::current_dir().unwrap_or(PathBuf::new())
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
                    return Err(ConvertError::InvalidForm);
                } else {
                    n as usize
                }
            }
            _ => {
                // Passing in anything other than an integer is an error
                return Err(ConvertError::InvalidForm);
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
