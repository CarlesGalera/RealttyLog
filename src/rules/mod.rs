pub mod color;

use serde::{Deserialize, Serialize};

pub use color::RgbColor;

/// Correspon a "Regla de ressaltat" de l'spec (FR-001–FR-005, data-model.md).
/// La prioritat és implícita per la posició dins `RuleSet::rules`, no un
/// camp d'aquesta struct (research.md, decisió 2). `Serialize`/`Deserialize`
/// (Fase 4) hi són pel mateix motiu que a `RgbColor`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightRule {
    pub keyword: String,
    pub color: RgbColor,
    pub enabled: bool,
    pub filter: bool,
}

impl HighlightRule {
    /// Coincidència insensible a majúscules (FR-002). Una paraula clau
    /// buida o només espais no compleix mai —estat assolible en editar una
    /// regla existent des de la interfície— en lloc de coincidir amb
    /// qualsevol línia.
    fn matches(&self, text: &str) -> bool {
        self.enabled
            && !self.keyword.trim().is_empty()
            && text.to_lowercase().contains(&self.keyword.to_lowercase())
    }
}

/// Correspon a "Estat de filtratge del mur" de l'spec, més la col·lecció de
/// regles que el determina (data-model.md). `version` s'incrementa a cada
/// mutació perquè `LogViewState` sàpiga quan invalidar la seva memorització
/// (research.md, decisió 4).
#[derive(Debug, Default)]
pub struct RuleSet {
    rules: Vec<HighlightRule>,
    version: u64,
}

impl RuleSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reconstrueix un `RuleSet` a partir de regles ja carregades (Fase 4,
    /// `config::load`). `version` sempre comença a 0: no té cap sentit
    /// entre execucions diferents (research.md de la Fase 4, decisió 4;
    /// data-model.md).
    pub fn from_rules(rules: Vec<HighlightRule>) -> Self {
        Self { rules, version: 0 }
    }

    pub fn rules(&self) -> &[HighlightRule] {
        &self.rules
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    /// Afegeix una regla al final (màxima prioritat, research.md decisió
    /// 2). Rebutja una paraula clau buida o només espais (FR-004) sense
    /// modificar res.
    pub fn add(&mut self, keyword: impl Into<String>, color: RgbColor) -> Result<(), &'static str> {
        let keyword = keyword.into();
        if keyword.trim().is_empty() {
            return Err("la paraula clau no pot ser buida");
        }
        self.rules.push(HighlightRule {
            keyword,
            color,
            enabled: true,
            filter: false,
        });
        self.version += 1;
        Ok(())
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.rules.len() {
            self.rules.remove(index);
            self.version += 1;
        }
    }

    /// Accés mutable per a l'edició des de `ui/rules_panel.rs` (paraula,
    /// color, activació, filtre). Qui el crida MUST invocar `bump_version`
    /// si ha canviat res, perquè la memorització de `LogViewState` es
    /// refresqui.
    pub fn rule_mut(&mut self, index: usize) -> Option<&mut HighlightRule> {
        self.rules.get_mut(index)
    }

    pub fn bump_version(&mut self) {
        self.version += 1;
    }

    /// Retorna la regla activa de més prioritat (la més recent) que
    /// coincideix amb `text`, si n'hi ha (research.md, decisió 2).
    pub fn matching_rule(&self, text: &str) -> Option<(usize, &HighlightRule)> {
        self.rules
            .iter()
            .enumerate()
            .rev()
            .find(|(_, rule)| rule.matches(text))
    }

    /// `true` si cap regla activa té el filtre actiu, o si `text` compleix
    /// almenys una de les que sí (OR, research.md decisió 3).
    pub fn is_visible(&self, text: &str) -> bool {
        let mut any_filter = false;
        for rule in &self.rules {
            if rule.enabled && rule.filter {
                any_filter = true;
                if rule.matches(text) {
                    return true;
                }
            }
        }
        !any_filter
    }

    pub fn has_active_filter(&self) -> bool {
        self.rules.iter().any(|rule| rule.enabled && rule.filter)
    }
}
