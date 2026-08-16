# Data Model: Cerca i tailing

Entitats derivades de `spec.md` §Key Entities, més les entitats de suport que fan falta per
complir el principi III de la constitució (memòria acotada) tant en cercar com en seguir.
Cap d'aquestes entitats es persisteix: totes viuen només en memòria mentre l'aplicació té un
directori obert o un fitxer seguit.

## LogDirectory (Directori de logs)

Correspon a l'entitat "Directori de logs" de l'spec.

| Camp | Descripció |
|---|---|
| `root` | Camí absolut al directori obert |
| `files` | Llista de camins de fitxers de log detectats (inclosos els de subdirectoris), vigent en el moment d'obrir-lo (FR-001) |

**Validació**: en obrir, el camí ha de ser un directori legible; si no, es mostra l'error a
la interfície en lloc de crear un `LogDirectory` invàlid.

## SearchQuery (Cerca)

Correspon a l'entitat "Cerca" de l'spec.

| Camp | Descripció |
|---|---|
| `text` | Text lliure cercat (FR-002) |
| `status` | `Running` \| `Completed` \| `Cancelled` (FR-005) |
| `matches` | Llista ordenada de `SearchMatch` trobades fins ara, acotada a un màxim fix (FR-008, research.md decisió 3) |

**Comportament**: pertany a un `LogDirectory`. Els fitxers considerats són els de
`LogDirectory.files` en el moment de llançar la cerca (assumpció "foto fixa" de l'spec); un
fitxer nou al directori mentre `status == Running` no s'hi afegeix. Passar `status` a
`Cancelled` atura els fils de cerca (research.md, decisió 2) i deixa `matches` tal com
estava.

## SearchMatch (Coincidència)

Correspon a l'entitat "Coincidència" de l'spec.

| Camp | Descripció |
|---|---|
| `file_path` | Fitxer on s'ha trobat la coincidència |
| `byte_offset` | Posició dins el fitxer on comença la línia coincident |
| `line_context` | Fragment de text al voltant de la coincidència, per mostrar-lo a la llista de resultats sense obrir el fitxer (FR-004) |

**Comportament**: en clicar-la (FR-009), és el pont cap a `FollowedFile`: n'obre (o en
reutilitza) una instància per `file_path` i la posiciona a `byte_offset` mitjançant el
`LineIndex` d'aquell fitxer (research.md, decisió 6), en lloc de posicionar-se al final com
faria una obertura directa.

## FollowedFile (Fitxer seguit)

Correspon a l'entitat "Fitxer seguit" de l'spec. S'hi arriba des d'un `SearchMatch` (FR-009)
o obrint-lo directament (FR-012).

| Camp | Descripció |
|---|---|
| `path` | Camí absolut al fitxer que s'està seguint |
| `state` | `Live` \| `Paused` \| `Unavailable` (FR-019, FR-021) |
| `read_offset` | Byte fins on s'ha llegit el fitxer en directe |
| `last_known_len` | Mida del fitxer a l'última lectura, per detectar truncament (research.md, decisió 5) |

**Validació**: en obrir, el camí ha d'existir i ser legible (FR-012); si no, no es crea la
instància i es mostra l'error a la interfície en lloc de crear un `FollowedFile` invàlid.

**Transicions d'estat**:

- `Live → Paused`: l'usuari desplaça la vista cap amunt (FR-017).
- `Paused → Live`: l'usuari activa "tornar al directe" (FR-018).
- `Live | Paused → Unavailable`: el fitxer deixa d'estar accessible (FR-021).
- `Unavailable → Live`: el fitxer torna a estar accessible; es reprèn en mode directe
  (FR-021).
- Un esdeveniment de rotació (research.md, decisió 5) reinicia `read_offset` a 0 però no
  canvia `state` (FR-020): rotar el fitxer no equival a perdre'l de vista.

## Line (Línia)

Correspon a l'entitat "Línia" de l'spec.

| Camp | Descripció |
|---|---|
| `content` | Text de la línia, decodificat amb pèrdua UTF-8 (research.md, decisió 9) |
| `sequence` | Índex d'arribada, monòtonament creixent, per mantenir l'ordre original (FR-015) |

**Validació**: el contingut es conserva tal com arriba, sense retallar-lo ni alterar-lo,
excepte la substitució de bytes invàlids (FR-022).

## ViewportCache (finestra en memòria)

Entitat de suport, no present a l'spec com a concepte de negoci, però necessària per complir
FR-024/SC-005 sense violar FR-025/SC-009 (research.md, decisió 6). **No és "tot l'historial
en memòria"**: només la porció que es mostra en pantalla més un marge de pre-càrrega —tant
si s'hi arriba seguint en directe com saltant des d'un `SearchMatch` (FR-010, context al
voltant del salt).

| Camp | Descripció |
|---|---|
| `capacity` | Nombre màxim de `Line` retingudes a la finestra activa |
| `lines` | Col·lecció ordenada de `Line` visibles ara mateix, la més antiga al capdavant |
| `window_start` | Número de línia (per `sequence`) on comença la finestra actual dins el fitxer |

**Comportament**: en desplaçar-se dins de la finestra actual, no cal llegir res més. En
desplaçar-se'n fora (amunt, avall, o en saltar-hi des d'un `SearchMatch`), es recalcula
`window_start` i es recarreguen les `Line` corresponents des del fitxer, via `LineIndex`, no
des de memòria. Un `ViewportCache` pertany a un únic `FollowedFile`; en tancar el fitxer o
obrir-ne un altre, es crea un `ViewportCache` nou i buit.

## LineIndex (índex dispers d'offsets)

Entitat de suport que fa possible saltar a un punt qualsevol de l'historial —incloent-hi un
`byte_offset` que arriba d'un `SearchMatch`— sense escanejar el fitxer sencer cada vegada
(research.md, decisió 6).

| Camp | Descripció |
|---|---|
| `checkpoints` | Llista ordenada de `(sequence, byte_offset)`, un cada N línies (per exemple, cada 1.000) |

**Comportament**: es construeix de manera incremental a mesura que RealttyLog llegeix el
fitxer, tant en seguir-lo en directe com en respondre a un salt de l'usuari o d'un
`SearchMatch`. Per posicionar-se en un `byte_offset` o `sequence` concrets, es cerca el
`checkpoint` més proper per sota i s'escaneja linealment des d'allà fins a la posició
exacta. En rotar el fitxer (FR-020), l'índex es reinicia junt amb `read_offset` de
`FollowedFile`.

## Relacions

```text
LogDirectory 1 ── N SearchQuery
SearchQuery  1 ── N SearchMatch
SearchMatch  N ── 1 FollowedFile (per file_path; obrir el mateix fitxer des de dos
                                   resultats reutilitza la mateixa instància)
FollowedFile 1 ── 1 ViewportCache 1 ── N Line (finestra activa, acotada)
FollowedFile 1 ── 1 LineIndex 1 ── N checkpoint (dispers, tot el fitxer)
```

Un `LogDirectory` pot tenir diverses `SearchQuery` (una per cerca llançada); cada
`SearchQuery` acumula `SearchMatch` fins al límit de FR-008. Un `FollowedFile` té exactament
un `ViewportCache` (finestra visible, acotada per `capacity`) i un `LineIndex` (checkpoints
dispersos sobre tot el fitxer). El `LineIndex` és el que permet que `FollowedFile` satisfaci
FR-025 (repassar tot l'historial) i que un `SearchMatch` s'hi pugui posicionar directament,
sense que cap dels dos impliqui carregar el fitxer sencer a memòria.
