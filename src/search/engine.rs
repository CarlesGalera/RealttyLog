use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use grep_regex::{RegexMatcher, RegexMatcherBuilder};
use grep_searcher::{BinaryDetection, Searcher, SearcherBuilder, Sink, SinkMatch};

use super::result::SearchMatch;
use crate::encoding::decode_lossy;

/// Nombre màxim de resultats que es mostren per cerca (FR-008; research.md,
/// decisió 3). Evita que un terme molt comú aclapari la interfície o la
/// memòria; l'usuari pot refinar la cerca si en necessita més.
pub const MAX_RESULTS: usize = 500;

pub enum SearchEvent {
    Match(SearchMatch),
    /// Emès quan tots els fitxers s'han cercat (o s'ha cancel·lat, o s'ha
    /// arribat al límit de resultats). Marca la transició Running → Completed
    /// de `SearchQuery` (data-model.md).
    Finished,
}

/// Llança una cerca de `text` a tots els `files` en un pool de fils
/// (research.md, decisió 2), retornant un canal pel qual arriben els
/// resultats a mesura que es troben. `cancel` es pot activar des de fora
/// per aturar la cerca (FR-005) sense perdre els resultats ja enviats.
pub fn search(text: &str, files: Vec<PathBuf>, cancel: Arc<AtomicBool>) -> Receiver<SearchEvent> {
    let (tx, rx) = mpsc::channel();

    let matcher = match build_literal_matcher(text) {
        Ok(matcher) => matcher,
        Err(_) => {
            let _ = tx.send(SearchEvent::Finished);
            return rx;
        }
    };

    let thread_count = thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1);
    let files = Arc::new(Mutex::new(files.into_iter()));
    let found = Arc::new(AtomicUsize::new(0));

    let worker_handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let files = Arc::clone(&files);
            let matcher = matcher.clone();
            let tx = tx.clone();
            let cancel = Arc::clone(&cancel);
            let found = Arc::clone(&found);
            thread::spawn(move || {
                while !cancel.load(Ordering::Relaxed) && found.load(Ordering::Relaxed) < MAX_RESULTS
                {
                    let next_file = files.lock().unwrap().next();
                    let Some(path) = next_file else {
                        break;
                    };
                    search_file(&matcher, &path, &tx, &cancel, &found);
                }
            })
        })
        .collect();

    // Un fil supervisor s'encarrega d'esperar tots els treballadors i, un
    // cop acabats, notificar la fi de la cerca — mantenint el canal `rx`
    // immediatament disponible per al fil de la GUI mentre tant.
    thread::spawn(move || {
        for handle in worker_handles {
            let _ = handle.join();
        }
        let _ = tx.send(SearchEvent::Finished);
    });

    rx
}

/// Cerca de text literal (Assumptions de l'spec: sense expressions regulars,
/// sense distingir majúscules/minúscules per defecte), construïda escapant
/// el text de l'usuari abans de passar-lo al motor de `grep-regex`.
fn build_literal_matcher(text: &str) -> Result<RegexMatcher, grep_regex::Error> {
    RegexMatcherBuilder::new()
        .case_insensitive(true)
        .build(&regex::escape(text))
}

fn search_file(
    matcher: &RegexMatcher,
    path: &Path,
    tx: &Sender<SearchEvent>,
    cancel: &AtomicBool,
    found: &AtomicUsize,
) {
    // FR-007: un fitxer que no es pot obrir es descarta sense aturar la resta.
    let Ok(file) = std::fs::File::open(path) else {
        return;
    };
    let mut searcher: Searcher = SearcherBuilder::new()
        // Igual que ripgrep: si un fitxer conté un byte nul, es tracta com a
        // binari i es descarta (FR-007) en lloc d'escanejar-lo sencer.
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build();
    let mut sink = MatchCollector {
        path,
        tx,
        cancel,
        found,
    };
    let _ = searcher.search_file(matcher, &file, &mut sink);
}

struct MatchCollector<'a> {
    path: &'a Path,
    tx: &'a Sender<SearchEvent>,
    cancel: &'a AtomicBool,
    found: &'a AtomicUsize,
}

impl Sink for MatchCollector<'_> {
    type Error = std::io::Error;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> Result<bool, Self::Error> {
        let line_context = decode_lossy(mat.bytes()).trim_end().to_string();
        let result = SearchMatch {
            file_path: self.path.to_path_buf(),
            byte_offset: mat.absolute_byte_offset(),
            line_context,
        };
        let _ = self.tx.send(SearchEvent::Match(result));
        let now_found = self.found.fetch_add(1, Ordering::Relaxed) + 1;

        // `false` atura la cerca en aquest fitxer (i, indirectament, la
        // resta en aturar-se cada fil abans del pròxim): cancel·lació o
        // límit de resultats assolit.
        Ok(!self.cancel.load(Ordering::Relaxed) && now_found < MAX_RESULTS)
    }
}
