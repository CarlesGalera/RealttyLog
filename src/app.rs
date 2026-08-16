use eframe::egui;

use crate::tailer::{FollowedFile, OpenAt};
use crate::ui::log_view::LogViewState;
use crate::ui::search_view::SearchViewState;

/// Estat de l'aplicació: la cerca es manté sempre viva perquè "tornar als
/// resultats" (FR-011) no impliqui repetir-la; `open_file` només és `Some`
/// mentre l'usuari mira un fitxer concret.
#[derive(Default)]
pub struct App {
    pub search: SearchViewState,
    pub open_file: Option<LogViewState>,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(log_view) = &mut self.open_file {
                if log_view.ui(ui) {
                    self.open_file = None;
                }
                return;
            }

            let Some(clicked) = self.search.ui(ui) else {
                return;
            };
            match FollowedFile::open(
                clicked.file_path.clone(),
                OpenAt::Offset(clicked.byte_offset),
            ) {
                Ok(file) => self.open_file = Some(LogViewState::new(file)),
                Err(err) => self.search.set_open_error(format!(
                    "No s'ha pogut obrir {}: {err}",
                    clicked.file_path.display()
                )),
            }
        });
    }
}
