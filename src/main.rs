extern crate clap;
extern crate config;
extern crate glob;

use clap::{App, Arg};
use config::{Config, File};
use glob::glob;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = "contrail";

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
        // .arg(Arg::with_name("shell")
        //          .long("shell")
        //          .value_name("SHELL")
        //          .takes_value(true)
        //          .possible_values(&["bash", "zsh", "fish", "powershell"]))
        .get_matches();
    
    let mut settings = Config::default();

    if let Some(f) = matches.value_of("config") {
        // Merge in the default configs
        // c.merge(File::new(f, FileFormat::Toml).required(false))
        //     .expect("Failed to merge in config file!");
    }

    // Merge in the module config files
    settings.merge(glob("conf/*")
                   .unwrap()
                   .map(|path| File::from(path.unwrap()))
                   .collect::<Vec<_>>())
        .unwrap();
}
