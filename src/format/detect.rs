use super::{json, jwt, xml, DetectedPayload, PayloadKind};

/// Detecta el primer payload (JSON, XML, HTML o JWT) que apareix a `line`
/// (FR-001–FR-005). Prova els quatre formats i es queda amb el que
/// comença més a l'esquerra, no amb un ordre de prioritat fix, perquè
/// "el primer que apareix" (spec.md, Edge Cases) valgui també quan es
/// barregen formats diferents a la mateixa línia.
pub fn detect(line: &str) -> Option<DetectedPayload> {
    let mut candidates = Vec::with_capacity(2);

    if let Some((start, end)) = jwt::detect_shape(line) {
        candidates.push(DetectedPayload {
            kind: PayloadKind::Jwt,
            start,
            end,
        });
    }
    if let Some((start, end)) = json::detect(line) {
        candidates.push(DetectedPayload {
            kind: PayloadKind::Json,
            start,
            end,
        });
    }
    if let Some((kind, start, end)) = xml::detect(line) {
        candidates.push(DetectedPayload { kind, start, end });
    }

    candidates.into_iter().min_by_key(|c| c.start)
}
