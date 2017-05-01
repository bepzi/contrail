use ansi_term::Color;
use config::Config;
use clap::Shell;

use utils::{Error, FormatResult};

use modules;

/// Formats the exit code module using whatever options are present in
/// the config file provided.
///
/// Returns an `Error` if it encounters any errors while parsing the
/// config file.
pub fn format_exit_code(c: &Config,
                        exit_code: u8,
                        next_bg: Option<Color>,
                        shell: Shell)
                        -> Result<FormatResult, Error> {
    let mut options = modules::read_options("exit_code", c)?;

    let style_success = modules::read_style("modules.exit_code.style_success", c)?;
    let style_error = modules::read_style("modules.exit_code.style_error", c)?;

    // A command exited successfully if and only if the exit code is 0
    if exit_code == 0 {
        options.style = style_success;
    } else {
        options.style = style_error;
    }

    let format_result = FormatResult {
        output: Some(modules::format_for_module(exit_code.to_string(), &options, next_bg, shell)),
        next_bg: options.style.background,
    };

    Ok(format_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code() {
        let mut c = Config::new();

        c.set("modules.exit_code.style_success.background", "green")
            .unwrap();
        c.set("modules.exit_code.style_error.background", "red")
            .unwrap();

        // Exit code of 0 should be green
        let result = format_exit_code(&c, 0, None, Shell::Bash).unwrap();
        assert_eq!(result.next_bg, Some(Color::Green));

        // Exit code of non-zero should be red
        let result = format_exit_code(&c, 1, None, Shell::Bash).unwrap();
        assert_eq!(result.next_bg, Some(Color::Red));
    }
}
