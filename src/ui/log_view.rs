use std::collections::HashMap;

use eframe::egui;

use crate::format::{self, jwt, PayloadKind, StyledLine, TokenKind};
use crate::rules::{RgbColor, RuleSet};
use crate::tailer::{FollowState, FollowedFile};
use crate::ui::rules_panel;

/// Nombre màxim d'entrades que es mantenen a cadascuna de les memòries cau
/// de la Fase 2 (research.md, decisió 6). En superar-lo, es buiden
/// senceres: és una mica més barroer que descartar només la més antiga,
/// però igual de vàlid per acotar la memòria i molt més senzill.
const DETECTION_CACHE_CAP: usize = 4000;
const EXPANDED_CACHE_CAP: usize = 200;

/// Vista d'un fitxer obert: les línies carregades, l'indicador de directe/
/// pausat, els payloads detectats i desplegats (Fase 2), el ressaltat i
/// filtratge per regla (Fase 3), i l'acció de tornar a la llista de
/// resultats (FR-010, FR-011, FR-017–FR-019).
pub struct LogViewState {
    pub file: FollowedFile,
    detected: HashMap<u64, Option<PayloadKind>>,
    expanded: HashMap<u64, Vec<StyledLine>>,
    rule_match: HashMap<u64, (Option<RgbColor>, bool)>,
    rule_match_version: u64,
    show_rules_panel: bool,
    rules_panel: rules_panel::RulesPanelState,
}

impl LogViewState {
    pub fn new(file: FollowedFile) -> Self {
        Self {
            file,
            detected: HashMap::new(),
            expanded: HashMap::new(),
            rule_match: HashMap::new(),
            rule_match_version: 0,
            show_rules_panel: false,
            rules_panel: rules_panel::RulesPanelState::default(),
        }
    }

