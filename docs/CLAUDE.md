# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Kona is a Rust project that aims to build a Claude Code clone - a command-line interface for interacting with the Anthropic API. The project is in its early stages of development, with the main functionality to be implemented in phases as outlined in PLAN.md.

## Development Commands

### Build and Run

```bash
# Build the project
cargo build

# Run the project
cargo run

# Build with optimizations for release
cargo build --release

# Run tests
cargo test

# Check for errors without building
cargo check
```

## Project Architecture

Kona is organized as a Rust CLI application with the following planned components:

1. **CLI Framework**: Using clap for argument parsing and command-line interface
2. **API Client**: For communication with Anthropic's Claude API
3. **Terminal UI**:
   - REPL interface for interactive mode
   - Rich terminal UI using Ratatui (planned for later phases)
4. **Tool Framework**: For executing functions like file operations and shell commands
5. **Storage System**: For conversation history and configuration management

The development is planned in phases:
1. Project setup and basic CLI
2. API communication with Anthropic
3. Core features (interactive mode, file operations, bash execution)
4. Advanced features (tool orchestration, enhanced UI)
5. Testing and deployment

## Configuration

The project will use:
- Environment variables for API keys (ANTHROPIC_API_KEY)
- Configuration files for persistent settings
- Command-line arguments for runtime options

## Planned Key Dependencies

- clap: CLI argument parsing
- tokio: Async runtime
- reqwest: HTTP requests
- serde: JSON serialization
- rustyline: REPL interface
- Ratatui: Terminal UI (for advanced phases)
- tracing: Logging infrastructure