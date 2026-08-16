use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

/// Un avís del fil de fons cap al `FollowedFile` que l'ha demanat
/// (research.md, decisió 8). Deliberadament lleuger: només diu "va a
/// comprovar-ho", no porta contingut ni cap judici sobre disponibilitat —
/// `FollowedFile::poll()`, al fil principal, és l'única font de veritat
/// sobre l'estat real del fitxer (una sola comprovació de metadades, no
/// dues d'independents que es puguin desincronitzar). Així un
/// `FollowedFile` pausat tampoc acumula feina ni memòria mentre ningú el
/// mira.
pub struct FollowSignal;

/// Llança un fil que vigila `path` amb `notify` i n'envia un avís pel canal
/// retornat cada vegada que hi ha (o podria haver-hi) un canvi (FR-009,
/// FR-012, FR-020, FR-021).
pub fn watch(path: PathBuf) -> Receiver<FollowSignal> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || run(&path, &tx));
    rx
}

fn run(path: &Path, tx: &Sender<FollowSignal>) {
    let (notify_tx, notify_rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher: Option<RecommendedWatcher> = notify::recommended_watcher(move |res| {
        let _ = notify_tx.send(res);
    })
    .ok();
    if let Some(w) = &mut watcher {
        let _ = w.watch(path, RecursiveMode::NonRecursive);
    }

    loop {
        // Es desperta amb cada esdeveniment de `notify` i, com a xarxa de
        // seguretat, com a màxim cada segon per si el sistema de fitxers no
        // n'emet cap (unitats de xarxa que es desconnecten, o un fitxer
        // esborrat i recreat massa de pressa perquè aquest mateix `watch`
        // el detecti a temps).
        let _ = notify_rx.recv_timeout(Duration::from_secs(1));
        if tx.send(FollowSignal).is_err() {
            return; // el receptor ha plegat: FollowedFile s'ha descartat
        }
    }
}
