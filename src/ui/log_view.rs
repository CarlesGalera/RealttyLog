use eframe::egui;

use crate::tailer::{FollowState, FollowedFile};

/// Vista d'un fitxer obert: les línies carregades, l'indicador de directe/
/// pausat i l'acció de tornar a la llista de resultats (FR-010, FR-011,
/// FR-017–FR-019).
pub struct LogViewState {
    pub file: FollowedFile,
}

impl LogViewState {
    pub fn new(file: FollowedFile) -> Self {
        Self { file }
    }

    /// Retorna `true` si l'usuari ha demanat tornar a la llista de
    /// resultats (FR-011): l'estat de la cerca no es toca, és qui crida qui
    /// decideix descartar aquesta vista.
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        if self.file.poll() {
            ui.ctx().request_repaint();
        }

        let mut back_requested = false;
        ui.horizontal(|ui| {
            if ui.button("< Resultats").clicked() {
                back_requested = true;
            }
            ui.label(self.file.path.display().to_string());
            self.state_indicator(ui);
        });

        // Un desplaçament manual mentre se segueix en directe és senyal
        // inequívoc que l'usuari vol repassar l'historial (FR-006): es
        // pausa abans de dibuixar la llista, perquè aquest mateix frame ja
        // no forci la vista cap avall i el desplaçament es senti immediat.
        let user_scrolled = ui.input(|i| i.smooth_scroll_delta.y.abs() > 0.0);
        if user_scrolled && self.file.state == FollowState::Live {
            self.file.pause();
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for line in &self.file.viewport.lines {
                    ui.label(&line.content);
                }
                if self.file.state == FollowState::Live {
                    // FR-005: autoscroll a l'última línia mentre se segueix
                    // en directe.
                    ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                }
            });

        back_requested
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
