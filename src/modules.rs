use ansi_term::{ANSIString, Colour, Style};
use config::Config;

/// Merges in the default values for the program
pub fn merge_defaults(c: &mut Config) {
    c.set_default("global.modules",
                     vec!["exit_code", "directory", "git", "prompt"])
        .unwrap();
    c.set_default("global.foreground", "bright_white").unwrap();
    c.set_default("global.background", "blue").unwrap();
    c.set_default("global.style", "default").unwrap();
    c.set_default("global.separator", "").unwrap();
    c.set_default("global.padding_left", " ").unwrap();
    c.set_default("global.padding_right", " ").unwrap();

    c.set_default("modules.directory.max_depth", 4).unwrap();

    c.set_default("modules.exit_code.bg_success", "green").unwrap();
    c.set_default("modules.exit_code.bg_error", "red").unwrap();

    // c.set_default("modules.git.symbol_clean", "").unwrap();
    c.set_default("modules.git.symbol_insertion", "+").unwrap();
    c.set_default("modules.git.symbol_deletion", "-").unwrap();
    c.set_default("modules.git.symbol_push", "⇡").unwrap();
    // c.set_default("modules.git.symbol_pull", "⇣").unwrap();
    c.set_default("modules.git.show_diff_stats", true).unwrap();
    c.set_default("modules.git.show_unpushed", true).unwrap();

    c.set_default("modules.prompt.output", "$").unwrap();
    c.set_default("modules.prompt.bg_success", "green").unwrap();
    c.set_default("modules.prompt.bg_error", "red").unwrap();
}

/// Generic formatting method
pub fn format_module<'a>(c: &mut Config,
                         name: &'a str,
                         output: Option<String>,
                         last_successful: Option<&str>)
                         -> (Option<&'a str>, Option<ANSIString<'static>>) {
    // Formatting was not successful if there was nothing to format
    if c.get_str(&format!("modules.{}.output", name)).is_none() && output.is_none() {
        return (None, None);
    }

    // Get config options
    let fg = c.get_str(&format!("modules.{}.foreground", name))
        .unwrap_or_else(|| c.get_str("global.foreground").unwrap_or_default());
    let fg = string_to_colour(fg);

    let bg = c.get_str(&format!("modules.{}.background", name))
        .unwrap_or_else(|| c.get_str("global.background").unwrap_or_default());
    let bg = string_to_colour(bg);

    let padding_left = c.get_str(&format!("modules.{}.padding_left", name))
        .unwrap_or_else(|| c.get_str("global.padding_left").unwrap_or_default());
    let padding_right = c.get_str(&format!("modules.{}.padding_right", name))
        .unwrap_or_else(|| c.get_str("global.padding_right").unwrap_or_default());
    let separator = c.get_str(&format!("modules.{}.separator", name))
        .unwrap_or_else(|| c.get_str("global.separator").unwrap_or_default());

    let main_style = c.get_str(&format!("modules.{}.style", name))
        .unwrap_or_else(|| c.get_str("global.style").unwrap_or_default());
    let main_style = string_to_style(main_style).on(bg).fg(fg);

    // Override the output if there is "output" in this module's config
    let output = if let Some(s) = c.get_str(&format!("modules.{}.output", name)) {
        s
    } else {
        output.unwrap()
    };

    // Format the main content
    let mut content = format!("\\[{}\\]{}{}{}\\[{}\\]",
                              main_style.prefix(),
                              padding_left,
                              output,
                              padding_right,
                              main_style.suffix());

    // Format the separator
    let main_style = main_style.on(fg).fg(bg);
    if let Some(name) = last_successful {
        // There is a visible module that comes after this one
        let next_bg = c.get_str(&format!("modules.{}.background", name))
            .unwrap_or_else(|| c.get_str("global.background").unwrap_or_default());
        let next_bg = string_to_colour(next_bg);

        content = format!("{}\\[{}\\]{}\\[{}\\]",
                          content,
                          main_style.on(next_bg).prefix(),
                          separator,
                          main_style.on(next_bg).suffix());
    } else {
        // This is the final module
        content = format!("{}\\[{}\\]{}\\[{}\\]",
                          content,
                          bg.prefix(),
                          separator,
                          bg.suffix());
    }

    (Some(name), Some(ANSIString::from(content)))
}

