use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::utils::error::{KonaError, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub system_prompt: Option<String>,
    pub history_size: usize,
    pub use_streaming: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "anthropic/claude-3-sonnet".to_string(),
            max_tokens: 1024,
            system_prompt: Some("You are Claude, an AI assistant by Anthropic. You are helping the user via the Kona CLI interface.".to_string()),
            history_size: 100,
            use_streaming: true,  // Enable streaming by default for a better experience
        }
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        let mut config = Config::default();

        // Try to load from config file first
        if let Some(config_from_file) = Self::load_from_file() {
            debug!("Loaded configuration from file");
            config = config_from_file;
        } else {
            debug!("No config file found or error reading it, using default config");
        }

        // Environment variables override config file settings
        Self::apply_env_overrides(&mut config)?;

        // API key is required
        if config.api_key.trim().is_empty() {
            return Err(KonaError::ConfigError(
                "API key is required. Set it in the config file or with KONA_OPENROUTER_API_KEY environment variable.".to_string(),
            ));
        }

        // Validate API key
        if config.api_key == "your_api_key_here" ||
           (config.api_key.starts_with("sk-ant-api") && config.api_key.contains("not-a-real-key")) {
            return Err(KonaError::ConfigError(
                "Invalid API key. Please set a valid API key in the config file or as an environment variable.".to_string(),
            ));
        }

        Ok(config)
    }

    // Load configuration from a TOML file
    fn load_from_file() -> Option<Self> {
        let config_path = Self::get_config_path()?;
        debug!("Looking for config file at: {:?}", config_path);

        match fs::read_to_string(&config_path) {
            Ok(content) => {
                match toml::from_str::<Config>(&content) {
                    Ok(config) => Some(config),
                    Err(e) => {
                        debug!("Error parsing config file: {}", e);
                        None
                    }
                }
            },
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    debug!("Error reading config file: {}", e);
                }
                None
            }
        }
    }

    // Get the path to the configuration file
    pub fn get_config_path() -> Option<PathBuf> {
        if let Some(mut config_dir) = dirs::config_dir() {
            config_dir.push("kona");
            fs::create_dir_all(&config_dir).ok()?;
            config_dir.push("config.toml");
            Some(config_dir)
        } else {
            None
        }
    }

    // Apply environment variable overrides to the configuration
    fn apply_env_overrides(config: &mut Self) -> Result<()> {
        // API key from environment (highest priority)
        // First try KONA_OPENROUTER_API_KEY (preferred)
        let api_key = env::var("KONA_OPENROUTER_API_KEY").ok()
            // Then try KONA_API_KEY as second option
            .or_else(|| env::var("KONA_API_KEY").ok())
            // Then try OPENROUTER_API_KEY as fallback for backward compatibility
            .or_else(|| env::var("OPENROUTER_API_KEY").ok());

        if let Some(api_key) = api_key {
            // Clean the API key to remove any whitespace
            let cleaned_api_key = api_key.trim().to_string();
            config.api_key = cleaned_api_key;
        }

        // Model override
        if let Ok(model) = env::var("KONA_MODEL") {
            config.model = model;
        }

        // Max tokens override
        if let Ok(max_tokens_str) = env::var("KONA_MAX_TOKENS") {
            if let Ok(max_tokens) = max_tokens_str.parse::<u32>() {
                config.max_tokens = max_tokens;
            } else {
                debug!("Invalid KONA_MAX_TOKENS value: {}", max_tokens_str);
            }
        }

        // System prompt override
        if let Ok(system_prompt) = env::var("KONA_SYSTEM_PROMPT") {
            config.system_prompt = Some(system_prompt);
        }

        // History size override
        if let Ok(history_size_str) = env::var("KONA_HISTORY_SIZE") {
            if let Ok(history_size) = history_size_str.parse::<usize>() {
                config.history_size = history_size;
            } else {
                debug!("Invalid KONA_HISTORY_SIZE value: {}", history_size_str);
            }
        }

        // Streaming override
        if let Ok(streaming_str) = env::var("KONA_USE_STREAMING") {
            config.use_streaming = streaming_str.to_lowercase() == "true" ||
                                  streaming_str == "1" ||
                                  streaming_str.to_lowercase() == "yes";
        }

        Ok(())
    }

    // Create a default config file if it doesn't exist
    pub fn create_default_config_file() -> Result<PathBuf> {
        let config_path = Self::get_config_path()
            .ok_or_else(|| KonaError::ConfigError("Could not determine config directory".to_string()))?;

        // Check if file already exists
        if config_path.exists() {
            return Ok(config_path);
        }

        // Create a default config
        let default_config = Config::default();

        // Serialize to TOML
        let toml_content = toml::to_string_pretty(&default_config)
            .map_err(|e| KonaError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        // Write to file
        fs::write(&config_path, toml_content)
            .map_err(|e| KonaError::ConfigError(format!("Failed to write config file: {}", e)))?;

        info!("Created default config file at: {:?}", config_path);

        Ok(config_path)
    }
}