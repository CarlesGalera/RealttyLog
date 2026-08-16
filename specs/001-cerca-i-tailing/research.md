# Research: Cerca i tailing

## Decisió 1 — Motor de cerca multi-fitxer: `ignore` + `grep-searcher`

**Decision**: per cercar text a tot un directori (FR-001–FR-008), fem servir els crates
`ignore` (recorregut de directoris, el mateix que fa servir `ripgrep` per travessar arbres
de fitxers de manera eficient) i `grep-searcher`/`grep-matcher` (cerca en streaming dins
d'un fitxer, línia a línia, sense carregar-lo sencer).

**Rationale**: és exactament el motor que fa `ripgrep` tan ràpid en directoris grans, ja
provat en producció per milions d'usuaris, i cobreix de sèrie el recorregut de subdirectoris
(FR-001), la detecció de fitxers no llegibles com a text (FR-007) i la cerca en streaming
(FR-003) sense haver de reimplementar-ho.

**Alternatives considered**: recórrer directoris a mà amb `std::fs::read_dir` recursiu i
cercar amb `str::contains` línia a línia — descartat: reimplementaria pitjor el que ja fa
`ripgrep`, i perdríem detalls ja resolts (fitxers binaris, permisos, simbolic links) que
`ignore` gestiona correctament.

## Decisió 2 — Cerca en un pool de fils, cancel·lable

**Decision**: cada cerca (FR-002) es reparteix entre un pool de fils fix (per exemple, un
per nucli de CPU disponible), cadascun processant un subconjunt dels fitxers detectats per
`ignore`. Els resultats es van enviant a la GUI a mesura que apareixen (no s'espera que
acabi tota la cerca per mostrar el primer resultat). Un senyal de cancel·lació compartit
(`AtomicBool` o equivalent) es consulta entre fitxers i, si està activat, els fils aturen la
cerca deixant els resultats trobats fins aquell moment (FR-005).

**Rationale**: processar els fitxers en paral·lel és l'única manera realista de complir
SC-002 (primer resultat en <3 s sobre 20 GB en total) quan hi ha desenes de fitxers; mostrar
resultats a mesura que arriben, en lloc d'esperar que tot acabi, fa que l'aplicació se senti
responsiva des del primer instant encara que la cerca completa trigui més.

**Alternatives considered**: cerca seqüencial, un fitxer rere l'altre — descartada: en un
directori amb molts fitxers grans, el primer resultat trigaria tant com trigués a arribar-hi
seqüencialment, incompatible amb SC-002.

## Decisió 3 — Acotar els resultats mostrats

**Decision**: la llista de resultats es talla a un nombre màxim fix (per exemple, els
primers 500), amb un indicador de "hi ha més resultats, refina la cerca" en lloc de
seguir acumulant-ne sense límit (FR-008).

**Rationale**: una cerca d'un terme molt comú en un directori gran pot generar desenes de
milers de coincidències; mostrar-les totes trencaria tant el rendiment de la interfície com
el principi III (memòria acotada). Acotar els resultats és independent d'acotar la cerca en
si: la cerca es pot cancel·lar igualment un cop hi ha prou resultats útils.

**Alternatives considered**: paginació completa amb totes les coincidències accessibles —
descartada per a aquesta fase: exigiria mantenir totes les coincidències en memòria o
re-cercar per pàgina, complexitat no justificada quan el cas d'ús real és trobar el fitxer
correcte, no exhaurir totes les aparicions d'un terme comú.

## Decisió 4 — Detecció de canvis en directe: `notify` en lloc de sondeig (polling)

**Decision**: un cop l'usuari segueix un fitxer concret (via cerca o obert directament), fem
servir el crate `notify`, que embolcalla els mecanismes natius del sistema operatiu (inotify
a Linux, ReadDirectoryChangesW a Windows) per rebre un esdeveniment quan el fitxer canvia de
mida.

**Rationale**: notificació instantània sense cap cicle de sondeig actiu; consum de CPU
pràcticament nul en repòs, coherent amb el principi III de la constitució.

**Alternatives considered**: sondeig periòdic (`fs::metadata` cada N ms) — descartat perquè
o bé introdueix latència (interval llarg) o bé malgasta CPU (interval curt) sense cap
avantatge sobre `notify`.

## Decisió 5 — Detecció de rotació de log

**Decision**: en cada esdeveniment de `notify`, comparem la mida actual del fitxer amb
l'últim offset llegit. Si la mida actual és **menor** que l'offset, interpretem que el
fitxer s'ha truncat i reprenem la lectura des del byte 0. Si el fitxer desapareix i es torna
a crear amb el mateix camí (reemplaçament), el reobrim quan arriba l'esdeveniment de
creació.

**Rationale**: cobreix els dos patrons de rotació documentats a l'spec (FR-020): truncament
a mida zero i reemplaçament amb el mateix nom, que són els que fan servir `logrotate` i la
majoria de loggers amb rotació integrada.

**Alternatives considered**: seguiment per identitat de fitxer (inode a Linux, file index a
Windows) via crates com `same-file` — descartat per a aquesta fase: afegeix una dependència
i complexitat multiplataforma addicional per cobrir casos (renombrar el fitxer mentre un
altre procés continua escrivint al descriptor antic) que no formen part de l'abast acordat
a l'spec. Es pot reconsiderar si apareix un cas real que ho requereixi.

## Decisió 6 — Finestra acotada en memòria, historial reconstruïble des de disc

**Decision**: en memòria només es manté una finestra acotada de línies decodificades al
voltant del punt de lectura actual —el que es mostra en pantalla més un marge de
pre-càrrega—, en un búfer circular (`VecDeque<Line>`) amb capacitat fixa. **Cap contingut es
descarta de debò**: quan l'usuari es desplaça més enllà d'aquesta finestra (o salta a un
resultat de cerca llunyà, FR-009), RealttyLog torna a llegir aquell tram directament del
fitxer. Per no haver d'escanejar el fitxer sencer cada vegada, es manté un índex lleuger
d'offsets de línia —un checkpoint `(número de línia, byte offset)` cada N línies (per
exemple, cada 1.000), no de totes— que es construeix de manera incremental a mesura que es
llegeix el fitxer (tant en seguir-lo com en cercar-hi, ja que la cerca també travessa el
fitxer línia a línia). Per saltar a un punt intermedi, es va al checkpoint indexat més
proper i s'escaneja linealment des d'allà fins a la línia exacta.

