.PHONY: build test test-unit test-integration run clean doc

# Build the application
build:
	cargo build

# Build with optimizations
release:
	cargo build --release

# Run all tests
test:
	cargo test

# Run unit tests only
test-unit:
	cargo test --lib

# Run integration tests
test-integration:
	cargo test --test '*'

# Run the application
run:
	cargo run

# Clean build artifacts
clean:
	cargo clean

# Generate documentation
doc:
	cargo doc --no-deps --open

# Check code for errors
check:
	cargo check

# Format code
fmt:
	cargo fmt

# Run lints
lint:
	cargo clippy

# Set up a .env file template
setup-env:
	@if [ ! -f .env ]; then \
		echo "Creating .env file template"; \
		echo "# Get your API key from https://openrouter.ai" > .env; \
		echo "ANTHROPIC_API_KEY=your_openrouter_api_key_here" >> .env; \
		echo "# KONA_MODEL=claude-3-sonnet-20240229" >> .env; \
		echo "# KONA_MAX_TOKENS=1024" >> .env; \
		echo "# KONA_USE_STREAMING=true" >> .env; \
		echo ".env file created. Please edit it with your OpenRouter API key."; \
	else \
		echo ".env file already exists."; \
	fi

# Default target
default: build