use kvfmt::parser::{emit_logfmt, parse_json};
use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = if args.len() > 1 {
        std::fs::read_to_string(&args[1]).unwrap_or_else(|e| die(&e.to_string()))
    } else {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s).unwrap_or_else(|e| die(&e.to_string()));
        s
    };
    let pairs = parse_json(&input).unwrap_or_else(|e| die(&e));
    print!("{}", emit_logfmt(&pairs));
}

fn die(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1);
}
