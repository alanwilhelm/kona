use crate::api::{Message, ResponseStream};
use crate::config::Config;
use crate::utils::error::Result;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// Mock API client for testing
pub struct MockOpenRouterClient {
    pub config: Config,
    pub response: Arc<Mutex<String>>,
}

impl MockOpenRouterClient {
    pub fn new(config: Config, response: String) -> Self {
        Self {
            config,
            response: Arc::new(Mutex::new(response)),
        }
    }
    
    pub fn set_response(&self, response: String) {
        let mut r = self.response.lock().unwrap();
        *r = response;
    }
    
    pub async fn send_message(&self, _message: &str) -> Result<String> {
        let response = self.response.lock().unwrap().clone();
        Ok(response)
    }
    
    pub async fn send_message_with_history(&self, _messages: Vec<Message>) -> Result<String> {
        let response = self.response.lock().unwrap().clone();
        Ok(response)
    }
    
    pub async fn send_message_streaming(&self, _message: &str) -> Result<ResponseStream> {
        let response = self.response.lock().unwrap().clone();
        let (sender, receiver) = mpsc::channel(10);
        
        // Clone response for the spawned task
        let response_clone = response.clone();
        
        tokio::spawn(async move {
            // Split the response into chunks to simulate streaming
            // For simplicity, we'll split by spaces
            for word in response_clone.split_whitespace() {
                let _ = sender.send(Ok(word.to_string() + " ")).await;
                // Add a small delay to simulate streaming
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        });
        
        Ok(ResponseStream::new(receiver))
    }
    
    pub async fn send_message_streaming_with_history(&self, _messages: Vec<Message>) -> Result<ResponseStream> {
        self.send_message_streaming("").await
    }
}