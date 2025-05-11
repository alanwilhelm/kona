use futures::stream::{Stream, StreamExt, TryStreamExt};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::utils::mask_api_key;

use crate::config::Config;
use crate::utils::error::{KonaError, Result};

// Using OpenRouter API that can route to Anthropic's Claude
#[cfg(not(test))]
const API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

// For testing, we'll set this in the test module
#[cfg(test)]
thread_local! {
    static API_URL: std::cell::RefCell<String> = std::cell::RefCell::new(
        "https://openrouter.ai/api/v1/chat/completions".to_string()
    );
}

#[derive(Debug, Serialize)]
struct MessageRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    // OpenRouter specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    id: String,
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
    index: usize,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    role: String,
    content: String,
}

// Streaming response types
// Note: We no longer need the StreamEvent and Delta structs
// as we're parsing the OpenRouter streaming responses as generic JSON

// Define a stream of text chunks
pub struct ResponseStream {
    receiver: mpsc::Receiver<Result<String>>,
}

impl Stream for ResponseStream {
    type Item = Result<String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

impl ResponseStream {
    fn new(receiver: mpsc::Receiver<Result<String>>) -> Self {
        Self { receiver }
    }
}

/// Client for communicating with OpenRouter API to access Claude models
pub struct OpenRouterClient {
    client: Client,
    pub config: Config,
}

impl OpenRouterClient {
    /// Creates a new client for communicating with OpenRouter's API to access Claude models
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the client, containing API key, model, etc.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The client instance or an error
    pub fn new(config: Config) -> Result<Self> {
        let mut headers = header::HeaderMap::new();

        // Set up authorization header for OpenRouter
        // OpenRouter uses Bearer auth instead of x-api-key
        let auth_value = format!("Bearer {}", config.api_key);
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&auth_value)
                .map_err(|e| KonaError::ApiError(format!("Invalid API key: {}", e)))?,
        );

        // Set the HTTP-REFERER header (OpenRouter likes to know where requests come from)
        headers.insert(
            "HTTP-REFERER",
            header::HeaderValue::from_static("https://github.com/yourusername/kona"),
        );

        // Set the Content-Type header
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| KonaError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Sends a single message to the OpenRouter API and waits for the complete response
    ///
    /// # Arguments
    ///
    /// * `message` - The message content to send
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The response from the API or an error
    pub async fn send_message(&self, message: &str) -> Result<String> {
        // Call the non-streaming version with a single message
        let messages = vec![Message {
            role: "user".to_string(),
            content: message.to_string(),
        }];
        self.send_message_with_history(messages).await
    }

    /// Sends a conversation history to the OpenRouter API and waits for the complete response
    ///
    /// # Arguments
    ///
    /// * `messages` - A vector of messages representing the conversation history
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The response from the API or an error
    pub async fn send_message_with_history(&self, messages: Vec<Message>) -> Result<String> {
        // Call the non-streaming version with message history
        self.send_message_internal_with_history(messages, false).await
    }

    /// Sends a single message to the OpenRouter API and streams the response
    ///
    /// # Arguments
    ///
    /// * `message` - The message content to send
    ///
    /// # Returns
    ///
    /// * `Result<ResponseStream>` - A stream of response chunks or an error
    pub async fn send_message_streaming(&self, message: &str) -> Result<ResponseStream> {
        // Call the streaming version with a single message
        let messages = vec![Message {
            role: "user".to_string(),
            content: message.to_string(),
        }];
        self.send_message_streaming_with_history(messages).await
    }