// Converts a string (from the config file) to a Colour
// See: https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg
fn string_to_colour(s: String) -> Colour {
    if let Ok(i) = s.parse::<u8>() {
        Colour::Fixed(i)
    } else {
        let s = s.to_lowercase();
        match s.as_ref() {
            "black" => Colour::Fixed(0),
            "bright_black" => Colour::Fixed(8),
            "red" => Colour::Fixed(1),
            "bright_red" => Colour::Fixed(9),
            "green" => Colour::Fixed(2),
            "bright_green" => Colour::Fixed(10),
            "yellow" => Colour::Fixed(3),
            "bright_yellow" => Colour::Fixed(11),
            "blue" => Colour::Fixed(4),
            "bright_blue" => Colour::Fixed(12),
            "purple" => Colour::Fixed(5),
            "bright_purple" => Colour::Fixed(13),
            "cyan" => Colour::Fixed(6),
            "bright_cyan" => Colour::Fixed(14),
            "white" => Colour::Fixed(7),
            "bright_white" => Colour::Fixed(15),
            _ => panic!("Invalid color option: {} in config file!", s),
        }
    }
}

fn string_to_style(s: String) -> Style {
    let s = s.to_lowercase();
    match s.as_ref() {
        "default" | "" | "normal" => Style::new(),
        "bold" => Style::new().bold(),
        "dimmed" => Style::new().dimmed(),
        "italic" => Style::new().italic(),
        "underline" => Style::new().underline(),
        "blink" => Style::new().blink(),
        "reverse" => Style::new().reverse(),
        "hidden" => Style::new().hidden(),
        "strikethrough" => Style::new().strikethrough(),
        _ => panic!("Unknown style property: {} in config file!", s),
    }
}

pub fn format_module_prompt<'a>(c: &mut Config,
                                last_successful: Option<&'a str>,
                                exit_code: &str)
                                -> (Option<&'a str>, Option<ANSIString<'static>>) {
    let bg = match exit_code.as_ref() {
        "0" => c.get_str("modules.prompt.bg_success").unwrap_or_default(),
        _ => c.get_str("modules.prompt.bg_error").unwrap_or_default(),
    };
    c.set("modules.prompt.background", bg).unwrap();

    let output = c.get_str("modules.prompt.output").unwrap_or_default();

    format_module(c, "prompt", Some(output), last_successful)
}

pub fn format_module_exit_code<'a>(c: &mut Config,
                                   last_successful: Option<&'a str>,
                                   exit_code: &str)
                                   -> (Option<&'a str>, Option<ANSIString<'static>>) {
    let bg = match exit_code.as_ref() {
        "0" => c.get_str("modules.exit_code.bg_success").unwrap_or_default(),
        _ => c.get_str("modules.exit_code.bg_error").unwrap_or_default(),
    };
    c.set("modules.exit_code.background", bg).unwrap();

    format_module(c, "exit_code", Some(exit_code.to_string()), last_successful)
}

pub fn format_module_directory<'a>(c: &mut Config,
                                   last_successful: Option<&'a str>)
                                   -> (Option<&'a str>, Option<ANSIString<'static>>) {
    use std::env;
    use std::path::PathBuf;

    let home = env::var("HOME").unwrap();
    let cwd = env::current_dir().unwrap();

    // Convert "/home/user/directory" to "~/directory"
    let mut shortened_cwd: PathBuf;
    if let Ok(stripped_cwd) = cwd.strip_prefix(&home) {
        shortened_cwd = PathBuf::from("~").join(stripped_cwd);
    } else {
        shortened_cwd = env::current_dir().unwrap();
    }

    let depth = shortened_cwd.components().count();

    // Max number of directories we want to see
    let max_depth = c.get_int("modules.directory.max_depth").unwrap_or_default() as usize;

    if depth > max_depth {
        let comp_iter = shortened_cwd.clone();
        let comp_iter = comp_iter.components();

        shortened_cwd = PathBuf::new();
        for (i, component) in comp_iter.enumerate() {
            if i < (max_depth / 2) || i >= (depth - (max_depth / 2)) {
                shortened_cwd.push(component.as_os_str());
            } else if i == (max_depth / 2) {
                shortened_cwd.push("...");
            }
        }
    }

    format_module(c,
                  "directory",
                  Some(format!("{}", shortened_cwd.display())),
                  last_successful)
}

