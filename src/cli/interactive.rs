// Interactive Mode Implementation

use colored::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
// Unused but will be used when implementing streaming
#[allow(unused_imports)]
use std::io;
use tracing::{debug, error, info};

use crate::api::OpenRouterClient;
use crate::history::storage::{Conversation, ConversationStorage};
use crate::utils::error::{KonaError, Result};
use crate::utils::mask_api_key;

// Convert rustyline errors to our error type
impl From<ReadlineError> for KonaError {
    fn from(error: ReadlineError) -> Self {
        KonaError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Readline error: {}", error),
        ))
    }
}

// Main interactive mode function
pub async fn start_interactive_mode(client: OpenRouterClient) -> Result<()> {
    // For simplicity, use fallback mode for now
    // TODO: Implement conversation history when we've fixed the current issues
    fallback_interactive_mode(client).await
}

// Fallback mode without conversation history
async fn fallback_interactive_mode(mut client: OpenRouterClient) -> Result<()> {
    println!("{}", format!("🌴 {} v{}", "Kona", env!("CARGO_PKG_VERSION")).green().bold());
    println!("Enter your message (use {} for help, {} to exit)", "/help".blue(), "/exit".blue());
    println!("Press Enter to send, Shift+Enter for new line");
    println!();

    let history_file = match dirs::home_dir() {
        Some(mut path) => {
            path.push(".kona_history");
            Some(path)
        }
        None => None,
    };

    // Initialize rustyline with the simpler DefaultEditor
    let mut rl = DefaultEditor::new()?;
    
    // Set max history entries based on config
    let history_size = client.config.history_size;
    debug!("Setting history size to {}", history_size);

    // Load history if available
    if let Some(ref history_path) = history_file {
        match rl.load_history(history_path) {
            Ok(_) => debug!("Successfully loaded history"),
            Err(err) => debug!("No previous history: {}", err),
        }
        
        // Note: DefaultEditor doesn't have a history_len method
        // In a full implementation, we would manage history size differently
    }

    // Main REPL loop
    let mut conversation_history = Vec::new();

    loop {
        let prompt = format!("{} ", "You:".green().bold());
        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }

                // Add valid input to history
                rl.add_history_entry(line.clone())?;

                // Process commands
                if line.starts_with('/') {
                    match line.trim() {
                        "/help" => {
                            println!("\n{}", "Available commands:".yellow());
                            println!("  {} - Show this help", "/help".blue());
                            println!("  {} - Clear the conversation", "/clear".blue());
                            println!("  {} - Show current configuration", "/config".blue());
                            println!("  {} - Create default config file", "/init".blue());
                            println!("  {} - Change the current model", "/model [model_name]".blue());
                            println!("  {} - Toggle streaming mode", "/stream".blue());
                            println!("  {} - Exit Kona", "/exit".blue());
                            println!();
                            continue;
                        }
                        "/clear" => {
                            conversation_history.clear();
                            println!("\n{}\n", "Conversation cleared.".yellow());
                            continue;
                        }
                        "/config" => {
                            // Show configuration
                            println!("\n{}", "Current configuration:".yellow());
                            println!("  API Key: {}", mask_api_key(&client.config.api_key));
                            println!("  Model: {}", client.config.model);
                            println!("  Max Tokens: {}", client.config.max_tokens);
                            println!("  System Prompt: {:?}", client.config.system_prompt);
                            println!("  History Size: {}", client.config.history_size);
                            println!("  Streaming: {}", if client.config.use_streaming { "enabled".green() } else { "disabled".yellow() });

                            if let Some(path) = crate::config::Config::get_config_path() {
                                println!("\n  Config file: {:?}", path);
                                if path.exists() {
                                    println!("  Config file exists: Yes");
                                } else {
                                    println!("  Config file exists: No (using defaults)");
                                    println!("  Use {} to create a config file", "/init".blue());
                                }
                            }
                            println!();
                            continue;
                        }
                        "/init" => {
                            // Create default config
                            println!("\n{}", "Creating default config file...".yellow());
                            match crate::config::Config::create_default_config_file() {
                                Ok(path) => {
                                    println!("  Created default config file at: {:?}", path);
                                    println!("  Please edit this file to add your API key and other settings");
                                }
                                Err(err) => {
                                    println!("  {} {}", "Error:".red(), err);
                                }
                            }
                            println!();
                            continue;
                        }
                        s if s.starts_with("/model") => {
                            // Change model or show current model
                            let parts: Vec<&str> = s.split_whitespace().collect();
                            if parts.len() >= 2 {
                                // Change the model
                                let new_model = parts[1].to_string();
                                println!("\n{} {} -> {}", "Changing model:".yellow(), client.config.model.blue(), new_model.green());
                                client.config.model = new_model;
                            } else {
                                // Show current model
                                println!("\n{} {}", "Current model:".yellow(), client.config.model.green());
                                println!("To change models, use /model <model_name>");
                                println!("Supported Claude models via OpenRouter:");
                                println!("  - anthropic/claude-3-opus");
                                println!("  - anthropic/claude-3-sonnet");
                                println!("  - anthropic/claude-3-haiku");
                                println!("  - anthropic/claude-3.5-sonnet");
                                println!("  - anthropic/claude-3.5-haiku");
                            }
                            println!();
                            continue;
                        },
                        "/stream" => {
                            // Toggle streaming mode
                            client.config.use_streaming = !client.config.use_streaming;
                            let status = if client.config.use_streaming { "enabled" } else { "disabled" };
                            println!("\n{} {}\n", "Streaming mode:".yellow(), status.green());
                            continue;
                        }
                        "/exit" => {
                            println!("\n{}\n", "Goodbye!".green());
                            break;
                        }
                        _ => {
                            println!("\n{} {}\n", "Unknown command:".red(), line);
                            continue;
                        }
                    }
                }

                // Store user message
                conversation_history.push(line.clone());

                // Send message to API
                println!("\n{} ", "Claude:".purple().bold());
                
                // Use streaming or non-streaming based on config
                if client.config.use_streaming {
                    // Use the streaming API
                    use futures::StreamExt;
                    use std::io::{self, Write};

                    match client.send_message_streaming(&line).await {
                        Ok(mut stream) => {
                            let mut full_response = String::new();

                            // Process the stream
                            while let Some(chunk_result) = stream.next().await {
                                match chunk_result {
                                    Ok(chunk) => {
                                        print!("{}", chunk);
                                        io::stdout().flush().ok(); // Ensure text appears immediately
                                        full_response.push_str(&chunk);
                                    }
                                    Err(err) => {
                                        error!("Stream error: {}", err);
                                        println!("\n{}: {}", "Error".red().bold(), err);
                                        break;
                                    }
                                }
                            }

                            println!("\n"); // Add newline after response
                            conversation_history.push(full_response);
                        }
                        Err(err) => {
                            error!("API error: {}", err);
                            println!("{}: {}\n", "Error".red().bold(), err);
                        }
                    }
                } else {
                    // Standard non-streaming mode
                    match client.send_message(&line).await {
                        Ok(response) => {
                            println!("{}\n", response);
                            conversation_history.push(response);
                        }
                        Err(err) => {
                            error!("API error: {}", err);
                            println!("{}: {}\n", "Error".red().bold(), err);
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C
                println!("\n{}\n", "Interrupted. Goodbye!".green());
                break;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D
                println!("\n{}\n", "End of input. Goodbye!".green());
                break;
            }
            Err(err) => {
                error!("Readline error: {}", err);
                println!("{}: {}\n", "Error".red().bold(), err);
                break;
            }
        }
    }

    // Save history
    if let Some(ref history_path) = history_file {
        match rl.save_history(history_path) {
            Ok(_) => debug!("Successfully saved history"),
            Err(err) => error!("Error saving history: {}", err),
        }
    }

    info!("Interactive mode exited");
    Ok(())
}

// Interactive mode with conversation history - TODO for future implementation
#[allow(dead_code)]
async fn interactive_mode_with_history(
    _client: OpenRouterClient,
    _storage: &mut ConversationStorage,
    _conversation: &mut Conversation,
) -> Result<()> {
    // To be implemented
    Ok(())
}