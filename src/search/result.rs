use std::path::PathBuf;

/// Una coincidència d'una cerca (FR-004): on és i quin fragment de context
/// té al voltant, sense necessitat d'obrir el fitxer per reconèixer-la.
/// Correspon a l'entitat "Coincidència" de data-model.md.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub file_path: PathBuf,
    pub byte_offset: u64,
    pub line_context: String,
}
