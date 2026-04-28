# Agent Workflow for DotAgent

This project is a Rust clone of teenycode. Use `cargo` for all development and verification workflows.

## Cargo Workflow

`cargo` is the primary CLI for the full development lifecycle.

### Develop

- `cargo check` - Fast compile checks
- `cargo test` - Run tests
- `cargo fmt` - Format code
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` - Lint with warnings as errors

### Build

- `cargo build` - Debug build
- `cargo build --release` - Optimized release build
- `cargo run` - Run the CLI locally

## CI Integration

GitHub Actions workflows in this repository run equivalent checks:

- CI: formatting, clippy, tests, and release build validation
- Release: tag-triggered verification and release artifact publishing

## Common Pitfalls

- Do not bypass `cargo` with ad-hoc build scripts when standard Cargo commands are sufficient.
- Keep behavior parity with teenycode for tool names, tool semantics, and CLI ergonomics.
- When changing file-tool behavior, update tests in `tests/tools_test.rs` and `tests/env_test.rs`.

## Review Checklist for Agents

- [ ] Run `cargo fmt`.
- [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo test`.
