---

description: "Task list template for feature implementation"
---

# Tasks: Ressaltat per paraula clau i filtratge instantani

**Input**: Design documents from `/specs/003-ressaltat-i-filtratge/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: plan.md decideix cobrir `rules/` amb `cargo test`, aïllat de la GUI (mateix
patró que `format/` a la Fase 2).

**Organization**: les tasques s'agrupen per user story per poder-les implementar i provar
de manera independent.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: es pot fer en paral·lel (fitxers diferents, sense dependències pendents)
- **[Story]**: a quina user story pertany (US1-US3, segons spec.md)

## Path Conventions

Crate binari únic de Rust (plan.md): `src/` i `tests/` a l'arrel del repositori.

---

## Phase 1: Setup

- [ ] T001 Declarar `pub mod rules;` a `src/lib.rs` i crear l'esquelet buit de
      `src/rules/mod.rs` i `src/rules/color.rs` (research.md: cap dependència nova)

---

## Phase 2: Foundational (Blocking Prerequisites)

**⚠️ CRITICAL**: cap user story pot començar fins que aquesta fase estigui completa

- [ ] T002 [P] Implementar `RgbColor` a `src/rules/color.rs` (data-model.md)
- [ ] T003 Implementar `HighlightRule` i `RuleSet` (`add`, `remove`, `matching_rule`,
      `is_visible`, comptador `version`) a `src/rules/mod.rs` (data-model.md; research.md,
      decisions 1-4) — depèn de T002
- [ ] T004 Afegir `rules: RuleSet` a `App` (`src/app.rs`) i passar-lo per referència
      mutable a `LogViewState::ui()` (research.md, decisió 5) — depèn de T003

**Checkpoint**: fonaments llestos

---

## Phase 3: User Story 1 - Ressaltar línies per paraula clau (Priority: P1) 🎯 MVP

**Goal**: definir una regla de paraula clau i color, i veure-la aplicada a l'instant a les
línies que la compleixen, tant carregades com en directe.

**Independent Test**: amb un fitxer que barreja línies `INFO`, `WARN` i `ERROR`, definir
una regla `ERROR` → vermell i comprovar que només aquelles línies es ressalten.

### Tests for User Story 1 ⚠️

- [ ] T005 [P] [US1] Test: la coincidència és insensible a majúscules i minúscules
      (FR-002), en `tests/rules_integration.rs`
- [ ] T006 [P] [US1] Test: amb dues regles actives que coincideixen amb la mateixa línia,
      `matching_rule` retorna la creada més recentment (research.md, decisió 2), en
      `tests/rules_integration.rs`
- [ ] T007 [P] [US1] Test: `RuleSet::add` amb la paraula clau buida o només espais es
      rebutja (FR-004), en `tests/rules_integration.rs`

### Implementation for User Story 1

- [ ] T008 [US1] Crear `src/ui/rules_panel.rs` amb un formulari per afegir una regla
      (paraula + color) i la llista de regles existents amb el seu color — depèn de T004
- [ ] T009 [US1] A `src/ui/log_view.rs`, afegir el botó "Regles" que obre/tanca el panell,
      i la memorització `rule_match: HashMap<u64, Option<usize>>` amb
      `rule_match_version` (research.md, decisió 4; data-model.md) — depèn de T003
- [ ] T010 [US1] Aplicar el color de `matching_rule` a cada línia visible en pintar-la, a
      `src/ui/log_view.rs` (FR-006, FR-007, FR-008) — depèn de T009

**Checkpoint**: US1 funcional i comprovable independentment (ressaltat en carregar i en
directe)

---

## Phase 4: User Story 2 - Filtrar el mur per nivell o paraula clau (Priority: P1)

**Goal**: activar el filtre d'una o més regles amaga la resta de línies de la vista, sense
tocar el fitxer; desactivar-lo les recupera totes.

**Independent Test**: amb una regla `ERROR` ja definida sobre un fitxer de línies mixtes,
activar-ne el filtre i comprovar que només les línies que la compleixen queden visibles.

### Tests for User Story 2 ⚠️

- [ ] T011 [P] [US2] Test: sense cap regla amb `filter = true`, `is_visible` és sempre
      cert (FR-009), en `tests/rules_integration.rs`
- [ ] T012 [P] [US2] Test: amb dues regles amb `filter = true`, `is_visible` és cert si la
      línia en compleix almenys una — OR, no AND (research.md, decisió 3), en
      `tests/rules_integration.rs`
- [ ] T013 [P] [US2] Test: desactivar una regla (`enabled = false`) li atura el filtre
      encara que `filter` fos `true` (US3, escenari 3), en `tests/rules_integration.rs`

### Implementation for User Story 2

- [ ] T014 [US2] Afegir el botó de filtre a cada regla del panell, en
      `src/ui/rules_panel.rs` — depèn de T008
- [ ] T015 [US2] Filtrar les línies no visibles en iterar `self.file.viewport.lines` dins
      `ui()` de `src/ui/log_view.rs` (FR-010, FR-013), sense tocar `ViewportCache` ni
      `LineIndex` — depèn de T010, T014
- [ ] T016 [US2] Mostrar un avís explícit quan el filtre actiu no compleix cap línia
      (FR-012), en `src/ui/log_view.rs` — depèn de T015

**Checkpoint**: US1+US2 funcionals — ressaltar i filtrar sobre les mateixes regles

---

## Phase 5: User Story 3 - Gestionar les regles durant la sessió (Priority: P2)

**Goal**: afegir, editar, (des)activar i esborrar regles des del panell, amb efecte
immediat, i que sobrevisquin a tancar i obrir fitxers diferents dins la mateixa sessió.

**Independent Test**: afegir una regla, editar-ne el color, desactivar-la sense esborrar-la
(comprovar que deixa de ressaltar i filtrar), esborrar-la (comprovar que desapareix de la
llista i del mur).

### Tests for User Story 3 ⚠️

- [ ] T017 [P] [US3] Test: editar el `keyword` o el `color` d'una regla existent no li
      canvia la posició al `Vec` (i per tant la prioritat), en `tests/rules_integration.rs`
- [ ] T018 [P] [US3] Test: esborrar una regla fa que deixi d'aparèixer a `matching_rule` i
      d'afectar `is_visible` per a qualsevol línia, en `tests/rules_integration.rs`

### Implementation for User Story 3

- [ ] T019 [US3] Afegir edició (paraula, color), (des)activació i esborrat de regla des de
      `src/ui/rules_panel.rs` (FR-003) — depèn de T014
- [ ] T020 [US3] Validar manualment (quickstart Escenari M) que `App::rules` sobreviu a
      tancar un fitxer i obrir-ne un altre — depèn de T004, T019

**Checkpoint**: totes les user stories (P1 i P2) funcionals

---

## Phase 6: Polish & Cross-Cutting Concerns

- [ ] T021 [P] Executar `cargo clippy --all-targets` i corregir els avisos
- [ ] T022 Executar els escenaris K–N de `quickstart.md` d'extrem a extrem amb el binari
      real (sota Xvfb + `xdotool`, com a les Fases 1 i 2)
- [ ] T023 Tornar a comprovar que el binari en mode `release` es manté dins el rang de
      5–15 MB (constitució) — no s'ha afegit cap dependència nova, però cal confirmar-ho

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sense dependències
- **Foundational (Phase 2)**: depèn de Setup — bloqueja totes les user stories
- **User Stories (Phase 3-5)**: totes depenen de Foundational
  - US1 no depèn de cap altra story
  - US2 reutilitza el panell de regles introduït per US1 i hi afegeix el toggle de filtre
  - US3 amplia el mateix panell amb edició/(des)activació/esborrat; sense US1 no hi hauria
    cap regla a gestionar
- **Polish (Phase 6)**: depèn de totes les user stories desitjades

### Within Each User Story

- Els tests s'escriuen i fallen abans d'implementar
- La lògica de `rules/` (sense `egui`) abans de la GUI (`ui/rules_panel.rs`,
  `ui/log_view.rs`) que hi depèn

### Parallel Opportunities

- Dins de US1: T005-T007 (tests) en paral·lel
- Dins de US2: T011-T013 (tests) en paral·lel
- Dins de US3: T017-T018 (tests) en paral·lel

---

## Implementation Strategy

### MVP First (User Story 1 sola)

1. Setup + Foundational
2. User Story 1
3. **ATURAR-SE I VALIDAR**: definir una regla `ERROR` i comprovar que ressalta les línies
   que la compleixen, tant ja carregades com en directe

### Incremental Delivery

1. Setup + Foundational → `RuleSet` llest, posseït per `App`
2. + US1 → veure d'un cop d'ull què importa (MVP)
3. + US2 → amagar la resta i centrar-se només en el que importa
4. + US3 → gestionar les regles sense limitar-se a la que ve fixada al codi

---

## Notes

- `[P]` = fitxers diferents, sense dependències pendents
- Fer commit després de cada tasca o grup lògic
- Aturar-se a cada checkpoint per validar la story independentment
