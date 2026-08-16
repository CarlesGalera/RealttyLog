use std::path::PathBuf;

use ignore::WalkBuilder;

/// El directori de logs obert i els fitxers que conté, inclosos els de
/// subdirectoris (FR-001). Correspon a l'entitat "Directori de logs" de
/// data-model.md: és una foto fixa del moment en què s'obre.
#[derive(Debug, Clone)]
pub struct LogDirectory {
    pub root: PathBuf,
    pub files: Vec<PathBuf>,
}

impl LogDirectory {
    /// Obre `root` i en llista els fitxers, recorrent subdirectoris.
    ///
    /// A diferència de `ripgrep`, aquí no volem respectar `.gitignore` ni
    /// ocultar fitxers ocults: un directori de logs no és un repositori de
    /// codi, i qualsevol fitxer que hi hagi és rellevant. Les entrades que no
    /// es poden llegir (permisos, enllaços trencats) es descarten en
    /// silenci (FR-007) en lloc d'aturar tot el llistat.
    pub fn open(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let files = WalkBuilder::new(&root)
            .hidden(false)
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false)
            .ignore(false)
            .parents(false)
            .build()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_some_and(|ft| ft.is_file()))
            .map(|entry| entry.into_path())
            .collect();
        Self { root, files }
    }
}
