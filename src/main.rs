use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use kvfmt::{json_to_logfmt, logfmt_to_json, KvfmtError};

/// Convert between JSON and logfmt (key=value) log lines.
#[derive(Parser)]
#[command(
    name = "kvfmt",
    version,
    about = "Bidirectional JSON ↔ logfmt converter",
    long_about = "Read JSON or logfmt from a file or stdin and emit the other format.\n\
                  Logfmt is space-separated key=value pairs; bare keys mean true."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Parse logfmt input and write JSON (default when no subcommand path is used via `to-json` alias)
    ToJson(ToJsonOpts),
    /// Parse JSON object input and write logfmt
    ToLogfmt(ToLogfmtOpts),
}

#[derive(Parser)]
struct ToJsonOpts {
    /// Pretty-print JSON with indentation
    #[arg(long)]
    pretty: bool,
    /// Input file (default: stdin)
    #[arg(value_name = "FILE")]
    input: Option<String>,
}

#[derive(Parser)]
struct ToLogfmtOpts {
    /// Input file (default: stdin)
    #[arg(value_name = "FILE")]
    input: Option<String>,
}

fn read_input(path: &Option<String>) -> Result<String, KvfmtError> {
    match path {
        Some(p) => Ok(fs::read_to_string(p)?),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}

fn run() -> Result<(), KvfmtError> {
    let cli = Cli::parse();
    match cli.command {
        Command::ToJson(opts) => {
            let input = read_input(&opts.input)?;
            let out = logfmt_to_json(input.trim_end(), opts.pretty)?;
            print!("{out}");
            if !out.ends_with('\n') {
                println!();
            }
        }
        Command::ToLogfmt(opts) => {
            let input = read_input(&opts.input)?;
            let out = json_to_logfmt(input.trim_end())?;
            println!("{out}");
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
