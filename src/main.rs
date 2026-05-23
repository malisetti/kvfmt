mod logfmt;
mod stream;

use std::io::{self, Write};
use stream::{Mode, open_input};

const USAGE: &str = "\
kvfmt v0.1.0 — JSON ↔ logfmt converter (streaming variant)

USAGE:
    kvfmt json2logfmt [FILE|-]
    kvfmt logfmt2json [FILE|-]

Reads input line-by-line (BufReader); emits one output line per input line and
flushes after each line. FILE defaults to stdin; use '-' for stdin explicitly.
";

fn main() {
    if let Err(code) = run() {
        eprintln!("kvfmt: {code}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().ok_or_else(|| {
        format!("missing subcommand\n{USAGE}")
    })?;

    let mode = match cmd.as_str() {
        "json2logfmt" | "j2l" => Mode::JsonToLogfmt,
        "logfmt2json" | "l2j" => Mode::LogfmtToJson,
        "-h" | "--help" | "help" => {
            print!("{USAGE}");
            return Ok(());
        }
        "-V" | "--version" => {
            println!("kvfmt 0.1.0 (streaming)");
            return Ok(());
        }
        other => return Err(format!("unknown subcommand: {other}\n{USAGE}")),
    };

    let path = args.next();
    if args.next().is_some() {
        return Err(format!("too many arguments\n{USAGE}"));
    }

    let reader = open_input(path.as_deref())?;
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    stream::process_reader(mode, reader, &mut writer)?;
    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod cli_tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn version_flag() {
        Command::cargo_bin("kvfmt")
            .unwrap()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("0.1.0"));
    }

    #[test]
    fn stdin_json2logfmt() {
        Command::cargo_bin("kvfmt")
            .unwrap()
            .arg("json2logfmt")
            .write_stdin("{\"k\":\"v\"}\n")
            .assert()
            .success()
            .stdout(predicate::str::contains("k=v"));
    }

    #[test]
    fn file_logfmt2json() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "a=b").unwrap();
        tmp.flush().unwrap();

        Command::cargo_bin("kvfmt")
            .unwrap()
            .arg("logfmt2json")
            .arg(tmp.path())
            .assert()
            .success()
            .stdout(predicate::str::contains("\"a\""));
    }
}
