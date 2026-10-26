/*******************************************************************************
* 
*   🌿  MAIN
* 
*******************************************************************************/
//! # 💬 Description
//! 
//! * Declare global constants and define the `main` function.
//! * Initialize `env_logger` and test its macros.
//! * Input a string from the user with `stdin`.
//! 
//! # 🚧 Things to Do
//! 
//! * Find (or make) a more appropriate `Error` to return from `main`.
//! 
//! # 👀 See Also
//! 
//! * 🔗 [`env_logger`](https://crates.io/crates/env_logger)
//! 
//! ## 📝 Note
//! 
//! I assume these links are correct. Multiple use of names in different namespaces
//! can make the documentation difficult to navigate at times.

// Keep the `rustc` output clean as possible
#![allow(warnings)]

use std::env::current_exe;
use std::fs::ReadDir;
// Rust crates
use std::io;
use std::io::{ // Needed for `flush` (?)
    IsTerminal,
    Read,
    // self,
    stdin, // Standard input
    stdout, // Standard output
    Write,
};

use std::path::{ Path, PathBuf };

use std::process::{ Command, Stdio };

use std::sync::{ OnceLock };

// Command line parsing crate
use clap::Parser;

// Logging crates
use env_logger;
use hw::constants::PROGRAM_NAME;
use log::{
    LevelFilter,
    debug,
    error,
    info,
    trace,
    warn,
};

// Rust `readline` module
use rustyline; // So as not to mask any other `Error`
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor,
    // Result,
};

use shlex::split;

// Local packages
use hw::constants::{
    ERROR_PICT,
    SCRATCH_HEAD,
};

use hw::files::home;

// Constant chars
const PERIOD_CHAR: char = '.';
// Constant strings
const GREETING: &str = "Hello, "; // Format string
const NAME: &str = "🌎 "; // Default name
const DATA_DIR_NAME: &str = "data"; // Name of the app's user data folder.

// REPL command strings
const HELP: &str = "help";
const QUIT: &str = "quit";
const CONVERT: &str = "convert";

enum Commands {
    HELP,
    QUIT,
}

// Global variable
pub static arguments: OnceLock<Args> = OnceLock::new();

#[derive(Debug)]
struct Error {
    code: u64,
}

impl From<ReadlineError> for Error {
    fn from (_: ReadlineError) -> Self {
        Self {
            code: 0,
        }
    }
}

// #[derive(Parser)]
#[derive(Debug, Parser)]
#[command(version)]
pub struct Args {
    /// Enable debug logging
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,
    /// Enable verbose logging
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
    /// Enable warnings
    #[arg(short = 'w', long = "warn")]
    pub warnings: bool,
    /// Enable trace logging
    #[arg(short = 't', long = "trace")]
    pub trace: bool,
    /// Other arguments
    pub args: Vec<String>,
}

enum CalcResult {
    Ok(f64),
    Error(String),
}

#[derive(Debug, Parser)]
#[command(version)]
pub struct QuitParser {
    /// Enable debug logging
    #[arg(short = 'f', long = "force")]
    pub force: bool,
}

enum QuitResult {
    Ok(),
    Error(String),
}

#[derive(Debug, Parser)]
pub struct HelpParser {
    pub cmd: String,
}

/******************************************************************************
*
*   init_log()
*
******************************************************************************/

/// Initialize the logging module `env_logger`
fn init_log() -> () {

    // Check for options that change the logging level
    let level = if arguments.get().unwrap().trace { // More detailed debug output
        LevelFilter::Trace
    } else if arguments.get().unwrap().debug { // Debugging output
        LevelFilter::Debug
    } else if arguments.get().unwrap().verbose { // Extra output
        LevelFilter::Info
    } else if arguments.get().unwrap().warnings { // `-w` stops warnings
        LevelFilter::Error
    } else {
        LevelFilter::Warn // Show warnings by default
    };

    // Set the logging level
    env_logger::Builder::new().filter_level(level).init();
}

/******************************************************************************
*
*   input_name()
*
******************************************************************************/

/// Prompt the user for their name.
///
/// # Return
///
/// `Option<String, ()>` Containing the `input_name` or `NAME`.
fn input_name() -> Option<String> {
    let mut name = String::new();
    print!("Enter your name: "); // No newline
    io::stdout().flush().unwrap(); // Sends anything waiting in the buffer out
    io::stdin().read_line(&mut name).unwrap(); // Input a `String`
    name = name.trim().to_string(); // Without `to_string` it's a `&str`
    if name.is_empty() { // Use the default `NAME`
        name = String::from(NAME);
    }
    Some(name) // Return a string either way
}

