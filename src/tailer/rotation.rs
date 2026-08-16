/// Detecta si el fitxer seguit s'ha truncat: la mida actual és menor que
/// l'última coneguda, senyal inequívoc que el contingut ja llegit ja no hi
/// és (research.md, decisió 5; FR-020).
///
/// El patró de reemplaçament (fitxer nou amb el mateix nom) es cobreix
/// indirectament: en obrir sempre pel camí, i no per un descriptor retingut,
/// un fitxer nou hi apareix igual; si comença petit (el cas habitual de
/// `logrotate`), aquesta mateixa comprovació ho detecta.
pub fn detect_rotation(current_len: u64, last_known_len: u64) -> bool {
    current_len < last_known_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrinking_length_is_a_rotation() {
        assert!(detect_rotation(0, 5_000));
        assert!(detect_rotation(10, 5_000));
    }

    #[test]
    fn growing_or_stable_length_is_not_a_rotation() {
        assert!(!detect_rotation(5_000, 5_000));
        assert!(!detect_rotation(5_001, 5_000));
    }
}
