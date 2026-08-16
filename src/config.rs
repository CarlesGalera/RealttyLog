//! Persistència de les regles de ressaltat i filtratge (Fase 4) en un
//! fitxer JSON al costat de l'executable. Cap funció d'aquest mòdul fa
//! pànic amb un fitxer absent, il·legible o parcialment invàlid: la
//! configuració és una comoditat, mai un requisit per poder obrir i llegir
//! logs (research.md, decisió 4).

use std::path::{Path, PathBuf};

use crate::rules::{HighlightRule, RuleSet};

const FILE_NAME: &str = "realttylog-rules.json";

/// Ubicació real del fitxer de configuració: sempre relatiu a
/// `current_exe()`, mai a un directori de configuració del sistema
/// operatiu (FR-008, research.md decisió 2). `None` si `current_exe()`
/// falla (rar) — en aquest cas es renuncia a persistir per a aquesta
/// execució, sense que això impedeixi fer servir l'aplicació.
pub fn config_path() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()?
        .parent()
        .map(|dir| dir.join(FILE_NAME))
}

/// Llegeix i deserialitza les regles des de `path`. Sense dependre de
/// `current_exe()` perquè es pugui provar amb un fitxer temporal
/// qualsevol (research.md, decisió 5).
pub fn load_from(path: &Path) -> RuleSet {
    let Ok(text) = std::fs::read_to_string(path) else {
        return RuleSet::new();
    };
    let Ok(raw) = serde_json::from_str::<Vec<serde_json::Value>>(&text) else {
        return RuleSet::new();
    };
    let rules: Vec<HighlightRule> = raw
        .into_iter()
        .filter_map(|value| serde_json::from_value::<HighlightRule>(value).ok())
        .filter(|rule| !rule.keyword.trim().is_empty())
        .collect();
    RuleSet::from_rules(rules)
}

/// Desa les regles a `path`. Els errors d'escriptura (per exemple,
/// directori sense permisos) s'ignoren silenciosament (FR-003): la sessió
/// en curs no en depèn.
pub fn save_to(path: &Path, rules: &RuleSet) {
    let Ok(text) = serde_json::to_string_pretty(rules.rules()) else {
        return;
    };
    let _ = std::fs::write(path, text);
}

/// Carrega des de la ubicació real (`config_path()`). `RuleSet` buit si no
/// es pot resoldre la ubicació o el fitxer no existeix.
pub fn load() -> RuleSet {
    match config_path() {
        Some(path) => load_from(&path),
        None => RuleSet::new(),
    }
}

/// Desa a la ubicació real (`config_path()`). No fa res si no es pot
/// resoldre la ubicació.
pub fn save(rules: &RuleSet) {
    if let Some(path) = config_path() {
        save_to(&path, rules);
    }
}
