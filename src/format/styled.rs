/// Classifica un fragment de text dins d'un payload formatat, perquè `ui/`
/// el tradueixi a color (research.md, decisió 5). Comú a JSON, XML/HTML i
/// als dos blocs JSON d'un JWT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Key,
    StringValue,
    Number,
    BoolNull,
    Punctuation,
    TagName,
    AttrName,
    AttrValue,
    Comment,
    PlainText,
}

/// Una línia del payload ja formatat i indentat: una seqüència de
/// fragments de text amb el seu `TokenKind`. La indentació és part del
/// text de cada fragment (espais com a `PlainText`), no un camp numèric a
/// part (data-model.md).
#[derive(Debug, Clone, Default)]
pub struct StyledLine {
    pub segments: Vec<(String, TokenKind)>,
}

impl StyledLine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, text: impl Into<String>, kind: TokenKind) -> &mut Self {
        self.segments.push((text.into(), kind));
        self
    }

    /// El text pla de la línia, sense estils — útil per als tests i per
    /// verificar que el formatatge no perd cap dada (FR-010).
    pub fn plain_text(&self) -> String {
        self.segments.iter().map(|(t, _)| t.as_str()).collect()
    }
}
