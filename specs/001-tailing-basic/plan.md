# Implementation Plan: Tailing bàsic

**Branch**: `001-tailing-basic` | **Date**: 2026-08-15 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-tailing-basic/spec.md`

## Summary

Aplicació d'escriptori que obre un fitxer de log, el segueix en temps real (com `tail -f`)
i el mostra en una finestra pròpia amb autoscroll que es pausa sol quan l'usuari repassa
l'historial. Aproximació tècnica: Rust + `eframe`/`egui` per a la GUI, un fil de fons que
llegeix per streaming amb el crate `notify` per detectar canvis, i un búfer de línies acotat
en memòria perquè el consum de RAM no depengui de la mida del fitxer.

## Technical Context

**Language/Version**: Rust, edició 2021, canal estable més recent

**Primary Dependencies**: `eframe`/`egui` (GUI nativa multiplataforma) · `notify`
(esdeveniments de sistema de fitxers: inotify a Linux, ReadDirectoryChangesW a Windows)

**Storage**: N/A — no hi ha persistència pròpia en aquesta fase; només es llegeix el fitxer
que l'usuari obre

**Testing**: `cargo test` per a la lògica de seguiment (rotació, búfer acotat, decodificació)
aïllada de la GUI, més validació manual amb `quickstart.md`

**Target Platform**: Windows 10/11 (prioritari) · Linux (secundari)

**Project Type**: aplicació d'escriptori (binari únic amb GUI)

**Performance Goals**: línia nova visible en <1 s (SC-001) · obertura i posicionament al
final d'un fitxer de 5 GB en <2 s (SC-002)

**Constraints**: consum de memòria <20 MB per sobre del punt de partida després de 30 min
seguint un fitxer de diversos GB (SC-003) · binari de 5–15 MB · sense runtime extern

**Scale/Scope**: un fitxer seguit per finestra · fitxers de fins a diversos GB · una sola
pantalla en aquesta fase

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principi | Com el compleix aquest pla | Estat |
|---|---|---|
| I. Portabilitat sense dependències | `eframe` i `notify` es compilen estàticament dins un únic binari; cap dels dos requereix runtime ni webview instal·lat al sistema | ✅ |
| II. Multiplataforma des d'un sol codi base | `eframe`/`egui` i `notify` ja abstreuen Windows/Linux; el codi de seguiment no necessita cap branca `#[cfg(target_os)]` més enllà de les que ja resolen aquests crates internament | ✅ |
| III. Rendiment natiu i memòria acotada | Lectura per streaming amb desplaçament (offset) en lloc de carregar el fitxer sencer; búfer de línies en memòria amb mida màxima fixa (veure research.md, decisió 3) | ✅ |
| IV. Interfície gràfica, no de terminal | Finestra `eframe`, sense cap component de terminal | ✅ |
| V. Desenvolupament dirigit per especificacions | Aquest pla parteix de `spec.md`, validat abans de tocar codi | ✅ |

Cap violació. La secció *Complexity Tracking* queda buida.

**Re-avaluació post-disseny (Fase 1)**: la decisió d'acotar l'historial en memòria a un
nombre fix de línies (research.md, decisió 3) i la de posicionar-se al final llegint per
blocs des de la cua del fitxer (decisió 4) reforcen el principi III en lloc de trencar-lo;
no introdueixen cap dependència nova.

## Project Structure

### Documentation (this feature)

```text
specs/001-tailing-basic/
├── plan.md              # Aquest fitxer
├── spec.md              # Especificació funcional
├── research.md          # Fase 0 — decisions tècniques
├── data-model.md        # Fase 1 — entitats
├── quickstart.md        # Fase 1 — validació manual
├── checklists/
│   └── requirements.md  # Validació de qualitat de l'spec
└── tasks.md              # Fase 2 — pendent de /speckit-tasks
```

No hi ha `contracts/`: RealttyLog és una aplicació d'escriptori sense interfície externa
(API, CLI d'un altre procés) que calgui documentar com a contracte en aquesta fase.

### Source Code (repository root)

```text
src/
├── main.rs                  # Punt d'entrada, arrencada d'eframe
├── app.rs                   # Estat de l'aplicació i bucle update() d'egui
├── tailer/
│   ├── mod.rs
│   ├── watcher.rs            # Subscripció a notify, esdeveniments de canvi de fitxer
│   ├── reader.rs             # Lectura incremental per offset, decodificació UTF-8 amb pèrdua
│   └── rotation.rs           # Detecció de truncament/reemplaçament del fitxer seguit
├── linebuffer.rs             # Búfer circular de línies amb capacitat fixa
└── ui/
    ├── mod.rs
    └── log_view.rs            # Vista desplaçable, lògica d'autoscroll i pausa

tests/
└── tailer_integration.rs     # Proves contra fitxers temporals: creixement, rotació, UTF-8 invàlid
```

**Structure Decision**: crate binari únic de Rust (sense workspace ni sub-crates: no cal
aquesta complexitat per a una sola aplicació d'escriptori). `tailer/` conté tota la lògica
independent de la GUI i és el que cobreixen els tests de `cargo test`; `ui/` només hi
depèn, mai al revés.

## Complexity Tracking

Sense violacions de la constitució. Cap complexitat que calgui justificar.
