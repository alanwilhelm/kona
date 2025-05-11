// Utility functions module
pub mod error;
#[cfg(test)]
mod tests;

pub fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 8 {
        return "****".to_string();
    }

    let prefix = &api_key[0..4];
    let suffix = &api_key[api_key.len() - 4..];
    format!("{}****{}", prefix, suffix)
}