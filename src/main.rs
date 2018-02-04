extern crate clap;

#[macro_use]
extern crate lazy_static;

use clap::{App, Arg, ArgMatches};

use std::thread;
use std::sync::mpsc;
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = "contrail";

lazy_static! {
    // thread::spawn takes a closure where everything used has a
    // static lifetime, so this must be defined as static
    static ref MATCHES: ArgMatches<'static> = App::new(APP_NAME)
        .version(VERSION)
        .about("Fast and configurable shell prompter")
        .arg(
            Arg::with_name("command")
                .short("c")
                .long("command")
                .value_name("CMD")
                .help("Command to be run and inserted into the output")
                .takes_value(true)
                .required(true)
                .multiple(true),
        )
        .get_matches();
}

fn main() {
    let commands: Vec<_> = MATCHES.values_of("command").unwrap().collect();

    let (send, recv) = mpsc::channel();

    for (i, each) in commands.iter().enumerate() {
        let tx = mpsc::Sender::clone(&send);
        let cmd = String::clone(&each.to_string());

        thread::spawn(move || {
            // Start the command call
            let result = Command::new(&cmd)
                .output()
                .expect(&format!("failed to execute commmand {}", cmd));

            if !result.status.success() {
                panic!("command {} failed with exit code {}", cmd, result.status);
            }

            let stdout = String::from_utf8(result.stdout)
                .expect(&format!("output of command {} was not valid utf8", cmd));

            // Send the output of the command and its future position
            // in the final vector
            tx.send((i, stdout)).unwrap();
        });
    }

    // Allow the receiver to close with all senders closed
    drop(send);

    // Convert the results into the final printed out vector
    let mut results: Vec<(usize, String)> = recv.iter().collect();
    results.sort();

    for each in &results {
        print!("{}", each.1);
    }
}
