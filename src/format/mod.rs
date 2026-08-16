//! Detecció i formatatge de payloads JSON, XML, HTML i JWT dins d'una
//! línia de log (Fase 2). No depèn d'`egui`: `ui/log_view.rs` és qui
//! tradueix `TokenKind` a colors concrets (research.md, decisió 5).

pub mod detect;
pub mod json;
pub mod jwt;
pub mod styled;
pub mod xml;

pub use styled::{StyledLine, TokenKind};

/// El format detectat en una línia (FR-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadKind {
    Json,
    Xml,
    Html,
    Jwt,
}

/// El resultat de la detecció sobre una línia concreta: quin format s'hi
/// ha trobat i on comença i acaba dins la línia (FR-001–FR-005).
/// Correspon a l'entitat "Payload detectat" de data-model.md.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedPayload {
    pub kind: PayloadKind,
    pub start: usize,
    pub end: usize,
}
