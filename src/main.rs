//! A Rust implementation of a simpIL interpreter.

use argh::FromArgs;
use parser::Parser;
use scanner::Scanner;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use interpreter::Interpreter;
use io::BufReader;
use tracing_subscriber as tsub;

/// Traverse and execute a syntax tree.
mod interpreter;
/// Turn a token iterator into a statement iterator.
mod parser;
/// Turn a string into a token iterator.
mod scanner;
/// Definitions of the simpIL syntax.
mod syntax;
/// Definitions of the simpIL tokens.
mod tokens;

#[doc(hidden)]
pub(crate) type Error = Box<dyn std::error::Error>;

#[doc(hidden)]
pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Run simpilrs on a simpIL script.
#[derive(FromArgs)]
struct CommandStruct {
    #[argh(positional)]
    file_name: Option<String>,
}

/// Run a program from a file, or as an interactive prompt.
fn main() -> Result<()> {
    tsub::fmt::init();
    let cmd: CommandStruct = argh::from_env();

    match cmd.file_name {
        Some(f) => run_file(f)?,
        None => run_prompt()?,
    };

    Ok(())
}

/// Print the prompt to stdout
fn prompt() -> io::Result<()> {
    print!("> ");
    io::stdout().flush()
}

/// Interactive script mode.
fn run_prompt() -> Result<()> {
    let stdin = std::io::stdin();
    prompt()?;
    for line in stdin.lock().lines() {
        match line {
            Ok(l) => run(l)?,
            Err(_) => break,
        };
        prompt()?;
    }

    Ok(())
}

/// Load script from file.
fn run_file(file_name: String) -> Result<()> {
    let reader = BufReader::new(File::open(file_name)?);
    for line in reader.lines() {
        let line = line?;
        run(line)?;
    }
    Ok(())
}

/// Run the whole pipeline, including the interpreter.
fn run(code: String) -> Result<()> {
    let scanner = Scanner::new(&code);
    println!("{}", &scanner);
    let parser = Parser::new(scanner);
    println!("{}", &parser);
    let _ = Interpreter::new(parser);

    Ok(())
}

#[doc(hidden)]
fn report(line: usize, column: usize, message: &str) {
    println!("[line {}, column {}] Error {{ {} }}", line, column, message);
}
