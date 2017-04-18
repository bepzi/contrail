use std::env;
use std::path::PathBuf;

use ansi_term::Color;
use config::Config;
use clap::Shell;
use git2::Repository;

use utils::{ModuleError, FormatResult};

use modules;

pub fn format_git(c: &Config,
                  next_bg: Option<Color>,
                  shell: Shell)
                  -> Result<FormatResult, ModuleError> {
    let options = modules::read_options("git", c)?;

    // Fail silently if the "current directory" doesn't exist
    let cwd = env::current_dir().unwrap_or(PathBuf::new());

    if let Ok(repo) = Repository::discover(cwd) {
        let local = repo.head()?;
        if let Some(name) = local.shorthand() {
            return Ok(FormatResult {
                          output: Some(modules::format_for_module(name, &options, next_bg, shell)),
                          next_bg: options.style.background,
                      });
        }
    }

    Ok(FormatResult {
           output: None,
           next_bg: next_bg,
       })
}
