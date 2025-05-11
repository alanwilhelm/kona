// API client module
pub mod client;
#[cfg(test)]
pub mod mock;

pub use client::{OpenRouterClient, Message, ResponseStream};