# Implementation Plan: Detecció i formatatge de payloads

**Branch**: `002-deteccio-format` | **Date**: 2026-08-16 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-deteccio-format/spec.md`

## Summary

Detectar, línia a línia dins la finestra ja carregada (Fase 1), si hi ha un payload JSON,
XML, HTML o JWT, marcar-ho amb un indicador sense tocar el text condensat, i permetre
desplegar-lo inline amb indentació i ressaltat per tipus de dada. Aproximació tècnica:
`serde_json` per JSON, `quick-xml` per XML i HTML (en mode permissiu per HTML), `base64`
per als segments d'un JWT. La detecció es memoritza per línia i el format complet només es
calcula quan l'usuari desplega una línia concreta, perquè cap de les dues coses interfereixi
amb el seguiment en directe.

## Technical Context

**Language/Version**: Rust, edició 2021, canal estable més recent (continuació de la Fase 1)

**Primary Dependencies**: `serde_json` (parsing i valors JSON) · `quick-xml` (tokenització
XML/HTML) · `base64` (segments d'un JWT) — s'afegeixen a les de la Fase 1 (`eframe`/`egui`,
`notify`, `ignore`, `grep-searcher`/`grep-regex`)

**Storage**: N/A — cap canvi respecte la Fase 1

**Testing**: `cargo test` per a la detecció i el formatatge (`format/`), aïllats de la GUI,
amb casos coneguts de JSON/XML/HTML/JWT vàlids i invàlids

**Target Platform**: Windows 10/11 (prioritari) · Linux (secundari) — sense canvis

**Project Type**: aplicació d'escriptori (binari únic amb GUI) — sense canvis

**Performance Goals**: desplegar un JSON de 100 KB en <200 ms (SC-002); la detecció no ha
d'alentir el mur ni el seguiment en directe (SC-003)

**Constraints**: la detecció es limita a les línies carregades a `ViewportCache` (FR-014);
cap dependència nova que trenqui el rang de 5–15 MB del binari (es tornarà a mesurar, com a
la Fase 1 T043)

**Scale/Scope**: detecció memoritzada per línia; format complet calculat i memoritzat només
per a les línies que l'usuari desplega, mai per a totes les carregades

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principi | Com el compleix aquest pla | Estat |
|---|---|---|
| I. Portabilitat sense dependències | `serde_json`, `quick-xml` i `base64` són crates purs de Rust, sense dependències de sistema; es compilen dins el mateix binari únic | ✅ |
| II. Multiplataforma des d'un sol codi base | Les tres dependències noves són multiplataforma per naturalesa (parsing de text, sense res específic de SO) | ✅ |
| III. Rendiment natiu i memòria acotada | Detecció memoritzada i acotada a la finestra carregada (FR-014); el format complet només es calcula i es guarda per a línies desplegades explícitament, mai per a totes | ✅ |
| IV. Interfície gràfica, no de terminal | El desplegament és inline dins la mateixa finestra `eframe` ja existent | ✅ |
| V. Desenvolupament dirigit per especificacions | Aquest pla parteix de `spec.md`, ja validat | ✅ |

Cap violació. La secció *Complexity Tracking* queda buida.

**Re-avaluació post-disseny (Fase 1)**: separar `format/` (lògica de detecció i formatatge,
sense dependència d'`egui`) de la representació que consumeix `ui/` reforça el principi V
del `Structure Decision` original (`ui/` depèn de `search/`/`tailer/`, mai al revés): aquí
`ui/` passa a dependre també de `format/`, seguint el mateix patró.

## Project Structure

### Documentation (this feature)

```text
specs/002-deteccio-format/
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
├── format/
│   ├── mod.rs
│   ├── detect.rs             # Punt d'entrada: detecta JWT, JSON, XML o HTML en una línia
│   ├── json.rs                # Parsing amb serde_json + conversió a línies estilades
│   ├── xml.rs                 # Tokenització amb quick-xml (mode estricte XML / permissiu HTML)
│   ├── jwt.rs                 # Detecció de la forma d'un JWT + descodificació base64url
│   └── styled.rs              # Tipus compartit: StyledLine, TokenKind (sense dependència d'egui)
├── search/                     # Fase 1, sense canvis
├── tailer/                     # Fase 1, sense canvis
└── ui/
    ├── search_view.rs          # Fase 1, sense canvis
    └── log_view.rs              # Amplia cada línia amb l'indicador i el desplegament

tests/
└── format_integration.rs       # Detecció i formatatge contra payloads coneguts (vàlids i invàlids)
```

**Structure Decision**: `format/` és un mòdul nou, paral·lel a `search/` i `tailer/`, sense
dependència d'`egui`: exposa `StyledLine`/`TokenKind` com a tipus propis perquè `ui/` els
tradueixi a colors concrets. `log_view.rs` és l'únic fitxer de la Fase 1 que es toca —
`search_view.rs`, `search/` i `tailer/` queden intactes.

## Complexity Tracking

Sense violacions de la constitució. Cap complexitat que calgui justificar.
