// Special Mac-friendly interactive mode

use colored::*;
use std::io::{self, Write};
use std::process::Command;
use tracing::{debug, error, info};

use crate::api::OpenRouterClient;
use crate::utils::error::Result;
use crate::utils::mask_api_key;

// Main interactive mode function for Mac
pub async fn start_mac_mode(mut client: OpenRouterClient) -> Result<()> {
    println!("{}", format!("🌴 {} v{}", "Kona", env!("CARGO_PKG_VERSION")).green().bold());
    println!("Mac-friendly interactive mode");
    println!("Type a message and press Return to send");
    println!("Type /exit to quit, /help for more commands\n");

    // Keep track of conversation for history
    let mut conversation_history = Vec::new();
    
    loop {
        // Prompt for input
        print!("{} ", "You:".green().bold());
        io::stdout().flush()?;
        
        // Use osascript to get input in a Mac-friendly way
        let input = get_mac_input()?;
        
        // Check if we got empty input - retry
        if input.is_empty() {
            continue;
        }
        
        let trimmed_input = input.trim();
        debug!("Received input: '{}'", trimmed_input);
        
        // Process commands
        if trimmed_input.starts_with('/') {
            let command = trimmed_input.split_whitespace().next().unwrap_or(trimmed_input);
            debug!("Processing command: {}", command);
            
            match command {
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
                "/model" => {
                    // Change model or show current model
                    let parts: Vec<&str> = trimmed_input.split_whitespace().collect();
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
                    println!("\n{} {}\n", "Unknown command:".red(), trimmed_input);
                    continue;
                }
            }
        }

        // Regular message - store in history
        conversation_history.push(input.clone());
        
        // Send message to API
        println!("\n{} ", "Claude:".purple().bold());
        
        // Use streaming or non-streaming based on config
        if client.config.use_streaming {
            // Use the streaming API
            use futures::StreamExt;
            
            match client.send_message_streaming(trimmed_input).await {
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
            match client.send_message(trimmed_input).await {
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

    info!("Mac interactive mode exited");
    Ok(())
}

// Function to get input from the user using Mac's osascript
fn get_mac_input() -> Result<String> {
    let script = r#"
    set theResponse to display dialog "Enter your message:" default answer "" buttons {"Send"} default button "Send"
    return text returned of theResponse
    "#;
    
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| crate::utils::error::KonaError::IoError(e))?;
    
    let input = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(input.trim().to_string())
}