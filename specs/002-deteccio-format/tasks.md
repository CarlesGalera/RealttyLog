---

description: "Task list template for feature implementation"
---

# Tasks: Detecció i formatatge de payloads

**Input**: Design documents from `/specs/002-deteccio-format/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: plan.md decideix cobrir `format/` amb `cargo test`, aïllat de la GUI.

**Organization**: les tasques s'agrupen per user story per poder-les implementar i provar
de manera independent.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: es pot fer en paral·lel (fitxers diferents, sense dependències pendents)
- **[Story]**: a quina user story pertany (US1-US3, segons spec.md)

## Path Conventions

Crate binari únic de Rust (plan.md): `src/` i `tests/` a l'arrel del repositori.

---

## Phase 1: Setup

- [X] T001 Afegir a `Cargo.toml` les dependències `serde_json`, `quick-xml` i `base64`
      (research.md, decisions 1, 2 i 4)

---

## Phase 2: Foundational (Blocking Prerequisites)

**⚠️ CRITICAL**: cap user story pot començar fins que aquesta fase estigui completa

- [X] T002 [P] Crear `src/format/styled.rs` amb `TokenKind` i `StyledLine` (research.md,
      decisió 5; data-model.md)
- [X] T003 [P] Crear `src/format/mod.rs` amb `PayloadKind` i `DetectedPayload`
      (data-model.md) i declarar els submòduls

**Checkpoint**: fonaments llestos

---

## Phase 3: User Story 1 - Reconèixer d'un cop d'ull quines línies porten un payload (Priority: P1) 🎯 MVP

**Goal**: marcar amb un indicador les línies que contenen un JSON, XML, HTML o JWT vàlids,
sense tocar el text condensat del mur.

**Independent Test**: amb un fitxer que barreja línies de text pla i línies amb un JSON o
XML complet, obrir-lo i comprovar que només les segones porten l'indicador.

### Tests for User Story 1 ⚠️

- [X] T004 [P] [US1] Test: un JSON vàlid es detecta i una clau solta invàlida no (FR-001,
      FR-004), en `tests/format_integration.rs`
- [X] T005 [P] [US1] Test: un XML ben format es detecta com a `Xml` (FR-001), en
      `tests/format_integration.rs`
- [X] T006 [P] [US1] Test: un fragment HTML permissiu (`<br>` sense tancar) es detecta com
      a `Html` (Edge Case), en `tests/format_integration.rs`
- [X] T007 [P] [US1] Test: la forma d'un JWT es detecta com a `Jwt` (FR-011), en
      `tests/format_integration.rs`
- [X] T008 [P] [US1] Test: una línia de text pla no es detecta (FR-004), en
      `tests/format_integration.rs`

### Implementation for User Story 1

- [X] T009 [P] [US1] Implementar la detecció JSON (validesa via `serde_json::from_str`) en
      `src/format/json.rs` (depèn de T003)
- [X] T010 [P] [US1] Implementar la detecció XML estricta i HTML permissiva amb
      `quick-xml` (research.md, decisions 2-3) en `src/format/xml.rs` (depèn de T003)
- [X] T011 [P] [US1] Implementar la detecció de la forma d'un JWT (tres segments
      base64url, sense descodificar encara) en `src/format/jwt.rs` (depèn de T003)
- [X] T012 [US1] Implementar `detect::detect()`, que prova JSON, XML, HTML i JWT en ordre
      (FR-005) en `src/format/detect.rs` (depèn de T009-T011)
- [X] T013 [US1] Afegir la memorització `detected: HashMap<u64, Option<PayloadKind>>` i
      l'indicador visual a `src/ui/log_view.rs` (depèn de T012; research.md, decisió 6)

**Checkpoint**: US1 funcional i comprovable independentment (indicador visible, mur intacte)

---

## Phase 4: User Story 2 - Desplegar el payload amb indentació i ressaltat (Priority: P1)

**Goal**: clicar l'indicador d'una línia mostra el payload JSON/XML/HTML formatat, i es pot
tornar a condensar.

**Independent Test**: amb una línia que conté un JSON conegut, clicar-ne l'indicador i
comprovar que es mostra indentat sense perdre cap dada.

### Tests for User Story 2 ⚠️

- [X] T014 [P] [US2] Test: un JSON es formata a `Vec<StyledLine>` conservant totes les
      dades de l'original (FR-010), en `tests/format_integration.rs`
- [X] T015 [P] [US2] Test: un XML/HTML es formata indentat per nivell d'imbricació
      (Acceptance Scenario 2), en `tests/format_integration.rs`

### Implementation for User Story 2

- [X] T016 [US2] Implementar `Value` → `Vec<StyledLine>` amb `TokenKind` per tipus de dada
      en `src/format/json.rs` (depèn de T009)
- [X] T017 [US2] Implementar tokens XML/HTML → `Vec<StyledLine>` indentats per nivell en
      `src/format/xml.rs` (depèn de T010)
- [X] T018 [US2] Afegir la memorització `expanded: HashMap<u64, Vec<StyledLine>>`, l'acció
      de desplegar/condensar independent per línia, i el mapeig `TokenKind` → `Color32` en
      `src/ui/log_view.rs` (depèn de T016, T017, T013)

**Checkpoint**: US1+US2 funcionals — detectar i desplegar JSON/XML/HTML

---

## Phase 5: User Story 3 - Descodificar un JWT (Priority: P2)

**Goal**: clicar l'indicador d'una línia amb un JWT mostra la capçalera i el payload
descodificats com a JSON.

**Independent Test**: amb una línia que conté un JWT conegut, clicar-ne l'indicador i
comprovar que capçalera i payload es mostren amb els valors correctes.

### Tests for User Story 3 ⚠️

- [X] T019 [P] [US3] Test: un JWT conegut decodifica capçalera i payload als valors
      exactes (SC-005), en `tests/format_integration.rs`
- [X] T020 [P] [US3] Test: un JWT amb el payload corromput mostra un avís en lloc de
      trencar-se (Acceptance Scenario 3), en `tests/format_integration.rs`

### Implementation for User Story 3

- [X] T021 [US3] Implementar la descodificació base64url dels dos primers segments i el
      parsing com a JSON, reutilitzant `format/json.rs` (research.md, decisió 4) en
      `src/format/jwt.rs` (depèn de T011, T016)
- [X] T022 [US3] Mostrar la signatura tal com és, marcada com a no desxifrable (FR-012) en
      `src/format/jwt.rs`
- [X] T023 [US3] Connectar el tipus `Jwt` al desplegament (dos blocs JSON) en
      `src/ui/log_view.rs` (depèn de T018, T021)

**Checkpoint**: totes les user stories (P1 i P2) funcionals

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T024 [P] Executar `cargo clippy --all-targets` i corregir els avisos
- [X] T025 Executar els escenaris G–J de `quickstart.md` d'extrem a extrem amb el binari
      real (com a la Fase 1: sota Xvfb + `xdotool`, no només llegits)
- [X] T026 Tornar a comprovar que el binari en mode `release` es manté dins el rang de
      5–15 MB (constitució) després d'afegir `serde_json`, `quick-xml` i `base64`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sense dependències
- **Foundational (Phase 2)**: depèn de Setup — bloqueja totes les user stories
- **User Stories (Phase 3-5)**: totes depenen de Foundational
  - US1 no depèn de cap altra story
  - US2 reutilitza els tipus de detecció de US1 però formata per separat
  - US3 depèn del formatador JSON que introdueix US2 (decisió 4: reutilitzar-lo, no
    duplicar-lo)
- **Polish (Phase 6)**: depèn de totes les user stories desitjades

### Within Each User Story

- Els tests s'escriuen i fallen abans d'implementar
- Detecció (US1) abans de formatatge complet (US2)
- El nucli (`format/`) abans de la GUI (`ui/log_view.rs`) que hi depèn

### Parallel Opportunities

- Dins de US1: T004-T008 (tests) en paral·lel; T009-T011 (detecció per format) en
  paral·lel
- Dins de US2: T014-T015 (tests) en paral·lel; T016-T017 (formatatge per format) en
  paral·lel
- Dins de US3: T019-T020 (tests) en paral·lel

---

## Implementation Strategy

### MVP First (User Story 1 sola)

1. Setup + Foundational
2. User Story 1
3. **ATURAR-SE I VALIDAR**: obrir un fitxer amb JSON/XML/HTML/JWT barrejats amb text pla i
   comprovar que només les línies correctes porten l'indicador

### Incremental Delivery

1. Setup + Foundational → base llesta
2. + US1 → saber quines línies val la pena mirar (MVP)
3. + US2 → llegir-les de veritat, indentades i amb colors
4. + US3 → JWT descodificat reutilitzant el mateix formatador

---

## Notes

- `[P]` = fitxers diferents, sense dependències pendents
- Fer commit després de cada tasca o grup lògic
- Aturar-se a cada checkpoint per validar la story independentment
