use serde::{Deserialize, Serialize};

/// Color propi de `rules/`, sense dependència d'`egui` (mateix patró que
/// `format::TokenKind` a la Fase 2): `ui/` el tradueix a `egui::Color32` en
/// pintar. `Serialize`/`Deserialize` (Fase 4) hi són perquè `config.rs` el
/// pugui persistir sense que `rules/` sàpiga res del fitxer de
/// configuració.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
