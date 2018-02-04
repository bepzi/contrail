extern crate clap;

use clap::{App, Arg};

use std::thread;
use std::sync::mpsc;

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
    let (send, recv) = mpsc::channel();
    
    for cmd in commands.iter() {
        let tx = mpsc::Sender::clone(&send);
        
        thread::spawn(move || {
            // Start the command call
            
            // Send the output of the command and its future position
            // in the final vector
            tx.send("Yo").unwrap();
        });
    }
    
    // Allow the receiver to close with all senders closed
    drop(send);

    // Convert the results into the final printed out vector
    let result = recv.iter();
    for item in result {
        println!("{}", item);
    }
}
