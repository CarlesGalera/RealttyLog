---

description: "Task list template for feature implementation"
---

# Tasks: Configuració portable de les regles

**Input**: Design documents from `/specs/004-configuracio-portable/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: plan.md decideix cobrir `config::load_from`/`save_to` amb `cargo test`, aïllat
de la GUI i de `current_exe()` real (mateix patró que `format/` i `rules/`).

**Organization**: les tasques s'agrupen per user story per poder-les implementar i provar
de manera independent.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: es pot fer en paral·lel (fitxers diferents, sense dependències pendents)
- **[Story]**: a quina user story pertany (US1-US3, segons spec.md)

## Path Conventions

Crate binari únic de Rust (plan.md): `src/` i `tests/` a l'arrel del repositori.

---

## Phase 1: Setup

- [X] T001 Afegir `serde` amb la característica `derive` a `Cargo.toml` (research.md,
      decisió 1)

---

## Phase 2: Foundational (Blocking Prerequisites)

**⚠️ CRITICAL**: cap user story pot començar fins que aquesta fase estigui completa

- [X] T002 [P] Afegir `#[derive(Serialize, Deserialize)]` a `RgbColor` en
      `src/rules/color.rs` — depèn de T001
- [X] T003 [P] Afegir `#[derive(Serialize, Deserialize)]` a `HighlightRule` i implementar
      `RuleSet::from_rules(Vec<HighlightRule>) -> Self` (`version = 0`) en
      `src/rules/mod.rs` — depèn de T001
- [X] T004 Crear `src/config.rs` amb `load_from(path)`, `save_to(path, rules)`,
      `config_path()`, `load()` i `save()` (research.md, decisions 2, 4, 5;
      data-model.md) — depèn de T002, T003
- [X] T005 Declarar `pub mod config;` a `src/lib.rs`

**Checkpoint**: fonaments llestos

---

## Phase 3: User Story 1 - Les regles hi són en tornar a obrir l'aplicació (Priority: P1) 🎯 MVP

**Goal**: definir regles, tancar l'aplicació del tot, tornar-la a obrir, i trobar-les tal
com es van deixar.

**Independent Test**: definir dues o tres regles, tancar l'aplicació, tornar-la a obrir, i
comprovar que hi són amb els mateixos valors.

### Tests for User Story 1 ⚠️

- [X] T006 [P] [US1] Test: desar un `RuleSet` i tornar-lo a carregar des del mateix path
      reprodueix les mateixes regles (paraula, color, actiu, filtre) (FR-001, SC-001), en
      `tests/config_integration.rs`
- [X] T007 [P] [US1] Test: carregar des d'un path que no existeix retorna un `RuleSet`
      buit sense error (FR-005), en `tests/config_integration.rs`

### Implementation for User Story 1

- [X] T008 [US1] Afegir `last_saved_version: u64` a `App` i un constructor `App::new()`
      que crida `config::load()` (research.md, decisió 5) en `src/app.rs` — depèn de T004
- [X] T009 [US1] Canviar `#[derive(Default)]` d'`App` per un `impl Default` que crida
      `Self::new()`, en `src/app.rs` — depèn de T008
- [X] T010 [US1] A `App::ui()`, desar quan `rules.version() != last_saved_version`
      (research.md, decisió 3) en `src/app.rs` — depèn de T008

**Checkpoint**: US1 funcional i comprovable independentment (tancar i reobrir conserva les
regles)

---

## Phase 4: User Story 2 - La configuració viatja amb l'executable (Priority: P1)

**Goal**: copiar l'executable i el fitxer de configuració a un altre directori preserva
les regles; copiar només l'executable arrenca net.

**Independent Test**: copiar l'executable i `realttylog-rules.json` a un directori nou,
executar-lo des d'allà, i comprovar que les regles hi són.

### Tests for User Story 2 ⚠️

- [X] T011 [P] [US2] Test: `config_path()` es resol relatiu al directori pare de
      `current_exe()`, mai a un directori fix independent de la ubicació de l'executable
      (FR-008), en `tests/config_integration.rs`

### Implementation for User Story 2

