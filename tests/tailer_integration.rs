//! Tests d'integració del seguiment de fitxers (User Stories 2-4), aïllats
//! de la GUI (plan.md, Testing).

use std::fs;
use std::path::PathBuf;

use realttylog::tailer::{FollowedFile, OpenAt};

fn tempfile_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "realttylog-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

/// T017: obrir un fitxer en un offset concret mostra la línia i el context
/// correctes (FR-009, FR-010).
#[test]
fn open_at_offset_shows_line_and_context() {
    let path = tempfile_path("tailer-offset.log");
    let mut content = String::new();
    for i in 0..20 {
        content.push_str(&format!("línia {i}\n"));
    }
    fs::write(&path, &content).unwrap();

    let target_line = "línia 10\n";
    let offset = content.find(target_line).unwrap() as u64;

    let file = FollowedFile::open(path.clone(), OpenAt::Offset(offset)).unwrap();

    let contents: Vec<&str> = file
        .viewport
        .lines
        .iter()
        .map(|l| l.content.as_str())
        .collect();
    assert!(
        contents.contains(&"línia 10"),
        "línia trobada: {contents:?}"
    );
    assert!(
        contents.contains(&"línia 9"),
        "context anterior: {contents:?}"
    );
    assert!(
        contents.contains(&"línia 11"),
        "context posterior: {contents:?}"
    );

    fs::remove_file(&path).unwrap();
}
