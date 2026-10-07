use super::normalizar_url;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, PartialEq)]
pub enum Origen {
    Repo(String),
    RepoSinRemoto(PathBuf),
    Desconocido,
}

pub fn origen_de_archivo(ruta: &Path) -> Origen {
    let es_enlace = std::fs::symlink_metadata(ruta).map(|m| m.file_type().is_symlink()).unwrap_or(false);
    if !es_enlace {
        return Origen::Desconocido;
    }
    let Ok(real) = std::fs::canonicalize(ruta) else { return Origen::Desconocido };
    let dir = if real.is_dir() { real.clone() } else { real.parent().map(Path::to_path_buf).unwrap_or(real) };
    let Some(raiz) = git(&dir, &["rev-parse", "--show-toplevel"]).map(PathBuf::from) else { return Origen::Desconocido };
    match git(&raiz, &["remote", "get-url", "origin"]) {
        Some(url) => Origen::Repo(normalizar_url(&url)),
        None => Origen::RepoSinRemoto(raiz),
    }
}

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let salida = Command::new("git").arg("-C").arg(dir).args(args).output().ok()?;
    let texto = String::from_utf8(salida.stdout).ok()?.trim().to_string();
    (salida.status.success() && !texto.is_empty()).then_some(texto)
}

/// El repo de GitHub que más veces nombra el binario; suele ser el propio (p. ej. rtk-ai/rtk).
pub fn url_en_binario(ruta: &Path) -> Option<String> {
    let bytes = std::fs::read(ruta).ok()?;
    let prefijo = b"https://github.com/";
    let valido = |b: u8| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.';
    let mut conteo: HashMap<String, (usize, usize)> = HashMap::new();
    let mut i = 0;
    while let Some(pos) = bytes[i..].windows(prefijo.len()).position(|w| w == prefijo) {
        let inicio = i + pos + prefijo.len();
        let mut fin = inicio;
        let mut barras = 0;
        while fin < bytes.len() && (valido(bytes[fin]) || (bytes[fin] == b'/' && barras == 0)) {
            if bytes[fin] == b'/' { barras += 1; }
            fin += 1;
        }
        let ruta_repo = String::from_utf8_lossy(&bytes[inicio..fin]).trim_end_matches('.').to_string();
        if barras == 1 && !ruta_repo.ends_with('/') {
            let orden = conteo.len();
            conteo.entry(ruta_repo).or_insert((0, orden)).0 += 1;
        }
        i = fin.max(inicio);
    }
    conteo
        .into_iter()
        .max_by(|a, b| a.1 .0.cmp(&b.1 .0).then(b.1 .1.cmp(&a.1 .1)))
        .map(|(r, _)| normalizar_url(&format!("https://github.com/{r}")))
}
