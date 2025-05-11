use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use tracing::debug;

use crate::api::Message;
use crate::utils::error::{KonaError, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
}

impl Conversation {
    pub fn new(title: String) -> Self {
        let now = Utc::now();
        let id = format!("{}", uuid::Uuid::new_v4());
        
        Self {
            id,
            title,
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
        }
    }
    
    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(Message {
            role: "user".to_string(),
            content,
        });
        self.updated_at = Utc::now();
    }
    
    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(Message {
            role: "assistant".to_string(),
            content,
        });
        self.updated_at = Utc::now();
    }
    
    pub fn to_summary(&self) -> ConversationSummary {
        ConversationSummary {
            id: self.id.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            message_count: self.messages.len(),
        }
    }
}

pub struct ConversationStorage {
    storage_dir: PathBuf,
    conversations: HashMap<String, ConversationSummary>,
}

impl ConversationStorage {
    pub fn new() -> Result<Self> {
        let storage_dir = Self::get_storage_dir()?;
        let conversations = Self::load_conversation_index(&storage_dir)?;
        
        Ok(Self {
            storage_dir,
            conversations,
        })
    }
    
    fn get_storage_dir() -> Result<PathBuf> {
        let mut dir = match dirs::data_dir() {
            Some(dir) => dir,
            None => return Err(KonaError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not determine data directory",
            ))),
        };
        
        dir.push("kona");
        dir.push("conversations");
        
        // Create directory if it doesn't exist
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| {
                KonaError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to create conversation directory: {}", e),
                ))
            })?;
        }
        
        Ok(dir)
    }
    
    fn get_index_path(storage_dir: &PathBuf) -> PathBuf {
        let mut path = storage_dir.clone();
        path.push("index.json");
        path
    }
    
    fn get_conversation_path(&self, id: &str) -> PathBuf {
        let mut path = self.storage_dir.clone();
        path.push(format!("{}.json", id));
        path
    }
    
    fn load_conversation_index(storage_dir: &PathBuf) -> Result<HashMap<String, ConversationSummary>> {
        let index_path = Self::get_index_path(storage_dir);
        
        if !index_path.exists() {
            return Ok(HashMap::new());
        }
        
        let content = fs::read_to_string(&index_path).map_err(|e| {
            KonaError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read conversation index: {}", e),
            ))
        })?;
        
        serde_json::from_str(&content).map_err(|e| {
            KonaError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to parse conversation index: {}", e),
            ))
        })
    }
    
    fn save_conversation_index(&self) -> Result<()> {
        let index_path = Self::get_index_path(&self.storage_dir);
        
        let content = serde_json::to_string_pretty(&self.conversations).map_err(|e| {
            KonaError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to serialize conversation index: {}", e),
            ))
        })?;
        
        fs::write(&index_path, content).map_err(|e| {
            KonaError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write conversation index: {}", e),
            ))
        })
    }
    
    pub fn get_all_conversations(&self) -> Vec<ConversationSummary> {
        let mut conversations: Vec<_> = self.conversations.values().cloned().collect();
        conversations.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)); // Sort newest first
        conversations
    }
    
    pub fn create_conversation(&mut self, title: String) -> Result<Conversation> {
        let conversation = Conversation::new(title);
        
        // Add to index
        self.conversations.insert(
            conversation.id.clone(),
            conversation.to_summary(),
        );
        
        // Save index
        self.save_conversation_index()?;
        
        Ok(conversation)
    }
    
    pub fn save_conversation(&mut self, conversation: &Conversation) -> Result<()> {
        // Update index
        self.conversations.insert(
            conversation.id.clone(),
            conversation.to_summary(),
        );
        
        // Save index
        self.save_conversation_index()?;
        
        // Save conversation
        let path = self.get_conversation_path(&conversation.id);
        let content = serde_json::to_string_pretty(conversation).map_err(|e| {
            KonaError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to serialize conversation: {}", e),
            ))
        })?;
        
        fs::write(&path, content).map_err(|e| {
            KonaError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write conversation: {}", e),
            ))
        })?;
        
        debug!("Saved conversation to {}", path.display());
        Ok(())
    }
    
    pub fn load_conversation(&self, id: &str) -> Result<Conversation> {
        if !self.conversations.contains_key(id) {
            return Err(KonaError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Conversation not found: {}", id),
            )));
        }
        
        let path = self.get_conversation_path(id);
        let content = fs::read_to_string(&path).map_err(|e| {
            KonaError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read conversation: {}", e),
            ))
        })?;
        
        serde_json::from_str(&content).map_err(|e| {
            KonaError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to parse conversation: {}", e),
            ))
        })
    }
    
    pub fn delete_conversation(&mut self, id: &str) -> Result<()> {
        if !self.conversations.contains_key(id) {
            return Err(KonaError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Conversation not found: {}", id),
            )));
        }
        
        // Remove from index
        self.conversations.remove(id);
        
        // Save index
        self.save_conversation_index()?;
        
        // Delete conversation file
        let path = self.get_conversation_path(id);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                KonaError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to delete conversation: {}", e),
                ))
            })?;
        }
        
        Ok(())
    }
}