# Data Model: Detecció i formatatge de payloads

Entitats derivades de `spec.md` §Key Entities, més els tipus de suport que fan falta per
mantenir `format/` sense dependència d'`egui` (research.md, decisió 5) i la memòria acotada
(research.md, decisió 6). Cap d'aquestes entitats es persisteix: viuen només en memòria
mentre la línia corresponent és a la finestra carregada o desplegada.

## PayloadKind (Format detectat)

Enum simple, sense dades pròpies.

```text
Json | Xml | Html | Jwt
```

## DetectedPayload

Correspon al concepte "Payload detectat" de l'spec: el resultat de la detecció sobre una
línia concreta (FR-001–FR-005).

| Camp | Descripció |
|---|---|
| `kind` | `PayloadKind` trobat |
| `start` | Offset (en caràcters) dins el text de la línia on comença el payload |
| `end` | Offset on acaba |

**Validació**: només existeix si la detecció (research.md, decisions 1-4) ha tingut èxit;
una línia sense payload vàlid no en genera cap instància (FR-004).

## TokenKind

Enum que classifica un fragment de text dins un payload formatat, perquè `ui/` el tradueixi
a color (research.md, decisió 5). Comú a JSON, XML/HTML i als dos blocs JSON d'un JWT.

```text
Key | StringValue | Number | BoolNull | Punctuation
| TagName | AttrName | AttrValue | Comment | PlainText
```

## StyledLine

Una línia del payload ja formatat i indentat, com una seqüència de fragments de text amb el
seu `TokenKind`.

| Camp | Descripció |
|---|---|
| `segments` | `Vec<(String, TokenKind)>`, en ordre de lectura |

**Comportament**: la indentació és part del text de cada `StyledLine` (espais inicials com
a `TokenKind::PlainText`), no un camp numèric separat — més senzill de renderitzar línia a
línia dins la llista ja existent de `log_view.rs`.

## Estat de desplegament (a `LogViewState`, Fase 1)

Correspon a "Estat de desplegament" de l'spec: si una línia concreta es mostra condensada o
desplegada (FR-006–FR-009). No és una entitat pròpia, és estat afegit a `LogViewState`:

| Camp | Descripció |
|---|---|
| `detected: HashMap<u64, Option<PayloadKind>>` | Memorització per `byte_offset` de línia (research.md, decisió 6) |
| `expanded: HashMap<u64, Vec<StyledLine>>` | Format complet, només per a línies desplegades explícitament |

**Comportament**: una línia és "desplegada" si i només si `expanded` conté una entrada per
al seu `byte_offset`. Clicar l'indicador d'una línia condensada calcula el format (si no hi
és ja a `expanded`) i l'hi afegeix; clicar-la desplegada l'elimina de `expanded` (FR-008).
Cada entrada és independent (FR-009): eliminar-ne una no afecta les altres. Igual que
`ViewportCache` (Fase 1), tots dos mapes s'acoten a una mida màxima i descarten l'entrada
més antiga en superar-la.

## Relacions

```text
Line (tailer, Fase 1) ── byte_offset ── DetectedPayload (0 o 1, via `detected`)
Line (tailer, Fase 1) ── byte_offset ── Vec<StyledLine> (0 o 1, via `expanded`, si desplegada)
DetectedPayload ── kind: PayloadKind
StyledLine ── N segments (String, TokenKind)
```

`format/` no coneix `tailer::Line` ni `ViewportCache`: rep només el text de la línia (`&str`)
i retorna `Option<DetectedPayload>` o `Vec<StyledLine>`. És `ui/log_view.rs` qui lliga
aquests resultats al `byte_offset` de cada `Line` carregada.
