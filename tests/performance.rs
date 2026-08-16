//! Proves de rendiment a escala real (User Story 5). Generen fitxers de
//! diversos GB, així que es marquen `#[ignore]` — no s'executen en cada
//! `cargo test`, només a mà amb `cargo test --test performance -- --ignored
//! --test-threads=1` (T037, T038; SC-002, SC-004).

use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

use realttylog::search::engine::{search, SearchEvent};
use realttylog::tailer::{FollowedFile, OpenAt};

fn tempfile_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "realttylog-perf-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Omple `path` amb `target_bytes` de línies repetitives, escrivint en blocs
/// d'1 MB (no línia a línia) perquè generar diversos GB de dades de prova
/// no es converteixi en el coll d'ampolla del test.
fn generate_large_file(path: &Path, target_bytes: u64, needle: Option<&str>) {
    let line = "línia de prova per omplir espai amb contingut repetitiu i realista\n";
    let mut chunk = String::with_capacity(1 << 20);
    while chunk.len() < (1 << 20) {
        chunk.push_str(line);
    }
    let chunk = chunk.into_bytes();

    let mut writer = BufWriter::with_capacity(1 << 20, fs::File::create(path).unwrap());
    let mut written = 0u64;
    while written < target_bytes {
        writer.write_all(&chunk).unwrap();
        written += chunk.len() as u64;
    }
    if let Some(needle) = needle {
        writer.write_all(needle.as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
    }
    writer.flush().unwrap();
}

/// T038 (SC-004): obrir directament un fitxer de 5 GB es posiciona al
/// final en menys de 2 segons.
#[test]
#[ignore = "genera un fitxer de 5 GB; es executa a mà"]
fn opening_a_5gb_file_positions_at_end_under_2_seconds() {
    let dir = tempfile_dir("open-end");
    let path = dir.join("gran.log");
    generate_large_file(&path, 5 * 1024 * 1024 * 1024, None);

    let start = Instant::now();
    let file = FollowedFile::open(path.clone(), OpenAt::End).unwrap();
    let elapsed = start.elapsed();

    assert!(
        !file.viewport.lines.is_empty(),
        "la finestra hauria de tenir contingut carregat"
    );
    assert!(
        elapsed < Duration::from_secs(2),
        "obrir al final ha trigat {elapsed:?}, per sobre del límit de 2 s (SC-004)"
    );

    fs::remove_dir_all(&dir).unwrap();
}

/// T037 (SC-002): cercar un text en un conjunt de fitxers de fins a 20 GB
/// en total mostra el primer resultat en menys de 3 segons.
#[test]
#[ignore = "genera ~20 GB de fitxers de prova; es executa a mà"]
fn search_over_20gb_shows_first_result_under_3_seconds() {
    let dir = tempfile_dir("search-20gb");
    let per_file = 5 * 1024 * 1024 * 1024u64; // 4 fitxers * 5 GB = 20 GB
    let files: Vec<PathBuf> = (0..4).map(|i| dir.join(format!("f{i}.log"))).collect();

    for (i, path) in files.iter().enumerate() {
        let needle = if i == files.len() - 1 {
            Some("ERROR agulla al paller de 20 gigabytes")
        } else {
            None
        };
        generate_large_file(path, per_file, needle);
    }

    let start = Instant::now();
    let rx = search(
        "agulla al paller de 20 gigabytes",
        files,
        Arc::new(AtomicBool::new(false)),
    );
    let first_match = rx.recv_timeout(Duration::from_secs(30));
    let elapsed = start.elapsed();

    assert!(
        matches!(first_match, Ok(SearchEvent::Match(_))),
        "s'esperava una coincidència abans del termini de 30 s"
    );
    assert!(
        elapsed < Duration::from_secs(3),
        "el primer resultat ha trigat {elapsed:?}, per sobre del límit de 3 s (SC-002)"
    );

    fs::remove_dir_all(&dir).unwrap();
}
