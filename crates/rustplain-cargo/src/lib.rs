use rustplain_diag::{parse_line, JsonMessage, ParseError};
use std::io::{self, BufRead, BufReader};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct CargoExit {
    pub code: i32,
}

pub fn run<F>(subcommand: &str, extra_args: &[String], mut on_line: F) -> io::Result<CargoExit>
where
    F: FnMut(&str, Result<JsonMessage, ParseError>),
{
    let mut command = Command::new("cargo");
    command
        .arg(subcommand)
        .arg("--message-format=json")
        .args(extra_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    let mut child = command.spawn()?;
    let stdout = child
        .stdout
        .take()
        .expect("stdout was requested from the cargo child");
    for line in BufReader::new(stdout).lines() {
        let line = line?;
        on_line(&line, parse_line(&line));
    }
    let status = child.wait()?;
    Ok(CargoExit {
        code: status.code().unwrap_or(1),
    })
}
