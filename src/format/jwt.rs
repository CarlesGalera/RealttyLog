use std::sync::OnceLock;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use regex::Regex;

use super::styled::StyledLine;

fn jwt_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Capçalera i payload d'un JWT real codifiquen com a mínim un objecte
    // JSON petit; exigir-hi un mínim de 10 caràcters evita marcar coses com
    // un número de versió amb punts com si fos un token (FR-004).
    RE.get_or_init(|| {
        Regex::new(r"[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]+").unwrap()
    })
}

/// Cerca la forma d'un JWT (tres segments base64url separats per punts)
/// dins `line` (FR-011). No el descodifica encara: això només passa quan
/// l'usuari desplega la línia (User Story 3).
pub fn detect_shape(line: &str) -> Option<(usize, usize)> {
    jwt_pattern().find(line).map(|m| (m.start(), m.end()))
}

/// El resultat de descodificar un segment d'un JWT com a JSON.
pub enum JwtPart {
    Ok(Vec<StyledLine>),
    Invalid,
}

pub struct DecodedJwt {
    pub header: JwtPart,
    pub payload: JwtPart,
    /// La signatura tal com és: no es pot desxifrar sense la clau (FR-012).
    pub signature: String,
}

/// Descodifica els dos primers segments d'un token com a JSON, reutilitzant
/// el mateix formatador (research.md, decisió 4). La signatura es
/// conserva tal qual.
pub fn decode(token: &str) -> Option<DecodedJwt> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some(DecodedJwt {
        header: decode_json_part(parts[0]),
        payload: decode_json_part(parts[1]),
        signature: parts[2].to_string(),
    })
}

fn decode_json_part(segment: &str) -> JwtPart {
    let Ok(bytes) = URL_SAFE_NO_PAD.decode(segment) else {
        return JwtPart::Invalid;
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return JwtPart::Invalid;
    };
    match super::json::parse(text) {
        Some(value) => JwtPart::Ok(super::json::format_value(&value)),
        None => JwtPart::Invalid,
    }
}
