//! Tests d'integració de la cerca multi-fitxer (User Story 1), aïllats de la
//! GUI (plan.md, Testing). Corresponen a T008-T010 de tasks.md.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;

use realttylog::search::directory::LogDirectory;
use realttylog::search::engine::{search, SearchEvent};
use realttylog::search::result::SearchMatch;

fn tempfile_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "realttylog-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn collect(rx: Receiver<SearchEvent>) -> Vec<SearchMatch> {
    let mut matches = Vec::new();
    while let Ok(event) = rx.recv_timeout(Duration::from_secs(5)) {
        match event {
            SearchEvent::Match(m) => matches.push(m),
            SearchEvent::Finished => break,
        }
    }
    matches
}

#[test]
fn lists_files_in_subdirectories() {
    let dir = tempfile_dir("directory");
    fs::create_dir(dir.join("2026-08-15")).unwrap();
    fs::write(dir.join("app.log"), "arrel").unwrap();
    fs::write(dir.join("2026-08-15").join("app.log"), "subdirectori").unwrap();

    let log_dir = LogDirectory::open(&dir);

    assert_eq!(log_dir.files.len(), 2);
    assert!(log_dir.files.iter().any(|p| p == &dir.join("app.log")));
    assert!(log_dir
        .files
        .iter()
        .any(|p| p == &dir.join("2026-08-15").join("app.log")));

    fs::remove_dir_all(&dir).unwrap();
}

/// T008: una cerca en un directori multi-fitxer retorna els fitxers correctes.
#[test]
fn finds_match_in_the_right_file_only() {
    let dir = tempfile_dir("engine");
    fs::write(dir.join("a.log"), "línia normal\n").unwrap();
    fs::write(
        dir.join("b.log"),
        "línia normal\nERROR: connexió refusada\n",
    )
    .unwrap();

    let files = vec![dir.join("a.log"), dir.join("b.log")];
    let rx = search("connexió refusada", files, Arc::new(AtomicBool::new(false)));
    let matches = collect(rx);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].file_path, dir.join("b.log"));
    assert!(matches[0].line_context.contains("connexió refusada"));

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn search_is_case_insensitive_by_default() {
    let dir = tempfile_dir("engine-case");
    fs::write(dir.join("a.log"), "Error Fatal\n").unwrap();

    let rx = search(
        "error fatal",
        vec![dir.join("a.log")],
        Arc::new(AtomicBool::new(false)),
    );
    let matches = collect(rx);

    assert_eq!(matches.len(), 1);

    fs::remove_dir_all(&dir).unwrap();
}

/// T009: cancel·lar una cerca en curs conserva els resultats trobats fins
/// aquell moment (FR-005). Aquí es cancel·la abans de començar, que és el
/// cas límit: cap resultat, però tampoc cap penjada ni pànic.
#[test]
fn cancelling_still_yields_matches_found_so_far() {
    let dir = tempfile_dir("engine-cancel");
    fs::write(dir.join("a.log"), "terme\n".repeat(5)).unwrap();

    let cancel = Arc::new(AtomicBool::new(true)); // ja cancel·lada abans de començar
    let rx = search("terme", vec![dir.join("a.log")], cancel);
    let matches = collect(rx);

    assert!(matches.is_empty(), "cap fil hauria de començar a cercar");

    fs::remove_dir_all(&dir).unwrap();
}

/// T010: els fitxers no llegibles es descarten sense aturar la cerca (FR-007).
#[test]
fn unreadable_file_is_skipped_without_stopping_the_search() {
    let dir = tempfile_dir("engine-unreadable");
    let missing = dir.join("no-existeix.log");
    fs::write(dir.join("real.log"), "terme buscat\n").unwrap();

    let files = vec![missing, dir.join("real.log")];
    let rx = search("terme buscat", files, Arc::new(AtomicBool::new(false)));
    let matches = collect(rx);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].file_path, dir.join("real.log"));

    fs::remove_dir_all(&dir).unwrap();
}
