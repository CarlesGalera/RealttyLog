use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

use crate::encoding::decode_lossy;

use super::Line;

const READ_CHUNK: usize = 64 * 1024;

/// Cerca cap enrere des de `from` fins trobar el començament d'una línia
/// que sigui com a mínim `n` línies abans de `from` (o l'inici del fitxer,
/// si n'hi ha menys), llegint per blocs en lloc d'escanejar tot el fitxer.
/// Compartit entre "posicionar-se al final" (research.md, decisió 7) i
/// "mostrar context al voltant d'un salt" (FR-010).
pub fn find_offset_n_lines_before(file: &mut File, from: u64, n: usize) -> io::Result<u64> {
    if n == 0 || from == 0 {
        return Ok(from);
    }
    // Si `from` ja és l'inici d'una línia, el primer salt de línia trobat
    // escanejant cap enrere és el que tanca la línia immediatament anterior
    // — és a dir, tornar-hi ens deixaria altre cop a `from`. Cal el
    // (n+1)-èsim salt per quedar `n` línies completes per darrere.
    let target = n + 1;
    let mut pos = from;
    let mut newlines_found = 0usize;
    let mut buf = vec![0u8; READ_CHUNK];
    while pos > 0 && newlines_found < target {
        let chunk_len = READ_CHUNK.min(pos as usize);
        pos -= chunk_len as u64;
        file.seek(SeekFrom::Start(pos))?;
        file.read_exact(&mut buf[..chunk_len])?;
        for i in (0..chunk_len).rev() {
            if buf[i] == b'\n' {
                newlines_found += 1;
                if newlines_found == target {
                    return Ok(pos + i as u64 + 1);
                }
            }
        }
    }
    Ok(pos)
}

/// Llegeix cap endavant des de `start_offset`, com a màxim `max_lines`
/// línies (o fins al final del fitxer), assignant `sequence` de manera
/// consecutiva a partir de `first_sequence`. Retorna les línies i el nou
/// offset de lectura (FR-015, FR-022: decodificació UTF-8 amb pèrdua).
pub fn read_lines_forward(
    file: &mut File,
    start_offset: u64,
    first_sequence: u64,
    max_lines: usize,
) -> io::Result<(Vec<Line>, u64)> {
    file.seek(SeekFrom::Start(start_offset))?;
    let mut lines = Vec::new();
    let mut current = Vec::new();
    let mut current_start = start_offset;
    let mut buf = [0u8; READ_CHUNK];
    let mut offset = start_offset;
    'outer: loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            if !current.is_empty() {
                lines.push(Line {
                    content: decode_lossy(&current),
                    byte_offset: current_start,
                    sequence: first_sequence + lines.len() as u64,
                });
            }
            break;
        }
        for &b in &buf[..n] {
            offset += 1;
            if b == b'\n' {
                lines.push(Line {
                    content: decode_lossy(&current),
                    byte_offset: current_start,
                    sequence: first_sequence + lines.len() as u64,
                });
                current.clear();
                current_start = offset;
                if lines.len() >= max_lines {
                    break 'outer;
                }
            } else {
                current.push(b);
            }
        }
    }
    Ok((lines, offset))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(content: &str) -> (std::path::PathBuf, File) {
        let path = std::env::temp_dir().join(format!(
            "realttylog-reader-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut f = File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        (path.clone(), File::open(&path).unwrap())
    }

    #[test]
    fn reads_lines_forward_with_offsets_and_sequence() {
        let (path, mut file) = write_temp("a\nbb\nccc\n");
        let (lines, offset) = read_lines_forward(&mut file, 0, 0, 10).unwrap();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].content, "a");
        assert_eq!(lines[0].byte_offset, 0);
        assert_eq!(lines[0].sequence, 0);
        assert_eq!(lines[1].content, "bb");
        assert_eq!(lines[1].byte_offset, 2);
        assert_eq!(lines[2].content, "ccc");
        assert_eq!(offset, 9);

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn finds_offset_a_fixed_number_of_lines_before() {
        let (path, mut file) = write_temp("l0\nl1\nl2\nl3\nl4\n");
        // Cada línia "lN\n" fa 3 bytes, així que l3 comença al byte 9.
        let offset = find_offset_n_lines_before(&mut file, 9, 2).unwrap();
        assert_eq!(offset, 3); // 2 línies abans de l3 -> l1, que comença al byte 3

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn stops_at_start_of_file_when_fewer_lines_exist() {
        let (path, mut file) = write_temp("l0\nl1\n");
        let offset = find_offset_n_lines_before(&mut file, 6, 100).unwrap();
        assert_eq!(offset, 0);

        std::fs::remove_file(&path).unwrap();
    }
}
