# Agent Workflow for DotAgent

This project is a Rust CLI coding agent. Use [mr boxington](https://mr-boxington.jdx.dev)
(`mbx`) for compiling Cargo commands so worktrees share one cache. Keep
`cargo fmt` and `cargo install` as plain `cargo`.

## Cargo Workflow

`mbx` wraps compiling cargo subcommands; `cargo fmt` stays plain.

### Develop

- `mbx check` - Fast compile checks
- `mbx test` - Run tests
- `cargo fmt` - Format code
- `mbx clippy --workspace --all-targets --all-features -- -D warnings` - Lint with warnings as errors

### Build

- `mbx build` - Debug build
- `mbx build --release` - Optimized release build
- `mbx run` - Run the CLI locally

## CI Integration

GitHub Actions workflows in this repository run equivalent checks:

- CI: formatting, clippy, tests, and release build validation
- Release: tag-triggered verification and release artifact publishing

## Common Pitfalls

- Do not bypass `mbx`/`cargo` with ad-hoc build scripts when standard Cargo commands are sufficient.
- Keep behavior parity for tool names, tool semantics, and CLI ergonomics.
- When changing file-tool behavior, update tests in `tests/tools_test.rs` and `tests/env_test.rs`.

## Review Checklist for Agents

- [ ] Run `cargo fmt`.
- [ ] Run `mbx clippy --workspace --all-targets --all-features -- -D warnings`.
- [ ] Run `mbx test`.
