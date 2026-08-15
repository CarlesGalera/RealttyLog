# Research: Tailing bàsic

## Decisió 1 — Detecció de canvis: `notify` en lloc de sondeig (polling)

**Decision**: fem servir el crate `notify`, que embolcalla els mecanismes natius del
sistema operatiu (inotify a Linux, ReadDirectoryChangesW a Windows) per rebre un
esdeveniment quan el fitxer seguit canvia de mida.

**Rationale**: notificació instantània sense cap cicle de sondeig actiu; consum de CPU
pràcticament nul en repòs, cosa que ajuda a complir el principi III de la constitució.

**Alternatives considered**: sondeig periòdic (`fs::metadata` cada N ms) — descartat perquè
o bé introdueix latència (interval llarg) o bé malgasta CPU (interval curt) sense cap
avantatge sobre `notify`.

## Decisió 2 — Detecció de rotació de log

**Decision**: en cada esdeveniment de `notify`, comparem la mida actual del fitxer amb
l'últim offset llegit. Si la mida actual és **menor** que l'offset, interpretem que el
fitxer s'ha truncat i reprenem la lectura des del byte 0. Si el fitxer desapareix i es torna
a crear amb el mateix camí (reemplaçament), el reobrim quan arriba l'esdeveniment de
creació.

**Rationale**: cobreix els dos patrons de rotació documentats a l'spec (FR-009): truncament
a mida zero i reemplaçament amb el mateix nom, que són els que fan servir `logrotate` i la
majoria de loggers amb rotació integrada.

**Alternatives considered**: seguiment per identitat de fitxer (inode a Linux, file index a
Windows) via crates com `same-file` — descartat per a aquesta fase: afegeix una dependència
i complexitat multiplataforma addicional per cobrir casos (renombrar el fitxer mentre un
altre procés continua escrivint al descriptor antic) que no formen part de l'abast acordat
a l'spec. Es pot reconsiderar si apareix un cas real que ho requereixi.

## Decisió 3 — Búfer de línies acotat en memòria

**Decision**: les línies mostrades es guarden en un búfer circular (`VecDeque<Line>`) amb
una capacitat màxima fixa. Quan s'hi afegeix una línia nova i s'ha superat la capacitat,
es descarta la més antiga.

**Rationale**: és l'única manera de garantir SC-003 (memòria acotada, independent de la
mida del fitxer) quan el fitxer creix sense límit durant una sessió llarga. És el mateix
compromís que fan la majoria d'eines de tailing (SnakeTail, `tail -f` amb un terminal de
scrollback limitat).

**Alternatives considered**: mantenir totes les línies vistes des de l'obertura — descartat,
violaria directament el principi III per a sessions llargues en fitxers molt actius; accés
aleatori per `mmap` a tot el fitxer per recuperar historial sota demanda — descartat per a
aquesta fase per la complexitat que afegeix (gestió de mapes de memòria multiplataforma,
invalidació en rotar); es pot revisar en una fase futura si calgués desplaçament il·limitat
cap enrere.

## Decisió 4 — Posicionament inicial al final d'un fitxer gran

**Decision**: en obrir un fitxer, en lloc de llegir-lo des del byte 0, obtenim la seva mida
per metadades i llegim cap enrere en blocs de mida fixa (per exemple 64 KB) comptant salts
de línia fins a tenir prou línies per omplir la vista inicial (o fins arribar al principi
del fitxer).

**Rationale**: és l'únic enfocament que compleix SC-002 (obrir un fitxer de 5 GB i mostrar-
ne el final en <2 s) sense llegir mai el fitxer sencer, coherent amb FR-002.

**Alternatives considered**: llegir des del principi fins al final — descartat, el temps
d'obertura creixeria linealment amb la mida del fitxer i trencaria SC-002 en fitxers grans.

## Decisió 5 — Comunicació entre el fil de lectura i la GUI

**Decision**: cada fitxer seguit té un fil de sistema operatiu (`std::thread`) dedicat que
llegeix i envia línies i esdeveniments d'estat (en directe / no disponible) a través d'un
canal `std::sync::mpsc`. El fil de la GUI els consumeix a cada frame i demana repintar
(`ctx.request_repaint()`) només quan arriba contingut nou.

**Rationale**: no calen fils de sistema ni un runtime asíncron (`tokio`) per gestionar un
sol fitxer per finestra; mantenir-ho a la biblioteca estàndard és coherent amb el principi I
(portabilitat sense dependències) i redueix la mida del binari.

**Alternatives considered**: `tokio` + canals asíncrons — descartat per a aquesta fase:
afegeix una dependència pesant i un runtime sencer per gestionar un únic fil de fons, sense
cap benefici real amb un sol fitxer seguit.

## Decisió 6 — Gestió de contingut no vàlid com a UTF-8

**Decision**: cada bloc de bytes es decodifica amb `String::from_utf8_lossy`, que substitueix
les seqüències de bytes invàlides pel caràcter de reemplaçament (`U+FFFD`) en lloc d'aturar
la lectura.

**Rationale**: cobreix FR-011 sense dependències addicionals; l'assumpció de l'spec és que
UTF-8 és la codificació per defecte dels logs moderns, així que no cal detecció de
codificació general en aquesta fase.

**Alternatives considered**: `encoding_rs` per a detecció i conversió de múltiples
codificacions — descartat, fora de l'abast acordat (veure Assumptions de l'spec).

## Resum de dependències resultants

| Crate | Ús | Per què cap alternativa més pesant |
|---|---|---|
| `eframe`/`egui` | GUI nativa multiplataforma | ja decidit a la constitució (principi IV) |
| `notify` | esdeveniments de sistema de fitxers | ja decidit a la constitució |
| *(cap més)* | — | `std::thread`, `std::sync::mpsc` i `String::from_utf8_lossy` de la biblioteca estàndard cobreixen la resta |