/******************************************************************************
*
*   get_piped_input()
*
******************************************************************************/

/// Retrieve input from a pipe, if there is any.
///
/// # Return
///
/// Piped input as a `String`.
pub fn get_piped_input() -> Option<String> {
    if !io::stdin().is_terminal() {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).unwrap();

        if input.ends_with('\n') {
            input.pop();
        }

        Some(input)
    } else {
        None
    }
}

/******************************************************************************
*
*   calculate()
*
******************************************************************************/
fn calculate(a: f64, operation: &str, b: f64) -> CalcResult {
    match operation {
        "+" => CalcResult::Ok(a + b),
        "-" => CalcResult::Ok(a - b),
        "*" => CalcResult::Ok(a * b),
        "/" => {
            if b == 0.0 {
                CalcResult::Error(String::from("Error: division by zero"))
            } else {
                CalcResult::Ok(a / b)
            }
        }
        _ => CalcResult::Error(
            format!("Error: operation '{}' not recognized. Use +, -, * or /", operation)
        ),
    }
}

// ── Input/output functions ────────────────────────────────────────────────

/// Reads a line of text from the user with a prompt
fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read");
    input.trim().to_string()
}

/// Reads a floating-point number from the user, retrying if input is invalid
fn read_number(prompt: &str) -> f64 {
    loop {
        let text = read_line(prompt);
        match text.parse::<f64>() {
            // turbofish `::<_>` syntax annotates the return type
            Ok(n) => return n,
            Err(_) => println!("Please enter a valid number (e.g. 3.14)"),
        }
    }
}

/// TODO: Put something a little more helpful in here.
fn help (args: &Vec<String>) {
    println!("God helps those who help themselves!");
}

/// Convert a temperature from Celsius to Fahrenheit or vice versa
/// # Arguments
/// * `line` - A string containing the temperature and unit to convert
/// # Example
/// ```
/// convert("39c -> f");
/// ```
/// The input string should be parsed for the only elements that are
/// relevant to the conversion. The rest of the string is ignored.
/// 
/// Conversion to or from Kelvin should be supported, with Kelvin temps
/// indicated by the letter 'k'. Temperature letters should be case-insensitive.
/// The input string should be parsed for the only elements that are relevant to the conversion.
/// The rest of the string is ignored.
///  
fn convert(line: &str) {
    let words: Vec<_> = line.split_whitespace().skip(1).collect();
    if words.is_empty() {
        println!("Usage: convert <temperature><unit> -> <unit> (e.g. convert 39C -> F)");
        return;
    }

    let source = words.iter().enumerate().find_map(|(index, word)| {
        if let Ok(value) = word.parse::<f64>() {
            let unit = words
                .get(index + 1)
                .and_then(|unit| parse_temperature_unit(unit))?;
            return Some((index + 1, value, unit));
        }

        let suffix = word.chars().last()?;
        let unit = parse_temperature_unit(&suffix.to_string())?;
        let number = &word[..word.len() - suffix.len_utf8()];
        let value = number.parse::<f64>().ok()?;
        Some((index, value, unit))
    });

    let Some((source_end, value, source_unit)) = source else {
        println!("Could not find a temperature. Example: convert 39C -> F");
        return;
    };
    let target_unit = words
        .iter()
        .skip(source_end + 1)
        .find_map(|word| parse_temperature_unit(word));
    let Some(target_unit) = target_unit else {
        println!("Could not find a target unit. Example: convert 39C -> F");
        return;
    };

    let converted = convert_temperature(value, source_unit, target_unit);
    println!(
        "{value} {} = {converted:.2} {}",
        source_unit.to_ascii_uppercase(),
        target_unit.to_ascii_uppercase()
    );
}

fn convert_temperature(value: f64, source_unit: char, target_unit: char) -> f64 {
    let celsius = match source_unit {
        'c' => value,
        'f' => (value - 32.0) * 5.0 / 9.0,
        'k' => value - 273.15,
        _ => unreachable!(),
    };
    match target_unit {
        'c' => celsius,
        'f' => celsius * 9.0 / 5.0 + 32.0,
        'k' => celsius + 273.15,
        _ => unreachable!(),
    }
}

