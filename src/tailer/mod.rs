//! Seguiment d'un fitxer concret, en directe o a partir d'un salt (User
//! Stories 2-4).

pub mod index;
pub mod reader;
pub mod viewport;

use std::fs::File;
use std::io;
use std::path::PathBuf;

pub use index::LineIndex;
pub use viewport::ViewportCache;

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
}

impl FollowedFile {
    /// Obre `path` i el posiciona segons `at` (FR-009, FR-012, FR-013),
    /// carregant una finestra inicial de línies amb context al voltant.
    pub fn open(path: PathBuf, at: OpenAt) -> io::Result<Self> {
        let mut file = File::open(&path)?;
        let last_known_len = file.metadata()?.len();

        let (target_offset, state) = match at {
            OpenAt::End => (last_known_len, FollowState::Live),
            OpenAt::Offset(offset) => (offset, FollowState::Paused),
        };
        let window_offset =
            reader::find_offset_n_lines_before(&mut file, target_offset, CONTEXT_LINES)?;

        let (lines, read_offset) =
            reader::read_lines_forward(&mut file, window_offset, 0, viewport::CAPACITY)?;

        let mut index = LineIndex::new();
        for line in &lines {
            index.record(line.sequence, line.byte_offset);
        }

        let mut viewport = ViewportCache::new();
        viewport.set_lines(0, lines);

        Ok(Self {
            path,
            state,
            read_offset,
            last_known_len,
            index,
            viewport,
        })
    }
}
