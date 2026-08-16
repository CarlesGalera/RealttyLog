# Implementation Plan: Cerca i tailing

**Branch**: `001-cerca-i-tailing` | **Date**: 2026-08-15 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-cerca-i-tailing/spec.md`

## Summary

Aplicació d'escriptori que obre un directori de logs, hi cerca text a tots els fitxers
(incloent-hi subdirectoris) sense carregar-los sencers a memòria, permet saltar d'un
resultat al fitxer i la línia exactes, i des d'allà segueix el fitxer en temps real (com
`tail -f`) amb autoscroll que es pausa sol quan l'usuari repassa l'historial. Aproximació
tècnica: Rust + `eframe`/`egui` per a la GUI, cerca amb `ignore`+`grep-searcher` (el motor
que fa servir `ripgrep`) executada en un pool de fils, i seguiment en directe amb `notify` +
un búfer de línies acotat en memòria complementat per un índex dispers d'offsets perquè
l'historial sencer sigui sempre accessible des de disc.

## Technical Context

**Language/Version**: Rust, edició 2021, canal estable més recent

**Primary Dependencies**: `eframe`/`egui` (GUI nativa multiplataforma) · `notify`
(esdeveniments de sistema de fitxers) · `ignore` + `grep-searcher`/`grep-matcher` (cerca de
text en streaming a través d'arbres de directoris, el mateix motor que `ripgrep`)

**Storage**: N/A — no hi ha persistència pròpia en aquesta fase; només es llegeixen els
fitxers que l'usuari obre o cerca

**Testing**: `cargo test` per a la lògica de cerca i seguiment (rotació, búfer acotat,
decodificació, cancel·lació de cerca) aïllada de la GUI, més validació manual amb
`quickstart.md`

**Target Platform**: Windows 10/11 (prioritari) · Linux (secundari)

**Project Type**: aplicació d'escriptori (binari únic amb GUI)

**Performance Goals**: primer resultat d'una cerca sobre 20 GB en <3 s (SC-002) · línia nova
visible en <1 s (SC-003) · obertura directa i posicionament al final d'un fitxer de 5 GB en
<2 s (SC-004)

**Constraints**: consum de memòria <20 MB per sobre del punt de partida després de 30 min
seguint un fitxer de diversos GB (SC-005) · binari de 5–15 MB · sense runtime extern · cap
fitxer es carrega mai sencer a memòria, ni en cercar ni en seguir

**Scale/Scope**: un directori obert per finestra, amb un nombre no acotat de fitxers de fins
a diversos GB cadascun · un sol fitxer seguit en directe alhora · una llista de resultats i
una vista de fitxer en aquesta fase

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principi | Com el compleix aquest pla | Estat |
|---|---|---|
| I. Portabilitat sense dependències | `eframe`, `notify`, `ignore` i `grep-searcher` es compilen estàticament dins un únic binari; cap requereix runtime ni webview instal·lat al sistema | ✅ |
| II. Multiplataforma des d'un sol codi base | Els quatre crates ja abstreuen Windows/Linux; cap branca `#[cfg(target_os)]` pròpia més enllà de les que ja resolen internament | ✅ |
| III. Rendiment natiu i memòria acotada | Cerca i seguiment tots dos per streaming amb offset, mai el fitxer sencer; búfer de línies i índex dispers amb mida màxima fixa (veure research.md) | ✅ |
| IV. Interfície gràfica, no de terminal | Finestra `eframe`, sense cap component de terminal | ✅ |
| V. Desenvolupament dirigit per especificacions | Aquest pla parteix de `spec.md`, revisat i corregit abans de tocar codi arran d'una observació de l'usuari sobre no perdre historial | ✅ |

Cap violació. La secció *Complexity Tracking* queda buida.

**Re-avaluació post-disseny (Fase 1)**: incorporar `ignore`+`grep-searcher` per a la cerca
multi-fitxer és una dependència nova respecte al pla original d'aquesta feature, però és
exactament el motor que fa servir `ripgrep` per streaming sense carregar fitxers sencers:
reforça el principi III en lloc de posar-lo en risc, i evita reimplementar des de zero una
cerca eficient en arbres de directoris.

## Project Structure

### Documentation (this feature)

```text
specs/001-cerca-i-tailing/
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
├── app.rs                   # Estat de l'aplicació (cerca vs. fitxer obert) i bucle update()
├── search/
│   ├── mod.rs
│   ├── directory.rs          # Llistat de fitxers d'un directori (amb subdirectoris)
│   ├── engine.rs             # Cerca en streaming amb ignore + grep-searcher, cancel·lable
│   └── result.rs             # Tipus Coincidència (fitxer, offset, fragment de context)
├── tailer/
│   ├── mod.rs
│   ├── watcher.rs             # Subscripció a notify, esdeveniments de canvi de fitxer
│   ├── reader.rs              # Lectura incremental per offset, decodificació UTF-8 amb pèrdua
│   ├── rotation.rs            # Detecció de truncament/reemplaçament del fitxer seguit
│   ├── index.rs               # Índex dispers d'offsets per saltar a qualsevol punt del fitxer
│   └── viewport.rs            # Finestra acotada de línies visibles, recarregada des de l'índex
└── ui/
    ├── mod.rs
    ├── search_view.rs          # Camp de cerca, llista de directori i de resultats
    └── log_view.rs             # Vista desplaçable, context al voltant d'un salt, autoscroll i pausa

tests/
├── search_integration.rs     # Proves de cerca contra directoris temporals: multi-fitxer, cancel·lació, permisos
└── tailer_integration.rs     # Proves de seguiment: creixement, rotació, salt a offset, UTF-8 invàlid
```

**Structure Decision**: crate binari únic de Rust (sense workspace ni sub-crates). `search/`
i `tailer/` són independents de la GUI i cobreixen els tests de `cargo test`; `search/`
produeix `Coincidència`, que `app.rs` tradueix en l'obertura d'un `tailer::FollowedFile`
posicionat a l'offset indicat — és el punt d'unió entre les dues User Story P1 inicials
(cercar i saltar-hi) i les tres que ja hi havia (seguir, pausar, fitxers grans). `ui/` només
depèn de `search/` i `tailer/`, mai al revés.

## Complexity Tracking

Sense violacions de la constitució. Cap complexitat que calgui justificar.
