# Contributing to Kona

Thank you for your interest in contributing to Kona! This document provides guidelines and instructions for contributing to this project.

## Development Setup

1. **Clone the repository**:
   ```
   git clone https://github.com/yourusername/kona.git
   cd kona
   ```

2. **Set up API Key**:
   Create a `.env` file in the project root with your OpenRouter API key:
   ```
   ANTHROPIC_API_KEY=your_openrouter_key_here
   ```
   Or use the provided make command:
   ```
   make setup-env
   ```

   Note: You'll need an OpenRouter API key to interact with Claude models. Sign up at [OpenRouter](https://openrouter.ai) to get one. While the environment variable is still named `ANTHROPIC_API_KEY` for backward compatibility, it should now contain an OpenRouter API key.

3. **Build the project**:
   ```
   cargo build
   ```

## Development Workflow

### Building

- Build in debug mode: `cargo build` or `make build`
- Build in release mode: `cargo build --release` or `make release`

### Testing

- Run all tests: `cargo test` or `make test`
- Run only unit tests: `make test-unit`
- Run integration tests: `make test-integration`

Note: Integration tests for the CLI are marked `#[ignore]` by default to avoid requiring an API key during automated testing. To run these tests:
```
cargo test -- --ignored
```

### Code Formatting and Linting

- Format code: `cargo fmt` or `make fmt`
- Lint code: `cargo clippy` or `make lint`

## Project Structure

- `src/` - Source code directory
  - `api/` - Code for the OpenRouter API client (for Claude models)
  - `cli/` - Command-line interface code
  - `config/` - Configuration management
  - `history/` - Conversation history storage
  - `utils/` - Utility functions and error handling
- `tests/` - Integration tests

## Pull Request Process

1. Fork the repository and create a new branch for your feature or bug fix.
2. Make your changes and ensure they include appropriate tests.
3. Ensure your code passes all tests and formatting checks.
4. Create a pull request with a clear description of the changes.
5. Address any feedback from the code review.

## Feature Roadmap

See the [PLAN.md](PLAN.md) file for details on upcoming features and development priorities.

## License

By contributing to this project, you agree that your contributions will be licensed under the project's license.

## Questions?

If you have any questions or need assistance, please open an issue in the repository.