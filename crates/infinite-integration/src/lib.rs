//! Infinite Integration - PixygonServer API client
//!
//! Provides authentication, character management, AI chat, and game state persistence.

pub mod error;
pub mod types;
pub mod auth;
pub mod character;
pub mod character_item;
pub mod game_story;
pub mod ai_chat;
pub mod client;

pub use client::{IntegrationClient, PendingRequest};
pub use error::IntegrationError;
pub use types::*;