**Rationale**: el fitxer de log és la font de veritat i ja és durador al disc; RealttyLog és
un visor, no un generador de dades, i amagar contingut que encara existeix al fitxer seria
enganyós per a algú que hi busca alguna cosa concreta en producció (FR-025, SC-009). El
principi III (memòria acotada) s'aplica a la finestra activa i a l'índex —que és petit,
només offsets, no contingut—, no pas al nombre total de línies del fitxer. A més, reutilitzar
aquest índex quan s'obre un resultat de cerca (FR-009, FR-010) evita haver de rellegir el
fitxer des del principi per mostrar-ne el context.

**Alternatives considered**:
- Mantenir totes les línies vistes des de l'obertura en memòria — descartat: RAM
  proporcional a la mida del fitxer, incompatible amb SC-005 en sessions llargues.
- Descartar les línies més antigues en superar una capacitat fixa (disseny inicial
  d'aquest document, corregit aquí arran d'una observació de l'usuari) — descartat: amagaria
  contingut que encara existeix al disc, contradient directament el propòsit d'un visor de
  logs i el FR-025 de l'spec.
- Indexar l'offset de cada línia del fitxer sencer en obrir-lo — descartat: en un fitxer de
  diversos GB amb desenes de milions de línies, l'índex mateix pot pesar centenars de MB,
  violant igualment el pressupost de memòria. Per això l'índex és dispers (cada N línies),
  no complet.

## Decisió 7 — Posicionament inicial al final d'un fitxer gran

**Decision**: en obrir un fitxer directament (sense passar per un resultat de cerca), en
lloc de llegir-lo des del byte 0, obtenim la seva mida per metadades i llegim cap enrere en
blocs de mida fixa (per exemple 64 KB) comptant salts de línia fins a tenir prou línies per
omplir la vista inicial (o fins arribar al principi del fitxer). Quan s'obre des d'un
resultat de cerca (FR-009), es fa servir directament l'offset ja conegut de la coincidència
en lloc de posicionar-se al final.

**Rationale**: és l'únic enfocament que compleix SC-004 (obrir un fitxer de 5 GB i mostrar-
ne el final en <2 s) sense llegir mai el fitxer sencer, coherent amb FR-013.

**Alternatives considered**: llegir des del principi fins al final — descartat, el temps
d'obertura creixeria linealment amb la mida del fitxer i trencaria SC-004 en fitxers grans.

## Decisió 8 — Comunicació entre els fils de fons i la GUI

**Decision**: tant una cerca en curs com un fitxer seguit en directe corren en fils de
sistema operatiu (`std::thread`) dedicats, que envien resultats/línies i esdeveniments
d'estat a través de canals `std::sync::mpsc`. El fil de la GUI els consumeix a cada frame i
demana repintar (`ctx.request_repaint()`) només quan arriba contingut nou.

**Rationale**: no cal un runtime asíncron (`tokio`) per gestionar un pool de cerca i un fil
de seguiment; mantenir-ho a la biblioteca estàndard és coherent amb el principi I
(portabilitat sense dependències) i redueix la mida del binari.

**Alternatives considered**: `tokio` + canals asíncrons — descartat: afegeix una dependència
pesant i un runtime sencer sense cap benefici real per a un pool de fils senzill i un fil de
seguiment.

## Decisió 9 — Gestió de contingut no vàlid com a UTF-8

**Decision**: cada bloc de bytes, tant en cercar com en seguir, es decodifica amb
`String::from_utf8_lossy`, que substitueix les seqüències de bytes invàlides pel caràcter de
reemplaçament (`U+FFFD`) en lloc d'aturar la lectura.

**Rationale**: cobreix FR-022 sense dependències addicionals; l'assumpció de l'spec és que
UTF-8 és la codificació per defecte dels logs moderns, així que no cal detecció de
codificació general en aquesta fase.

**Alternatives considered**: `encoding_rs` per a detecció i conversió de múltiples
codificacions — descartat, fora de l'abast acordat (veure Assumptions de l'spec).

## Resum de dependències resultants

| Crate | Ús | Per què cap alternativa més pesant |
|---|---|---|
| `eframe`/`egui` | GUI nativa multiplataforma | ja decidit a la constitució (principi IV) |
| `notify` | esdeveniments de sistema de fitxers | ja decidit a la constitució |
| `ignore` | recorregut de directoris per a la cerca | evita reimplementar pitjor el que ja fa `ripgrep` (decisió 1) |
| `grep-searcher`/`grep-matcher` | cerca de text en streaming | mateix motiu (decisió 1) |
| *(cap més)* | — | `std::thread`, `std::sync::mpsc` i `String::from_utf8_lossy` de la biblioteca estàndard cobreixen la resta |
