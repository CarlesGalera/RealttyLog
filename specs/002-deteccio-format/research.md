# Research: Detecció i formatatge de payloads

## Decisió 1 — Parsing JSON amb `serde_json`

**Decision**: `serde_json::from_str` per detectar i parsejar JSON, treballant amb
`serde_json::Value` per construir la representació estilada.

**Rationale**: és l'estàndard de facto de l'ecosistema Rust, ja previst des del document de
concepte original (§3, "JSON/HTML Parser"), petit i sense dependències de sistema. Fer
servir el `Value` genèric (en lloc de `deserialize::<T>` a un tipus concret) encaixa amb un
visor que no sap res del contingut real dels logs.

**Alternatives considered**: escriure un parser JSON propi — descartat, reinventaria pitjor
el que ja fa `serde_json` de manera provada.

## Decisió 2 — XML i HTML amb `quick-xml`, en dos modes

**Decision**: `quick-xml::Reader` per tokenitzar tant XML com HTML. Per XML, en mode
estricte (etiquetes aparellades, sintaxi vàlida): si falla, no és XML vàlid (FR-004). Per
HTML, en mode permissiu (`check_end_names(false)`, sense exigir aparellament estricte ni
`&` escapat): cobreix elements buits sense tancar (`<br>`, `<img>`) i entitats soltes que
un parser XML estricte rebutjaria.

**Rationale**: una sola dependència per a tots dos formats, ja prevista al document de
concepte original. `quick-xml` és petit, no requereix dependències de sistema, i el seu
mode de baix nivell (esdeveniments d'obertura/tancament d'etiqueta) és exactament el que
cal per indentar per nivell d'imbricació — no calen ni DOM ni consultes CSS/XPath.

**Alternatives considered**: `html5ever` (el parser HTML5 "de veritat", usat per Servo) —
descartat per aquesta fase: és una família de crates més gran, pensada per construir un DOM
complet, que faria créixer el binari per sobre del que aporta a un simple indentador de
text. Si mai calgués interpretar HTML realment trencat de manera fiable, val la pena
reconsiderar-ho.

## Decisió 3 — Distingir XML d'HTML

**Decision**: primer es prova com a XML estricte. Si el text comença per `<?xml` o
parseja com a XML ben format amb `quick-xml` en mode estricte, es tracta com a XML. Si no,
es prova en mode permissiu; si tokenitza com a marcatge vàlid (almenys una etiqueta
reconeguda), es tracta com a HTML. Si cap dels dos mode ho accepta, no es marca res
(FR-004).

**Rationale**: cobreix el cas real —quan un log porta un XML de veritat sol tenir prolog o
ser ben format; l'HTML que apareix en producció (fragments de pàgines d'error, respostes
d'API) sovint no ho és— sense necessitat de detectar `<!DOCTYPE html>` com a únic senyal,
que fallaria amb fragments HTML solts (Edge Case de l'spec).

**Alternatives considered**: mirar només l'etiqueta arrel (`<html>`, `<?xml`) —
descartat: un fragment de taula HTML sense arrel `<html>` és un cas real (Edge Case de
l'spec) que aquest mètode no cobriria.

## Decisió 4 — JWT: detecció per forma i descodificació amb `base64`

**Decision**: es detecta un JWT per la seva forma (`^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$`,
tres segments base64url). Es descodifiquen els dos primers segments amb el crate `base64`
(variant `URL_SAFE_NO_PAD`, que és la que fa servir l'estàndard JWT) i es parsegen com a
JSON amb `serde_json`, reutilitzant el mateix formatador de la decisió 1. El tercer segment
(signatura) es mostra tal com és (FR-012), sense intentar-lo interpretar.

**Rationale**: reutilitzar el formatador de JSON per als dos primers segments d'un JWT
evita duplicar cap lògica de ressaltat; només cal un pas previ de descodificació base64url,
que és tot el que un JWT necessita per esdevenir llegible.

**Alternatives considered**: un crate dedicat a JWT (per exemple `jsonwebtoken`) —
descartat: aquests crates estan pensats per *validar* signatures (exigeixen una clau), no
per mostrar-les; per a un visor de només lectura, `base64` + `serde_json` ja fan tota la
feina que cal.

## Decisió 5 — Representació intermèdia sense dependència d'`egui`

**Decision**: `format/` produeix `Vec<StyledLine>`, on `StyledLine = Vec<(String,
TokenKind)>` i `TokenKind` és un enum propi (`Key`, `StringValue`, `Number`, `BoolNull`,
`Punctuation`, `TagName`, `AttrName`, `AttrValue`, `Comment`, `PlainText`). `ui/log_view.rs`
tradueix cada `TokenKind` a un `egui::Color32` concret en pintar.

**Rationale**: manté `format/` provable amb `cargo test` sense dependre d'`egui` (com ja fan
`search/` i `tailer/`), i respecta el `Structure Decision` de la Fase 1: la lògica no
gràfica no depèn mai de la capa de GUI.

**Alternatives considered**: retornar directament text amb colors d'`egui::Color32`
incrustats des de `format/` — descartat: acoblaria un mòdul de lògica pura a la llibreria
de GUI sense cap benefici real.

## Decisió 6 — Memorització en dos nivells

**Decision**: `LogViewState` manté dos mapes acotats, indexats per `byte_offset` de la
línia (estable dins la sessió de seguiment, a diferència de la posició dins el `VecDeque`
de `ViewportCache`, que canvia en desplaçar-se):

1. `detected: HashMap<u64, Option<PayloadKind>>` — es omple la primera vegada que una línia
   es dibuixa, cost mínim per a la resta de frames.
2. `expanded: HashMap<u64, Vec<StyledLine>>` — només conté les línies que l'usuari ha
   desplegat explícitament; el format complet (car per a payloads grans, SC-002) es calcula
   un sol cop per línia, no cada frame.

Tots dos mapes s'acoten a una mida màxima (com `ViewportCache`, decisió 6 de la Fase 1):
en superar-la, es descarta l'entrada més antiga.

**Rationale**: sense memorització, un JSON gran es tornaria a parsejar i indentar a cada
frame (fins a 60 vegades per segon) només per estar visible, violant SC-003. Acotar els
mapes evita que una sessió llarga amb moltes línies estructurades diferents faci créixer la
memòria sense límit, coherent amb el principi III de la constitució.

**Alternatives considered**: guardar la detecció dins la pròpia struct `Line` de
`tailer::mod` — descartat: acoblaria `tailer` (llegir i seguir fitxers) a `format`
(interpretar-ne el contingut), trencant la separació de mòduls ja establerta a la Fase 1.

## Resum de dependències resultants

| Crate | Ús | Per què cap alternativa més pesant |
|---|---|---|
| `serde_json` | parsing i valors JSON | estàndard de l'ecosistema, ja previst al document de concepte (decisió 1) |
| `quick-xml` | tokenització XML i HTML | una sola dependència per a tots dos formats, sense construir cap DOM (decisions 2-3) |
| `base64` | descodificació dels segments d'un JWT | mínima i específica; un crate de validació JWT faria més del que cal (decisió 4) |
