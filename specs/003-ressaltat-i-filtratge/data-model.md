# Data Model: Ressaltat per paraula clau i filtratge instantani

Entitats derivades de `spec.md` §Key Entities. Cap es persisteix (Assumptions: sense
persistència en aquesta fase): viuen només en memòria mentre l'aplicació és oberta.

## RgbColor

Color propi de `rules/`, sense dependència d'`egui` (research.md, decisió 5 de la Fase 2,
mateix patró aplicat aquí).

| Camp | Descripció |
|---|---|
| `r`, `g`, `b` | `u8` cadascun |

`ui/rules_panel.rs` i `ui/log_view.rs` el tradueixen a `egui::Color32` en pintar.

## HighlightRule (Regla de ressaltat)

Correspon a "Regla de ressaltat" de l'spec (§Key Entities, FR-001–FR-005).

| Camp | Descripció |
|---|---|
| `keyword` | Text no buit (FR-004); la coincidència és per subcadena, insensible a majúscules (FR-002) |
| `color` | `RgbColor` assignat |
| `enabled` | Si és `false`, la regla no ressalta ni filtra cap línia (US3, escenari 3) |
| `filter` | Si és `true` **i** `enabled` és `true`, les línies que no la compleixen queden amagades (FR-009–FR-011) |

**Validació**: `keyword` no pot ser buit ni només espais (FR-004, Edge Case); es rebutja en
crear o editar la regla, no es guarda mai un estat invàlid.

**Prioritat**: implícita per la posició al `Vec` de `RuleSet` (research.md, decisió 2) — no
és un camp de la struct.

## RuleSet

Correspon a "Estat de filtratge del mur" de l'spec, més la col·lecció de regles que el
determina.

| Camp | Descripció |
|---|---|
| `rules: Vec<HighlightRule>` | En ordre de creació; les més recents al final |
| `version: u64` | S'incrementa a cada mutació (afegir, editar, activar/desactivar, canviar filtre, esborrar); research.md, decisió 4 |

**Operacions**:

- `add(keyword, color)` — rebutja `keyword` buit (retorna `Err`, no modifica res); si
  s'accepta, la regla nova s'afegeix al final (màxima prioritat) i `version` s'incrementa.
- `remove(index)` — esborra la regla i incrementa `version`.
- `matching_rule(text: &str) -> Option<&HighlightRule>` — recorre `rules` de darrere cap
  endavant, retorna la primera `enabled` que coincideixi (research.md, decisió 2).
- `is_visible(text: &str) -> bool` — `true` si cap regla `enabled` té `filter = true`, o si
  almenys una que ho té coincideix amb `text` (research.md, decisió 3).

Editar `keyword`, `color`, `enabled` o `filter` d'una regla existent (per índex) també
incrementa `version`, sense canviar-ne la posició (i per tant la prioritat).

## Coincidència memoritzada (a `LogViewState`)

Estat afegit a `LogViewState` (mateix mòdul que `detected`/`expanded` de la Fase 2), no una
entitat pròpia de negoci:

| Camp | Descripció |
|---|---|
| `rule_match: HashMap<u64, (Option<usize>, bool)>` | Per `byte_offset` de línia: índex a `rules.rules` de la regla que hi coincideix per ressaltar-la (si n'hi ha), i si la línia és visible sota el filtre actiu |
| `rule_match_version: u64` | Versió de `RuleSet` amb què es va omplir `rule_match` |

**Comportament**: abans de llegir `rule_match`, si `rule_match_version != rules.version()`,
es buida `rule_match` sencer i s'actualitza `rule_match_version` (research.md, decisió 4).
Els dos valors (regla de ressaltat i visibilitat) es calculen junts per línia i es
memoritzen en un sol pas: no són el mateix càlcul (`matching_rule` és la de més prioritat;
`is_visible` mira totes les regles amb filtre actiu, encara que no siguin la de més
prioritat), però totes dues depenen només del text de la línia i de `rules`, així que
comparteixen la mateixa invalidació per versió.

## Relacions

```text
App ── posseeix ── RuleSet (research.md, decisió 5)
RuleSet ── conté ── Vec<HighlightRule>
Line (tailer, Fase 1) ── byte_offset ── Option<usize> (índex a HighlightRule, via `rule_match`)
HighlightRule ── color: RgbColor
```

`rules/` no coneix `tailer::Line` ni `ViewportCache`: rep només el text de la línia (`&str`)
i retorna `Option<&HighlightRule>` o `bool`. És `ui/log_view.rs` qui lliga aquests resultats
al `byte_offset` de cada `Line` carregada, igual que ja fa amb `format::detect`.
