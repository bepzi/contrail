extern crate clap;
extern crate config;

use clap::{App, Arg};
use config::Config;

use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = "contrail";
const HOME: &str = env!("HOME");

fn main() {
    const MAIN_CONFIG_NAME: &str = "config";

    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about("Fast and configurable shell prompter")
        // .arg(Arg::with_name("exit_code")
        //          .short("e")
        //          .long("exit_code")
        //          .value_name("CODE")
        //          .help("Exit code of the last-executed command")
        //          .takes_value(true))
        .arg(Arg::with_name("config")
                 .short("c")
                 .long("config")
                 .value_name("PATH")
                 .help("Alternate location for the config file")
                 .takes_value(true))
        // .arg(Arg::with_name("shell")
        //          .long("shell")
        //          .value_name("SHELL")
        //          .takes_value(true)
        //          .possible_values(&["bash", "zsh", "fish", "powershell"]))
        .get_matches();

    let mut settings: Config = Config::default();

    let config_path: PathBuf = if let Some(path) = matches.value_of("config") {
        PathBuf::from(path)
    } else {
        // User didn't specify a custom folder for config files
        [HOME, ".config", APP_NAME, MAIN_CONFIG_NAME]
            .iter()
            .collect()
    };

    // TODO: Don't panic if we're using the default folder location
    if !config_path.exists() {
        panic!(
            "config file \"{}\" does not exist",
            config_path.to_str().unwrap()
        );
    }

    let config_path = config_path.to_str().unwrap();

    settings
        .merge(config::File::with_name(config_path))
        .unwrap();
}
