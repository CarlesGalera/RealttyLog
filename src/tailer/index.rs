use std::collections::BTreeMap;

/// Interval de línies entre cada checkpoint dispers (research.md, decisió
/// 6): prou petit per saltar-hi ràpid, prou gran perquè l'índex no pesi en
/// un fitxer de desenes de milions de línies.
const CHECKPOINT_INTERVAL: u64 = 1000;

/// Índex dispers `sequence -> byte_offset`, construït incrementalment a
/// mesura que es llegeix el fitxer. Correspon a l'entitat `LineIndex` de
/// data-model.md: permet tornar a un punt qualsevol de l'historial sense
/// escanejar-lo sencer cada vegada (FR-025).
#[derive(Debug, Default)]
pub struct LineIndex {
    checkpoints: BTreeMap<u64, u64>,
}

impl LineIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra un checkpoint si `sequence` cau en un múltiple de
    /// `CHECKPOINT_INTERVAL`. Es crida per cada línia llegida, tant en
    /// directe (US3) com en respondre a un salt (US2).
    pub fn record(&mut self, sequence: u64, byte_offset: u64) {
        if sequence.is_multiple_of(CHECKPOINT_INTERVAL) {
            self.checkpoints.insert(sequence, byte_offset);
        }
    }

    /// El checkpoint conegut més proper, igual o per sota de `sequence`.
    /// Retorna `(0, 0)` si encara no n'hi ha cap prou avançat.
    pub fn nearest_checkpoint_at_or_before(&self, sequence: u64) -> (u64, u64) {
        self.checkpoints
            .range(..=sequence)
            .next_back()
            .map(|(&s, &o)| (s, o))
            .unwrap_or((0, 0))
    }

    /// En rotar el fitxer (FR-020), l'índex es reinicia junt amb
    /// `read_offset` de `FollowedFile`.
    pub fn reset(&mut self) {
        self.checkpoints.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_nearest_checkpoint_at_or_before() {
        let mut idx = LineIndex::new();
        idx.record(0, 0);
        idx.record(1000, 54_321);
        idx.record(2000, 108_642);

        assert_eq!(idx.nearest_checkpoint_at_or_before(1500), (1000, 54_321));
        assert_eq!(idx.nearest_checkpoint_at_or_before(999), (0, 0));
        assert_eq!(idx.nearest_checkpoint_at_or_before(2000), (2000, 108_642));
    }

    #[test]
    fn ignores_non_checkpoint_sequences() {
        let mut idx = LineIndex::new();
        idx.record(500, 12_345); // no és múltiple de CHECKPOINT_INTERVAL

        assert_eq!(idx.nearest_checkpoint_at_or_before(500), (0, 0));
    }
}