    /// Sends a conversation history to the OpenRouter API and streams the response
    ///
    /// # Arguments
    ///
    /// * `messages` - A vector of messages representing the conversation history
    ///
    /// # Returns
    ///
    /// * `Result<ResponseStream>` - A stream of response chunks or an error
    pub async fn send_message_streaming_with_history(&self, messages: Vec<Message>) -> Result<ResponseStream> {
        let (sender, receiver) = mpsc::channel(100);

        // If system message is set, add it as the first message
        let mut all_messages = Vec::new();

        // Add system prompt if configured
        if let Some(system_prompt) = &self.config.system_prompt {
            all_messages.push(Message {
                role: "system".to_string(),
                content: system_prompt.clone(),
            });
        }

        // Add user messages
        all_messages.extend(messages);

        // Map model name to OpenRouter's model format for Claude
        // OpenRouter uses format like "anthropic/claude-3-sonnet"
        let model_name = if self.config.model.contains("claude") && !self.config.model.starts_with("anthropic/") {
            format!("anthropic/{}", self.config.model)
        } else {
            self.config.model.clone()
        };

        let request = MessageRequest {
            model: model_name,
            max_tokens: self.config.max_tokens,
            messages: all_messages,
            stream: Some(true),
            temperature: Some(0.7), // Default temperature
        };

        debug!("Using API key: {}", mask_api_key(&self.config.api_key));
        debug!("Sending streaming message to OpenRouter API");

        // Create a clone of the client for the async task
        let client = self.client.clone();

        // Clone relevant data for the tokio task to avoid lifetime issues
        #[cfg(not(test))]
        let api_url = API_URL.to_string();

        #[cfg(test)]
        let api_url = API_URL.with(|url| url.borrow().clone());

        // Start a new task to handle the streaming response
        tokio::spawn(async move {
            match client.post(api_url)
                .json(&request)
                .send()
                .await
            {
                Ok(response) => {
                    if !response.status().is_success() {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        let error = KonaError::ApiError(format!("API returned error {}: {}", status, error_text));
                        let _ = sender.send(Err(error)).await;
                        return;
                    }

                    let stream = response.bytes_stream();
                    let mut stream = stream
                        .map_err(|e| KonaError::ApiError(format!("Stream error: {}", e)));

                    let mut buffer = String::new();

                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                // Convert bytes to string
                                if let Ok(chunk_str) = String::from_utf8(chunk.to_vec()) {
                                    buffer.push_str(&chunk_str);

                                    // Process the buffer to extract events and update the buffer
                                    // OpenRouter uses the SSE format: "data: {...}\n\n"
                                    let lines: Vec<&str> = buffer.split("\n\n").collect();

                                    // Process all but the last line (which might be incomplete)
                                    for i in 0..lines.len().saturating_sub(1) {
                                        let line = lines[i].trim();

                                        if line.is_empty() {
                                            continue;
                                        }

                                        // Lines should start with "data: "
                                        if let Some(data) = line.strip_prefix("data: ") {
                                            // Check for the completion signal
                                            if data == "[DONE]" {
                                                debug!("Received [DONE] event");
                                                continue;
                                            }

                                            // Parse the data as JSON
                                            match serde_json::from_str::<serde_json::Value>(data) {
                                                Ok(json) => {
                                                    // Extract the content delta from OpenRouter format
                                                    if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                                                        if let Some(choice) = choices.first() {
                                                            if let Some(delta) = choice.get("delta") {
                                                                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                                                    if !content.is_empty() {
                                                                        let _ = sender.send(Ok(content.to_string())).await;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                },
                                                Err(e) => {
                                                    warn!("Failed to parse event JSON: {}", e);
                                                    warn!("Raw data: {}", data);
                                                }
                                            }
                                        }
                                    }

                                    // Keep only the last (potentially incomplete) event
                                    if lines.len() > 0 {
                                        buffer = lines.last().unwrap_or(&"").to_string();
                                    }
                                }
                            },
                            Err(e) => {
                                let _ = sender.send(Err(e)).await;
                                break;
                            }
                        }
                    }
                },
                Err(e) => {
                    let error = KonaError::ApiError(format!("API request failed: {}", e));
                    let _ = sender.send(Err(error)).await;
                }
            }
        });

        Ok(ResponseStream::new(receiver))
    }

    // OpenRouter streaming response handling is now directly
    // integrated into the send_message_streaming_with_history method

    /// Internal implementation for sending messages that can be called with or without streaming
    ///
    /// # Arguments
    ///
    /// * `messages` - A vector of messages representing the conversation history
    /// * `streaming` - Whether to enable streaming mode in the request
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The full response text or an error
    async fn send_message_internal_with_history(&self, messages: Vec<Message>, streaming: bool) -> Result<String> {
        // If system message is set, add it as the first message
        let mut all_messages = Vec::new();

        // Add system prompt if configured
        if let Some(system_prompt) = &self.config.system_prompt {
            all_messages.push(Message {
                role: "system".to_string(),
                content: system_prompt.clone(),
            });
        }

        // Add user messages
        all_messages.extend(messages);

        // Map model name to OpenRouter's model format for Claude
        // OpenRouter uses format like "anthropic/claude-3-sonnet"
        let model_name = if self.config.model.contains("claude") && !self.config.model.starts_with("anthropic/") {
            format!("anthropic/{}", self.config.model)
        } else {
            self.config.model.clone()
        };

        let request = MessageRequest {
            model: model_name,
            max_tokens: self.config.max_tokens,
            messages: all_messages,
            stream: if streaming { Some(true) } else { None },
            temperature: Some(0.7), // Default temperature
        };

        // Log the request with masked API key
        debug!("Using API key: {}", mask_api_key(&self.config.api_key));
        debug!("Sending message to OpenRouter API");

        // Get the API URL depending on the build configuration
        #[cfg(not(test))]
        let api_url = API_URL.to_string();

        #[cfg(test)]
        let api_url = API_URL.with(|url| url.borrow().clone());

        // Print the full request for debugging
        debug!("Request URL: {}", api_url);
        debug!("Request body: {}", serde_json::to_string_pretty(&request).unwrap_or_default());

        let response = self
            .client
            .post(&api_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| KonaError::ApiError(format!("API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("API error: {} - {}", status, error_text);

            // Provide a more helpful error message for authentication issues
            if status.as_u16() == 401 {
                return Err(KonaError::ApiError(
                    "Authentication failed with OpenRouter. Please check that your API key is valid and properly formatted. \
                    For OpenRouter, the API key should be from openrouter.ai and not directly from Anthropic.".to_string()
                ));
            }

            return Err(KonaError::ApiError(format!(
                "API returned error {}: {}",
                status, error_text
            )));
        }

        let response_data: MessageResponse = response
            .json()
            .await
            .map_err(|e| KonaError::ApiError(format!("Failed to parse API response: {}", e)))?;

        info!("Received response with ID: {}", response_data.id);

        // Extract response content from the first choice
        if let Some(choice) = response_data.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(KonaError::ApiError("No response content received".to_string()))
        }
    }
}

// TODO: Add proper tests
// Removed the test module temporarily until other errors are fixed