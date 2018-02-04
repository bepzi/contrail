extern crate clap;

use clap::{App, Arg};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = "contrail";

fn main() {
    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about("Fast and configurable shell prompter")
        .arg(
            Arg::with_name("command")
                .short("c")
                .long("command")
                .value_name("CMD")
                .help("Command to be executed and inserted into the output")
                .takes_value(true)
                .required(true)
                .multiple(true),
        )
        .get_matches();

    let commands: Vec<_> = matches.values_of("command").unwrap().collect();
    for cmd in commands.iter() {
        println!("{:?}", cmd);
    }
}
