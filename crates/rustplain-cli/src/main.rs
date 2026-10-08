use clap::{Parser, Subcommand};
use rustplain_diag::{parse_line, JsonMessage};
use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
    process::ExitCode,
};

#[derive(Debug, Parser)]
#[command(
    name = "rustplain",
    about = "plain-language explanations for rustc diagnostics"
)]
struct Args {
    #[arg(long, global = true, value_name = "FILE", conflicts_with = "raw")]
    from_json: Option<PathBuf>,
    #[arg(long, global = true)]
    raw: bool,
    #[arg(long, global = true)]
    show_original: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Check(CargoArgs),
    Build(CargoArgs),
    Test(CargoArgs),
    Run(CargoArgs),
    Clippy(CargoArgs),
}

#[derive(Debug, clap::Args)]
struct CargoArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    if let Some(command) = args.command {
        return run_cargo(command, args.raw, args.show_original);
    }

    let input = match read_input(args.from_json.as_ref()) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("rustplain: {error}");
            return ExitCode::from(2);
        }
    };
    if args.raw {
        print!("{input}");
        return ExitCode::SUCCESS;
    }
    let mut had_error = false;
    for line in input.lines().filter(|line| !line.trim().is_empty()) {
        match parse_line(line) {
            Ok(JsonMessage::Diagnostic(diagnostic)) => {
                had_error |= diagnostic.level == "error" || diagnostic.level.starts_with("error:");
                print!(
                    "{}",
                    rustplain_render::render(&diagnostic, args.show_original)
                );
            }
            Ok(JsonMessage::Other(_))
            | Ok(JsonMessage::Artifact(_))
            | Ok(JsonMessage::BuildFinished(_)) => {}
            Err(error) => eprintln!("rustplain: {error}"),
        }
    }
    if had_error {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn run_cargo(command: Command, raw: bool, show_original: bool) -> ExitCode {
    let (subcommand, extra_args) = match command {
        Command::Check(args) => ("check", args.args),
        Command::Build(args) => ("build", args.args),
        Command::Test(args) => ("test", args.args),
        Command::Run(args) => ("run", args.args),
        Command::Clippy(args) => ("clippy", args.args),
    };

    let result = rustplain_cargo::run(subcommand, &extra_args, |line, parsed| {
        if raw {
            println!("{line}");
            return;
        }

        match parsed {
            Ok(JsonMessage::Diagnostic(diagnostic)) => {
                print!("{}", rustplain_render::render(&diagnostic, show_original));
            }
            Ok(JsonMessage::Artifact(_))
            | Ok(JsonMessage::BuildFinished(_))
            | Ok(JsonMessage::Other(_))
            | Err(_) => {
                // Cargo's machine-readable stream is also the source of truth
                // for consumers that need artifacts or an unrecognised event.
                println!("{line}");
            }
        }
    });

    match result {
        Ok(exit) => ExitCode::from(exit.code.clamp(0, u8::MAX as i32) as u8),
        Err(error) => {
            eprintln!("rustplain: failed to run cargo: {error}");
            ExitCode::from(2)
        }
    }
}

fn read_input(path: Option<&PathBuf>) -> io::Result<String> {
    let mut input = String::new();
    match path {
        Some(path) if path.as_os_str() == "-" => {
            io::stdin().read_to_string(&mut input)?;
        }
        Some(path) => {
            File::open(path)?.read_to_string(&mut input)?;
        }
        None => {
            io::stdin().read_to_string(&mut input)?;
        }
    }
    Ok(input)
}
