use argh::FromArgs;
use scanner::Scanner;
use std::fs::File;
use std::io;
use std::io::prelude::*;

mod scanner;
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

    for token in tokens {
        println!("{:?}", token);
    }

    Ok(())
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn report(line: usize, location: &str, message: &str) {
    println!("[line {}] Error {} where: {}", line, location, message);
}
