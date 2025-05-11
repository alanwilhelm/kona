#[cfg(test)]
mod tests {
    use super::storage::{Conversation, ConversationStorage};
    use uuid::Uuid;
    
    #[test]
    fn test_conversation_new() {
        let title = "Test Conversation".to_string();
        let conversation = Conversation::new(title.clone());
        
        assert_eq!(conversation.title, title);
        assert!(Uuid::parse_str(&conversation.id).is_ok());
        assert!(conversation.messages.is_empty());
    }
    
    #[test]
    fn test_conversation_add_messages() {
        let mut conversation = Conversation::new("Test".to_string());
        
        // Add a user message
        conversation.add_user_message("Hello".to_string());
        assert_eq!(conversation.messages.len(), 1);
        assert_eq!(conversation.messages[0].role, "user");
        assert_eq!(conversation.messages[0].content, "Hello");
        
        // Add an assistant message
        conversation.add_assistant_message("Hi there!".to_string());
        assert_eq!(conversation.messages.len(), 2);
        assert_eq!(conversation.messages[1].role, "assistant");
        assert_eq!(conversation.messages[1].content, "Hi there!");
    }
    
    #[test]
    fn test_conversation_to_summary() {
        let mut conversation = Conversation::new("Test".to_string());
        conversation.add_user_message("Hello".to_string());
        conversation.add_assistant_message("Hi there!".to_string());
        
        let summary = conversation.to_summary();
        
        assert_eq!(summary.id, conversation.id);
        assert_eq!(summary.title, conversation.title);
        assert_eq!(summary.created_at, conversation.created_at);
        assert_eq!(summary.updated_at, conversation.updated_at);
        assert_eq!(summary.message_count, 2);
    }
}