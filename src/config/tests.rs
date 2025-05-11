#[cfg(test)]
mod tests {
    use super::Config;
    use std::env;
    
    fn setup() {
        env::remove_var("ANTHROPIC_API_KEY");
        env::remove_var("KONA_MODEL");
        env::remove_var("KONA_MAX_TOKENS");
        env::remove_var("KONA_SYSTEM_PROMPT");
        env::remove_var("KONA_HISTORY_SIZE");
        env::remove_var("KONA_USE_STREAMING");
    }
    
    #[test]
    fn test_config_defaults() {
        setup();
        
        // Set API key to avoid error
        env::set_var("ANTHROPIC_API_KEY", "sk-ant-api-test-key-123456789");
        
        let config = Config::new().unwrap();
        
        assert_eq!(config.api_key, "sk-ant-api-test-key-123456789");
        assert_eq!(config.model, "claude-3-sonnet-20240229");
        assert_eq!(config.max_tokens, 1024);
        assert_eq!(config.system_prompt, Some("You are Claude, an AI assistant by Anthropic. You are helping the user via the Kona CLI interface.".to_string()));
        assert_eq!(config.history_size, 100);
        assert_eq!(config.use_streaming, true);
    }
    
    #[test]
    fn test_config_env_override() {
        setup();
        
        env::set_var("ANTHROPIC_API_KEY", "sk-ant-api-custom-key");
        env::set_var("KONA_MODEL", "claude-3-opus-20240229");
        env::set_var("KONA_MAX_TOKENS", "2048");
        env::set_var("KONA_SYSTEM_PROMPT", "Custom system prompt");
        env::set_var("KONA_HISTORY_SIZE", "50");
        env::set_var("KONA_USE_STREAMING", "false");
        
        let config = Config::new().unwrap();
        
        assert_eq!(config.api_key, "sk-ant-api-custom-key");
        assert_eq!(config.model, "claude-3-opus-20240229");
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(config.system_prompt, Some("Custom system prompt".to_string()));
        assert_eq!(config.history_size, 50);
        assert_eq!(config.use_streaming, false);
    }
    
    #[test]
    fn test_config_invalid_api_key() {
        setup();
        
        // No API key
        let result = Config::new();
        assert!(result.is_err());
        
        // Empty API key
        env::set_var("ANTHROPIC_API_KEY", "");
        let result = Config::new();
        assert!(result.is_err());
        
        // Template API key
        env::set_var("ANTHROPIC_API_KEY", "your_api_key_here");
        let result = Config::new();
        assert!(result.is_err());
        
        // Invalid test key 
        env::set_var("ANTHROPIC_API_KEY", "sk-ant-api-not-a-real-key");
        let result = Config::new();
        assert!(result.is_err());
    }
}