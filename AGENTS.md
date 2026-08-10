# Agent Development Guide

A file for coding agents.

## Commands

- Building: `cargo build`
- Testing: `cargo test`
- Formatting: `cargo fmt`
- Linting: `cargo clippy --all-targets`

## Directory Structure

- `src/`: Source code and unit tests
- `tests/`: E2E tests exercising CLI

## Development Guidelines

1. Never commit code. Leave all changes unstaged.
2. Development isn't done until all code is formatted, compilation succeeds, all tests pass, and clippy surfaces no issues.
3. Abide by the following principle as much as possible without sacrificing clarity or correctness: make illegal state unrepresentable.
4. All code needs to be easily testable with automated unit tests.
5. Do not add comments unless they explain what the code cannot.
