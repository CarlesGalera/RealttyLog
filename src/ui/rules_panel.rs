use eframe::egui;

use crate::rules::{RgbColor, RuleSet};

/// Estat transitori del formulari d'afegir regla (Fase 3, US1/US3). Viu a
/// `LogViewState`, no a `RuleSet`: és estat de la interfície (què hi ha
/// escrit al formulari abans de prémer "Afegir"), no una regla ja desada.
pub struct RulesPanelState {
    new_keyword: String,
    new_color: egui::Color32,
    error: Option<String>,
}

impl Default for RulesPanelState {
    fn default() -> Self {
        Self {
            new_keyword: String::new(),
            new_color: egui::Color32::from_rgb(220, 90, 90),
            error: None,
        }
    }
}

/// Dibuixa el panell de regles: formulari per afegir-ne una de nova i la
/// llista de les existents, amb edició, (des)activació, filtre i esborrat
/// en línia (FR-001–FR-003, FR-009).
pub fn ui(ui: &mut egui::Ui, rules: &mut RuleSet, state: &mut RulesPanelState) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(8, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Nova regla:");
                ui.text_edit_singleline(&mut state.new_keyword);
                ui.color_edit_button_srgba(&mut state.new_color);
                if ui.button("Afegir").clicked() {
                    let color = RgbColor::new(
                        state.new_color.r(),
                        state.new_color.g(),
                        state.new_color.b(),
                    );
                    match rules.add(state.new_keyword.trim(), color) {
                        Ok(()) => {
                            state.new_keyword.clear();
                            state.error = None;
                        }
                        Err(msg) => state.error = Some(msg.to_string()),
                    }
                }
            });
            if let Some(err) = &state.error {
                ui.colored_label(egui::Color32::RED, err);
            }

            let mut to_remove = None;
            for index in 0..rules.rules().len() {
                let mut changed = false;
                if let Some(rule) = rules.rule_mut(index) {
                    ui.horizontal(|ui| {
                        let mut color =
                            egui::Color32::from_rgb(rule.color.r, rule.color.g, rule.color.b);
                        if ui.color_edit_button_srgba(&mut color).changed() {
                            rule.color = RgbColor::new(color.r(), color.g(), color.b());
                            changed = true;
                        }
                        if ui.text_edit_singleline(&mut rule.keyword).changed() {
                            changed = true;
                        }
                        if ui.checkbox(&mut rule.enabled, "Activa").changed() {
                            changed = true;
                        }
                        if ui.checkbox(&mut rule.filter, "Filtra").changed() {
                            changed = true;
                        }
                        if ui.button("Esborrar").clicked() {
                            to_remove = Some(index);
                        }
                    });
                }
                if changed {
                    rules.bump_version();
                }
            }
            if let Some(index) = to_remove {
                rules.remove(index);
            }
        });
}
