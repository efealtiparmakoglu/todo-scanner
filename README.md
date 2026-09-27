# todo-scanner

**Finds TODO, FIXME, HACK, XXX and NOTE comments in a source tree and
prints them as `file:line: [TAG] text`. Rust, zero dependencies.**

```
$ todo-scanner src
src/main.rs:14: [FIXME] the parser rejects valid input here
src/lib.rs:9: [TODO] write the doc comment
found 2 TODO-type comment(s)
```

The oldest signal in a codebase is a comment asking for help. todo-scanner
walks a directory, recognises the usual tags and exits non-zero when it
finds any — so it can run in CI as a debt report.

## What it does

- Walks a directory tree (the current directory by default), skipping
  `target/`, `node_modules/`, `.git/` and other noise.
- Recognises `TODO`, `FIXME`, `HACK` and `XXX` (uppercase, anywhere on
  the line) in `.rs`, `.py`, `.js`, `.go` and `.c` files, and reports
  one marker per line.
- Prints `file:line: [TAG] text`, which pipes straight into an editor
  (`todo-scanner . | editor +grep`) or a report.
- Exits 1 when anything is found, 0 when the tree is clean.

## Usage

```sh
todo-scanner            # scan the current directory
todo-scanner src/       # scan one tree
todo-scanner . | wc -l  # how many markers are left
```

Exit codes: 0 no markers, 1 markers found, 2 usage error.

## Development

```sh
cargo test --release
```

## License

MIT — see [LICENSE](LICENSE).
