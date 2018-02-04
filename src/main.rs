extern crate clap;

use clap::{App, Arg};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = "contrail";

fn main() {
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
}
