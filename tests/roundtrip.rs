use kvfmt::parser::{emit_json, emit_logfmt, parse_json, parse_logfmt};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn lib_roundtrip() {
    let json = r#"{"a":"x","b":"42","c":"true","d":"null"}"#;
    let lf = emit_logfmt(&parse_json(json).unwrap());
    let back = emit_json(&parse_logfmt(&lf).unwrap()).unwrap();
    let v1: serde_json::Value = serde_json::from_str(json).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&back).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn j2k_k2j_bin_roundtrip() {
    let json = r#"{"msg":"hello world","n":"99","ok":"true"}"#;
    let j2k = env!("CARGO_BIN_EXE_j2k");
    let k2j = env!("CARGO_BIN_EXE_k2j");
    let mut j2k_proc = Command::new(j2k)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    j2k_proc.stdin.as_mut().unwrap().write_all(json.as_bytes()).unwrap();
    let j2k_out = j2k_proc.wait_with_output().unwrap();
    assert!(j2k_out.status.success());
    let mut k2j_proc = Command::new(k2j)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    k2j_proc
        .stdin
        .as_mut()
        .unwrap()
        .write_all(&j2k_out.stdout)
        .unwrap();
    let k2j_out = k2j_proc.wait_with_output().unwrap();
    assert!(k2j_out.status.success());
    let v1: serde_json::Value = serde_json::from_str(json).unwrap();
    let out = std::str::from_utf8(&k2j_out.stdout).unwrap().trim();
    let v2: serde_json::Value = serde_json::from_str(out).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn quoted_values() {
    let lf = r#"k="a b" v=plain"#;
    let pairs = parse_logfmt(lf).unwrap();
    assert_eq!(pairs, vec![("k".into(), "a b".into()), ("v".into(), "plain".into())]);
    let back = parse_logfmt(&emit_logfmt(&pairs)).unwrap();
    assert_eq!(pairs, back);
}
