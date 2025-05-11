use clap::Parser;
use dotenv::dotenv;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

mod cli;
mod api;
mod config;
mod utils;
mod history;

use api::OpenRouterClient;
use utils::mask_api_key;
use cli::cli::{Cli, Commands};
use cli::interactive;
use cli::tui;
// Will be used later
// use history::storage::ConversationStorage;
use config::Config;

fn setup_logging(verbosity: u8) {
    let level = match verbosity {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file if present
    match dotenv() {
        Ok(_) => info!("Loaded environment variables from .env file"),
        Err(e) => info!("No .env file found or error loading it: {}", e),
    };

    // Parse command line arguments
    let cli = Cli::parse();

    // Setup logging based on verbosity flag
    setup_logging(cli.verbose);

    info!("Starting Kona v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let mut config = match Config::new() {
        Ok(config) => config,
        Err(err) => {
            error!("Failed to load configuration: {}", err);
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };

    // Display API key if in debug mode
    if cli.debug {
        println!("Debug mode enabled");
        println!("API Key: {}", config.api_key);
        println!("API Key length: {}", config.api_key.len());
        println!("API Key (masked): {}", mask_api_key(&config.api_key));
        println!("Model: {}", config.model);
    } else {
        // Always show a masked version in normal logging
        info!("Using API Key: {}", mask_api_key(&config.api_key));
        info!("API Key length: {}", config.api_key.len());
        info!("Using Model: {}", config.model);
    }

    // Override streaming based on command line flags
    // --no-streaming takes precedence over --streaming
    if cli.no_streaming {
        config.use_streaming = false;
        info!("Streaming disabled via command line flag");
    } else if !cli.streaming {
        config.use_streaming = false;
        info!("Streaming disabled via command line flag");
    }

    // Create API client
    // Clone the config for the client
    let config_for_client = config.clone();

    let client = match OpenRouterClient::new(config_for_client) {
        Ok(client) => client,
        Err(err) => {
            error!("Failed to create API client: {}", err);
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };

    // Process commands
    match cli.command {
        Some(Commands::Ask { query }) => {
            println!("Asking Claude: {}", query);

            // Use streaming if enabled in config
            if config.use_streaming {
                use futures::StreamExt;
                use std::io::{self, Write};

                match client.send_message_streaming(&query).await {
                    Ok(mut stream) => {
                        println!("\nClaude:");

                        // Process the stream
                        while let Some(chunk_result) = stream.next().await {
                            match chunk_result {
                                Ok(chunk) => {
                                    print!("{}", chunk);
                                    io::stdout().flush().ok(); // Ensure text appears immediately
                                }
                                Err(err) => {
                                    error!("Stream error: {}", err);
                                    eprintln!("\nError: {}", err);
                                    std::process::exit(1);
                                }
                            }
                        }

                        println!("\n"); // Add newline after response
                    }
                    Err(err) => {
                        error!("API call failed: {}", err);
                        eprintln!("Error: {}", err);
                        std::process::exit(1);
                    }
                }
            } else {
                // Use non-streaming API
                match client.send_message(&query).await {
                    Ok(response) => {
                        println!("\nClaude: {}", response);
                    }
                    Err(err) => {
                        error!("API call failed: {}", err);
                        eprintln!("Error: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        },
        Some(Commands::Init { force }) => {
            // Handle initialization without creating the API client
            match Config::get_config_path() {
                Some(path) => {
                    if path.exists() && !force {
                        println!("Config file already exists at: {:?}", path);
                        println!("Use --force to overwrite existing config");
                        return;
                    }

                    match Config::create_default_config_file() {
                        Ok(path) => {
                            println!("Created default config file at: {:?}", path);
                            println!("Please edit this file to add your API key and other settings");
                        },
                        Err(err) => {
                            error!("Failed to create config file: {}", err);
                            eprintln!("Error: {}", err);
                            std::process::exit(1);
                        }
                    }
                },
                None => {
                    error!("Could not determine config directory");
                    eprintln!("Error: Could not determine config directory");
                    std::process::exit(1);
                }
            }
        },
        Some(Commands::Config) => {
            // Show current configuration
            println!("Current configuration:");
            println!("API Key: {}", mask_api_key(&config.api_key));
            println!("Model: {}", config.model);
            println!("Max Tokens: {}", config.max_tokens);
            println!("System Prompt: {:?}", config.system_prompt);
            println!("History Size: {}", config.history_size);
            println!("Streaming: {}", if config.use_streaming { "enabled" } else { "disabled" });

            // Show config file location
            if let Some(path) = Config::get_config_path() {
                println!("\nConfig file location: {:?}", path);
                if path.exists() {
                    println!("Config file exists: Yes");
                } else {
                    println!("Config file exists: No (using defaults)");
                }
            } else {
                println!("\nConfig file location: Could not determine");
            }
        },
        None => {
            // No subcommand was used, run TUI or interactive mode
            info!("Starting interactive mode with TUI");

            // Check if config file exists, suggest creating one if not
            if let Some(path) = Config::get_config_path() {
                if !path.exists() {
                    println!("No config file found at: {:?}", path);
                    println!("Using environment variables and defaults");
                    println!("Type /help for more information\n");
                }
            }

            // Try to use the TUI mode first, fall back to simple interactive mode if it fails
            match tui::start_tui_mode(client.clone()).await {
                Ok(_) => {
                    info!("TUI mode exited successfully");
                }
                Err(err) => {
                    // Check the error type/message to provide better feedback
                    let err_message = format!("{}", err);

                    // If it's a terminal compatibility error, show a more user-friendly message
                    if err_message.contains("Terminal environment not compatible") ||
                       err_message.contains("Device not configured") ||
                       err_message.contains("Unsupported") {
                        info!("Terminal doesn't support TUI features");
                        println!("Your terminal doesn't support advanced UI features.");
                    } else {
                        // Generic error for other issues
                        error!("Failed to start TUI mode: {}", err);
                    }

                    println!("Falling back to basic interactive mode...");

                    if let Err(err) = interactive::start_interactive_mode(client).await {
                        error!("Interactive mode error: {}", err);
                        eprintln!("Error: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
