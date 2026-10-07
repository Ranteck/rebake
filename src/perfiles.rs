use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct PerfilDetectado {
    pub nombre_sugerido: String,
    pub dir: PathBuf,
}

pub fn detectar(home: &Path) -> Result<Vec<PerfilDetectado>> {
    let mut encontrados = Vec::new();
    for entrada in fs::read_dir(home).with_context(|| format!("no pude listar {}", home.display()))? {
        let entrada = entrada?;
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        let sugerido = match nombre.as_str() {
            ".claude" => "claude".to_string(),
            n => match n.strip_prefix(".claude-") {
                Some(resto) if !resto.is_empty() => resto.to_string(),
                _ => continue,
            },
        };
        let dir = entrada.path();
        if dir.is_dir() && dir.join("settings.json").is_file() {
            encontrados.push(PerfilDetectado { nombre_sugerido: sugerido, dir });
        }
    }
    encontrados.sort_by(|a, b| a.dir.cmp(&b.dir));
    Ok(encontrados)
}
