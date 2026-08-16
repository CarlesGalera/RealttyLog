use eframe::egui;

use crate::tailer::FollowedFile;

/// Vista d'un fitxer obert: les línies carregades i l'acció de tornar a la
/// llista de resultats (FR-010, FR-011).
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
        let mut back_requested = false;
        ui.horizontal(|ui| {
            if ui.button("< Resultats").clicked() {
                back_requested = true;
            }
            ui.label(self.file.path.display().to_string());
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            for line in &self.file.viewport.lines {
                ui.label(&line.content);
            }
        });

        back_requested
    }
}
