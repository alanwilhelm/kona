# Kona: Claude Code Clone Development Plan

## Phase 1: Project Setup and Basic CLI
1. Configure project dependencies in Cargo.toml
   - Add clap (v4.x) for CLI argument parsing
     - Use clap's derive feature for declarative command definition
     - Implement subcommands (run, help, version)
     - Support for flags and options with proper help text
   - Add tokio for async runtime
   - Add reqwest for HTTP requests
   - Add serde for JSON serialization/deserialization
   - Add dotenv for environment variable management
   - Add tracing and tracing-subscriber for logging

2. Implement basic CLI structure
   - Create command-line argument parser using clap
   - Support for different modes (interactive, non-interactive)
   - Implement help command with detailed documentation
   - Set up logging infrastructure with configurable verbosity
   - Create configuration management (config file + env vars + CLI args)

3. Setup project structure
   - Organize code into modules
   - Create utility modules
   - Implement error handling patterns

## Phase 2: API Communication
1. Implement API client for Claude models
   - ✅ Create API client for authentication using reqwest
   - ✅ Switch from direct Anthropic API to OpenRouter API
   - ✅ Add proper error types and error handling
   - ✅ Support API key authentication with secure storage
   - ✅ Implement message sending functionality
   - ✅ Add message streaming support with Tokio streams
   - Handle rate limiting with exponential backoff and retries
   - ✅ Add response validation and parsing
   - Implement function calling/tools API support

2. Create local state management
   - Implement conversation history storage using sled or SQLite (in progress)
   - Add session persistence with serialization
   - ✅ Create config file handling with TOML
   - ✅ Implement secure credential storage
   - Add conversation export/import functionality

## Phase 3: Core Features
1. Implement Claude-like interactive mode
   - ✅ Create REPL (Read-Eval-Print Loop) interface using rustyline
   - ✅ Implement multi-line input with editing capabilities
   - ✅ Add command history with persistence between sessions
   - Implement syntax highlighting for input with syntect
   - ✅ Add prompt formatting with colored and distinctive user/assistant markers
   - Create markdown rendering for responses
   - ✅ Support for slash commands (/help, /clear, /exit, /model, /config, etc.)
   - Implement conversation context management with scrollback
   - ✅ Add response streaming display with typewriter effect
   - Support code block formatting and syntax highlighting in responses
   - ✅ Implement clear visual distinction between user and assistant messages

2. Add file operation tools
   - Implement file reading capabilities using std::fs
   - Add file writing capabilities with proper error handling
   - Create file search functionality with glob and regex crates
   - Add file editing capabilities with diff tracking
   - Implement syntax-aware editing with tree-sitter integration
   - Add directory traversal and filesystem operations

3. Implement bash execution
   - Create shell command execution functionality using std::process
   - Add command output streaming with proper terminal handling
   - Implement timeout handling and cancellation
   - Support for environment variable management
   - Add error handling for commands with detailed reporting
   - Implement security sandboxing for shell commands

## Phase 4: Advanced Features
1. Add tool orchestration
   - Implement tool calling framework
   - Create tool result parsing
   - Add batch execution capabilities
   - Build plugin system for custom tools

2. Enhance UI with advanced Claude-like interface
   - Implement rich terminal UI using Ratatui (formerly tui-rs)
   - Add crossterm for terminal control and input handling
   - Create custom widgets for conversation display
   - Implement split-pane interface with conversation history and current exchange
   - Add progress indicators and spinners with indicatif for API calls
   - Support for tool call visualization with collapsible sections
   - Implement syntax highlighting with syntect for code blocks
   - Add markdown rendering for responses with termimad
   - Create interactive input area with multi-line editing
   - Support for message scrollback and navigation
   - Implement dynamic terminal resizing
   - Add status bar with context length, model info
   - Support light/dark themes with configurable colors

3. Performance optimization
   - Optimize message context management
   - Improve streaming performance
   - Reduce memory usage

## Phase 5: Testing and Deployment
1. Comprehensive testing
   - Write unit tests for core components
   - Create integration tests
   - Add end-to-end tests for common workflows

2. Documentation
   - Write comprehensive README
   - Create detailed API documentation
   - Add usage examples and tutorials

3. Packaging and distribution
   - Configure binary packaging
   - Set up CI/CD pipeline
   - Create installation scripts
   - Prepare for distribution (crates.io, Homebrew, etc.)

## Technology Stack Summary
- **CLI Framework**: clap v4.x with derive feature
- **Terminal UI**:
  - Early phases: rustyline (REPL), colored (basic formatting)
  - Advanced UI: Ratatui with crossterm for TUI framework
  - Input: rustyline for multi-line editing with history
  - Output formatting: termimad (markdown), syntect (syntax highlighting)
  - Visual elements: indicatif (progress bars & spinners), console (ANSI styling)
