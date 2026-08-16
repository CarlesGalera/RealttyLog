# Data Model: Configuració portable de les regles

No introdueix cap entitat de negoci nova: persisteix les que ja va definir la Fase 3
(`HighlightRule`, `RgbColor`) tal com són, amb dues macros de derivació afegides.

## Format del fitxer de configuració

Un array JSON, un element per regla, a `<directori de l'executable>/realttylog-rules.json`:

```json
[
  {
    "keyword": "ERROR",
    "color": { "r": 220, "g": 90, "b": 90 },
    "enabled": true,
    "filter": false
  },
  {
    "keyword": "WARN",
    "color": { "r": 216, "g": 220, "b": 90 },
    "enabled": true,
    "filter": true
  }
]
```

Correspondència directa amb els camps públics de `HighlightRule` (Fase 3, data-model.md):
`keyword`, `color` (un objecte `RgbColor` amb `r`, `g`, `b`), `enabled`, `filter`. No es
persisteix cap prioritat explícita: es reconstrueix implícitament per l'ordre dels
elements a l'array, exactament com `RuleSet::rules` (Fase 3, research.md decisió 2) —
la primera regla de l'array és la de menys prioritat, l'última la de més.

`RuleSet::version` (Fase 3) MUST NOT persistir-se: és un comptador de memorització en
memòria (research.md, decisió 4 de la Fase 3), sense cap sentit entre execucions
diferents. En carregar, es reconstrueix un `RuleSet` nou amb `version = 0` via
`RuleSet::from_rules(rules)`.

## Tolerància a errors (FR-006, FR-007)

| Situació | Comportament |
|---|---|
| El fitxer no existeix | `RuleSet` buit, com un primer ús |
| El fitxer existeix però no és JSON vàlid | `RuleSet` buit |
| El fitxer és un array JSON vàlid, però un element no és un objecte `HighlightRule` vàlid (camp absent, tipus incorrecte) | Aquell element es descarta; la resta es carreguen amb normalitat |
| Una regla vàlida però amb `keyword` buit o només espais | Es descarta igual que si s'hagués intentat crear via `RuleSet::add` (Fase 3, FR-004) |

## Relacions

```text
App ── posseeix ── RuleSet (Fase 3)
RuleSet ── conté ── Vec<HighlightRule>
config::load()/save() ── (de)serialitza ── Vec<HighlightRule> ⇄ realttylog-rules.json
```

`config` no coneix `App` ni la GUI: rep o retorna sempre un `RuleSet` (o, a
`load_from`/`save_to`, un `&Path` explícit). És `App::new()` i `App::ui()` qui decideixen
quan cridar-lo (research.md, decisions 3 i 5).
