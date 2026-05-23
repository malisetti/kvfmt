# kvfmt v0.1.0 (ergonomic)

Bidirectional **JSON ↔ logfmt** converter. Ergonomic variant: `clap` derive CLI, `thiserror` errors, rich `--help`, and `--pretty` JSON output.

## Build

```bash
cargo build --release
cargo test
```

## Usage

```bash
# logfmt → JSON (compact)
echo 'foo=bar a=14 f' | kvfmt to-json

# logfmt → JSON (pretty)
echo 'foo=bar a=14' | kvfmt to-json --pretty

# JSON → logfmt
echo '{"foo":"bar","a":14}' | kvfmt to-logfmt

kvfmt --help
kvfmt to-json --help
```

Bare keys in logfmt (no `=`) decode as boolean `true`. Quoted values support spaces and escapes.
