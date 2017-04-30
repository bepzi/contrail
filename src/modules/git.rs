use std::env;

use ansi_term::Color;
use config::Config;
use clap::Shell;
use git2::{Branch, Repository};

use utils::{Error, FormatResult};

use modules;

/// Finds and formats information about the current git repository, if
/// any.
///
/// Returns an `Error` if there is an error while reading the config
/// file. Errors encountered while fetching information about the
/// current repository are simply ignored.
pub fn format_git(c: &Config, next_bg: Option<Color>, shell: Shell) -> Result<FormatResult, Error> {
    // This is one of the few modules that actually can return `None`
    // for its output. If that happens, no part of the module
    // (separator, padding, etc.) will show up in the prompt. (It will
    // be effectively "skipped")

    let options = modules::read_options("git", c)?;

    let cwd = if let Ok(cwd) = env::current_dir() {
        cwd
    } else {
        // Problem while getting the current directory, just skip this
        // module.
        return Ok(FormatResult::default());
    };

    // Holds the final output
    let mut output = String::new();

    if let Ok(repo) = Repository::discover(cwd) {
        // Find and print the branch name ("master", etc...), but if
        // the repository exists and the HEAD doesn't, just return
        let local = if let Ok(h) = repo.head() {
            h
        } else {
            return Ok(FormatResult::default());
        };

        if let Some(name) = local.shorthand() {
            output.push_str(name);
        }

        // Show whether or not the current working directory has been
        // modified. If errors are encountered, just don't display
        // anything for this part.
        if let Ok(diff) = repo.diff_index_to_workdir(None, None) {
            if let Ok(stats) = diff.stats() {
                if stats.files_changed() > 0 {
                    output.push_str(" +");
                }
            }
        }

        // Show whether whether or not the current working
        // directory is ahead/behind upstream. If errors are
        // encountered AT ANY POINT, don't display anything.
        let local = Branch::wrap(local);
        if let Ok(upstream) = local.upstream() {
            let local_ref = local.get();
            let upstream_ref = upstream.get();

            if let Some(local_target) = local_ref.target() {
                if let Some(upstream_target) = upstream_ref.target() {
                    if let Ok((ahead, behind)) =
                        repo.graph_ahead_behind(local_target, upstream_target) {
                        // Show commits ahead
                        if ahead > 0 {
                            output.push_str(&format!(" ⇡{}", ahead));
                        }

                        // Show commits behind
                        if behind > 0 {
                            output.push_str(&format!(" ⇣{}", behind));
                        }
                    }
                }
            }
        }

        if output.is_empty() {
            Ok(FormatResult::default())
        } else {
            // If we get here, we *at least* have a branch name we can
            // format.
            Ok(FormatResult {
                   output: Some(modules::format_for_module(output, &options, next_bg, shell)),
                   next_bg: options.style.background,
               })
        }
    } else {
        // Current working directory wasn't a git repository. Harmless
        // error, return `None` so that we don't print anything, and
        // just move on to the next module.
        Ok(FormatResult::default())
    }
}
