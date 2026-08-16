//! Tests d'integració del seguiment de fitxers (User Stories 2-4), aïllats
//! de la GUI (plan.md, Testing).

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use realttylog::tailer::{FollowState, FollowedFile, OpenAt};

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

/// Fa `poll()` repetidament fins que `condition` sigui certa o passin 5 s
/// (marge generós per si `notify` triga a arribar en aquest entorn i cal
/// esperar la xarxa de seguretat d'1 s del watcher).
fn wait_until(file: &mut FollowedFile, condition: impl Fn(&FollowedFile) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        file.poll();
        if condition(file) {
            return;
        }
        if Instant::now() >= deadline {
            panic!("condició no assolida abans del termini");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// T025: les línies noves apareixen en seguir un fitxer en creixement
/// (FR-014, SC-003).
#[test]
fn new_lines_appear_while_following_a_growing_file() {
    let path = tempfile_path("tailer-growing.log");
    fs::write(&path, "línia inicial\n").unwrap();

    let mut file = FollowedFile::open(path.clone(), OpenAt::End).unwrap();
    assert_eq!(file.state, FollowState::Live);

    let mut f = fs::OpenOptions::new().append(true).open(&path).unwrap();
    writeln!(f, "línia nova").unwrap();
    drop(f);

    wait_until(&mut file, |f| {
        f.viewport.lines.iter().any(|l| l.content == "línia nova")
    });

    fs::remove_file(&path).unwrap();
}

/// T026: una rotació (truncament i reemplaçament) es detecta i el
/// seguiment continua (FR-020, SC-007).
#[test]
fn rotation_is_detected_and_following_continues() {
    let path = tempfile_path("tailer-rotation.log");
    fs::write(&path, "abans de la rotació\n").unwrap();

    let mut file = FollowedFile::open(path.clone(), OpenAt::End).unwrap();

    // Truncament a mida zero, com fa `logrotate` amb `copytruncate`. Es
    // comprova el truncament abans d'escriure-hi contingut nou perquè el
    // test sigui determinista: si s'escrivissin tots dos seguits sense
    // pausa, un `poll()` podria no arribar mai a observar l'estat buit
    // intermedi (el fitxer sempre tindria una mida igual o més gran que
    // l'última coneguda) — cosa que a la pràctica no passa, perquè qui
    // escriu el log no ho fa al mateix instant que es trunca.
    fs::write(&path, "").unwrap();
    wait_until(&mut file, |f| f.viewport.lines.is_empty());
    assert_eq!(
        file.state,
        FollowState::Live,
        "el seguiment continua sense error"
    );

    let mut f = fs::OpenOptions::new().append(true).open(&path).unwrap();
    writeln!(f, "després del truncament").unwrap();
    drop(f);

    wait_until(&mut file, |f| {
        f.viewport
            .lines
            .iter()
            .any(|l| l.content == "després del truncament")
    });

    // Reemplaçament: un fitxer nou amb el mateix camí, amb un contingut
    // deliberadament més curt que l'anterior perquè `detect_rotation` no
    // necessiti observar cap estat intermedi per detectar-ho.
    fs::remove_file(&path).unwrap();
    fs::write(&path, "fitxer reemplaçat\n").unwrap();

    wait_until(&mut file, |f| {
        f.viewport
            .lines
            .iter()
            .any(|l| l.content == "fitxer reemplaçat")
    });

    fs::remove_file(&path).unwrap();
}

/// T033: pausar deixa de llegir contingut nou (FR-017) sense perdre'l
/// (FR-025), i reprendre salta a la cua actual (FR-018).
#[test]
fn pause_stops_reading_and_resume_catches_up_to_the_tail() {
    let path = tempfile_path("tailer-pause.log");
    fs::write(&path, "línia inicial\n").unwrap();

    let mut file = FollowedFile::open(path.clone(), OpenAt::End).unwrap();
    assert_eq!(file.state, FollowState::Live);

    file.pause();
    assert_eq!(file.state, FollowState::Paused);
    assert!(!file.has_new_content_while_paused);

    let mut f = fs::OpenOptions::new().append(true).open(&path).unwrap();
    writeln!(f, "arribada mentre estava pausat").unwrap();
    drop(f);

    wait_until(&mut file, |f| f.has_new_content_while_paused);
    assert!(
        !file
            .viewport
            .lines
            .iter()
            .any(|l| l.content == "arribada mentre estava pausat"),
        "pausat no hauria de carregar contingut nou a la finestra"
    );

    file.resume_live().unwrap();
    assert_eq!(file.state, FollowState::Live);
    assert!(
        file.viewport
            .lines
            .iter()
            .any(|l| l.content == "arribada mentre estava pausat"),
        "en reprendre, la línia arribada mentre estava pausat ha de ser visible"
    );

    fs::remove_file(&path).unwrap();
}
