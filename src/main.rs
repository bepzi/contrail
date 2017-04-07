extern crate ansi_term;
extern crate clap;
extern crate config;
extern crate git2;

mod modules;

use ansi_term::{ANSIString, ANSIStrings};
use clap::{App, Arg};
use config::{Config, File as ConfigFile, FileFormat};

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
        .arg(Arg::with_name("generate_config")
                .short("g").long("generate_config")
                .help("Generates a default configuration file at the config file path given by the -c flag."))
        .get_matches();

    let mut c = Config::new();

    // Merge in the defaults
    modules::merge_defaults(&mut c);

    // Merge in the config file if provided
    if let Some(f) = matches.value_of("config") {
        if matches.is_present("generate_config") {
            create_config_file(&f);
        }
        c.merge(ConfigFile::new(f, FileFormat::Toml).required(false))
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

fn create_config_file(path: &str) {
    use std::io::{stdin, stdout, Write, BufRead};
    use std::fmt::{format};
    use std::path::{Path, PathBuf};
    // use std::env;
    // let home = env::var("HOME").unwrap();
    // The default path for contrail's config.toml file.
    // let formatted_path = format!("{}/.config/contrail/config.toml", path);

    let formatted_path = format!("{}", path);
    let config_path = Path::new(formatted_path.as_str());
    // Check if the file already exists
    match config_path.metadata() {
        Ok(_) => {
            // Potentially ask user if they would like to overwrite the existing file with a default one here.
            // File exists, do nothing.
        }
        Err(_) => {
            // File doesn't already exist, create a default config file for the user
            use std::fs::{create_dir_all, copy, File};

            //get user confirmation
            let stdout = stdout();
            let mut handle = stdout.lock();
            handle.write(format(
                format_args!("\n{:?}\n", config_path.as_os_str())
            ).as_bytes())
            .ok();     //assume we are ok, because we will exit later on no matter what.
            handle.write(b"Is this the correct path for the contrail configuration file? <y/n> ").ok();
            handle.flush().ok();

            let mut user_response = String::new();
            let stdin = stdin();
            let mut handle = stdin.lock();

            handle.read_line(&mut user_response).ok();

            match user_response.trim().to_lowercase().as_ref() {
                "y" => {
                    // println!("Creating a new default config file at ~/.config/contrail/config.toml");
                    println!("Creating a config file at '{:?}'", config_path.as_os_str());
                    let cargo_dir = env!("CARGO_MANIFEST_DIR");

                    let mut example_config_path = PathBuf::from(cargo_dir);
                    example_config_path.push("example_config.toml");


                    // Create all directories required for the path,
                    match create_dir_all(config_path.parent().unwrap()) {
                        Ok(_) => (),
                        Err(err) => println!("Error while creating a config directory: {}", err),
                    }

                    // Create the config_file,
                    match File::create(config_path) {
                        Ok(_) => (),
                        Err(err) => println!("Error creating the config file: {}", err),
                    }

                    // Copy contents of the example_config to config
                    match copy(example_config_path.as_os_str(), config_path.as_os_str()) {
                        Ok(_) => (),
                        Err(err) => println!("Error writing to the config file: {}", err),
                    }
                    println!("Finished writing config file!");

                }
                _ => ()
            }


        }
    }
}
