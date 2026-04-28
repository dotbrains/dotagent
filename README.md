# DotAgent

[![CI](https://github.com/dotbrains/dotagent/actions/workflows/ci.yml/badge.svg)](https://github.com/dotbrains/dotagent/actions/workflows/ci.yml)
[![Release](https://github.com/dotbrains/dotagent/actions/workflows/release.yml/badge.svg)](https://github.com/dotbrains/dotagent/actions/workflows/release.yml)
[![License: PolyForm Shield 1.0.0](https://img.shields.io/badge/License-PolyForm%20Shield%201.0.0-blue.svg)](https://polyformproject.org/licenses/shield/1.0.0/)

![Rust](https://img.shields.io/badge/-Rust-000000?style=flat-square&logo=rust&logoColor=white)
![OpenAI](https://img.shields.io/badge/-OpenAI-412991?style=flat-square&logo=openai&logoColor=white)
![macOS](https://img.shields.io/badge/-macOS-000000?style=flat-square&logo=apple&logoColor=white)
![Linux](https://img.shields.io/badge/-Linux-FCC624?style=flat-square&logo=linux&logoColor=black)

A tiny code-editing agent in Rust — a minimal, hackable CLI that talks to OpenAI and can read, list, and edit files in your working directory.

This repository is for **educational purposes**, to demonstrate the core elements of a minimal CLI coding agent, adapted from [How to Build an Agent](https://ampcode.com/notes/how-to-build-an-agent) by Amp.

## Features

- Chat-based CLI that calls OpenAI and uses tool calls
- File tools: `read_file`, `list_files`, `edit_file` (single, unique replacement or create new file)
- Prefers reading files over guessing; makes the smallest edit that satisfies a request
- Minimal dependencies and simple code you can tweak

> [!NOTE]
> No `bash` tool is included because it can be risky and this project is intended for educational use.
> You can add it yourself if needed.

## Requirements

- Rust toolchain (`cargo`, `rustc`)
- An OpenAI API key

## Quickstart

```sh
export OPENAI_API_KEY=sk-...
cargo run
```

Or put `OPENAI_API_KEY=sk-...` in a `.env` file in the directory where you run the binary.

If `OPENAI_API_KEY` is missing, the CLI prints setup instructions and exits.

## How it works (quick tour)

- `src/main.rs`: Entry point; checks `OPENAI_API_KEY`, starts the agent
- `src/agent.rs`: Chat loop, tool routing, and message state
- `src/tools.rs`: Three built-in tools implemented with Rust filesystem APIs
- `src/env.rs`: Loads `.env` from the current working directory

## Tools

The agent exposes three file tools:

- `read_file` (`path: string`): Reads and returns the text contents of a file at a relative path.
- `list_files` (`path?: string | null`): Lists files and directories at the given path (or current directory).
- `edit_file` (`path: string, old_str: string, new_str: string`): Replaces exactly one occurrence of `old_str` with `new_str` in the given file.

Conventions:

- All paths are relative to your current working directory.
- The agent prefers reading over guessing and aims to make the smallest possible change.

## Quality checks

```sh
cargo fmt
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## Pre-commit hooks

Install the repository hooks:

```sh
./scripts/setup-hooks.sh
```

This enables a `pre-commit` hook that runs:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --all-targets`

## Acknowledgements

Adapted from [How to Build an Agent](https://ampcode.com/notes/how-to-build-an-agent) by Amp.

## License

This project is licensed under the [PolyForm Shield License 1.0.0](https://polyformproject.org/licenses/shield/1.0.0/) — see [LICENSE](LICENSE) for details.
