use ansi_term::{ANSIString, Colour};
use config::Config;

/// Merges in the default values for the program
pub fn merge_defaults(c: &mut Config) {
    c.set_default("global.modules",
                     vec!["exit_code", "directory", "git", "prompt"])
        .unwrap();
    c.set_default("global.foreground", "bright_white").unwrap();
    c.set_default("global.background", "blue").unwrap();
    c.set_default("global.separator", "").unwrap();
    c.set_default("global.padding_left", " ").unwrap();
    c.set_default("global.padding_right", " ").unwrap();

    c.set_default("modules.exit_code.bg_success", "green").unwrap();
    c.set_default("modules.exit_code.bg_error", "red").unwrap();

    // c.set_default("modules.git.symbol_clean", "").unwrap();
    c.set_default("modules.git.symbol_insertion", "+").unwrap();
    c.set_default("modules.git.symbol_deletion", "-").unwrap();
    // c.set_default("modules.git.symbol_push", "⇡").unwrap();
    // c.set_default("modules.git.symbol_pull", "⇣").unwrap();
    c.set_default("modules.git.show_diff_stats", true).unwrap();

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
        .unwrap_or(c.get_str("global.foreground").unwrap_or_default());
    let fg = string_to_colour(fg);

    let bg = c.get_str(&format!("modules.{}.background", name))
        .unwrap_or(c.get_str("global.background").unwrap_or_default());
    let bg = string_to_colour(bg);

    let padding_left = c.get_str(&format!("modules.{}.padding_left", name))
        .unwrap_or(c.get_str("global.padding_left").unwrap_or_default());
    let padding_right = c.get_str(&format!("modules.{}.padding_right", name))
        .unwrap_or(c.get_str("global.padding_right").unwrap_or_default());
    let separator = c.get_str(&format!("modules.{}.separator", name))
        .unwrap_or(c.get_str("global.separator").unwrap_or_default());

    // Override the output if there is "output" in this module's config
    let output = if let Some(s) = c.get_str(&format!("modules.{}.output", name)) {
        s
    } else {
        output.unwrap()
    };

    // Format the main content
    let mut content = format!("\\[{}\\]{}{}{}\\[{}\\]",
                              fg.on(bg).prefix(),
                              padding_left,
                              output,
                              padding_right,
                              fg.on(bg).suffix());

    // Format the separator
    let fg = bg;
    if let Some(name) = last_successful {
        // There is a visible module that comes after this one
        let next_bg = c.get_str(&format!("modules.{}.background", name))
            .unwrap_or(c.get_str("global.background").unwrap_or_default());
        let next_bg = string_to_colour(next_bg);

        content = format!("{}\\[{}\\]{}\\[{}\\]",
                          content,
                          fg.on(next_bg).prefix(),
                          separator,
                          fg.on(next_bg).suffix());
    } else {
        // This is the final module
        content = format!("{}\\[{}\\]{}\\[{}\\]",
                          content,
                          fg.prefix(),
                          separator,
                          fg.suffix());
    }

    (Some(name), Some(ANSIString::from(content)))
}

// Converts a string (from the config file) to a Colour
// See: https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg
fn string_to_colour(s: String) -> Colour {
    if let Ok(i) = s.parse::<u8>() {
        return Colour::Fixed(i);
    }

    let s = s.to_lowercase();
    match s.as_ref() {
        "black" => Colour::Black,
        "bright_black" => Colour::Fixed(008),
        "red" => Colour::Red,
        "bright_red" => Colour::Fixed(009),
        "green" => Colour::Green,
        "bright_green" => Colour::Fixed(010),
        "yellow" => Colour::Yellow,
        "bright_yellow" => Colour::Fixed(011),
        "blue" => Colour::Blue,
        "bright_blue" => Colour::Fixed(012),
        "purple" => Colour::Purple,
        "bright_purple" => Colour::Fixed(013),
        "cyan" => Colour::Cyan,
        "bright_cyan" => Colour::Fixed(014),
        "white" => Colour::White,
        "bright_white" => Colour::Fixed(015),
        _ => panic!("Invalid color option: {} in config file!", s),
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
    use std::path;

    let home = env::var("HOME").unwrap();
    let cwd = env::current_dir().unwrap();

    let mut directory = env::current_dir().unwrap();

    if let Ok(stripped_dir) = cwd.strip_prefix(&home) {
        directory = path::PathBuf::from("~").join(stripped_dir);
    }

    let output = format!("{}", directory.display());

    format_module(c, "directory", Some(output), last_successful)
}

pub fn format_module_git<'a>(c: &mut Config,
                             last_successful: Option<&'a str>)
                             -> (Option<&'a str>, Option<ANSIString<'static>>) {
    use git2::Repository;
    use std::env;

    let mut output = String::new();

    let repo = Repository::discover(env::current_dir().unwrap());
    if let Ok(repo) = repo {
        if let Ok(reference) = repo.head() {
            // Current branch name
            output.push_str(reference.shorthand().unwrap());

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
        }
    }

    if output.is_empty() {
        return (None, None);
    }

    // let symbol_clean = c.get_str("modules.git.symbol_clean").unwrap_or_default();
    // let symbol_push = c.get_str("modules.git.symbol_push").unwrap_or_default();
    // let symbol_pull = c.get_str("modules.git.symbol_pull").unwrap_or_default();

    return format_module(c, "git", Some(output), last_successful);
}
