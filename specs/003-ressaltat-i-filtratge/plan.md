# Implementation Plan: Ressaltat per paraula clau i filtratge instantani

**Branch**: `003-ressaltat-i-filtratge` | **Date**: 2026-08-16 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-ressaltat-i-filtratge/spec.md`

## Summary

Un conjunt de regles (paraula clau + color + activa/desactivada + filtre) gestionat des
d'un panell nou, aplicat a les línies ja carregades a `ViewportCache` (Fase 1): cada línia
es ressalta amb el color de la regla de més prioritat que hi coincideix, i les línies que
no compleixen cap regla amb el filtre actiu queden amagades de la vista sense tocar el
fitxer ni l'índex. Aproximació tècnica: un mòdul `rules/` nou, sense dependència d'`egui`
(igual que `format/` a la Fase 2), amb la coincidència memoritzada per línia perquè avaluar
totes les regles contra totes les línies visibles no es refaci a cada frame.

## Technical Context

**Language/Version**: Rust, edició 2021, canal estable més recent (continuació de les Fases
1 i 2)

**Primary Dependencies**: cap de nova — reutilitza `eframe`/`egui` ja presents. La
coincidència és una simple cerca de subcadena insensible a majúscules (`str::to_lowercase`
+ `contains`), sense necessitat d'un crate de regex (Assumptions de l'spec: paraula clau
literal, no expressions regulars)

**Storage**: N/A — les regles viuen només en memòria durant la sessió (Assumptions:
persistència ajornada a la Fase 4)

**Testing**: `cargo test` per a la lògica de coincidència i prioritat (`rules/`), aïllada de
la GUI, amb casos de regles solapades, filtre buit i cerca insensible a majúscules

**Target Platform**: Windows 10/11 (prioritari) · Linux (secundari) — sense canvis

**Project Type**: aplicació d'escriptori (binari únic amb GUI) — sense canvis

**Performance Goals**: activar o desactivar un filtre actualitza la vista en <200 ms
(SC-002); ressaltar i filtrar no han d'alentir el seguiment en directe (SC-003)

**Constraints**: la coincidència de regles es limita a les línies carregades a
`ViewportCache` (mateix límit que la detecció de payloads de la Fase 2), no re-escaneja mai
el fitxer sencer; cap dependència nova que trenqui el rang de 5–15 MB del binari

**Scale/Scope**: coincidència memoritzada per línia i per "versió" del conjunt de regles
(un comptador que s'incrementa a cada canvi); es recalcula només quan la línia o les regles
canvien, mai a cada frame

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principi | Com el compleix aquest pla | Estat |
|---|---|---|
| I. Portabilitat sense dependències | Cap dependència nova: coincidència de subcadena amb `std` pur | ✅ |
| II. Multiplataforma des d'un sol codi base | Lògica de text pur, sense res específic de SO | ✅ |
| III. Rendiment natiu i memòria acotada | Coincidència limitada a `ViewportCache` (línies ja resident, mai tot el fitxer) i memoritzada per evitar recalcular a cada frame | ✅ |
| IV. Interfície gràfica, no de terminal | El panell de regles és una finestra/panell `egui` més, coherent amb la resta de la GUI | ✅ |
| V. Desenvolupament dirigit per especificacions | Aquest pla parteix de `spec.md`, ja validat | ✅ |

Cap violació. La secció *Complexity Tracking* queda buida.

**Re-avaluació post-disseny**: `rules/` segueix el mateix patró que `format/` (Fase 2) —
tipus i lògica sense dependència d'`egui`, perquè `ui/` els tradueixi a colors i widgets
concrets. `ui/` passa a dependre també de `rules/`, mai al revés.

## Project Structure

### Documentation (this feature)

```text
specs/003-ressaltat-i-filtratge/
├── plan.md              # Aquest fitxer
├── spec.md              # Especificació funcional
├── research.md          # Fase 0 — decisions tècniques
├── data-model.md         # Fase 1 — entitats
├── quickstart.md         # Fase 1 — validació manual
├── checklists/
│   └── requirements.md  # Validació de qualitat de l'spec
└── tasks.md              # Fase 2 — pendent de /speckit-tasks
```

No hi ha `contracts/`: continua sent una aplicació d'escriptori sense interfície externa.

### Source Code (repository root)

```text
src/
├── rules/
│   ├── mod.rs                 # HighlightRule, RuleSet: afegir/editar/(des)activar/esborrar,
│   │                            # coincidència amb prioritat, visibilitat per filtre
│   └── color.rs                # RgbColor senzill (sense dependència d'egui)
├── format/                     # Fase 2, sense canvis
├── search/                     # Fase 1, sense canvis
├── tailer/                     # Fase 1, sense canvis
├── app.rs                       # Passa a posseir el `RuleSet` (viu tota la sessió, no
│                                # només mentre un fitxer és obert)
└── ui/
    ├── search_view.rs          # Fase 1, sense canvis
    ├── rules_panel.rs           # Nou: llista de regles, afegir/editar/(des)activar/esborrar/filtrar
    └── log_view.rs               # Aplica ressaltat i filtre a cada línia; obre el panell

tests/
└── rules_integration.rs        # Coincidència, prioritat, filtre OR, cas insensible
```

**Structure Decision**: `rules/` és un mòdul nou, paral·lel a `format/`, sense dependència
d'`egui`. El `RuleSet` viu a `App` (no a `LogViewState`) perquè sobrevisqui a tancar i
obrir fitxers diferents dins la mateixa sessió — coherent amb l'esperit de les User Stories
1–3, que parlen de "la sessió" i no d'un fitxer concret — però es perd en tancar
l'aplicació (Assumptions: sense persistència en aquesta fase). `log_view.rs` rep el
`RuleSet` per referència mutable des d'`app.rs`, igual que ja rep `FollowedFile`.

## Complexity Tracking

Sense violacions de la constitució. Cap complexitat que calgui justificar.
