use eframe::egui;

use crate::tailer::FileViewState;
use crate::ui::search_view::SearchViewState;

/// Quina pantalla de nivell superior mostra l'aplicació: la cerca sobre un
/// directori (US1-US2) o un fitxer concret obert per seguir-lo (US2-US4).
pub enum ActiveView {
    Search(SearchViewState),
    File(FileViewState),
}

impl Default for ActiveView {
    fn default() -> Self {
        ActiveView::Search(SearchViewState::default())
    }
}

#[derive(Default)]
pub struct App {
    pub view: ActiveView,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| match &mut self.view {
            ActiveView::Search(state) => state.ui(ui),
            ActiveView::File(_state) => {
                ui.label("Vista de fitxer, pendent (US2/US3)");
            }
        });
    }
}
