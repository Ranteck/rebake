use anyhow::{Context, Result};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct PerfilDetectado {
    pub nombre_sugerido: String,
    pub dir: PathBuf,
}

pub fn detectar(home: &Path) -> Result<Vec<PerfilDetectado>> {
    let mut encontrados = Vec::new();
    for entrada in
        fs::read_dir(home).with_context(|| format!("no pude listar {}", home.display()))?
    {
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
        if dir.is_dir()
            && es_perfil(&dir, home)
                .with_context(|| format!("no pude reconocer el perfil {sugerido}"))?
        {
            encontrados.push(PerfilDetectado {
                nombre_sugerido: sugerido,
                dir,
            });
        }
    }
    encontrados.sort_by(|a, b| a.dir.cmp(&b.dir));
    Ok(encontrados)
}

pub(crate) fn es_perfil(dir: &Path, home: &Path) -> io::Result<bool> {
    Ok(es_archivo(&dir.join("settings.json"))?
        && (es_archivo(&dir.join(".claude.json"))?
            || (dir == home.join(".claude") && es_archivo(&home.join(".claude.json"))?)))
}

fn es_archivo(ruta: &Path) -> io::Result<bool> {
    match fs::metadata(ruta) {
        Ok(m) => Ok(m.is_file()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e),
    }
}
