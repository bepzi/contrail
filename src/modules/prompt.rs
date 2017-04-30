use ansi_term::Color;
use config::Config;
use clap::Shell;

use utils::{Error, FormatResult};

use modules;

pub fn format_prompt(c: &Config,
                     exit_code: u8,
                     next_bg: Option<Color>,
                     shell: Shell)
                     -> Result<FormatResult, Error> {
    let mut options = modules::read_options("prompt", c)?;

    let style_success = modules::read_style("modules.prompt.style_success", c)?;
    let style_error = modules::read_style("modules.prompt.style_error", c)?;

    // A command exited successfully if and only if the exit code is 0
    if exit_code == 0 {
        options.style = style_success;
    } else {
        options.style = style_error;
    }

    let format_result = FormatResult {
        output: Some(modules::format_for_module("$", &options, next_bg, shell)),
        next_bg: options.style.background,
    };

    Ok(format_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_prompt() {
        let mut c = Config::new();

        c.set("modules.prompt.style_success.background", "green")
            .unwrap();
        c.set("modules.prompt.style_error.background", "red")
            .unwrap();

        // Exit code of 0 should be green
        let result = format_prompt(&c, 0, None, Shell::Bash).unwrap();
        assert_eq!(result.next_bg, Some(Color::Green));

        // Exit code of non-zero should be red
        let result = format_prompt(&c, 1, None, Shell::Bash).unwrap();
        assert_eq!(result.next_bg, Some(Color::Red));
    }
}
