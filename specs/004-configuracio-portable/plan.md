# Implementation Plan: Configuració portable de les regles

**Branch**: `004-configuracio-portable` | **Date**: 2026-08-16 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/004-configuracio-portable/spec.md`

## Summary

Un mòdul `config` nou, responsable de llegir i escriure el conjunt de regles de la Fase 3
com a JSON en un fitxer al costat de l'executable (`std::env::current_exe()`), amb `serde`
per (de)serialitzar `HighlightRule`/`RgbColor`. `App` carrega les regles en arrencar i les
torna a desar automàticament cada vegada que `RuleSet::version()` canvia respecte a
l'última versió desada — sense cap acció explícita de l'usuari (FR-002) i sense bloquejar
mai l'arrencada si el fitxer no existeix o és il·legible (FR-005, FR-006).

## Technical Context

**Language/Version**: Rust, edició 2021, canal estable més recent (continuació de les
Fases 1-3)

**Primary Dependencies**: `serde` amb la característica `derive` (nova, però `serde_json`
ja l'arrossega transitivament des de la Fase 2 — només calen les macros de derivació per
als tipus propis) — s'afegeix a les ja presents (`eframe`/`egui`, `notify`, `ignore`,
`grep-searcher`/`grep-regex`, `serde_json`, `quick-xml`, `base64`)

**Storage**: un fitxer JSON (`realttylog-rules.json`) al mateix directori que l'executable
— cap base de dades ni carpeta de configuració del sistema (FR-008)

**Testing**: `cargo test` per a `config::load_from`/`save_to`, aïllat de la GUI i de
`current_exe()` real (rebent el `Path` com a paràmetre, no resolent-lo interiorment), amb
fitxers temporals

**Target Platform**: Windows 10/11 (prioritari) · Linux (secundari) — sense canvis

**Project Type**: aplicació d'escriptori (binari únic amb GUI) — sense canvis

**Performance Goals**: desar un canvi de regla no ha d'introduir cap alentiment perceptible
(SC-004, <200 ms, mateix llindar que Fases 2-3)

**Constraints**: el fitxer de configuració MUST resoldre's relatiu a `current_exe()`, mai a
un directori de configuració de l'usuari o del sistema operatiu (FR-008); un error en
llegir o escriure MUST NOT impedir arrencar ni interrompre la sessió (FR-003, FR-006);
cap dependència nova que trenqui el rang de 5–15 MB del binari

**Scale/Scope**: només el `RuleSet` de la Fase 3 (Assumptions de l'spec); cap altra
preferència (mida de finestra, últim fitxer obert) entra en aquesta fase

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principi | Com el compleix aquest pla | Estat |
|---|---|---|
| I. Portabilitat sense dependències | `serde` (derive) és un crate pur de Rust, sense dependències de sistema, ja arrossegat transitivament per `serde_json`; el fitxer de configuració viu al costat de l'executable, no en una carpeta del sistema (FR-008), que és el que fa l'eina portable de debò | ✅ |
| II. Multiplataforma des d'un sol codi base | `std::env::current_exe()` i `std::fs` són multiplataforma; cap camí específic de SO | ✅ |
| III. Rendiment natiu i memòria acotada | Desar és una operació puntual (un cop per canvi de versió, no cada frame) sobre un fitxer petit (unes desenes de regles com a molt) | ✅ |
| IV. Interfície gràfica, no de terminal | Cap canvi a la interfície: desar i carregar són silenciosos (Assumptions de l'spec), no introdueixen cap diàleg nou | ✅ |
| V. Desenvolupament dirigit per especificacions | Aquest pla parteix de `spec.md`, ja validat | ✅ |

Cap violació. La secció *Complexity Tracking* queda buida.

**Re-avaluació post-disseny**: `config` depèn de `rules` (per (de)serialitzar
`HighlightRule`/`RgbColor`) però `rules` no sap res de `config` — mateixa direcció de
dependència d'una via que ja segueixen `format/` i `rules/` respecte a `ui/`.

## Project Structure

### Documentation (this feature)

```text
specs/004-configuracio-portable/
├── plan.md              # Aquest fitxer
├── spec.md              # Especificació funcional
├── research.md          # Fase 0 — decisions tècniques
├── data-model.md         # Fase 1 — entitats
├── quickstart.md         # Fase 1 — validació manual
├── checklists/
│   └── requirements.md  # Validació de qualitat de l'spec
└── tasks.md              # Fase 2 — pendent de /speckit-tasks
```

No hi ha `contracts/`: continua sent una aplicació d'escriptori sense interfície externa;
el format del fitxer JSON es documenta a `data-model.md` en lloc d'un contracte separat.

### Source Code (repository root)

```text
src/
├── config.rs                   # Nou: config_path(), load()/save() (current_exe() real) i
│                                # load_from(path)/save_to(path) (provables amb un path
│                                # qualsevol)
├── rules/
│   ├── mod.rs                  # Fase 3 + `RuleSet::from_rules`, `derive(Serialize,
│   │                            # Deserialize)` a HighlightRule
│   └── color.rs                # + `derive(Serialize, Deserialize)` a RgbColor
├── format/                     # Fase 2, sense canvis
├── search/                     # Fase 1, sense canvis
├── tailer/                     # Fase 1, sense canvis
├── app.rs                       # `App::new()` carrega les regles; `App::ui()` les torna a
│                                # desar quan `RuleSet::version()` canvia
├── main.rs                      # `App::default()` → `App::new()`
└── ui/                          # Fase 2-3, sense canvis

tests/
└── config_integration.rs       # Desar i tornar a carregar; fitxer absent; fitxer
                                  # il·legible; una regla amb un camp invàlid enmig d'altres
                                  # de vàlides
```

**Structure Decision**: `config.rs` és un mòdul nou al nivell arrel (com `encoding.rs`),
no un submòdul de `rules/`: `rules/` es va dissenyar a la Fase 3 sense cap dependència
externa més enllà d'`std`, i barrejar-hi la serialització en trencaria aquell límit sense
necessitat — n'hi ha prou que `HighlightRule`/`RgbColor` derivin `Serialize`/`Deserialize`
perquè `config.rs` els pugui (de)serialitzar des de fora.

## Complexity Tracking

Sense violacions de la constitució. Cap complexitat que calgui justificar.
