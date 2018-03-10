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
        .about("Asynchronous command executor and output concatenator.")
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
        .arg(
            Arg::with_name("separator")
                .short("s")
                .help("Draw a separating line between each command's output")
                .requires("command")
        )
        .arg(
            Arg::with_name("newlines")
                .long("strip-newlines")
                .help("Behavior to take regarding stripping newlines")
                .takes_value(true)
                .possible_values(&["leading", "trailing", "all"])
        )
        .get_matches();
}

fn main() {
    let commands: Vec<_> = MATCHES.values_of("command").unwrap().collect();

    let (send, recv) = mpsc::channel();

    // We enumerate because we need to keep track of the original
    // calling order. The commands won't finish in the same order they
    // were called in.
    for (i, each) in commands.iter().enumerate() {
        let tx = mpsc::Sender::clone(&send);
        let input: String = String::clone(&each.to_string());

        thread::spawn(move || {
            let (cmd, args) = split_options_from_command(&input);

            // Start the command call
            let result = Command::new(&cmd)
                .args(&args)
                .output()
                .expect(&format!("failed to execute commmand {}", cmd));

            if !result.status.success() {
                panic!("command {} failed with {}", cmd, result.status);
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

    // Convert the results into the final printed out vector. Since
    // they were run asynchronously, they need to be put back into the
    // original order they were called in
    let mut results: Vec<(usize, String)> = recv.iter().collect();
    results.sort();

    for (i, each) in results.iter().enumerate() {
        if MATCHES.is_present("separator") {
            println!("#{}) `{}`", (i + 1), commands[i]);
        }

        if let Some(newline_behavior) = MATCHES.value_of("newlines") {
            print!("{}", strip_newlines(&each.1, newline_behavior));
        } else {
            print!("{}", each.1);
        }
    }
}

/// Removes newlines either from the beginning, end, or throughout an
/// input string. Valid stripping behaviors are "leading", "trailing",
/// or "all".
fn strip_newlines(input: &str, behavior: &str) -> String {
    let newlines: &[_] = &['\n', '\r'];

    match behavior {
        "leading" => input.trim_left_matches(newlines).to_string(),
        "trailing" => input.trim_right_matches(newlines).to_string(),
        "all" => input.replace(newlines, "").to_string(),
        _ => panic!(
            "unrecognized newline stripping behavior '{}', which should not happen",
            behavior
        ),
    }
}

/// Separates the whitespace-delimited arguments passed to a command
/// in a string. Returns a tuple with the first element being the
/// command itself, and the second element a Vec containing each
/// argument.
fn split_options_from_command(input: &str) -> (&str, Vec<&str>) {
    let mut args: Vec<&str> = input.split_whitespace().collect();

    if args.is_empty() {
        ("", vec![])
    } else {
        (args.remove(0), args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn invalid_stripping_behavior_panics() {
        strip_newlines("", "invalid behavior");
    }

    #[test]
    fn strip_leading_newlines() {
        let input = String::from("\nHello, world!");
        let expected = "Hello, world!";

        assert_eq!(expected, strip_newlines(&input, "leading"));
    }

    #[test]
    fn strip_trailing_newlines() {
        let input = String::from("Hello, world!\n\r");
        let expected = "Hello, world!";

        assert_eq!(expected, strip_newlines(&input, "trailing"));
    }

    #[test]
    fn strip_all_newlines() {
        let input = String::from("\rThis \nstring \rhad \nmany \nnewlines.\r");
        let expected = "This string had many newlines.";

        assert_eq!(expected, strip_newlines(&input, "all"))
    }

    #[test]
    fn no_option_commands() {
        struct Test<'a> {
            input: &'a str,
            expected: (&'a str, Vec<&'a str>),
        }

        let tests = vec![
            Test {
                input: "contrail",
                expected: ("contrail", Vec::new()),
            },
            Test {
                input: "contrail ",
                expected: ("contrail", Vec::new()),
            },
            Test {
                input: " contrail ",
                expected: ("contrail", Vec::new()),
            },
        ];

        for test in &tests {
            assert_eq!(test.expected, split_options_from_command(test.input));
        }
    }

    #[test]
    fn single_option_commands() {
        struct Test<'a> {
            input: &'a str,
            expected: (&'a str, Vec<&'a str>),
        }

        let tests = vec![
            Test {
                input: "contrail -v",
                expected: ("contrail", vec!["-v"]),
            },
            Test {
                input: " contrail -v ",
                expected: ("contrail", vec!["-v"]),
            },
            Test {
                input: "ls -al",
                expected: ("ls", vec!["-al"]),
            },
        ];

        for test in &tests {
            assert_eq!(test.expected, split_options_from_command(test.input));
        }
    }

    #[test]
    fn multiple_option_commands() {
        struct Test<'a> {
            input: &'a str,
            expected: (&'a str, Vec<&'a str>),
        }

        let tests = vec![
            Test {
                input: "contrail -v  -g -a",
                expected: ("contrail", vec!["-v", "-g", "-a"]),
            },
            Test {
                input: " contrail -v -- \"Hello\" ",
                expected: ("contrail", vec!["-v", "--", "\"Hello\""]),
            },
            // Probably not the desired result by the user, but it's
            // their responsibility to format their commands correctly
            Test {
                input: "contrail -v ls -al",
                expected: ("contrail", vec!["-v", "ls", "-al"]),
            },
        ];

        for test in &tests {
            assert_eq!(test.expected, split_options_from_command(test.input));
        }
    }
}