    /// Retorna `true` si l'usuari ha demanat tornar a la llista de
    /// resultats (FR-011): l'estat de la cerca no es toca, és qui crida qui
    /// decideix descartar aquesta vista.
    pub fn ui(&mut self, ui: &mut egui::Ui, rules: &mut RuleSet) -> bool {
        if self.file.poll() {
            ui.ctx().request_repaint();
        }

        // Un desplaçament manual mentre se segueix en directe és senyal
        // inequívoc que l'usuari vol repassar l'historial (FR-006). Cal
        // detectar-ho i pausar ABANS de dibuixar res: l'indicador de sota i
        // el força-scroll de la llista han de veure ja l'estat corregit en
        // aquest mateix frame, no un de retardat.
        let user_scrolled = ui.input(|i| i.smooth_scroll_delta.y.abs() > 0.0);
        if user_scrolled && self.file.state == FollowState::Live {
            self.file.pause();
            ui.ctx().request_repaint();
        }

        let mut back_requested = false;
        ui.horizontal(|ui| {
            if ui.button("< Resultats").clicked() {
                back_requested = true;
            }
            ui.label(self.file.path.display().to_string());
            self.state_indicator(ui);
            if ui.button("Regles").clicked() {
                self.show_rules_panel = !self.show_rules_panel;
            }
        });

        if self.show_rules_panel {
            rules_panel::ui(ui, rules, &mut self.rules_panel);
        }

        self.ensure_detection_cached();
        self.ensure_rule_match_cached(rules);

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let offsets: Vec<u64> = self
                    .file
                    .viewport
                    .lines
                    .iter()
                    .filter(|l| {
                        self.rule_match
                            .get(&l.byte_offset)
                            .is_none_or(|(_, visible)| *visible)
                    })
                    .map(|l| l.byte_offset)
                    .collect();

                if offsets.is_empty() && rules.has_active_filter() {
                    ui.colored_label(
                        egui::Color32::from_rgb(200, 140, 0),
                        "⚠ Cap línia compleix el filtre actiu",
                    );
                }

                for offset in offsets {
                    self.render_line(ui, offset);
                }
                if self.file.state == FollowState::Live {
                    // FR-005 (Fase 1): autoscroll a l'última línia mentre
                    // se segueix en directe.
                    ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                }
            });

        back_requested
    }

    /// Detecta el payload (si n'hi ha) de cada línia carregada que encara
    /// no s'hagi mirat, i el memoritza (FR-001, FR-014: només la finestra
    /// carregada, mai tot el fitxer).
    fn ensure_detection_cached(&mut self) {
        if self.detected.len() > DETECTION_CACHE_CAP {
            self.detected.clear();
        }
        for line in &self.file.viewport.lines {
            self.detected
                .entry(line.byte_offset)
                .or_insert_with(|| format::detect::detect(&line.content).map(|p| p.kind));
        }
    }

    /// Calcula, per a cada línia carregada, quina regla la ressalta i si és
    /// visible sota el filtre actiu, i ho memoritza (research.md, decisió
    /// 4). Si les regles han canviat des de l'últim càlcul, es buida tot
    /// abans de tornar a omplir-lo.
    fn ensure_rule_match_cached(&mut self, rules: &RuleSet) {
        if self.rule_match_version != rules.version() {
            self.rule_match.clear();
            self.rule_match_version = rules.version();
        }
        for line in &self.file.viewport.lines {
            self.rule_match.entry(line.byte_offset).or_insert_with(|| {
                let highlight = rules
                    .matching_rule(&line.content)
                    .map(|(_, rule)| rule.color);
                let visible = rules.is_visible(&line.content);
                (highlight, visible)
            });
        }
    }

    fn render_line(&mut self, ui: &mut egui::Ui, offset: u64) {
        let kind = self.detected.get(&offset).copied().flatten();
        let highlight_color = self
            .rule_match
            .get(&offset)
            .and_then(|(color, _)| *color)
            .map(to_color32);

        ui.horizontal(|ui| {
            if let Some(line) = self
                .file
                .viewport
                .lines
                .iter()
                .find(|l| l.byte_offset == offset)
            {
                match highlight_color {
                    Some(color) => {
                        ui.colored_label(color, &line.content);
                    }
                    None => {
                        ui.label(&line.content);
                    }
                }
            }
            if let Some(kind) = kind {
                if ui.small_button(badge(kind)).clicked() {
                    self.toggle_expand(offset, kind);
                }
            }
        });

        if let Some(styled) = self.expanded.get(&offset) {
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(16, 4))
                .show(ui, |ui| {
                    for line in styled {
                        render_styled_line(ui, line, highlight_color);
                    }
                });
        }
    }

    fn toggle_expand(&mut self, offset: u64, kind: PayloadKind) {
        if self.expanded.remove(&offset).is_some() {
            return;
        }
        let Some(line) = self
            .file
            .viewport
            .lines
            .iter()
            .find(|l| l.byte_offset == offset)
        else {
            return;
        };
        let Some(payload) = format::detect::detect(&line.content) else {
            return;
        };
        let text = &line.content[payload.start..payload.end];
        let styled = match kind {
            PayloadKind::Json => format::json::parse(text)
                .map(|v| format::json::format_value(&v))
                .unwrap_or_else(|| vec![invalid_line("El JSON ja no és vàlid.")]),
            PayloadKind::Xml | PayloadKind::Html => format::xml::format_markup(text),
            PayloadKind::Jwt => match jwt::decode(text) {
                Some(decoded) => jwt_styled_lines(&decoded),
                None => vec![invalid_line("El token JWT no té la forma esperada.")],
            },
        };

        if self.expanded.len() > EXPANDED_CACHE_CAP {
            self.expanded.clear();
        }
        self.expanded.insert(offset, styled);
    }

    fn state_indicator(&mut self, ui: &mut egui::Ui) {
        match self.file.state {
            FollowState::Live => {
                ui.colored_label(egui::Color32::from_rgb(0, 160, 0), "● En directe");
            }
            FollowState::Paused => {
                let label = if self.file.has_new_content_while_paused {
                    "⏸ Pausat — han arribat línies noves"
                } else {
                    "⏸ Pausat"
                };
                ui.colored_label(egui::Color32::from_rgb(200, 140, 0), label);
                // FR-007/FR-018: reprendre amb una sola acció, saltant a la
                // cua actual del fitxer.
                if ui.button("Tornar al directe").clicked() {
                    if let Err(err) = self.file.resume_live() {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("No s'ha pogut reprendre: {err}"),
                        );
                    }
                }
            }
            FollowState::Unavailable => {
                ui.colored_label(egui::Color32::RED, "⚠ Fitxer no disponible");
            }
        }
    }
}

