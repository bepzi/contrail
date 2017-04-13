extern crate ansi_term;
extern crate clap;
extern crate config;
extern crate git2;

use std::str::FromStr;

use clap::{App, Arg, Shell};
use config::{Config, File, FileFormat};

pub mod utils;
pub mod modules;

use utils::*;
use modules::*;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const APP_NAME: &'static str = "contrail";

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
        .expect("Exit code passed was not a u8!");

    let shell = if let Some(s) = matches.value_of("shell") {
        // This shouldn't panic, clap will enforce that correct shell
        // types were passed at runtime
        Shell::from_str(s).expect("Invalid shell type passed!")
    } else {
        Shell::Bash
    };

    let module_names: Vec<String> = if let Some(arr) = utils::ref_get_array("global.modules", &c) {
        // into_str() always succeeds, so it's safe to call unwrap()
        arr.into_iter().map(|m| m.into_str().unwrap()).collect()
    } else {
        vec![String::from("cwd"),
             String::from("git"),
             String::from("prompt")]
    };

    for name in &module_names {
        // Format the module
    }
}
