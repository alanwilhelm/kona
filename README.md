# Kona

Kona is a command-line interface for interacting with Claude AI models through OpenRouter's API. It aims to provide an experience similar to Claude Code but in a local CLI environment.

## Features

- Simple CLI for asking questions to Claude
- Interactive mode with command history and slash commands
- File operations (coming soon)
- Bash command execution (coming soon)

## Installation

### Prerequisites

- Rust and Cargo installed
- An OpenRouter API key (required)
  - Sign up at [OpenRouter](https://openrouter.ai) to get an API key
  - OpenRouter provides access to Claude and other models

### Building from Source

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/kona.git
   cd kona
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Run the executable:
   ```
   ./target/release/kona --help
   ```

## Configuration

### Using the Initialize Command

The easiest way to configure Kona is using the init command:

```
kona init
```

This will create a default configuration file that you can edit with your OpenRouter API key.

### Manual Configuration Options

1. **Environment Variables**:
   Create a `.env` file in the project root with your OpenRouter API key:

   ```
   # Both environment variables are supported, but OPENROUTER_API_KEY is preferred
   OPENROUTER_API_KEY=your_openrouter_api_key_here
   # Or for backward compatibility
   ANTHROPIC_API_KEY=your_openrouter_api_key_here
   ```

2. **Configuration File**:
   Kona looks for a configuration file at `~/.config/kona/config.toml` (macOS/Linux) or
   `%APPDATA%\kona\config.toml` (Windows).

   Example configuration:
   ```toml
   api_key = "your_openrouter_api_key_here"
   model = "claude-3-sonnet-20240229"
   max_tokens = 1024
   system_prompt = "You are Claude, an AI assistant by Anthropic. You are helping the user via the Kona CLI interface."
   history_size = 100
   use_streaming = true
   ```

3. **Model Configuration**:
   - The default model is `claude-3-sonnet-20240229`
   - You can specify a different model using the `KONA_MODEL` environment variable
   - All Claude models are accessible via OpenRouter

## Usage

### Ask a Question (Non-Interactive Mode)

```
kona ask "What is the capital of France?"
```

### Interactive Mode

Start the interactive REPL mode:

```
kona
```

In interactive mode, you can:
- Enter questions to send to Claude
- Use slash commands:
  - `/help` - Show available commands
  - `/clear` - Clear the conversation history
  - `/exit` - Exit the program
  - `/model` - Show or change the current model
  - `/config` - Show current configuration
  - `/streaming` - Toggle streaming mode on/off

Command history is saved between sessions, and you can navigate it with the up/down arrow keys.

### Verbosity

You can increase the logging verbosity with the `-v` flag:

```
kona -v      # Warning level logging
kona -vv     # Info level logging
kona -vvv    # Debug level logging
kona -vvvv   # Trace level logging
```

## Development

Follow the development plan in PLAN.md to contribute to the project.

## License

This project is open source and available under the [MIT License](LICENSE).