use quick_xml::events::Event;
use quick_xml::reader::Reader;

use super::styled::{StyledLine, TokenKind};
use super::PayloadKind;

const INDENT: &str = "  ";

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Cerca el primer XML o HTML dins `line` (research.md, decisió 3). Si
/// l'arrel té pinta de document HTML (`<html`, `<!DOCTYPE html`), es tracta
/// com a HTML encara que resulti ser estrictament ben format: un
/// `<html><body>...</body></html>` net és HTML per a qualsevol persona que
/// el llegeixi, no XML, i exigir-hi manca de bona formació per etiquetar-lo
/// com a tal donaria una classificació tècnicament certa però confusa.
/// Per a la resta de marcatge, es prova XML estricte primer i només si
/// falla, marcatge permissiu.
pub fn detect(line: &str) -> Option<(PayloadKind, usize, usize)> {
    for (idx, ch) in line.char_indices() {
        if ch != '<' {
            continue;
        }
        let sub = &line[idx..];
        if looks_like_html_document(sub) {
            if let Some(end) = detect_permissive_markup(sub) {
                return Some((PayloadKind::Html, idx, idx + end));
            }
        }
        if let Some(end) = detect_strict_xml(sub) {
            return Some((PayloadKind::Xml, idx, idx + end));
        }
        if let Some(end) = detect_permissive_markup(sub) {
            return Some((PayloadKind::Html, idx, idx + end));
        }
    }
    None
}

fn looks_like_html_document(sub: &str) -> bool {
    let head: String = sub
        .chars()
        .take(15)
        .collect::<String>()
        .to_ascii_lowercase();
    head.starts_with("<!doctype html") || head.starts_with("<html")
}

/// Un XML ben format: totes les etiquetes aparellades pel seu nom exacte
/// (FR-004). Retorna el nombre de bytes consumits fins tancar l'arrel.
fn detect_strict_xml(sub: &str) -> Option<usize> {
    let mut reader = Reader::from_str(sub);
    let mut depth: i32 = 0;
    let mut opened = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(_)) => {
                depth += 1;
                opened = true;
            }
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
                if opened && depth == 0 {
                    return Some(reader.buffer_position() as usize);
                }
            }
            Ok(Event::Empty(_)) => {
                opened = true;
                if depth == 0 {
                    return Some(reader.buffer_position() as usize);
                }
            }
            Ok(Event::Eof) => return None,
            Ok(_) => {}
            Err(_) => return None,
        }
    }
}

/// Marcatge prou versemblant per considerar-lo HTML (research.md, decisió
/// 2): no exigeix escapar `&` ni tancar elements buits (`<br>`, `<img>`),
/// a diferència d'un parser XML estricte. Escrit a mà en lloc de fer
/// servir `quick-xml` en mode permissiu: les seves opcions de relaxació
/// no cobreixen `&` sense escapar, que és habitual en HTML real.
fn detect_permissive_markup(sub: &str) -> Option<usize> {
    let mut i = 0;
    let mut depth: i32 = 0;
    let mut opened = false;
    while i < sub.len() {
        if !sub.is_char_boundary(i) || sub.as_bytes()[i] != b'<' {
            i += 1;
            continue;
        }
        let Some(rel_end) = sub[i..].find('>') else {
            break;
        };
        let tag = &sub[i + 1..i + rel_end];
        let end_pos = i + rel_end + 1;
        if let Some(name) = tag.strip_prefix('/') {
            if name.trim().is_empty() {
                return None;
            }
            depth -= 1;
            if opened && depth <= 0 {
                return Some(end_pos);
            }
        } else if tag.starts_with('!') || tag.starts_with('?') {
            // Comentari, DOCTYPE o instrucció de processament: no compta
            // per a la profunditat d'imbricació.
        } else {
            let name_end = tag
                .find(|c: char| c.is_whitespace() || c == '/')
                .unwrap_or(tag.len());
            let name = tag[..name_end].to_ascii_lowercase();
            if name.is_empty() || !name.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
                return None;
            }
            opened = true;
            if tag.trim_end().ends_with('/') || VOID_ELEMENTS.contains(&name.as_str()) {
                if depth == 0 {
                    return Some(end_pos);
                }
            } else {
                depth += 1;
            }
        }
        i = end_pos;
    }
    None
}

/// Formata un fragment de marcatge (XML o HTML) com a línies indentades
/// per nivell d'imbricació, amb etiquetes i atributs ressaltats (FR-007).
/// Fa servir `quick-xml` en mode permissiu (`check_end_names(false)`) amb
/// els dos formats: un cop se sap que el text ja ha passat la detecció,
/// no cal tornar a exigir-hi estricta bona formació per mostrar-lo.
pub fn format_markup(text: &str) -> Vec<StyledLine> {
    let mut reader = Reader::from_str(text);
    reader.config_mut().check_end_names = false;
    reader.config_mut().trim_text(true);

    let mut out = Vec::new();
    let mut level: usize = 0;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                out.push(tag_line(level, &e, false));
                level += 1;
            }
            Ok(Event::End(e)) => {
                level = level.saturating_sub(1);
                let mut line = StyledLine::new();
                line.push(indent(level), TokenKind::PlainText);
                line.push(
                    format!("</{}>", String::from_utf8_lossy(e.name().as_ref())),
                    TokenKind::TagName,
                );
                out.push(line);
            }
            Ok(Event::Empty(e)) => {
                out.push(tag_line(level, &e, true));
            }
            Ok(Event::Text(t)) => {
                let text = String::from_utf8_lossy(&t).trim().to_string();
                if !text.is_empty() {
                    let mut line = StyledLine::new();
                    line.push(indent(level), TokenKind::PlainText);
                    line.push(text, TokenKind::PlainText);
                    out.push(line);
                }
            }
            Ok(Event::Comment(c)) => {
                let mut line = StyledLine::new();
                line.push(indent(level), TokenKind::PlainText);
                line.push(
                    format!("<!--{}-->", String::from_utf8_lossy(&c)),
                    TokenKind::Comment,
                );
                out.push(line);
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }
    out
}

fn tag_line(level: usize, e: &quick_xml::events::BytesStart, self_closing: bool) -> StyledLine {
    let mut line = StyledLine::new();
    line.push(indent(level), TokenKind::PlainText);
    line.push("<", TokenKind::TagName);
    line.push(
        String::from_utf8_lossy(e.name().as_ref()).into_owned(),
        TokenKind::TagName,
    );
    for attr in e.attributes().flatten() {
        line.push(" ", TokenKind::PlainText);
        line.push(
            String::from_utf8_lossy(attr.key.as_ref()).into_owned(),
            TokenKind::AttrName,
        );
        line.push("=", TokenKind::Punctuation);
        // `unescape_value` és l'obsoleta; la substituta exigeix triar una
        // versió XML que no ve al cas per a un simple indentador de text.
        #[allow(deprecated)]
        let value = attr.unescape_value().unwrap_or_default();
        line.push(format!("{value:?}"), TokenKind::AttrValue);
    }
    line.push(if self_closing { "/>" } else { ">" }, TokenKind::TagName);
    line
}

fn indent(level: usize) -> String {
    INDENT.repeat(level)
}