pub fn format_module_git<'a>(c: &mut Config,
                             last_successful: Option<&'a str>)
                             -> (Option<&'a str>, Option<ANSIString<'static>>) {
    use git2::{Branch, Error, Oid, Reference, Repository};
    use std::env;

    let mut output = String::new();

    if let Ok(repo) = Repository::discover(env::current_dir().unwrap_or_default()) {
        let local = repo.head().unwrap();

        // Output current branch name
        output.push_str(local.shorthand().unwrap());

        let local = Branch::wrap(local);

        // Show local insertions and deletions
        let show_diffs = c.get_bool("modules.git.show_diff_stats").unwrap_or_default();
        if show_diffs {
            let diff_stats = repo.diff_index_to_workdir(None, None).unwrap();
            let diff_stats = diff_stats.stats().unwrap();

            if diff_stats.files_changed() > 0 {
                let symbol_deletion = c.get_str("modules.git.symbol_deletion").unwrap_or_default();
                let symbol_insertion = c.get_str("modules.git.symbol_insertion")
                    .unwrap_or_default();

                output.push_str(&format!(" ({}{}, {}{})",
                                         symbol_deletion,
                                         diff_stats.deletions(),
                                         symbol_insertion,
                                         diff_stats.insertions()));
            }
        }

        // Show unpushed commits
        let show_unpushed = c.get_bool("modules.git.show_unpushed").unwrap_or_default();
        if show_unpushed {
            if let Ok(upstream) = local.upstream() {
                println!("We have an upstream!");
            }
        }
    }

    // let repo: Rc<Result<Repository, Error>> = Rc::from(Repository::discover(env::current_dir()
    //                                                                             .unwrap()));
    // let local: Option<Reference> = match repo.clone().borrow() {
    //     Ok(r) => if let Ok(e) = r.head() { Some(e) } else { None },
    //     _ => None,
    // };

    /*     if let Ok(repo) = repo {
        if let Ok(local) = repo.head() {
            // Current branch name
            output.push_str(local.shorthand().unwrap());

            let show_diffs = c.get_bool("modules.git.show_diff_stats").unwrap_or_default();
            // Insertions and deletions
            if show_diffs {
                if let Ok(diff) = repo.diff_index_to_workdir(None, None) {
                    if let Ok(stats) = diff.stats() {
                        if stats.files_changed() > 0 {
                            let symbol_deletion = c.get_str("modules.git.symbol_deletion")
                                .unwrap_or_default();
                            let symbol_insertion = c.get_str("modules.git.symbol_insertion")
                                .unwrap_or_default();

                            output.push_str(&format!(" ({}{}, {}{})",
                                                     symbol_deletion,
                                                     stats.deletions(),
                                                     symbol_insertion,
                                                     stats.insertions()));
                        }
                    }
                }
            }

            let show_unpushed = c.get_bool("modules.git.show_unpushed").unwrap_or_default();
            // Unpushed commits
            if show_unpushed {
                let local_branch = Branch::wrap(local);
                if let Ok(upstream) = local_branch.upstream() {
                    let upstream_oid = Oid::from_str(upstream.name().unwrap().unwrap()).unwrap();
                    let local_oid = Oid::from_str(local_branch.name().unwrap().unwrap()).unwrap();
                    if let Ok((a, b)) = repo.graph_ahead_behind(local_oid, upstream_oid) {
                        let symbol_push = c.get_str("modules.git.symbol_push").unwrap_or_default();
                        println!("(a, b): ({}, {})", a, b);
                    }
                }
            }
        }
    } */

    if output.is_empty() {
        (None, None)
    } else {
        format_module(c, "git", Some(output), last_successful)
    }
    // let symbol_clean = c.get_str("modules.git.symbol_clean").unwrap_or_default();
    // let symbol_pull = c.get_str("modules.git.symbol_pull").unwrap_or_default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_colour() {
        const CONTENT: &'static str = "hi";

        struct Test<'a> {
            input: &'static str,
            expected: &'a str,
        }

        let tests = [Test {
                         input: "green",
                         expected: &format!("\x1B[38;5;2m{}\x1B[0m", CONTENT),
                     },
                     Test {
                         input: "bright_green",
                         expected: &format!("\x1B[38;5;10m{}\x1B[0m", CONTENT),
                     }];

        for test in &tests {
            let result = string_to_colour(test.input.to_string());
            let result = format!("{}", result.paint(CONTENT));

            assert_eq!(test.expected, result);
        }
    }

    #[test]
    #[should_panic]
    fn test_string_to_colour_invalid_input() {
        struct Test {
            input: &'static str,
        }

        let tests = [Test { input: "green" }, Test { input: "invalid" }];

        for test in &tests {
            string_to_colour(test.input.to_string());
        }
    }
}
