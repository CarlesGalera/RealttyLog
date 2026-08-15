<!--
Sync Impact Report
- Canvi de versió: (cap) → 1.0.0
- Ratificació inicial de la constitució del projecte RealttyLog
- Principis definits: I. Portabilitat sense dependències · II. Multiplataforma des d'un
  sol codi base · III. Rendiment natiu i memòria acotada · IV. Interfície gràfica, no de
  terminal · V. Desenvolupament dirigit per especificacions
- Seccions afegides: Restriccions tècniques · Flux de desenvolupament · Governança
- Plantilles revisades:
  ✅ .specify/templates/plan-template.md (Constitution Check compatible, sense canvis)
  ✅ .specify/templates/spec-template.md (sense canvis necessaris)
  ✅ .specify/templates/tasks-template.md (sense canvis necessaris)
- TODOs pendents: cap
-->

# Constitució de RealttyLog

## Core Principles

### I. Portabilitat sense dependències (NO NEGOCIABLE)

Cada versió publicada MUST distribuir-se com un binari únic, sense instal·lador, sense
runtime a part (`.NET`, `Java`, `Node.js`) i sense dependre d'un navegador (WebView) que
pugui no existir al servidor de destí. L'executable MUST arrencar i funcionar en un
servidor aïllat o sense connexió a Internet.

**Motiu**: és la raó de ser del projecte. lnav ja resol el mateix problema a Linux/macOS
però no té build natiu per Windows perquè depèn de curses; RealttyLog només val la pena si
no reintrodueix cap de les dependències que fan inviables les eines actuals en producció.

### II. Multiplataforma des d'un sol codi base

El mateix codi MUST compilar i córrer de manera nativa a Windows i Linux, sense forks de
codi per plataforma més enllà de la capa d'I/O de fitxers ja abstreta per les llibreries de
base (`notify`). macOS és desitjable però no bloqueja cap decisió si mai hi arriba a haver
un conflicte tècnic amb Windows o Linux.

**Motiu**: mantenir dos projectes separats (un per Windows, un per Linux) multiplicaria el
cost de manteniment d'un projecte fet per una sola persona amb ajuda d'IA.

### III. Rendiment natiu i memòria acotada

La lectura de fitxers en creixement MUST fer-se en streaming amb buffers acotats, mai
carregant el fitxer sencer a memòria. El consum de RAM en repòs MUST mantenir-se per sota
d'uns 20 MB i créixer de manera previsible, no lineal amb la mida del fitxer, encara que
aquest fitxer pesi diversos GB.

**Motiu**: l'eina ha de poder córrer en servidors amb recursos limitats sense competir amb
el procés que genera els logs que està llegint.

### IV. Interfície gràfica, no de terminal

RealttyLog MUST oferir una GUI d'escriptori (finestres, clics, panells desplegables), no una
interfície de terminal (TUI/ncurses). El panell de detall d'una línia amb JSON o HTML
incrustat MUST poder-se expandir amb un sol clic, amb ressaltat de sintaxi i indentació.

**Motiu**: és la diferència real respecte lnav, que és TUI i resulta incòmode en un
servidor Windows sense terminal decent — és l'avantatge competitiu del projecte, no un
detall estètic.

### V. Desenvolupament dirigit per especificacions

Cada funcionalitat MUST tenir spec i pla (`/speckit-specify`, `/speckit-plan`) abans
d'implementar-se (`/speckit-implement`), seguint el mateix flux de Spec Kit que fa servir
PLaB, encara que l'stack tècnic no sigui l'estàndard AppsCat. La programació la fa Claude;
l'usuari revisa i decideix en cada pas de conflicte o ambigüitat.

**Motiu**: separar metodologia de stack permet mantenir la disciplina de procés d'AppsCat en
un projecte amb un stack diferent, i deixa un rastre escrit de per què es va prendre cada
decisió.

## Restriccions tècniques

- **Llenguatge**: Rust (edició estable més recent).
- **GUI**: `egui`/`eframe`. Es prioritza velocitat d'iteració i comunitat sobre aspecte
  natiu de plataforma (decisió conscient, veure spec de la Fase 1).
- **Tailing de fitxers**: crate `notify` per als esdeveniments del sistema de fitxers.
- **Parsing**: `serde_json` per JSON, `quick-xml` per HTML/XML incrustat als logs.
- **Mida objectiu del binari**: 5–15 MB per plataforma.
- **Plataformes objectiu**: Windows 10/11 (prioritària), Linux (secundària).
- **Sense telemetria**: l'eina no envia cap dada fora del servidor on corre.

## Flux de desenvolupament

Cada fase del full de ruta (Tailing bàsic → Detection Engine → Smart Viewer Panel →
Configuració Portable) es tracta com una feature independent dins `specs/NNN-nom/`, amb
`spec.md` i `plan.md` abans de tocar codi. Els commits i el `push` a `RealttyLog` es fan
seguint les mateixes pràctiques de git ja establertes a la resta de projectes de l'usuari.

## Governance

Aquesta constitució preval sobre qualsevol pràctica ad hoc. Les modificacions requereixen
actualitzar aquest fitxer amb un nou número de versió (semver: MAJOR per canvis
incompatibles de principis, MINOR per afegir-ne, PATCH per aclariments) i un Sync Impact
Report al capçal. El `/speckit-plan` de cada feature MUST incloure una Constitution Check
que verifiqui el compliment dels cinc principis abans de passar a disseny.

**Version**: 1.0.0 | **Ratified**: 2026-08-15 | **Last Amended**: 2026-08-15
