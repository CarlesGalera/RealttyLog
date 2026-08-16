---

description: "Task list template for feature implementation"
---

# Tasks: Cerca i tailing

**Input**: Design documents from `/specs/001-cerca-i-tailing/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: plan.md ja decideix cobrir `search/` i `tailer/` amb `cargo test`, aïllats de la
GUI (Technical Context → Testing), així que els tasques de test hi són incloses.

**Organization**: les tasques s'agrupen per user story per poder-les implementar i provar
de manera independent.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: es pot fer en paral·lel (fitxers diferents, sense dependències pendents)
- **[Story]**: a quina user story pertany (US1–US5, segons spec.md)
- Cada tasca inclou el camí de fitxer exacte

## Path Conventions

Crate binari únic de Rust (plan.md, Structure Decision): `src/` i `tests/` a l'arrel del
repositori.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: inicialització del projecte Rust

- [ ] T001 Crear l'esquelet del crate binari (`Cargo.toml`, `src/main.rs` mínim) segons
      l'estructura de `plan.md`
- [ ] T002 [P] Afegir a `Cargo.toml` les dependències `eframe`/`egui`, `notify`, `ignore`,
      `grep-searcher` i `grep-matcher` (research.md, decisions 1 i 4)
- [ ] T003 [P] Configurar `rustfmt` i `clippy` com a base de format i lint del projecte

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: infraestructura compartida per totes les user stories

**⚠️ CRITICAL**: cap user story pot començar fins que aquesta fase estigui completa

- [ ] T004 Arrencar la finestra `eframe` i l'estat base de l'aplicació en `src/main.rs` i
      `src/app.rs`
- [ ] T005 Definir l'estat de vista de l'aplicació (vista de directori/cerca vs. vista de
      fitxer) en `src/app.rs`, perquè les user stories següents hi puguin commutar
- [ ] T006 [P] Implementar la decodificació UTF-8 amb pèrdua compartida per cerca i
      seguiment (research.md, decisió 9) en `src/encoding.rs`
- [ ] T007 [P] Crear els esquelets de mòdul `src/search/mod.rs` i `src/tailer/mod.rs`
      segons l'estructura de `plan.md`

**Checkpoint**: fonaments llestos — ja es pot començar qualsevol user story

---

## Phase 3: User Story 1 - Cercar text a tot un directori de logs (Priority: P1) 🎯 MVP

**Goal**: obrir un directori de logs i cercar-hi un text a tots els fitxers (incloent-hi
subdirectoris), veient una llista de resultats amb fitxer i fragment de context.

**Independent Test**: amb un directori de diversos fitxers de text on només un conté una
paraula concreta, cercar-la i comprovar que la llista de resultats assenyala el fitxer
correcte, sense cap altra funcionalitat.

### Tests for User Story 1 ⚠️

> Escriure aquests tests primer, comprovar que fallen abans d'implementar

- [ ] T008 [P] [US1] Test d'integració: una cerca en un directori multi-fitxer retorna els
      fitxers correctes, en `tests/search_integration.rs`
- [ ] T009 [P] [US1] Test d'integració: cancel·lar una cerca en curs conserva els resultats
      trobats fins aquell moment (FR-005), en `tests/search_integration.rs`
- [ ] T010 [P] [US1] Test d'integració: els fitxers no llegibles o binaris es descarten
      sense aturar la cerca (FR-007), en `tests/search_integration.rs`

### Implementation for User Story 1

- [ ] T011 [P] [US1] Implementar `LogDirectory` (llistat de fitxers via `ignore`, incl.
      subdirectoris, FR-001) en `src/search/directory.rs`
- [ ] T012 [P] [US1] Implementar el tipus `SearchMatch` (fitxer, offset, fragment de
      context, FR-004) en `src/search/result.rs`
- [ ] T013 [US1] Implementar `SearchQuery` i el motor de cerca cancel·lable en pool de fils
      amb `grep-searcher`/`grep-matcher` (FR-002, FR-003, FR-005; research.md, decisió 2) en
      `src/search/engine.rs` (depèn de T011, T012)
- [ ] T014 [US1] Acotar el nombre de resultats mostrats amb indicador de "refina la cerca"
      (FR-008; research.md, decisió 3) en `src/search/engine.rs`
- [ ] T015 [US1] Implementar la vista de cerca (camp de text, llistat de directori, llista
      de resultats amb context) en `src/ui/search_view.rs` (depèn de T013)
- [ ] T016 [US1] Connectar la vista de cerca a l'estat de l'aplicació (depèn de T005, T015)

**Checkpoint**: US1 funcional i comprovable independentment (cercar i veure resultats,
encara sense poder-hi saltar)

---

## Phase 4: User Story 2 - Saltar del resultat de cerca al fitxer i la línia exactes (Priority: P1)

**Goal**: en clicar un resultat de cerca, obrir el fitxer corresponent posicionat
exactament a la línia trobada, amb context al voltant.

**Independent Test**: amb una cerca ja feta i almenys un resultat, clicar-lo i comprovar que
s'obre el fitxer amb la línia trobada visible, sense desplaçar-se manualment.

### Tests for User Story 2 ⚠️

- [ ] T017 [P] [US2] Test d'integració: obrir un fitxer en un offset concret mostra la línia
      i el context correctes, en `tests/tailer_integration.rs`

### Implementation for User Story 2

- [ ] T018 [P] [US2] Implementar el nucli de `FollowedFile` (camí, estat, read_offset) en
      `src/tailer/mod.rs`
- [ ] T019 [P] [US2] Implementar `LineIndex` (checkpoints dispersos d'offset, research.md
      decisió 6) en `src/tailer/index.rs`
- [ ] T020 [US2] Implementar la lectura per offset amb decodificació lossy (FR-022) en
      `src/tailer/reader.rs` (depèn de T006, T018, T019)
- [ ] T021 [US2] Implementar `ViewportCache` (finestra acotada, recàrrega des de
      `LineIndex` en desplaçar-se'n fora) en `src/tailer/viewport.rs` (depèn de T019, T020)
- [ ] T022 [US2] Implementar la vista de fitxer (llista de línies desplaçable, context al
      voltant d'un salt, FR-010) en `src/ui/log_view.rs` (depèn de T021)
- [ ] T023 [US2] Connectar el clic d'un `SearchMatch` a obrir `FollowedFile` a
      `byte_offset` (FR-009) (depèn de T012, T018, T022)
- [ ] T024 [US2] Implementar "tornar a la llista de resultats" sense repetir la cerca
      (FR-011) (depèn de T016, T022)

**Checkpoint**: US1+US2 funcionals: cercar, saltar-hi, veure context

---

## Phase 5: User Story 3 - Seguir en directe el fitxer un cop localitzat (Priority: P1)

**Goal**: un fitxer obert (des d'una cerca o directament) es segueix en directe, mostrant
línies noves a mesura que es generen.

**Independent Test**: amb un script que afegeix línies a un fitxer mentre RealttyLog el té
obert, comprovar que cada línia nova hi apareix sense recarregar.

### Tests for User Story 3 ⚠️

- [ ] T025 [P] [US3] Test d'integració: línies noves apareixen en seguir un fitxer en
      creixement, en `tests/tailer_integration.rs`
- [ ] T026 [P] [US3] Test d'integració: una rotació (truncament i reemplaçament) es detecta
      i el seguiment continua (FR-020), en `tests/tailer_integration.rs`

### Implementation for User Story 3

- [ ] T027 [P] [US3] Implementar el watcher basat en `notify` (research.md, decisió 4) en
      `src/tailer/watcher.rs`
- [ ] T028 [US3] Implementar la detecció de rotació/truncament (research.md, decisió 5) en
      `src/tailer/rotation.rs` (depèn de T018, T027)
- [ ] T029 [US3] Implementar el posicionament inicial al final llegint per blocs enrere
      (FR-013; research.md, decisió 7) en `src/tailer/reader.rs` (depèn de T020)
- [ ] T030 [US3] Connectar l'arribada de línies en directe a `ViewportCache`/`LineIndex` via
      canal `mpsc` (research.md, decisió 8) en `src/tailer/mod.rs` (depèn de T021, T027)
- [ ] T031 [US3] Implementar obrir un fitxer directament, sense passar per cerca (FR-012),
      en `src/ui/search_view.rs` (depèn de T029)
- [ ] T032 [US3] Implementar l'estat "no disponible" i la represa en tornar a haver-hi
      accés (FR-021) en `src/tailer/watcher.rs` (depèn de T027)

**Checkpoint**: seguiment en directe funcional, tant des de cerca com obrint directament

---

## Phase 6: User Story 4 - Repassar l'historial sense perdre el fil (Priority: P1)

**Goal**: en desplaçar-se cap amunt durant un seguiment en directe, l'autoscroll es pausa
sol; tornar-hi el reprèn amb una sola acció.

**Independent Test**: amb un fitxer que rep línies noves contínuament, desplaçar-se amunt i
comprovar que la vista es queda quieta; tornar avall i comprovar que el seguiment es
reprèn.

### Tests for User Story 4 ⚠️

- [ ] T033 [P] [US4] Test unitari de la lògica de transició d'estat directe/pausat (FR-017,
      FR-018), en `tests/tailer_integration.rs`

### Implementation for User Story 4

- [ ] T034 [US4] Implementar les transicions Live↔Paused en desplaçar la vista (FR-017,
      FR-018) en `src/ui/log_view.rs` (depèn de T022)
- [ ] T035 [US4] Implementar l'indicador visible "en directe"/"pausat" i l'acció "tornar al
      directe" (FR-019) en `src/ui/log_view.rs` (depèn de T034)
- [ ] T036 [US4] Implementar l'indicador de línies noves arribades mentre està pausat
      (acceptance scenario 3) en `src/ui/log_view.rs` (depèn de T034)

**Checkpoint**: totes les user stories P1 funcionals, per separat i juntes

---

## Phase 7: User Story 5 - Fitxers grans sense penalització (Priority: P2)

**Goal**: cercar i seguir funcionen igual de bé amb fitxers de diversos gigabytes, sense
exhaurir memòria ni bloquejar-se.

**Independent Test**: generar un directori amb fitxers de diversos GB, cercar-hi text i
obrir-ne un en directe, mesurant temps i consum de memòria.

### Tests for User Story 5 ⚠️

- [ ] T037 [P] [US5] Test de rendiment (marcat `#[ignore]`, no s'executa en cada `cargo
      test`): cerca sobre ~20 GB de fitxers mostra el primer resultat en <3 s (SC-002), en
      `tests/search_integration.rs`
- [ ] T038 [P] [US5] Test de rendiment (`#[ignore]`): obertura directa d'un fitxer de 5 GB
      es posiciona al final en <2 s (SC-004), en `tests/tailer_integration.rs`