- [X] T012 [US2] Verificar manualment (quickstart Escenari P) que copiar l'executable i el
      fitxer de configuració a un directori nou preserva les regles, i que copiar només
      l'executable arrenca sense cap regla — depèn de T004, T010

**Checkpoint**: US1+US2 funcionals — la configuració sobreviu i viatja amb l'executable

---

## Phase 5: User Story 3 - Un fitxer de configuració trencat no impedeix treballar (Priority: P2)

**Goal**: un fitxer de configuració il·legible, absent, o amb una regla mal formada no
impedeix mai arrencar ni carregar la resta de regles vàlides.

**Independent Test**: escriure contingut invàlid al fitxer de configuració i comprovar que
l'aplicació arrenca amb normalitat, sense regles i sense cap diàleg bloquejant.

### Tests for User Story 3 ⚠️

- [X] T013 [P] [US3] Test: carregar des d'un fitxer amb contingut que no és JSON vàlid
      retorna un `RuleSet` buit sense pànic (FR-006), en `tests/config_integration.rs`
- [X] T014 [P] [US3] Test: un array JSON amb una regla vàlida i una amb el camp `color`
      absent carrega només la vàlida (FR-007), en `tests/config_integration.rs`
- [X] T015 [P] [US3] Test: desar en un path dins d'un directori inexistent no fa pànic
      (FR-003, l'error s'ignora), en `tests/config_integration.rs`

### Implementation for User Story 3

- [X] T016 [US3] Implementar la deserialització element a element amb `filter_map`
      (research.md, decisió 4) en `src/config.rs` — depèn de T004 (ja cobert per T004 si
      s'implementa directament; aquesta tasca queda com a verificació dels tests T013-T015)

**Checkpoint**: totes les user stories (P1 i P2) funcionals

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T017 [P] Executar `cargo clippy --all-targets` i corregir els avisos
- [X] T018 Executar els escenaris O–Q de `quickstart.md` d'extrem a extrem amb el binari
      real (sota Xvfb + `xdotool`, com a les Fases 1-3), incloent-hi la còpia real de
      l'executable a un directori nou (Escenari P)
- [X] T019 Tornar a comprovar que el binari en mode `release` es manté dins el rang de
      5–15 MB (constitució) després d'afegir `serde` amb `derive`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sense dependències
- **Foundational (Phase 2)**: depèn de Setup — bloqueja totes les user stories
- **User Stories (Phase 3-5)**: totes depenen de Foundational
  - US1 no depèn de cap altra story
  - US2 reutilitza `config_path()` introduït per US1/Foundational; és sobretot validació
    manual, no codi nou
  - US3 reforça la implementació de `config::load_from` que ja introdueix Foundational
    (T004): els tests en verifiquen la resiliència, no n'afegeixen de nova si T004 ja ho
    fa bé
- **Polish (Phase 6)**: depèn de totes les user stories desitjades

### Within Each User Story

- Els tests s'escriuen i fallen abans d'implementar
- La lògica de `config.rs` (sense GUI) abans de connectar-la a `App`

### Parallel Opportunities

- Dins de Foundational: T002-T003 (derives) en paral·lel
- Dins de US1: T006-T007 (tests) en paral·lel
- Dins de US3: T013-T015 (tests) en paral·lel

---

## Implementation Strategy

### MVP First (User Story 1 sola)

1. Setup + Foundational
2. User Story 1
3. **ATURAR-SE I VALIDAR**: definir regles, tancar l'aplicació del tot, tornar-la a obrir,
   comprovar que hi són

### Incremental Delivery

1. Setup + Foundational → `config.rs` llest, `RuleSet` (de)serialitzable
2. + US1 → les regles sobreviuen a tancar l'aplicació (MVP)
3. + US2 → confirmar que la portabilitat és certa, no només teòrica
4. + US3 → un fitxer trencat mai bloqueja l'eina principal

---

## Notes

- `[P]` = fitxers diferents, sense dependències pendents
- Fer commit després de cada tasca o grup lògic
- Aturar-se a cada checkpoint per validar la story independentment
