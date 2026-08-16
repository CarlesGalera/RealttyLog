//! Tests d'integració del ressaltat i filtratge per regla (Fase 3), aïllats
//! de la GUI (plan.md, Testing).

use realttylog::rules::{RgbColor, RuleSet};

const RED: RgbColor = RgbColor::new(255, 0, 0);
const YELLOW: RgbColor = RgbColor::new(255, 255, 0);

// ---------------------------------------------------------------------
// User Story 1 — ressaltar (T005-T007)
// ---------------------------------------------------------------------

/// T005: la coincidència és insensible a majúscules i minúscules (FR-002).
#[test]
fn matching_is_case_insensitive() {
    let mut rules = RuleSet::new();
    rules.add("ERROR", RED).unwrap();
    assert!(rules
        .matching_rule("2026-08-16 error connexió refusada")
        .is_some());
    assert!(rules
        .matching_rule("2026-08-16 Error connexió refusada")
        .is_some());
    assert!(rules
        .matching_rule("2026-08-16 info petició rebuda")
        .is_none());
}

/// T006: amb dues regles que coincideixen amb la mateixa línia, guanya la
/// creada més recentment (research.md, decisió 2).
#[test]
fn most_recently_created_rule_wins_on_overlap() {
    let mut rules = RuleSet::new();
    rules.add("connexió", RED).unwrap();
    rules.add("refusada", YELLOW).unwrap();

    let (_, winner) = rules.matching_rule("connexió refusada").unwrap();
    assert_eq!(winner.color, YELLOW);
}

/// T007: una paraula clau buida o només espais es rebutja (FR-004).
#[test]
fn add_rejects_an_empty_or_blank_keyword() {
    let mut rules = RuleSet::new();
    assert!(rules.add("", RED).is_err());
    assert!(rules.add("   ", RED).is_err());
    assert_eq!(rules.rules().len(), 0);
}

// ---------------------------------------------------------------------
// User Story 2 — filtrar (T011-T013)
// ---------------------------------------------------------------------

/// T011: sense cap regla amb filtre actiu, totes les línies són visibles
/// (FR-009).
#[test]
fn without_an_active_filter_everything_is_visible() {
    let mut rules = RuleSet::new();
    rules.add("ERROR", RED).unwrap();
    assert!(rules.is_visible("info petició rebuda"));
    assert!(rules.is_visible("error connexió refusada"));
}

/// T012: amb dues regles amb filtre actiu, una línia és visible si en
/// compleix almenys una — OR, no AND (research.md, decisió 3).
#[test]
fn active_filters_combine_with_or() {
    let mut rules = RuleSet::new();
    rules.add("ERROR", RED).unwrap();
    rules.add("WARN", YELLOW).unwrap();
    rules.rule_mut(0).unwrap().filter = true;
    rules.rule_mut(1).unwrap().filter = true;

    assert!(rules.is_visible("error connexió refusada"));
    assert!(rules.is_visible("warn latència alta"));
    assert!(!rules.is_visible("info petició rebuda"));
}

/// T013: desactivar una regla li atura el filtre encara que `filter` fos
/// `true` (US3, escenari 3).
#[test]
fn disabling_a_rule_also_stops_its_filter() {
    let mut rules = RuleSet::new();
    rules.add("ERROR", RED).unwrap();
    rules.rule_mut(0).unwrap().filter = true;
    rules.rule_mut(0).unwrap().enabled = false;

    assert!(rules.is_visible("error connexió refusada"));
    assert!(!rules.has_active_filter());
}

// ---------------------------------------------------------------------
// User Story 3 — gestionar (T017-T018)
// ---------------------------------------------------------------------

/// T017: editar la paraula o el color d'una regla existent no li canvia la
/// posició (i per tant la prioritat) al vector.
#[test]
fn editing_a_rule_does_not_change_its_priority_position() {
    let mut rules = RuleSet::new();
    rules.add("ERROR", RED).unwrap();
    rules.add("WARN", YELLOW).unwrap();

    // ERROR es va crear primer; l'editem sense que passi a tenir més
    // prioritat que WARN.
    rules.rule_mut(0).unwrap().color = YELLOW;

    let (winning_index, _) = rules
        .matching_rule("error i warn a la mateixa línia")
        .unwrap();
    assert_eq!(
        winning_index, 1,
        "WARN, creada després, hauria de continuar guanyant"
    );
}

/// T018: esborrar una regla fa que deixi d'afectar `matching_rule` i
/// `is_visible` per a qualsevol línia.
#[test]
fn removing_a_rule_stops_it_from_matching_or_filtering() {
    let mut rules = RuleSet::new();
    rules.add("ERROR", RED).unwrap();
    rules.rule_mut(0).unwrap().filter = true;

    rules.remove(0);

    assert!(rules.matching_rule("error connexió refusada").is_none());
    assert!(rules.is_visible("error connexió refusada"));
    assert!(!rules.has_active_filter());
}
