use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::Arc;

use eframe::egui;

use crate::search::directory::LogDirectory;
use crate::search::engine::{self, SearchEvent};
use crate::search::result::SearchMatch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchStatus {
    Idle,
    Running,
    Completed,
    Cancelled,
}

/// Estat de la vista de cerca: el directori obert (FR-001) i la cerca en
/// curs o acabada (FR-002–FR-008).
pub struct SearchViewState {
    directory_path_input: String,
    directory: Option<LogDirectory>,
    directory_error: Option<String>,
    open_error: Option<String>,
    query_text: String,
    status: SearchStatus,
    matches: Vec<SearchMatch>,
    receiver: Option<Receiver<SearchEvent>>,
    cancel: Option<Arc<AtomicBool>>,
}

impl Default for SearchViewState {
    fn default() -> Self {
        Self {
            directory_path_input: String::new(),
            directory: None,
            directory_error: None,
            open_error: None,
            query_text: String::new(),
            status: SearchStatus::Idle,
            matches: Vec::new(),
            receiver: None,
            cancel: None,
        }
    }
}

impl SearchViewState {
    /// Retorna el resultat que l'usuari ha clicat aquest frame, si n'hi ha
    /// (FR-009): qui crida decideix què fer-ne (obrir-lo com a `FollowedFile`).
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<SearchMatch> {
        self.drain_events(ui.ctx());

        ui.horizontal(|ui| {
            ui.label("Directori:");
            ui.text_edit_singleline(&mut self.directory_path_input);
            if ui.button("Obre").clicked() {
                self.open_directory();
            }
        });

        if let Some(error) = &self.directory_error {
            ui.colored_label(egui::Color32::RED, error);
        }
        if let Some(error) = &self.open_error {
            ui.colored_label(egui::Color32::RED, error);
        }

        let Some(directory) = &self.directory else {
            return None;
        };

        ui.label(format!(
            "{} fitxers detectats a {}",
            directory.files.len(),
            directory.root.display()
        ));

        ui.horizontal(|ui| {
            ui.label("Cerca:");
            let response = ui.text_edit_singleline(&mut self.query_text);
            let search_triggered = ui.button("Cerca").clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
            if search_triggered && !self.query_text.trim().is_empty() {
                self.start_search();
            }
            if self.status == SearchStatus::Running && ui.button("Cancel·la").clicked() {
                self.cancel_search();
            }
        });

        self.status_line(ui);

        let mut clicked = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for m in &self.matches {
                let text = format!("{}: {}", m.file_path.display(), m.line_context);
                if ui.selectable_label(false, text).clicked() {
                    clicked = Some(m.clone());
                }
            }
        });
        clicked
    }

    /// Es crida des de fora quan obrir el fitxer d'un resultat ha fallat
    /// (per exemple, s'ha esborrat entre la cerca i el clic).
    pub fn set_open_error(&mut self, message: String) {
        self.open_error = Some(message);
    }

    fn status_line(&self, ui: &mut egui::Ui) {
        match self.status {
            SearchStatus::Idle => {}
            SearchStatus::Running => {
                ui.label(format!(
                    "Cercant... {} resultats fins ara",
                    self.matches.len()
                ));
            }
            SearchStatus::Cancelled => {
                ui.label(format!(
                    "Cerca cancel·lada. {} resultats trobats.",
                    self.matches.len()
                ));
            }
            SearchStatus::Completed if self.matches.is_empty() => {
                ui.label("Cap coincidència.");
            }
            SearchStatus::Completed => {
                ui.label(format!("{} resultats.", self.matches.len()));
                if self.matches.len() >= engine::MAX_RESULTS {
                    ui.label(
                        "S'ha arribat al límit de resultats mostrats; refina la cerca per acotar-los.",
                    );
                }
            }
        }
    }

    fn open_directory(&mut self) {
        let path = std::path::PathBuf::from(self.directory_path_input.trim());
        if !path.is_dir() {
            self.directory_error = Some(format!("«{}» no és un directori", path.display()));
            self.directory = None;
            return;
        }
        self.directory_error = None;
        self.directory = Some(LogDirectory::open(path));
        self.matches.clear();
        self.status = SearchStatus::Idle;
    }

    fn start_search(&mut self) {
        let Some(directory) = &self.directory else {
            return;
        };
        let files = directory.files.clone();
        let query = self.query_text.trim().to_string();

        self.cancel_search();
        self.matches.clear();

        let cancel = Arc::new(AtomicBool::new(false));
        let rx = engine::search(&query, files, Arc::clone(&cancel));
        self.receiver = Some(rx);
        self.cancel = Some(cancel);
        self.status = SearchStatus::Running;
    }

    fn cancel_search(&mut self) {
        if let Some(cancel) = &self.cancel {
            cancel.store(true, Ordering::Relaxed);
        }
    }

    fn drain_events(&mut self, ctx: &egui::Context) {
        if self.receiver.is_some() {
            self.pump_receiver();
        }
        if self.status == SearchStatus::Running {
            ctx.request_repaint();
        }
    }

    fn pump_receiver(&mut self) {
        // Es pren possessió temporal del receptor perquè el bucle pugui
        // mutar la resta de `self` (resultats, estat) sense conflictes de
        // préstec; es torna a deixar a `self.receiver` si la cerca continua.
        let Some(rx) = self.receiver.take() else {
            return;
        };
        let mut finished = false;
        loop {
            match rx.try_recv() {
                Ok(SearchEvent::Match(m)) => self.matches.push(m),
                Ok(SearchEvent::Finished) | Err(TryRecvError::Disconnected) => {
                    finished = true;
                    break;
                }
                Err(TryRecvError::Empty) => break,
            }
        }
        if finished {
            let was_cancelled = self
                .cancel
                .as_ref()
                .is_some_and(|c| c.load(Ordering::Relaxed));
            self.status = if was_cancelled {
                SearchStatus::Cancelled
            } else {
                SearchStatus::Completed
            };
            self.cancel = None;
        } else {
            self.receiver = Some(rx);
        }
    }
}
