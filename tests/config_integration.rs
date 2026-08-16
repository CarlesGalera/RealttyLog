//! Tests d'integració de la persistència de regles (Fase 4), aïllats de la
//! GUI i de `current_exe()` real (plan.md, Testing): sempre criden les
//! variants que reben el `Path` com a paràmetre.

use std::fs;

use realttylog::config;
use realttylog::rules::{RgbColor, RuleSet};

fn temp_path(name: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "realttylog-config-test-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    path
}

// ---------------------------------------------------------------------
// User Story 1 — sobreviu a tancar l'aplicació (T006-T007)
// ---------------------------------------------------------------------

/// T006: desar i tornar a carregar des del mateix path reprodueix les
/// mateixes regles (FR-001, SC-001).
#[test]
fn save_then_load_roundtrips_the_same_rules() {
    let path = temp_path("roundtrip");
    let mut rules = RuleSet::new();
    rules.add("ERROR", RgbColor::new(220, 90, 90)).unwrap();
    rules.add("WARN", RgbColor::new(216, 220, 90)).unwrap();
    rules.rule_mut(1).unwrap().filter = true;
    rules.rule_mut(0).unwrap().enabled = false;

    config::save_to(&path, &rules);
    let loaded = config::load_from(&path);

    assert_eq!(loaded.rules().len(), 2);
    assert_eq!(loaded.rules()[0].keyword, "ERROR");
    assert!(!loaded.rules()[0].enabled);
    assert_eq!(loaded.rules()[1].keyword, "WARN");
    assert!(loaded.rules()[1].filter);
    assert_eq!(loaded.rules()[1].color, RgbColor::new(216, 220, 90));

    let _ = fs::remove_file(&path);
}

/// T007: carregar des d'un path que no existeix retorna un `RuleSet` buit
/// sense error (FR-005).
#[test]
fn load_from_a_missing_file_returns_an_empty_ruleset() {
    let path = temp_path("missing");
    let loaded = config::load_from(&path);
    assert_eq!(loaded.rules().len(), 0);
}

// ---------------------------------------------------------------------
// User Story 2 — viatja amb l'executable (T011)
// ---------------------------------------------------------------------

/// T011: `config_path()` es resol relatiu al directori pare de
/// `current_exe()`, no a una ubicació fixa independent d'on és
/// l'executable (FR-008).
#[test]
fn config_path_resolves_relative_to_current_exe() {
    let expected_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let path = config::config_path().expect("current_exe hauria de resoldre's en un test");
    assert_eq!(path.parent().unwrap(), expected_dir);
    assert_eq!(path.file_name().unwrap(), "realttylog-rules.json");
}

// ---------------------------------------------------------------------
// User Story 3 — un fitxer trencat no bloqueja l'aplicació (T013-T015)
// ---------------------------------------------------------------------

/// T013: contingut que no és JSON vàlid retorna un `RuleSet` buit sense
/// pànic (FR-006).
#[test]
fn load_from_invalid_json_returns_an_empty_ruleset() {
    let path = temp_path("invalid-json");
    fs::write(&path, "això no és JSON").unwrap();

    let loaded = config::load_from(&path);
    assert_eq!(loaded.rules().len(), 0);

    let _ = fs::remove_file(&path);
}

/// T014: un array JSON vàlid amb una regla ben formada i una amb el camp
/// `color` absent carrega només la vàlida (FR-007).
#[test]
fn a_malformed_rule_among_valid_ones_is_skipped_not_fatal() {
    let path = temp_path("partial-invalid");
    fs::write(
        &path,
        r#"[
            {"keyword": "ERROR", "color": {"r": 220, "g": 90, "b": 90}, "enabled": true, "filter": false},
            {"keyword": "TRENCADA", "enabled": true, "filter": false}
        ]"#,
    )
    .unwrap();

    let loaded = config::load_from(&path);
    assert_eq!(loaded.rules().len(), 1);
    assert_eq!(loaded.rules()[0].keyword, "ERROR");

    let _ = fs::remove_file(&path);
}

/// T015: desar en un directori inexistent no fa pànic; l'error s'ignora
/// (FR-003).
#[test]
fn save_to_an_unwritable_path_does_not_panic() {
    let mut path = temp_path("unwritable-dir");
    path.push("subdir-inexistent");
    path.push("rules.json");

    let mut rules = RuleSet::new();
    rules.add("ERROR", RgbColor::new(220, 90, 90)).unwrap();

    config::save_to(&path, &rules);
}
