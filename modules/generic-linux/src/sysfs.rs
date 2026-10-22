//! Leitura segura de atributos do sysfs.

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Atributos do sysfs são pequenos. Qualquer coisa maior que isto é descartada.
const MAX_ATTR_BYTES: u64 = 4096;

/// Lê um atributo como texto aparado. `None` se não existe, não é legível, não é UTF-8 ou é grande demais.
/// `name` precisa ser um nome simples (sem `/`), para o chamador não escapar da pasta.
pub fn read_attr(dir: &Path, name: &str) -> Option<String> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name == ".." || name == "." {
        return None;
    }
    let mut buf = Vec::new();
    File::open(dir.join(name)).ok()?.take(MAX_ATTR_BYTES + 1).read_to_end(&mut buf).ok()?;
    if buf.len() as u64 > MAX_ATTR_BYTES {
        return None;
    }
    String::from_utf8(buf).ok().map(|s| s.trim().to_string())
}

/// Lê um inteiro com sinal. Valores fora de `i32` ou não numéricos viram `None`.
pub fn read_i32(dir: &Path, name: &str) -> Option<i32> {
    read_attr(dir, name)?.parse::<i32>().ok()
}

/// Lê um inteiro sem sinal limitado a `max`.
pub fn read_u32_max(dir: &Path, name: &str, max: u32) -> Option<u32> {
    read_attr(dir, name)?.parse::<u32>().ok().filter(|v| *v <= max)
}
