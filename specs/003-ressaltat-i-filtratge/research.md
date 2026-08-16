# Research: Ressaltat per paraula clau i filtratge instantani

## Decisió 1 — Coincidència per subcadena literal, sense regex

**Decision**: cada regla es compara amb `line.to_lowercase().contains(&keyword.to_lowercase())`
(FR-002: insensible a majúscules per defecte). Sense cap crate de regex.

**Rationale**: l'spec ho documenta com a assumpció explícita — "paraula clau", no expressió
regular, per mantenir la interfície simple (un sol camp de text, sense ensenyar sintaxi de
regex a qui l'utilitzi). `std` ja ho resol sencer, sense dependència nova (Constitution
Check, principi I).

**Alternatives considered**: crate `regex` — descartat per a aquesta fase: cap requisit
funcional el demana, i afegiria pes al binari (encara que petit) sense cap regla real que
el necessiti. Si mai calen patrons més expressius, és un CR futur, no una ampliació
d'aquesta fase.

## Decisió 2 — Prioritat entre regles: la més recent guanya

**Decision**: `RuleSet` és essencialment un `Vec<HighlightRule>` en ordre de creació. Quan
diverses regles actives coincideixen amb la mateixa línia, guanya la que s'ha creat més
tard (recorregut del vector de darrere cap endavant, primer match trobat).

**Rationale**: és el criteri més fàcil d'explicar a qui l'utilitza ("la regla que has fet
més recentment mana") i no requereix cap camp de prioritat manual que l'usuari hagi de
configurar. Editar una regla existent no li canvia la posició al vector — només crear-ne
una de nova la posa al capdavant de la prioritat.

**Alternatives considered**: deixar triar una prioritat numèrica explícita a cada regla —
descartat: afegeix un control més a la interfície (FR-003 vol que gestionar regles sigui
senzill) per a un cas (línies que compleixen dues regles alhora) que no és el flux
principal.

## Decisió 3 — El filtre és OR entre regles amb `filter = true`

**Decision**: una línia és visible si cap regla té el filtre actiu, o si almenys una regla
amb `filter = true` hi coincideix. `filter` només té efecte si la regla també està
`enabled` — desactivar una regla (US3, escenari 3) li atura alhora el ressaltat i el
filtre, sense necessitat de dues comprovacions independents a la interfície.

**Rationale**: OR és l'operació que redueix soroll sense exigir que una línia contingui
totes les paraules alhora (Acceptance Scenario 3 de la User Story 2) — AND seria gairebé
inútil en la pràctica (`ERROR` i `WARN` mai apareixen juntes a la mateixa línia).
Acoblar `filter` a `enabled` en lloc de tractar-los com a banderes independents evita un
estat contradictori (una regla "filtrant" però "desactivada" no té sentit per a qui la
llegeix).

**Alternatives considered**: AND entre regles filtrades — descartat pel motiu anterior.
Un tercer estat explícit "filtre independent de l'activació" — descartat: cap escenari de
l'spec el necessita i complicaria la interfície sense benefici.

## Decisió 4 — Memorització invalidada per versió, no per contingut

**Decision**: `RuleSet` porta un comptador `version: u64` que s'incrementa a cada canvi
(afegir, editar, (des)activar, esborrar, canviar filtre). `LogViewState` guarda
`(u64, HashMap<u64, Option<usize>>)` — la versió amb què es va omplir el mapa, i per cada
`byte_offset` de línia, l'índex (si n'hi ha) de la regla que hi coincideix. Si la versió
emmagatzemada no coincideix amb `rules.version()`, es buida tot el mapa abans de continuar
(mateix patró que la memorització de la Fase 2, decisió 6: buidar sencer en lloc de
invalidar entrada per entrada).

**Rationale**: sense memorització, cada frame tornaria a comparar totes les regles contra
totes les línies visibles (fins a 60 vegades per segon), un cost innecessari quan ni el
contingut ni les regles han canviat. Un comptador de versió és més senzill que comparar el
conjunt de regles sencer a cada frame, i prou: els canvis de regles són esdeveniments rars
comparats amb el ritme de dibuixat.

**Alternatives considered**: recalcular sense memoritzar, confiant que la coincidència de
subcadena és barata — descartat: és barata per a una línia i una regla, però multiplicada
per totes les línies de `ViewportCache` i totes les regles actives, a 60 fps, deixa de
ser-ho amb prou regles definides; memoritzar-ho costa poc i elimina el dubte.

## Decisió 5 — El `RuleSet` viu a `App`, no a `LogViewState`

**Decision**: `App` posseeix `rules: RuleSet` com a germana de `search` i `open_file`, i la
passa per referència mutable a `LogViewState::ui()`.

**Rationale**: `LogViewState` es descarta i es torna a crear cada vegada que es tanca i
s'obre un fitxer (`app.rs`, `self.open_file = Some(LogViewState::new(file))`). Si el
`RuleSet` hi visqués a dins, definir una regla ERROR i després tornar als resultats de
cerca per obrir un altre fitxer la faria desaparèixer — contrari a l'esperit de "gestionar
les regles durant la sessió" (User Story 3), que parla de la sessió de l'aplicació, no
d'un fitxer concret. Guardar-lo a `App` el fa sobreviure mentre l'aplicació és oberta, i
prou (Assumptions: sense persistència en aquesta fase).

**Alternatives considered**: guardar-lo a `LogViewState` i acceptar que es perd en tancar
un fitxer — descartat pel motiu anterior. Persistir-lo a disc perquè sobrvisqui també a
tancar l'aplicació — explícitament fora d'abast, és la Fase 4 (Configuració Portable).

## Decisió 6 — "Desenes de milers de línies" (SC-002) i el límit de `ViewportCache`

**Decision**: SC-002 es verifica sobre el nombre de línies que `ViewportCache` manté
residents (2000, Fase 1), no sobre la mida total del fitxer. Un fitxer de desenes de milers
de línies compleix igualment SC-002 perquè el filtre només avalua les línies carregades a
la finestra activa (FR de l'spec: "actua sobre la finestra visible, no re-escaneja tot el
fitxer"), mai el fitxer sencer.

**Rationale**: cal deixar-ho explícit perquè no sembli una contradicció amb el límit de
memòria acotada de la Fase 1 — la garantia real és que el cost de (des)activar un filtre és
independent de la mida del fitxer, no que es processin desenes de milers de línies alhora.
Quickstart.md ho verificarà generant un fitxer gran però mesurant el filtre sobre el que hi
ha carregat en un moment donat.

**Alternatives considered**: reinterpretar SC-002 perquè parli explícitament de "línies
residents a la finestra activa" — es descarta reescriure l'spec ja aprovat per aquest
detall; aquesta nota de research.md n'és prou aclariment.

## Resum de dependències resultants

Cap dependència nova. Tota la lògica de coincidència, prioritat i filtre es construeix amb
`std` pur, reutilitzant l'`eframe`/`egui` ja present per al panell de regles.
