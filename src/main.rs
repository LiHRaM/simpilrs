use argh::FromArgs;
use parser::Parser;
use scanner::Scanner;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use interpreter::Interpreter;
use io::BufReader;
use tracing_subscriber as tsub;

mod interpreter;
mod parser;
mod scanner;
mod syntax;
mod tokens;

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
    let reader = BufReader::new(File::open(file_name)?);
    for line in reader.lines() {
        let line = line?;
        run(line)?;
    }
    Ok(())
}

fn run(code: String) -> Result<()> {
    let scanner = Scanner::new(&code);
    println!("{}", &scanner);
    let parser = Parser::new(scanner);
    println!("{}", &parser);
    let _ = Interpreter::new(parser);
    
    Ok(())
}

fn report(line: usize, column: usize, message: &str) {
    println!("[line {}, column {}] Error {{ {} }}", line, column, message);
}
