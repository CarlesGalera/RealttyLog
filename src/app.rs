use eframe::egui;

use crate::rules::RuleSet;
use crate::tailer::{FollowedFile, OpenAt};
use crate::ui::log_view::LogViewState;
use crate::ui::search_view::{SearchViewAction, SearchViewState};

/// Estat de l'aplicació: la cerca es manté sempre viva perquè "tornar als
/// resultats" (FR-011) no impliqui repetir-la; `open_file` només és `Some`
/// mentre l'usuari mira un fitxer concret. `rules` viu aquí, no a
/// `LogViewState`, perquè sobrevisqui a tancar i obrir fitxers diferents
/// dins la mateixa sessió (Fase 3, research.md decisió 5).
#[derive(Default)]
pub struct App {
    pub search: SearchViewState,
    pub open_file: Option<LogViewState>,
    pub rules: RuleSet,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(log_view) = &mut self.open_file {
                if log_view.ui(ui, &mut self.rules) {
                    self.open_file = None;
                }
                return;
            }

            let Some(action) = self.search.ui(ui) else {
                return;
            };
            let (path, at) = match action {
                SearchViewAction::OpenMatch(m) => (m.file_path, OpenAt::Offset(m.byte_offset)),
                SearchViewAction::OpenDirect(path) => (path, OpenAt::End),
            };
            match FollowedFile::open(path.clone(), at) {
                Ok(file) => self.open_file = Some(LogViewState::new(file)),
                Err(err) => self
                    .search
                    .set_open_error(format!("No s'ha pogut obrir {}: {err}", path.display())),
            }
        });
    }
}
