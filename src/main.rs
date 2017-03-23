extern crate ansi_term;
extern crate clap;
extern crate config;
extern crate git2;

mod modules;

use ansi_term::{ANSIString, ANSIStrings};
use clap::{App, Arg};
use config::{Config, File, FileFormat};
use std::path::PathBuf;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let matches = App::new("contrail")
        .version(VERSION)
        .about("Fast and configurable shell prompter")
        .arg(Arg::with_name("exit_code")
                 .short("e")
                 .long("exit_code")
                 .value_name("CODE")
                 .help("The exit code of the last-executed command")
                 .takes_value(true))
        .arg(Arg::with_name("config")
                 .short("c")
                 .long("config")
                 .value_name("FILE")
                 .help("The configuration file")
                 .takes_value(true))
        .arg(Arg::with_name("zsh").short("z").long("zsh").help("Enables ZSH mode"))
        .get_matches();

    let mut c = Config::new();

    // Merge in the defaults
    modules::merge_defaults(&mut c);

    // Merge in the config file if provided
    if let Some(f) = matches.value_of("config") {
        c.merge(File::new(f, FileFormat::Toml).required(false))
            .expect("Unable to read the config file!");
    }

    // Merge in the command-line arguments
    if matches.is_present("zsh") {
        c.set("global.shell", "zsh").unwrap();
    }

    // A few things to note:
    // - Calling `c.get()` is a workaround, `c.get_array()` consumes `self`
    // - We do `rev()` because it's easier to handle things like finding
    //   the next module's background color when you go in backwards order
    let module_names: Vec<String> = c.get("global.modules")
        .unwrap()
        .into_array()
        .unwrap_or_default()
        .into_iter()
        .map(|m| m.into_str().unwrap())
        .rev()
        .collect();

    let mut ansi_strings: Vec<ANSIString<'static>> = Vec::new();

    // The name of the last successfully formatted module
    let mut last_successful_format: Option<&str> = None;

    for name in &module_names {
        let formatted_module: Option<ANSIString<'static>>;

        let result = match name.as_ref() {
            "directory" => modules::format_module_directory(&mut c, last_successful_format),
            "exit_code" => {
                let exit_code = matches.value_of("exit_code").unwrap_or("255");
                modules::format_module_exit_code(&mut c, last_successful_format, exit_code)
            }
            "git" => modules::format_module_git(&mut c, last_successful_format),
            "prompt" => {
                let exit_code = matches.value_of("exit_code").unwrap_or("255");
                modules::format_module_prompt(&mut c, last_successful_format, exit_code)
            }
            _ => {
                // The output will be found in the config file
                modules::format_module(&mut c, name, None, last_successful_format)
            }
        };

        // Updates the last formatted module variable and prints it
        // out only if it was successful
        formatted_module = result.1;
        if result.0.is_some() {
            last_successful_format = result.0;
            ansi_strings.insert(0, formatted_module.unwrap());
        }

    }

    print!("{}", ANSIStrings(ansi_strings.as_slice()));
}
