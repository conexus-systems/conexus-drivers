//! Garante a regra do ADR-014: nenhum blob de fabricante entra neste repositório.

use std::fs;
use std::path::Path;

const FORBIDDEN: &[&str] = &["ko", "so", "bin", "img", "mbn", "elf", "apk", "zip", "tar", "gz"];

fn walk(dir: &Path, bad: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if name == "target" || name == ".git" {
            continue;
        }
        if path.is_dir() {
            walk(&path, bad);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if FORBIDDEN.contains(&ext.to_ascii_lowercase().as_str()) {
                bad.push(path.display().to_string());
            }
        }
    }
}

#[test]
fn repositorio_nao_contem_blobs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut bad = Vec::new();
    walk(&root, &mut bad);
    assert!(bad.is_empty(), "arquivos binários de fabricante no repositório: {bad:?}");
}
