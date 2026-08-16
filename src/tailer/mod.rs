//! Seguiment d'un fitxer concret, en directe o a partir d'un salt (User
//! Stories 2-4).

pub mod index;
pub mod reader;
pub mod rotation;
pub mod viewport;
pub mod watcher;

use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

pub use index::LineIndex;
pub use viewport::ViewportCache;
use watcher::FollowSignal;

/// Nombre de línies de context carregades a banda i banda d'un punt
/// d'obertura (FR-010): la meitat de la finestra activa a cada costat.
const CONTEXT_LINES: usize = viewport::CAPACITY / 2;

/// Una línia de contingut: el seu text, la posició al fitxer i l'ordre
/// d'arribada dins la sessió de seguiment (FR-015). Correspon a l'entitat
/// "Línia" de data-model.md.
#[derive(Debug, Clone)]
pub struct Line {
    pub content: String,
    pub byte_offset: u64,
    pub sequence: u64,
}

/// Estat de seguiment d'un `FollowedFile` (FR-019, FR-021).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowState {
    Live,
    Paused,
    Unavailable,
}

/// D'on parteix la posició inicial en obrir un fitxer.
pub enum OpenAt {
    /// FR-013: obertura directa, com `tail -f`.
    End,
    /// FR-009: salt des d'un resultat de cerca, a l'offset de la línia
    /// coincident.
    Offset(u64),
}

/// El fitxer que RealttyLog té obert en un moment donat, arribi d'un
/// resultat de cerca o s'obri directament. Correspon a l'entitat "Fitxer
/// seguit" de data-model.md.
pub struct FollowedFile {
    pub path: PathBuf,
    pub state: FollowState,
    pub read_offset: u64,
    pub last_known_len: u64,
    pub index: LineIndex,
    pub viewport: ViewportCache,
    /// FR-019: alguna cosa nova ha arribat mentre l'usuari repassava
    /// l'historial. No és un recompte exacte, només un avís.
    pub has_new_content_while_paused: bool,
    /// A quin estat tornar en recuperar l'accés (FR-021): si l'usuari havia
    /// pausat abans que el fitxer desaparegués, hi torna en lloc de saltar
    /// a directe sense que ningú ho hagi demanat.
    state_before_unavailable: Option<FollowState>,
    watch_rx: Receiver<FollowSignal>,
}

impl FollowedFile {
    /// Obre `path` i el posiciona segons `at` (FR-009, FR-012, FR-013),
    /// carregant una finestra inicial de línies amb context al voltant, i
    /// engega el seu seguiment en directe.
    pub fn open(path: PathBuf, at: OpenAt) -> io::Result<Self> {
        let mut file = File::open(&path)?;
        let last_known_len = file.metadata()?.len();

        // En obrir directament al final no hi ha "després" que mostrar, així
        // que s'omple tota la finestra cap enrere; en saltar a un resultat de
        // cerca, la meitat és context anterior i l'altra meitat arriba en
        // llegir cap endavant tot seguit.
        let (target_offset, lookback_lines, state) = match at {
            OpenAt::End => (last_known_len, viewport::CAPACITY, FollowState::Live),
            OpenAt::Offset(offset) => (offset, CONTEXT_LINES, FollowState::Paused),
        };
        let window_offset =
            reader::find_offset_n_lines_before(&mut file, target_offset, lookback_lines)?;

        let (lines, read_offset) =
            reader::read_lines_forward(&mut file, window_offset, 0, viewport::CAPACITY)?;

        let mut index = LineIndex::new();
        for line in &lines {
            index.record(line.sequence, line.byte_offset);
        }

        let mut viewport = ViewportCache::new();
        viewport.set_lines(0, lines);

        Ok(Self {
            path: path.clone(),
            state,
            read_offset,
            last_known_len,
            index,
            viewport,
            has_new_content_while_paused: false,
            state_before_unavailable: None,
            watch_rx: watcher::watch(path),
        })
    }

