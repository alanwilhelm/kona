#[cfg(test)]
mod tests {
    use super::mask_api_key;

    #[test]
    fn test_mask_api_key() {
        // Test with a standard-length key
        let key = "sk-ant-api123456789abcdefg";
        let masked = mask_api_key(key);
        assert_eq!(masked, "sk-a****defg");
        
        // Test with a short key (less than 8 chars)
        let short_key = "1234";
        let masked_short = mask_api_key(short_key);
        assert_eq!(masked_short, "****");
        
        // Test with an empty key
        let empty_key = "";
        let masked_empty = mask_api_key(empty_key);
        assert_eq!(masked_empty, "****");
        
        // Test with exactly 8 chars key
        let exact_key = "12345678";
        let masked_exact = mask_api_key(exact_key);
        assert_eq!(masked_exact, "1234****");
    }
}