### Implementation for User Story 5

- [ ] T039 [US5] Ajustar la mida del pool de fils de cerca i dels blocs de lectura
      (research.md, decisions 2 i 7) segons els resultats dels tests de rendiment, en
      `src/search/engine.rs` i `src/tailer/reader.rs`
- [ ] T040 [US5] Validar manualment SC-005 (memòria acotada 30 min seguint un fitxer gran) i
      SC-009 (historial complet accessible) seguint l'Escenari F de `quickstart.md` — no
      s'automatitza per no pagar 30 minuts de CI a cada execució

**Checkpoint**: tota la Fase 1 (cerca i tailing) validada a escala real

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: qualitat transversal a totes les user stories

- [ ] T041 [P] Executar `cargo clippy --all-targets` i corregir els avisos
- [ ] T042 Executar tots els escenaris de `quickstart.md` (A–F) d'extrem a extrem
- [ ] T043 Comprovar que el binari en mode `release` es manté dins el rang de 5–15 MB
      (constitució, Restriccions tècniques)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sense dependències — pot començar de seguida
- **Foundational (Phase 2)**: depèn de Setup — BLOQUEJA totes les user stories
- **User Stories (Phase 3-7)**: totes depenen de Foundational
  - US1 no depèn de cap altra story
  - US2 depèn dels tipus que produeix US1 (`SearchMatch`) i de la seva pròpia vista de
    fitxer
  - US3 reutilitza `FollowedFile`/`ViewportCache`/`LineIndex` que introdueix US2
  - US4 és una capa d'estat sobre la vista de fitxer que introdueix US2/US3
  - US5 no afegeix funcionalitat nova: valida i ajusta el rendiment de US1-US4 a escala
