//! Zalo client sub-modules — auth, messaging, session, crypto, WebSocket listener, bot.
pub mod auth;
pub mod session;
pub mod crypto;
pub mod messaging;
pub mod groups;
pub mod friends;
pub mod business;
pub mod listener;
pub mod models;
/// Zalo OA Bot Platform client (bot.zapps.me official API).
pub mod bot;
