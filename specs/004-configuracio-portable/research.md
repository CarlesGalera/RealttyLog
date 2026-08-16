# Research: Configuració portable de les regles

## Decisió 1 — JSON amb `serde` (derive), no INI

**Decision**: es desa com un array JSON de regles amb `serde_json::to_string_pretty` +
`#[derive(Serialize, Deserialize)]` a `HighlightRule` i `RgbColor`. El document de concepte
original oferia JSON o INI com a opcions; es tria JSON.

**Rationale**: `serde_json` ja és una dependència des de la Fase 2 (formatatge de
payloads); reutilitzar-lo per a la configuració no n'afegeix cap de nova de pes —només les
macros de derivació de `serde`, que `serde_json` ja arrossega transitivament. INI exigiria
un crate addicional (per exemple `ini` o `serde_ini`) per a un guany net zero: una regla té
camps niats de manera natural (color com a `{r,g,b}`), que INI expressa pitjor que JSON.

**Alternatives considered**: format binari propi (`bincode`) — descartat: l'spec (Key
Entities) exigeix un fitxer "llegible i editable a mà", i un format binari ho impediria.

## Decisió 2 — Ubicació: al costat de l'executable, no una carpeta de configuració del SO

**Decision**: `std::env::current_exe()?.parent()` + `realttylog-rules.json`. Mai
`dirs::config_dir()` ni equivalents (`%APPDATA%`, `~/.config`).

**Rationale**: és literalment el que demana FR-008 i el que fa que US2 (la configuració
viatja amb l'executable) sigui certa. Un directori de configuració del sistema és l'opció
"normal" per a la majoria d'aplicacions d'escriptori, però trencaria precisament la
propietat "portable, sense instal·lació" que defineix aquest projecte (constitució,
principi I) — la mateixa raó per la qual el binari mateix no s'instal·la enlloc.

**Alternatives considered**: directori de configuració estàndard del SO (via el crate
`dirs`) — descartat pel motiu anterior, a més d'evitar una dependència nova sense cap
benefici net per a aquest projecte.

## Decisió 3 — Desar: un cop per canvi de versió, no a cada mutació individual

**Decision**: `App::ui()` compara `self.rules.version()` amb un `last_saved_version`
guardat a `App`; si difereixen, desa i actualitza `last_saved_version`. No hi ha cap crida
a `config::save` directament des de `ui/rules_panel.rs`.

**Rationale**: `RuleSet::version()` (Fase 3, research.md decisió 4) ja és el senyal
"alguna cosa ha canviat" que fa servir `LogViewState` per invalidar la seva memorització;
reutilitzar-lo per decidir quan desar evita escampar crides d'I/O per tot `rules_panel.rs`
i centralitza la persistència en un sol lloc, sense dependre que cada camí d'edició
recordi cridar `save()`.

**Alternatives considered**: desar només en sortir de l'aplicació (a `Drop` o abans de
`std::process::exit`) — descartat: no sobreviuria a un tancament brusc (procés matat,
tall de corrent), i FR-002 vol que els canvis es desin sense cap acció explícita, no només
en un moment concret de sortida.

## Decisió 4 — Resiliència: mai bloquejar l'arrencada ni la sessió

**Decision**: `config::load_from` retorna sempre un `RuleSet` vàlid (buit si el fitxer no
existeix, no és llegible, o no és JSON vàlid). Es deserialitza primer com a
`Vec<serde_json::Value>` i després cada element individualment com a `HighlightRule`,
descartant amb `filter_map` els que fallin —així una sola regla amb un camp absent no fa
caure la resta (FR-007). `config::save_to` ignora els errors d'escriptura (`let _ =
std::fs::write(...)`) sense propagar-los: la sessió en curs no en depèn (FR-003).

**Rationale**: la persistència és una comoditat (Assumptions de l'spec), mai un requisit
per poder obrir i llegir logs, que és la funció principal de l'eina. Fer que qualsevol
error de disc talli l'aplicació seria posar la part accessòria per sobre de la principal.

**Alternatives considered**: propagar l'error de lectura/escriptura amb un `Result` fins a
`App` i mostrar-hi un avís a la interfície — descartat per aquesta fase: l'spec
(Assumptions) ja decideix que desar és silenciós, i afegir-hi un mecanisme d'avisos seria
abast no demanat.

## Decisió 5 — Provabilitat: separar el `Path` real de la lògica de (de)serialització

**Decision**: `config::load_from(path: &Path) -> RuleSet` i `config::save_to(path: &Path,
rules: &RuleSet)` no criden `current_exe()` internament; només ho fan els embolcalls
`config::load()`/`config::save()` que fa servir `App`. `tests/config_integration.rs` crida
sempre les variants amb `path`, sobre fitxers temporals (`std::env::temp_dir()` + un nom
únic, sense cap crate nou com `tempfile`).

**Rationale**: barrejar la resolució de `current_exe()` amb la lògica de (de)serialització
faria impossible provar-la sense escriure de debò al costat del binari de test —arriscat i
innecessari. La separació és el mateix patró que ja fa `rules/`: lògica pura, provable,
separada del punt d'entrada real.

**Alternatives considered**: cap — és l'única manera raonable de fer-ho provable sense
dependències noves.

## Resum de dependències resultants

| Crate | Ús | Per què cap alternativa més pesant |
|---|---|---|
| `serde` (característica `derive`) | (de)serialitzar `HighlightRule`/`RgbColor` | ja arrossegat transitivament per `serde_json` des de la Fase 2; només calen les macros (decisió 1) |
