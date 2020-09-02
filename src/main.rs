use argh::FromArgs;
use scanner::Scanner;
use parser::Parser;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use tracing_subscriber as tsub;

mod scanner;
mod tokens;
mod syntax;
mod visitor;
mod parser;

pub(crate) type Error = Box<dyn std::error::Error>;
pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Run simpilrs on a simpIL script.
#[derive(FromArgs)]
struct CommandStruct {
    #[argh(positional)]
    file_name: Option<String>,
}

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
    let reader = io::BufReader::new(File::open(file_name)?);
    for line in reader.lines() {
        let line = line?;
        run(line)?;
    }
    Ok(())
}

fn run(code: String) -> Result<()> {
    let scanner = Scanner::new(code);
    let tokens = scanner.scan_tokens()?;
    let parser = Parser::new(tokens);
    let _ = parser.parse();

    Ok(())
}

fn report(line: usize, column: usize, message: &str) {
    println!("[line {}, column {}] Error {{ {} }}", line, column, message);
}