fn badge(kind: PayloadKind) -> &'static str {
    match kind {
        PayloadKind::Json => "{ } JSON",
        PayloadKind::Xml => "</> XML",
        PayloadKind::Html => "</> HTML",
        PayloadKind::Jwt => "JWT",
    }
}

fn invalid_line(message: &str) -> StyledLine {
    let mut line = StyledLine::new();
    line.push(message.to_string(), TokenKind::PlainText);
    line
}

fn jwt_styled_lines(decoded: &jwt::DecodedJwt) -> Vec<StyledLine> {
    let mut out = Vec::new();

    let mut header_title = StyledLine::new();
    header_title.push("Capçalera:", TokenKind::Key);
    out.push(header_title);
    match &decoded.header {
        jwt::JwtPart::Ok(lines) => out.extend(lines.iter().cloned()),
        jwt::JwtPart::Invalid => out.push(invalid_line("  (no s'ha pogut descodificar)")),
    }

    let mut payload_title = StyledLine::new();
    payload_title.push("Payload:", TokenKind::Key);
    out.push(payload_title);
    match &decoded.payload {
        jwt::JwtPart::Ok(lines) => out.extend(lines.iter().cloned()),
        jwt::JwtPart::Invalid => out.push(invalid_line("  (no s'ha pogut descodificar)")),
    }

    let mut sig_title = StyledLine::new();
    sig_title.push("Signatura (no desxifrable): ", TokenKind::Key);
    sig_title.push(decoded.signature.clone(), TokenKind::PlainText);
    out.push(sig_title);

    out
}

/// `highlight` sobreescriu el color de tots els tokens quan la línia
/// condensada corresponent compleix una regla de ressaltat activa (Fase 3,
/// quickstart Escenari N): el text desplegat s'ha de veure ressaltat igual
/// que la línia condensada, en lloc del color per tipus de dada.
fn render_styled_line(ui: &mut egui::Ui, line: &StyledLine, highlight: Option<egui::Color32>) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for (text, kind) in &line.segments {
            ui.colored_label(highlight.unwrap_or_else(|| token_color(*kind)), text);
        }
    });
}

fn token_color(kind: TokenKind) -> egui::Color32 {
    match kind {
        TokenKind::Key => egui::Color32::from_rgb(120, 170, 255),
        TokenKind::StringValue => egui::Color32::from_rgb(140, 210, 140),
        TokenKind::Number => egui::Color32::from_rgb(220, 170, 90),
        TokenKind::BoolNull => egui::Color32::from_rgb(190, 140, 220),
        TokenKind::Punctuation => egui::Color32::from_rgb(150, 150, 150),
        TokenKind::TagName => egui::Color32::from_rgb(120, 170, 255),
        TokenKind::AttrName => egui::Color32::from_rgb(140, 200, 200),
        TokenKind::AttrValue => egui::Color32::from_rgb(140, 210, 140),
        TokenKind::Comment => egui::Color32::from_rgb(120, 120, 120),
        TokenKind::PlainText => egui::Color32::from_rgb(220, 220, 220),
    }
}

fn to_color32(color: RgbColor) -> egui::Color32 {
    egui::Color32::from_rgb(color.r, color.g, color.b)
}
