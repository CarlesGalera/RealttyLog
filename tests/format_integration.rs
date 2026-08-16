//! Tests d'integració de la detecció i el formatatge de payloads (Fase 2),
//! aïllats de la GUI (plan.md, Testing).

use realttylog::format::{detect, json, jwt, xml, PayloadKind};

// ---------------------------------------------------------------------
// User Story 1 — detecció (T004-T008)
// ---------------------------------------------------------------------

/// T004: un JSON vàlid es detecta i una clau solta invàlida no.
#[test]
fn detects_valid_json_and_rejects_a_stray_brace() {
    let line = r#"2026-08-16 DEBUG payload={"usuari":"pep","actiu":true}"#;
    let (start, end) = json::detect(line).unwrap();
    assert_eq!(&line[start..end], r#"{"usuari":"pep","actiu":true}"#);

    assert!(json::detect("error: {details missing").is_none());
}

/// T005: un XML ben format es detecta com a `Xml`.
#[test]
fn detects_well_formed_xml() {
    let line = "cos=<usuari><nom>Pep</nom><actiu>true</actiu></usuari> fi";
    let (kind, start, end) = xml::detect(line).unwrap();
    assert_eq!(kind, PayloadKind::Xml);
    assert_eq!(
        &line[start..end],
        "<usuari><nom>Pep</nom><actiu>true</actiu></usuari>"
    );
}

/// T006: un fragment HTML permissiu (`<br>` sense tancar) es detecta com a `Html`.
#[test]
fn detects_permissive_html_with_unclosed_void_element() {
    let line = "pàgina <div>text<br>més<img src=\"x.png\"></div> fi";
    let (kind, ..) = xml::detect(line).unwrap();
    assert_eq!(kind, PayloadKind::Html);
}

/// Trobat provant l'aplicació de debò (T025): un document HTML net i ben
/// tancat també és XML vàlid; sense el marcador d'arrel `<html>`
/// s'etiquetava com a XML, cosa que confonia qui ho llegia.
#[test]
fn well_formed_html_document_is_html_not_xml() {
    let line = "pagina <html><body><h1>404</h1></body></html> fi";
    let (kind, ..) = xml::detect(line).unwrap();
    assert_eq!(kind, PayloadKind::Html);
}

/// T007: la forma d'un JWT es detecta com a `Jwt`.
#[test]
fn detects_the_shape_of_a_jwt() {
    let line = "token=eyJhbGciOiJIUzI1NiJ9abcdef.eyJzdWIiOiIxIn1abcdef.signature123";
    let (start, end) = jwt::detect_shape(line).unwrap();
    assert!(line[start..end].contains('.'));
}

/// T008: una línia de text pla no es detecta.
#[test]
fn plain_text_line_is_never_detected() {
    assert!(detect::detect("2026-08-16 INFO petició rebuda").is_none());
}

#[test]
fn does_not_mistake_a_version_number_for_a_jwt() {
    assert!(jwt::detect_shape("versió 2026.08.16").is_none());
}

#[test]
fn picks_the_leftmost_match_when_a_line_mixes_formats() {
    let line = r#"{"a":1} enmig <b>x</b>"#;
    let payload = detect::detect(line).unwrap();
    assert_eq!(payload.kind, PayloadKind::Json);
    assert_eq!(payload.start, 0);
}

// ---------------------------------------------------------------------
// User Story 2 — formatatge (T014-T015)
// ---------------------------------------------------------------------

/// T014: un JSON es formata conservant totes les dades de l'original (FR-010).
#[test]
fn formats_json_preserving_all_data() {
    let value = json::parse(r#"{"a":1,"b":{"c":[true,null,"x"]}}"#).unwrap();
    let lines = json::format_value(&value);
    let plain: String = lines
        .iter()
        .map(|l| l.plain_text())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(plain.contains("\"a\": 1"));
    assert!(plain.contains("\"c\": ["));
    assert!(plain.contains("true"));
    assert!(plain.contains("null"));
    assert!(plain.contains("\"x\""));
}

/// T015: un XML/HTML es formata indentat per nivell d'imbricació.
#[test]
fn formats_markup_with_indentation_by_nesting_level() {
    let lines = xml::format_markup("<usuari><nom>Pep</nom></usuari>");
    let plain: Vec<String> = lines.iter().map(|l| l.plain_text()).collect();
    assert!(plain[0].contains("<usuari>"));
    assert!(plain[1].starts_with("  ") && plain[1].contains("<nom>"));
}

// ---------------------------------------------------------------------
// User Story 3 — JWT (T019-T020)
// ---------------------------------------------------------------------

const KNOWN_JWT: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IlBlcCJ9.dGVzdC1zaWduYXR1cmU";

/// T019: un JWT conegut decodifica capçalera i payload als valors exactes (SC-005).
#[test]
fn decodes_a_known_jwt_to_the_exact_original_values() {
    let decoded = jwt::decode(KNOWN_JWT).unwrap();

    let header_text = match &decoded.header {
        jwt::JwtPart::Ok(lines) => lines
            .iter()
            .map(|l| l.plain_text())
            .collect::<Vec<_>>()
            .join("\n"),
        jwt::JwtPart::Invalid => panic!("la capçalera hauria de decodificar"),
    };
    assert!(header_text.contains("\"alg\": \"HS256\""));
    assert!(header_text.contains("\"typ\": \"JWT\""));

    let payload_text = match &decoded.payload {
        jwt::JwtPart::Ok(lines) => lines
            .iter()
            .map(|l| l.plain_text())
            .collect::<Vec<_>>()
            .join("\n"),
        jwt::JwtPart::Invalid => panic!("el payload hauria de decodificar"),
    };
    assert!(payload_text.contains("\"sub\": \"1234567890\""));
    assert!(payload_text.contains("\"name\": \"Pep\""));
    assert_eq!(decoded.signature, "dGVzdC1zaWduYXR1cmU");
}

/// T020: un JWT amb el payload corromput mostra un avís en lloc de trencar-se.
#[test]
fn corrupted_jwt_payload_reports_invalid_instead_of_panicking() {
    let broken = "eyJhbGciOiJIUzI1NiJ9.dGhpcyBpcyBub3QgYmFzZTY0dXJsIGpzb24h.sig12345678";
    let decoded = jwt::decode(broken).unwrap();
    assert!(matches!(decoded.payload, jwt::JwtPart::Invalid));
}
