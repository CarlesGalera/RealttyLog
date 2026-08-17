use serde_json::Value;

use super::styled::{StyledLine, TokenKind};

const INDENT: &str = "  ";

/// Cerca el primer JSON vàlid dins `line`, escanejant cada `{`/`[` d'esquerra
/// a dreta fins trobar-ne un que parsegi (FR-001, FR-005). Retorna els
/// offsets en bytes d'inici i final dins `line`.
pub fn detect(line: &str) -> Option<(usize, usize)> {
    for (idx, ch) in line.char_indices() {
        if ch != '{' && ch != '[' {
            continue;
        }
        // Un `{`/`[` enganxat a un identificador (p. ex. el `[]` final de
        // la notació de tipus `System.String[]`) no és mai l'inici d'un
        // JSON real dins un log: sempre hi ha un separador (espai, `:`,
        // `=`...) abans, o és l'inici de línia.
        let prev_is_identifier = line[..idx]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_');
        if prev_is_identifier {
            continue;
        }
        let sub = &line[idx..];
        let mut stream = serde_json::Deserializer::from_str(sub).into_iter::<Value>();
        if let Some(Ok(_)) = stream.next() {
            let consumed = stream.byte_offset();
            if consumed > 0 {
                return Some((idx, idx + consumed));
            }
        }
    }
    None
}

/// Parseja `text` com a JSON. Es fa servir tant per al payload detectat amb
/// [`detect`] com per als segments descodificats d'un JWT (research.md,
/// decisió 4).
pub fn parse(text: &str) -> Option<Value> {
    serde_json::from_str(text).ok()
}

/// Formata un `Value` ja parsejat com a línies indentades i estilades per
/// tipus de dada (FR-007, FR-010).
pub fn format_value(value: &Value) -> Vec<StyledLine> {
    let mut out = Vec::new();
    let mut prefix = StyledLine::new();
    render(value, 0, &mut prefix, &mut out);
    out
}

fn indent(level: usize) -> String {
    INDENT.repeat(level)
}

fn render(value: &Value, level: usize, prefix: &mut StyledLine, out: &mut Vec<StyledLine>) {
    match value {
        Value::Object(map) if map.is_empty() => {
            prefix.push("{}", TokenKind::Punctuation);
            out.push(std::mem::take(prefix));
        }
        Value::Object(map) => {
            prefix.push("{", TokenKind::Punctuation);
            out.push(std::mem::take(prefix));
            let last = map.len().saturating_sub(1);
            for (i, (key, val)) in map.iter().enumerate() {
                let mut line = StyledLine::new();
                line.push(indent(level + 1), TokenKind::PlainText);
                line.push(format!("{key:?}"), TokenKind::Key);
                line.push(": ", TokenKind::Punctuation);
                render(val, level + 1, &mut line, out);
                if i != last {
                    if let Some(last_line) = out.last_mut() {
                        last_line.push(",", TokenKind::Punctuation);
                    }
                }
            }
            let mut close = StyledLine::new();
            close.push(indent(level), TokenKind::PlainText);
            close.push("}", TokenKind::Punctuation);
            out.push(close);
        }
        Value::Array(items) if items.is_empty() => {
            prefix.push("[]", TokenKind::Punctuation);
            out.push(std::mem::take(prefix));
        }
        Value::Array(items) => {
            prefix.push("[", TokenKind::Punctuation);
            out.push(std::mem::take(prefix));
            let last = items.len().saturating_sub(1);
            for (i, val) in items.iter().enumerate() {
                let mut line = StyledLine::new();
                line.push(indent(level + 1), TokenKind::PlainText);
                render(val, level + 1, &mut line, out);
                if i != last {
                    if let Some(last_line) = out.last_mut() {
                        last_line.push(",", TokenKind::Punctuation);
                    }
                }
            }
            let mut close = StyledLine::new();
            close.push(indent(level), TokenKind::PlainText);
            close.push("]", TokenKind::Punctuation);
            out.push(close);
        }
        Value::String(s) => {
            prefix.push(format!("{s:?}"), TokenKind::StringValue);
            out.push(std::mem::take(prefix));
        }
        Value::Number(n) => {
            prefix.push(n.to_string(), TokenKind::Number);
            out.push(std::mem::take(prefix));
        }
        Value::Bool(b) => {
            prefix.push(b.to_string(), TokenKind::BoolNull);
            out.push(std::mem::take(prefix));
        }
        Value::Null => {
            prefix.push("null", TokenKind::BoolNull);
            out.push(std::mem::take(prefix));
        }
    }
}