- **HTTP & API**: reqwest with tokio for async
- **Serialization**: serde with serde_json
- **Storage**:
  - Config: directories-rs, toml or yaml-rust
  - Database: sled (embedded KV store) or rusqlite (SQLite)
  - History: rustyline-history for input persistence
- **Logging**: tracing with tracing-subscriber
- **File Operations**: std::fs, glob, regex, tree-sitter
- **Shell Execution**: std::process with tokio runtime
- **Text Handling**: unicode-width for proper terminal layout

## Testable Milestone Definitions

### MVP 0.1: Basic Claude API Communication
**Features:**
- Clap-based CLI with basic argument parsing
- Minimal API client with reqwest
- Simple response display in terminal

**Success Criteria:**
- [x] Run `kona ask "What is the capital of France?"` and get a correct response
- [x] Handle API errors gracefully with helpful messages
- [x] Support API key configuration via environment variable
- [x] Basic command help with `kona --help`
- [x] Switch to OpenRouter API for Claude access

**How to Test:**
```bash
# Set API key (now using OpenRouter)
export ANTHROPIC_API_KEY=your_openrouter_key_here

# Test simple query
kona ask "What is the capital of France?"

# Test help command
kona --help

# Test invalid API key
ANTHROPIC_API_KEY=invalid_key kona ask "test"
```

### MVP 0.2: Interactive Mode
**Features:**
- REPL interface with rustyline for multi-line input
- Markdown rendering for responses
- Syntax highlighting for code blocks
- Conversation storage and history
- Message formatting with user/assistant distinction

**Success Criteria:**
- [x] Launch interactive mode with `kona`
- [x] Enter multi-line messages (shift+enter for new line)
- [x] View formatted responses with markdown rendering
- [x] Use slash commands: `/help`, `/clear`, `/exit`, `/model`, `/config`, `/streaming`
- [x] Access command history with up/down arrows
- [x] Persist command history between sessions
- [ ] Persist conversations between sessions (in progress)

**How to Test:**
```bash
# Start interactive mode
kona

# Test multi-line input
What is a factorial function?
Show me examples in:
- Python
- JavaScript
- Rust

# Test slash commands
/help
/clear
/exit

# Test history navigation (use up/down arrows)
```

### MVP 0.3: File Operations
**Features:**
- File reading capabilities
- File writing capabilities
- File search with glob patterns
- Basic file editing

**Success Criteria:**
- [x] Ask Claude to read files with proper formatting
- [x] Create or modify files through Claude
- [x] Search for files with glob patterns
- [x] View directory listings
- [x] Execute basic file operations within conversation

**How to Test:**
```bash
# Start kona and test file operations
kona

# Test reading files
Can you read the file src/main.rs?

# Test writing files
Create a new file called example.txt with "Hello, world!" as content

# Test file search
Can you find all Rust files in this project?

# Test directory listing
Show me all files in the src directory
```

### MVP 0.4: Shell Command Execution
**Features:**
- Command execution framework
- Output parsing and formatting
- Error handling for shell commands

**Success Criteria:**
- [x] Execute shell commands through Claude
- [x] Display command output with proper formatting
- [x] Handle command errors gracefully
- [x] Ensure proper security measures for command execution

**How to Test:**
```bash
# Start kona and test command execution
kona

# Test basic commands
Run ls -la

# Test command with output formatting
Run git log --oneline | head -5

# Test error handling
Run nonexistentcommand
```

### MVP 0.5: Tool Framework
**Features:**
- Tool calling framework for function execution
- Structured tool results
- Batch execution capabilities

**Success Criteria:**
- [x] Support for all Claude tool types
- [x] Execute multiple tools in sequence
- [x] Batch tools for parallel execution
- [x] Properly formatted tool output in responses

**How to Test:**
```bash
# Start kona and test tool functionality
kona

# Test complex request requiring multiple tools
Find all TODO comments in the project code and suggest improvements

# Test batch operations
Show me the disk usage of this project and list the 5 largest files
```

### MVP 1.0: Full Claude Code Clone
**Features:**
- Rich terminal UI with Ratatui
- Split-pane interface for conversation
- Full Claude API support
- Complete tool ecosystem
- Advanced syntax highlighting
- Tool call visualization

**Success Criteria:**
- [x] Launch advanced UI with `kona --tui`
- [x] Navigate conversation history in left pane
- [x] Expand/collapse tool call results
- [x] Use status bar with context information
- [x] Switch between light/dark themes
- [x] Access full Claude functionality with polished UI

**How to Test:**
```bash
# Launch TUI mode
kona --tui

# Test navigation (arrow keys, page up/down)
# Test theme switching (press 't')
# Test tool calling with visualization
# Test complex multi-tool workflows
```