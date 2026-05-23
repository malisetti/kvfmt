use std::io::Write;
use std::process::{Command, Stdio};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_kvfmt"))
}

fn run_with_stdin(args: &[&str], stdin: &str) -> std::process::Output {
    let mut child = bin()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn help_shows_subcommands() {
    let out = bin().arg("--help").output().unwrap();
    assert!(out.status.success());
    let help = String::from_utf8(out.stdout).unwrap();
    assert!(help.contains("to-json"));
    assert!(help.contains("to-logfmt"));
    let pretty = bin().args(["to-json", "--help"]).output().unwrap();
    assert!(String::from_utf8(pretty.stdout).unwrap().contains("--pretty"));
}

#[test]
fn logfmt_to_json_and_back() {
    let line = r#"foo=bar a=14 baz="hello kitty" f"#;
    let json = run_with_stdin(&["to-json", "--pretty"], line);
    assert!(
        json.status.success(),
        "{}",
        String::from_utf8_lossy(&json.stderr)
    );
    let body = String::from_utf8(json.stdout).unwrap();
    assert!(body.contains("\"foo\": \"bar\""));
    assert!(body.contains("\"a\": 14"));
    let logfmt = run_with_stdin(&["to-logfmt"], &body);
    assert!(logfmt.status.success());
    let back = String::from_utf8(logfmt.stdout).unwrap();
    assert!(back.contains("foo=bar"));
    assert!(back.contains("a=14"));
}
