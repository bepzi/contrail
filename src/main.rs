extern crate ansi_term;
extern crate clap;
extern crate config;
extern crate git2;

use std::str::FromStr;

use ansi_term::{ANSIString, ANSIStrings, Color};
use clap::{App, Arg, Shell};
use config::{Config, File, FileFormat};

mod utils;
mod modules;

use utils::*;
use modules::*;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const APP_NAME: &'static str = "contrail";
const CONFIG_ERR: &'static str = "There was a problem while parsing the config file!";

fn main() {
    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about("Fast and configurable shell prompter")
        .arg(Arg::with_name("exit_code")
                 .short("e")
                 .long("exit_code")
                 .value_name("CODE")
                 .help("Exit code of the last-executed command")
                 .takes_value(true))
        .arg(Arg::with_name("config")
                 .short("c")
                 .long("config")
                 .value_name("FILE")
                 .help("Location of configuration file")
                 .takes_value(true))
        .arg(Arg::with_name("shell")
                 .long("shell")
                 .value_name("SHELL")
                 .takes_value(true)
                 .possible_values(&["bash", "zsh", "fish", "powershell"]))
        .get_matches();

    let mut c = Config::new();

    if let Some(f) = matches.value_of("config") {
        c.merge(File::new(f, FileFormat::Toml).required(false))
            .expect("Failed to merge in config file!");
    }

    let exit_code = matches
        .value_of("exit_code")
        .unwrap_or("255")
        .parse::<u8>()
        .expect("Exit code passed as argument was not a u8!");

    let shell = if let Some(s) = matches.value_of("shell") {
        // This shouldn't panic, clap will enforce that correct shell
        // types were passed at runtime
        Shell::from_str(s).expect("Invalid shell type passed!")
    } else {
        Shell::Bash
    };

    let module_names: Vec<String> = if let Some(arr) = ref_get_array("global.modules", &c) {
        // into_str() always succeeds, so it's safe to call unwrap()
        arr.into_iter()
            .map(|m| m.into_str().unwrap())
            .rev()
            .collect()
    } else {
        vec![String::from("cwd"),
             String::from("git"),
             String::from("prompt")]
    };

    let mut formatted_strings: Vec<ANSIString<'static>> = Vec::new();

    let mut next_bg: Option<Color> = None;
    for name in &module_names {
        let result = match name.as_ref() {
            "prompt" => format_prompt(&c, exit_code, next_bg, shell).expect(CONFIG_ERR),
            "cwd" => format_cwd(&c, next_bg, shell).expect(CONFIG_ERR),
            s => format_generic(s, &c, next_bg, shell).expect(CONFIG_ERR),
        };

        // Only update the next_bg if we successfully formatted. The
        // Vec was reversed earlier (to make it possible for a module
        // to know the "next" module's background), so we must insert
        // at the beginning of the resultant formatted strings Vec
        if let Some(r) = result.output {
            next_bg = result.next_bg;
            formatted_strings.insert(0, r);
        }
    }

    print!("{}", ANSIStrings(formatted_strings.as_slice()));
}
