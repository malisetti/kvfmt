# kvfmt

JSON ↔ logfmt bidirectional converter (Rust CLI v0.1.0).

**Variant: streaming** — processes stdin (or a file) with `BufReader`, one line per record, flushing stdout after each emitted line. No whole-file buffering.

## Build

```bash
cargo build --release
```

## Usage

```bash
# JSON lines → logfmt lines
echo '{"at":"info","status":200}' | kvfmt json2logfmt

# logfmt lines → JSON lines
echo 'at=info status=200' | kvfmt logfmt2json

# explicit file or stdin
kvfmt json2logfmt access.log
kvfmt logfmt2json -   # stdin
```

## Tests

```bash
cargo test
```
