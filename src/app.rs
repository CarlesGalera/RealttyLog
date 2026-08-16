use eframe::egui;

use crate::search::SearchViewState;
use crate::tailer::FileViewState;

/// Quina pantalla de nivell superior mostra l'aplicació: la cerca sobre un
/// directori (US1-US2) o un fitxer concret obert per seguir-lo (US2-US4).
#[derive(Debug)]
pub enum ActiveView {
    Search(SearchViewState),
    File(FileViewState),
}

impl Default for ActiveView {
    fn default() -> Self {
        ActiveView::Search(SearchViewState::default())
    }
}

pub struct App {
    pub view: ActiveView,
}

impl Default for App {
    fn default() -> Self {
        Self {
            view: ActiveView::default(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| match &self.view {
            ActiveView::Search(state) => {
                ui.label(format!("{state:?} — vista de cerca, pendent (US1)"));
            }
            ActiveView::File(state) => {
                ui.label(format!("{state:?} — vista de fitxer, pendent (US2/US3)"));
            }
        });
    }
}