- **Polish (Phase 8)**: depèn de totes les user stories desitjades

### Within Each User Story

- Els tests s'escriuen i fallen abans d'implementar
- Les entitats/tipus abans dels serveis que les fan servir
- El nucli (`search/`, `tailer/`) abans de la GUI (`ui/`) que hi depèn
- La story es dona per completa abans de passar a la següent prioritat

### Parallel Opportunities

- Totes les tasques `[P]` de Setup i Foundational es poden fer en paral·lel
- Dins de US1: T008-T010 (tests) en paral·lel; T011-T012 (tipus base) en paral·lel
- Dins de US2: T017 (test) en paral·lel; T018-T019 (tipus base) en paral·lel
- Dins de US3: T025-T026 (tests) en paral·lel; T027 en paral·lel amb la resta de Setup de la
  story
- Dins de US5: T037-T038 (tests de rendiment) en paral·lel

---

## Parallel Example: User Story 1

```bash
# Tests de la User Story 1 junts:
Task: "Test d'integració: cerca multi-fitxer en tests/search_integration.rs"
Task: "Test d'integració: cancel·lació de cerca en tests/search_integration.rs"
Task: "Test d'integració: fitxers no llegibles descartats en tests/search_integration.rs"

# Tipus base de la User Story 1 junts:
Task: "LogDirectory en src/search/directory.rs"
Task: "SearchMatch en src/search/result.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 sola)

1. Completar Phase 1: Setup
2. Completar Phase 2: Foundational (bloqueja totes les stories)
3. Completar Phase 3: User Story 1
4. **ATURAR-SE I VALIDAR**: provar US1 independentment (cercar un directori i veure
   resultats)
5. Ja aporta valor real sol: saber en quin fitxer és l'error, encara sense poder-hi saltar
   ni seguir-lo

### Incremental Delivery

1. Setup + Foundational → base llesta
2. + US1 → cercar i veure resultats (MVP)
3. + US2 → saltar-hi amb context
4. + US3 → seguir en directe des d'allà
5. + US4 → repassar l'historial sense perdre el fil
6. + US5 → validat a escala de producció real (fitxers de diversos GB)

Cada increment és demostrable i no trenca l'anterior.

---

## Notes

- `[P]` = fitxers diferents, sense dependències pendents
- L'etiqueta `[Story]` lliga cada tasca a la seva user story per traçabilitat
- Verificar que els tests fallen abans d'implementar
- Fer commit després de cada tasca o grup lògic
- Aturar-se a cada checkpoint per validar la story independentment
