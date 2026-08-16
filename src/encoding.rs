//! Decodificació de bytes de fitxer compartida entre `search` i `tailer`
//! (research.md, decisió 9: UTF-8 amb pèrdua, sense dependències addicionals).

/// Decodifica un tros de bytes com a UTF-8, substituint les seqüències
/// invàlides pel caràcter de reemplaçament (U+FFFD) en lloc d'aturar-se.
pub fn decode_lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_utf8_is_preserved() {
        assert_eq!(decode_lossy("línia de prova".as_bytes()), "línia de prova");
    }

    #[test]
    fn invalid_bytes_become_replacement_char() {
        let invalid = [b'a', 0xFF, b'b'];
        assert_eq!(decode_lossy(&invalid), "a\u{FFFD}b");
    }
}
