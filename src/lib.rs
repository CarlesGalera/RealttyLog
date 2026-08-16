//! Biblioteca de RealttyLog. `main.rs` només arrenca la finestra; la lògica
//! viu aquí perquè `tests/*_integration.rs` la pugui exercitar de punta a
//! punta sense passar per la GUI (plan.md, Testing).

pub mod app;
pub mod config;
pub mod encoding;
pub mod format;
pub mod rules;
pub mod search;
pub mod tailer;
pub mod ui;
