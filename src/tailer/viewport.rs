use std::collections::VecDeque;

use super::Line;

/// Nombre de línies que es mantenen carregades a la finestra activa
/// (research.md, decisió 6): prou perquè desplaçar-se una mica no obligui a
/// rellegir el fitxer, sense dependre de la mida total del fitxer.
pub const CAPACITY: usize = 2000;

/// La porció de línies carregades en memòria per a un `FollowedFile`. Mai
/// és "tot l'historial": només el que es mostra més un marge — la resta es
/// torna a llegir del fitxer via `LineIndex` quan cal (FR-025).
#[derive(Debug, Default)]
pub struct ViewportCache {
    pub lines: VecDeque<Line>,
    pub window_start: u64,
}

impl ViewportCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_lines(&mut self, window_start: u64, lines: Vec<Line>) {
        self.window_start = window_start;
        self.lines = lines.into();
    }

    /// Afegeix una línia arribada en directe (US3), descartant la més
    /// antiga de la finestra si se supera `CAPACITY` — sense que això
    /// impliqui perdre-la de debò: continua al fitxer, recuperable via
    /// `LineIndex` (FR-025).
    pub fn push_live_line(&mut self, line: Line) {
        self.lines.push_back(line);
        while self.lines.len() > CAPACITY {
            self.lines.pop_front();
            self.window_start += 1;
        }
    }
}
