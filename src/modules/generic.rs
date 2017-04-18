use ansi_term::Color;
use config::Config;
use clap::Shell;

use utils::{ModuleError, FormatResult};

use modules;

/// Formats a user-defined module using whatever options are present
/// in the config file provided.
///
/// Returns a `ModuleError` if it encounters any errors parsing the
/// config file.
pub fn format_generic(name: &str,
                      c: &Config,
                      next_bg: Option<Color>,
                      shell: Shell)
                      -> Result<FormatResult, ModuleError> {
    let options = modules::read_options(name, c)?;

    if options.output.is_some() {
        Ok(FormatResult {
               output: Some(modules::format_for_module("", &options, next_bg, shell)),
               next_bg: options.style.background,
           })
    } else {
        Ok(FormatResult::default())
    }
}
