// Terminal UI Implementation with ratatui

use crate::api::OpenRouterClient;
use crate::utils::error::Result;
use crate::utils::mask_api_key;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;

// Message type for our UI
enum UiMessage {
    User(String),
    Assistant(String),
    Status(String),
    Command(String, String), // Command and its result
}

// Custom implementation of a text input widget
struct TextInput {
    text: String,
    cursor_position: usize,
    scroll_offset: usize,
}

impl TextInput {
    fn new() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            scroll_offset: 0,
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.text.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.text.remove(self.cursor_position);
                }
            }
            KeyCode::Delete => {
                if self.cursor_position < self.text.len() {
                    self.text.remove(self.cursor_position);
                }
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.text.len() {
                    self.cursor_position += 1;
                }
            }
            KeyCode::Home => {
                self.cursor_position = 0;
            }
            KeyCode::End => {
                self.cursor_position = self.text.len();
            }
            _ => {}
        }
    }

    fn get_text(&self) -> &str {
        &self.text
    }

    fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
        self.scroll_offset = 0;
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title("Input (Shift+Enter to send, Esc to exit)");

        let inner_area = input_block.inner(area);

        let mut text = Text::default();
        let content = Span::raw(&self.text);
        text.lines.push(Line::from(content));

        let input = Paragraph::new(text)
            .block(input_block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(input, area);

        // Show cursor
        if inner_area.width > 0 && inner_area.height > 0 {
            frame.set_cursor_position(
                (inner_area.x + self.cursor_position as u16, inner_area.y)
            );
        }
    }
}

pub struct Tui {
    client: OpenRouterClient,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    input_area: TextInput,
    messages: Vec<UiMessage>,
    should_quit: bool,
}

