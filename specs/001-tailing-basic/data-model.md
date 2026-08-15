# Data Model: Tailing bàsic

Entitats derivades de `spec.md` §Key Entities, més el búfer de suport que en fa falta per
complir el principi III de la constitució (memòria acotada). Cap d'aquestes entitats es
persisteix: totes viuen només en memòria mentre l'aplicació té un fitxer obert.

## FollowedFile (Fitxer seguit)

Correspon a l'entitat "Fitxer seguit" de l'spec.

| Camp | Descripció |
|---|---|
| `path` | Camí absolut al fitxer que s'està seguint |
| `state` | `Live` \| `Paused` \| `Unavailable` (FR-008, FR-010) |
| `read_offset` | Byte fins on s'ha llegit el fitxer |
| `last_known_len` | Mida del fitxer a l'última lectura, per detectar truncament (research.md, decisió 2) |

**Validació**: en obrir, el camí ha d'existir i ser legible (FR-001); si no, no es crea la
instància i es mostra l'error a la interfície en lloc de crear un `FollowedFile` invàlid.

**Transicions d'estat**:

- `Live → Paused`: l'usuari desplaça la vista cap amunt (FR-006).
- `Paused → Live`: l'usuari activa "tornar al directe" (FR-007).
- `Live | Paused → Unavailable`: el fitxer deixa d'estar accessible (FR-010).
- `Unavailable → Live`: el fitxer torna a estar accessible; es reprèn en mode directe
  (FR-010).
- Un esdeveniment de rotació (research.md, decisió 2) reinicia `read_offset` a 0 però no
  canvia `state` (FR-009): rotar el fitxer no equival a perdre'l de vista.

## Line (Línia)

Correspon a l'entitat "Línia" de l'spec.

| Camp | Descripció |
|---|---|
| `content` | Text de la línia, decodificat amb pèrdua UTF-8 (research.md, decisió 6) |
| `sequence` | Índex d'arribada, monòtonament creixent, per mantenir l'ordre original (FR-004) |

**Validació**: el contingut es conserva tal com arriba, sense retallar-lo ni alterar-lo,
excepte la substitució de bytes invàlids (FR-011).

## LineBuffer (búfer circular)

Entitat de suport, no present a l'spec com a concepte de negoci, però necessària per complir
FR-013/SC-003 (research.md, decisió 3).

| Camp | Descripció |
|---|---|
| `capacity` | Nombre màxim de `Line` retingudes |
| `lines` | Col·lecció ordenada de `Line`, la més antiga al capdavant |

**Comportament**: en afegir una `Line` nova, si `lines.len() > capacity`, es descarta la més
antiga. Un `LineBuffer` pertany a un únic `FollowedFile`; en tancar el fitxer o obrir-ne un
altre, es crea un `LineBuffer` nou i buit.

## Relacions

```text
FollowedFile 1 ── 1 LineBuffer 1 ── N Line
```

Un `FollowedFile` té exactament un `LineBuffer`; un `LineBuffer` conté com a màxim
`capacity` instàncies de `Line`, ordenades per `sequence`.