fn parse_temperature_unit(text: &str) -> Option<char> {
    match text.to_ascii_lowercase().as_str() {
        "c" => Some('c'),
        "f" => Some('f'),
        "k" => Some('k'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{convert_temperature, parse_temperature_unit};

    #[test]
    fn converts_between_temperature_scales() {
        assert!(
            (convert_temperature(39.0, 'c', 'f') - 102.2).abs() < f64::EPSILON * 128.0
        );
        assert!(convert_temperature(273.15, 'k', 'c').abs() < f64::EPSILON * 128.0);
    }

    #[test]
    fn parses_temperature_units_without_case_sensitivity() {
        assert_eq!(parse_temperature_unit("C"), Some('c'));
        assert_eq!(parse_temperature_unit("f"), Some('f'));
        assert_eq!(parse_temperature_unit("K"), Some('k'));
    }
}

/******************************************************************************
*
*   main()
*
******************************************************************************/

/// Initialize the program and exercise the output functions
fn main() -> () {
    // Paths may contain non-Unicode chars. For some reason. 🙄
    let PROGRAM_PATH = current_exe().expect("`current_exe` failed");
    let BINARY_EXE_NAME = PROGRAM_PATH.file_stem().unwrap().to_string_lossy().into_owned().to_string();
    println!("Running {}", BINARY_EXE_NAME);

    let mut USER_DATA_DIR = home();

    let args = Args::parse();       // Parse command line arguments and
    arguments.set(args).unwrap();   // store them in `arguments`.

    // Standard input / output
    let output = String::from(GREETING) + &input_name().unwrap();
    println!("{}", output);
    io::stdout().flush().unwrap();

    // Initialize `env_logger`
    init_log();

    // Logging macros send their output to `stderr`
    debug!("RAID! Kills bugs dead!");
    info!("Hello, 🌎 !");
    warn!("Danger, Will Robinson!");
    error!("I've got a bad feeling about this.");
    trace!("Screw you guys! I'm going home!");

    // Comment this line out to continue
    // return ();

    // Add more code here...

    let HISTORY_FNAME = "history.txt";
    let mut rl = DefaultEditor::new()
        .expect("Couldn't initialize `rustyline`!");

    info!("Starting `rustyline` REPL...");

    if rl.load_history(HISTORY_FNAME).is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())
                    .expect("Error saving input line to history file!");
                let words = split(&line.to_string()).unwrap();
                let cmd = &words[0];
                let nargs = words.len();
                // Parse a vector of strings
                match cmd.as_str() {
                    /// Convert a temperature from Celsius to Fahrenheit or vice versa
                    CONVERT => {
                        convert(&line);
                    }
                    /// Exit the program normally
                    QUIT => {
                        // TODO: Implement a confirmation dialog here.
                        // UPDATE: I implemented a confirmation dialog in Python using the `dialogs` module.
                        let request = r#"{
                            "dialog" : "question",
                            "title"  : "Quit",
                            "text"   : "Do you really want to quit?"
                        }"#;

                        let mut child = Command::new("py")
                            .args(["-m", "dialogs"])
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn()
                            .expect("Failed to create `Command`!");

                        child.stdin.take()
                                   .unwrap()
                                   .write_all(request.as_bytes())
                                   .expect("Write bytes to `stdin` failed!");

                        let mut response = String::new();
                        child.stdout.take()
                                    .unwrap()
                                    .read_to_string(&mut response)
                                    .expect("");

                        let status = child.wait().expect("Waiting returned an error!");

                        println!("Response: {response}");

                        // TODO: Parse the JSON response and check if the user clicked "Yes" or "No"
                        // For example, you could use the `serde_json` crate to parse the response:
                        // let json_response: serde_json::Value = serde_json::from_str(&response).expect("Failed to parse JSON response");
                        // let user_clicked_yes = json_response["clicked"] == serde_json::Value::Bool(true);

                        // Sure. I could do that. But the `dialogs` module returns a simple string.
                        // So, for now, I'll just check if the response is "yes" or "no".

                        // Or, since (if memory serves) the `dialogs` module returns a string, I could just check if the response is "True or "False".

                        let answer = response
                            .lines()
                            .rev()
                            .find(|line| !line.trim().is_empty())
                            .unwrap_or("")
                            .trim();

                        if answer == "True" {
                            println!("User clicked 'Yes'. Quitting...");
                            return;
                        } else {
                            println!("User clicked 'No'. Continuing...");
                        }

                    }
                    HELP => {
                        help(&words)
                    }
                    _ => {
                        error!("{}Unknown command! {}",
                              ERROR_PICT, SCRATCH_HEAD);
                        // continue;
                    }
                }
                // println!("Line: {:?}", words);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }

    rl.save_history(HISTORY_FNAME);
    ()

}