impl Tui {
    pub fn new(client: OpenRouterClient) -> Result<Self> {
        // Try to detect if the terminal is compatible
        // Check if we're in a valid terminal by testing basic operations
        if !Self::is_valid_terminal_env() {
            return Err(crate::utils::error::KonaError::IoError(io::Error::new(
                io::ErrorKind::Unsupported,
                "Terminal environment not compatible with TUI mode",
            )));
        }

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();

        // Use a more defensive approach with terminal operations
        match execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
            Ok(_) => {},
            Err(e) => {
                // Make sure to clean up if we failed
                let _ = disable_raw_mode();
                return Err(crate::utils::error::KonaError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to set up terminal: {}", e),
                )));
            }
        }

        let backend = CrosstermBackend::new(stdout);

        let terminal = match Terminal::new(backend) {
            Ok(t) => t,
            Err(e) => {
                // Clean up on failure
                let _ = disable_raw_mode();
                let mut stdout = io::stdout();
                let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);

                return Err(crate::utils::error::KonaError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to create terminal: {}", e),
                )));
            }
        };

        // Setup input area
        let input_area = TextInput::new();

        Ok(Self {
            client,
            terminal,
            input_area,
            messages: Vec::new(),
            should_quit: false,
        })
    }

    // Helper method to check if we're in a valid terminal environment
    fn is_valid_terminal_env() -> bool {
        // Try to get terminal size - this is a good indicator of terminal compatibility
        if let Err(_) = crossterm::terminal::size() {
            return false;
        }

        // Check if we can enable/disable raw mode briefly as a test
        if let Err(_) = enable_raw_mode() {
            return false;
        }
        let _ = disable_raw_mode(); // Be sure to reset back

        true
    }

    pub async fn run(&mut self) -> Result<()> {
        // Show welcome message
        self.messages.push(UiMessage::Status(format!(
            "🌴 Kona v{} - Welcome to the interactive mode",
            env!("CARGO_PKG_VERSION")
        )));
        self.messages.push(UiMessage::Status(
            "Type /help for a list of commands".to_string(),
        ));

        // Set up error recovery
        let result = self.run_ui_loop().await;

        // Always make sure to restore terminal state, even on errors
        self.restore_terminal();

        // Return any error from the UI loop
        result
    }

    // Main UI loop
    async fn run_ui_loop(&mut self) -> Result<()> {
        while !self.should_quit {
            if let Err(e) = self.draw() {
                // Try to restore terminal and bubble up the error
                self.restore_terminal();
                return Err(e);
            }

            // Poll for events with error handling
            match crossterm::event::poll(Duration::from_millis(100)) {
                Ok(true) => {
                    match crossterm::event::read() {
                        Ok(Event::Key(key)) => {
                            if let Err(e) = self.handle_key_event(key).await {
                                self.restore_terminal();
                                return Err(e);
                            }
                        },
                        Ok(_) => {}, // Other events are ignored
                        Err(e) => {
                            self.restore_terminal();
                            return Err(crate::utils::error::KonaError::IoError(
                                io::Error::new(io::ErrorKind::Other, format!("Event read error: {}", e))
                            ));
                        }
                    }
                },
                Ok(false) => {}, // No events ready
                Err(e) => {
                    self.restore_terminal();
                    return Err(crate::utils::error::KonaError::IoError(
                        io::Error::new(io::ErrorKind::Other, format!("Event poll error: {}", e))
                    ));
                }
            }
        }

        Ok(())
    }

    // Helper method to safely restore terminal state
    fn restore_terminal(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }

    fn draw(&mut self) -> Result<()> {
        // Create a copy of references to avoid borrowing issues
        let messages = &self.messages;
        let input_area = &self.input_area;

        self.terminal.draw(|frame| {
            let area = frame.area();

            // Create the layout
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(5)].as_ref())
                .margin(1)
                .split(area);

            // Messages area
            let messages_area = main_chunks[0];

            // Draw messages
            let mut items: Vec<ListItem> = Vec::new();

            for message in messages {
                match message {
                    UiMessage::User(content) => {
                        let header = Line::from(vec![
                            Span::styled(
                                "You: ",
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]);
                        items.push(ListItem::new(vec![header]));

                        // Split content into lines for better display
                        for line in content.lines() {
                            items.push(ListItem::new(line));
                        }
                        items.push(ListItem::new("")); // Add spacing
                    }
                    UiMessage::Assistant(content) => {
                        let header = Line::from(vec![
                            Span::styled(
                                "Claude: ",
                                Style::default()
                                    .fg(Color::Magenta)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]);
                        items.push(ListItem::new(vec![header]));

                        // Split content into lines for better display
                        for line in content.lines() {
                            items.push(ListItem::new(line));
                        }
                        items.push(ListItem::new("")); // Add spacing
                    }
                    UiMessage::Status(content) => {
                        let text = Line::from(vec![
                            Span::styled(
                                format!("System: {}", content),
                                Style::default().fg(Color::Yellow),
                            ),
                        ]);
                        items.push(ListItem::new(vec![text]));
                    }
                    UiMessage::Command(cmd, result) => {
                        let header = Line::from(vec![
                            Span::styled(
                                format!("Command [{}]: ", cmd),
                                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                            ),
                        ]);
                        items.push(ListItem::new(vec![header]));

                        // Split result into lines
                        for line in result.lines() {
                            items.push(ListItem::new(line));
                        }
                        items.push(ListItem::new("")); // Add spacing
                    }
                }
            }

            let messages_list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Conversation"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");

            frame.render_widget(messages_list, messages_area);

            // Input area
            let input_area_rect = main_chunks[1];
            input_area.render(frame, input_area_rect);
        })?;

        Ok(())
    }

    // This function is no longer needed as it's inlined in the draw function
    // to avoid borrowing issues

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key {
            // Quit on Escape
            KeyEvent {
                code: KeyCode::Esc, ..
            } => {
                self.should_quit = true;
            }
            // Send message on Shift+Enter
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::SHIFT,
                ..
            } => {
                self.send_message().await?;
            }
            // Normal input
            _ => {
                self.input_area.handle_key_event(key);
            }
        }
        Ok(())
    }

    async fn send_message(&mut self) -> Result<()> {
        let message = self.input_area.get_text();
        if message.is_empty() {
            return Ok(());
        }

        let message = message.to_string();
        self.input_area.clear();

        // Process commands
        if message.starts_with('/') {
            let cmd = message.trim();
            match cmd {
                "/help" => {
                    self.messages.push(UiMessage::Command(
                        "/help".to_string(),
                        "Available commands:
  /help - Show this help
  /clear - Clear the conversation
  /config - Show current configuration
  /model [name] - Show or change the model
  /stream - Toggle streaming mode
  /quit - Exit the application"
                            .to_string(),
                    ));
                }
                "/clear" => {
                    self.messages.clear();
                    self.messages.push(UiMessage::Status("Conversation cleared.".to_string()));
                }
                "/config" => {
                    let config_info = format!(
                        "API Key: {}
Model: {}
Max Tokens: {}
System Prompt: {:?}
History Size: {}
Streaming: {}",
                        mask_api_key(&self.client.config.api_key),
                        self.client.config.model,
                        self.client.config.max_tokens,
                        self.client.config.system_prompt,
                        self.client.config.history_size,
                        if self.client.config.use_streaming {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    );
                    self.messages
                        .push(UiMessage::Command("/config".to_string(), config_info));
                }
                cmd if cmd.starts_with("/model") => {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.len() >= 2 {
                        // Change the model
                        let old_model = self.client.config.model.clone();
                        let new_model = parts[1].to_string();
                        self.client.config.model = new_model.clone();
                        self.messages.push(UiMessage::Command(
                            "/model".to_string(),
                            format!("Model changed from {} to {}", old_model, new_model),
                        ));
                    } else {
                        // Show current model
                        self.messages.push(UiMessage::Command(
                            "/model".to_string(),
                            format!(
                                "Current model: {}

Supported Claude models via OpenRouter:
- anthropic/claude-3-opus
- anthropic/claude-3-sonnet
- anthropic/claude-3-haiku
- anthropic/claude-3.5-sonnet
- anthropic/claude-3.5-haiku

To change models, use /model <model_name>",
                                self.client.config.model
                            ),
                        ));
                    }
                }
                "/stream" => {
                    self.client.config.use_streaming = !self.client.config.use_streaming;
                    let status = if self.client.config.use_streaming {
                        "enabled"
                    } else {
                        "disabled"
                    };
                    self.messages.push(UiMessage::Command(
                        "/stream".to_string(),
                        format!("Streaming mode: {}", status),
                    ));
                }
                "/quit" => {
                    self.should_quit = true;
                }
                _ => {
                    self.messages.push(UiMessage::Command(
                        cmd.to_string(),
                        format!("Unknown command: {}", cmd),
                    ));
                }
            }
            return Ok(());
        }

        // Regular message
        self.messages.push(UiMessage::User(message.clone()));
        self.draw()?; // Update UI to show user message

        // Use streaming or non-streaming based on config
        if self.client.config.use_streaming {
            // Use the streaming API
            match self.client.send_message_streaming(&message).await {
                Ok(mut stream) => {
                    let mut full_response = String::new();
                    let mut current_response = String::new();

                    // Process the stream
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                full_response.push_str(&chunk);
                                current_response.push_str(&chunk);

                                // Update the UI every few characters or when we get a newline
                                if chunk.contains('\n') || current_response.len() > 10 {
                                    // Add or update assistant message
                                    if let Some(last_msg) = self.messages.last() {
                                        if matches!(last_msg, UiMessage::Assistant(_)) {
                                            self.messages.pop();
                                        }
                                    }
                                    self.messages.push(UiMessage::Assistant(full_response.clone()));
                                    current_response.clear();
                                    self.draw()?;
                                }
                            }
                            Err(err) => {
                                self.messages.push(UiMessage::Status(format!("Error: {}", err)));
                                self.draw()?;
                                break;
                            }
                        }
                    }

                    // Final update if needed
                    if !current_response.is_empty() {
                        // Add or update assistant message
                        if let Some(last_msg) = self.messages.last() {
                            if matches!(last_msg, UiMessage::Assistant(_)) {
                                self.messages.pop();
                            }
                        }
                        self.messages.push(UiMessage::Assistant(full_response));
                        self.draw()?;
                    }
                }
                Err(err) => {
                    self.messages
                        .push(UiMessage::Status(format!("API Error: {}", err)));
                    self.draw()?;
                }
            }
        } else {
            // Standard non-streaming mode
            match self.client.send_message(&message).await {
                Ok(response) => {
                    self.messages.push(UiMessage::Assistant(response));
                    self.draw()?;
                }
                Err(err) => {
                    self.messages
                        .push(UiMessage::Status(format!("API Error: {}", err)));
                    self.draw()?;
                }
            }
        }

        Ok(())
    }
}

// Main function to start the TUI mode
pub async fn start_tui_mode(client: OpenRouterClient) -> Result<()> {
    let mut tui = Tui::new(client)?;
    tui.run().await
}