    /// Pausa l'autoscroll (FR-017): el fil de seguiment continua vigilant
    /// el fitxer, però `poll()` deixa de llegir-ne contingut nou fins que
    /// es reprengui, perquè no es toqui la finestra que l'usuari mira.
    pub fn pause(&mut self) {
        self.state = FollowState::Paused;
        self.has_new_content_while_paused = false;
    }

    /// Reprèn el seguiment en directe (FR-018): salta a la cua actual del
    /// fitxer, com una obertura directa fresca, en lloc de reproduir tot el
    /// que hagi arribat mentre estava pausat. L'historial que es deixa
    /// enrere no es perd (FR-025): es pot tornar a saltar-hi cercant-lo de
    /// nou.
    pub fn resume_live(&mut self) -> io::Result<()> {
        *self = Self::open(self.path.clone(), OpenAt::End)?;
        Ok(())
    }

    /// Sequència que correspon a la pròxima línia que s'afegeixi al final de
    /// la finestra: es manté vàlida encara que `push_live_line` en descarti
    /// de velles, perquè `window_start` i `lines.len()` es compensen.
    fn next_sequence(&self) -> u64 {
        self.viewport.window_start + self.viewport.lines.len() as u64
    }

    /// Processa els avisos arribats del fil de seguiment des de l'última
    /// crida. Es crida cada frame des de la GUI (research.md, decisió 8).
    /// La disponibilitat i la mida del fitxer es comproven sempre aquí
    /// mateix, al fil principal: és l'única font de veritat, perquè el fil
    /// de fons només ha de dir "comprova-ho", no arriscar-se a decidir-ho
    /// pel seu compte i desincronitzar-se (una comprovació transitòria pot
    /// veure el fitxer desaparegut i reaparegut abans que el fil de fons
    /// se n'assabenti, si no s'unifiquen en un sol lloc). Retorna `true` si
    /// alguna cosa ha canviat i val la pena repintar.
    pub fn poll(&mut self) -> bool {
        let mut got_signal = false;
        while self.watch_rx.try_recv().is_ok() {
            got_signal = true;
        }
        if !got_signal {
            return false;
        }

        let Ok(metadata) = std::fs::metadata(&self.path) else {
            if self.state != FollowState::Unavailable {
                self.state_before_unavailable = Some(self.state);
                self.state = FollowState::Unavailable;
            }
            return true;
        };
        if self.state == FollowState::Unavailable {
            // FR-021: torna a haver-hi accés; es reprèn l'estat d'abans
            // (en directe o pausat) en lloc d'imposar-ne un.
            self.state = self
                .state_before_unavailable
                .take()
                .unwrap_or(FollowState::Live);
        }

        let current_len = metadata.len();
        if rotation::detect_rotation(current_len, self.last_known_len) {
            // El contingut carregat pertany a l'era anterior a la rotació:
            // ja no és coherent amb el fitxer nou, així que es descarta en
            // lloc de barrejar-lo amb el que hi ha ara (FR-020).
            self.read_offset = 0;
            self.index.reset();
            self.viewport = ViewportCache::new();
        }
        self.last_known_len = current_len;

        if self.state != FollowState::Live {
            // FR-006/FR-017: pausat, l'usuari repassa l'historial. No es
            // llegeix res nou perquè no es toqui la finestra que està
            // mirant; només s'anota que n'hi ha (FR-019). Res es perd
            // (FR-025): read_offset no avança, així que en reprendre es
            // recupera exactament des d'on s'havia deixat.
            self.has_new_content_while_paused |= current_len != self.read_offset;
            return true;
        }

        let Ok(mut file) = File::open(&self.path) else {
            self.state_before_unavailable = Some(self.state);
            self.state = FollowState::Unavailable;
            return true;
        };
        if let Ok((lines, new_offset)) = reader::read_lines_forward(
            &mut file,
            self.read_offset,
            self.next_sequence(),
            viewport::CAPACITY,
        ) {
            for line in lines {
                self.index.record(line.sequence, line.byte_offset);
                self.viewport.push_live_line(line);
            }
            self.read_offset = new_offset;
        }
        true
    }
}
