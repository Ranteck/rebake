use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn home() -> Result<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from).context("la variable HOME no está definida")
}

pub fn expandir(ruta: &str, home: &Path) -> PathBuf {
    match ruta.strip_prefix('~') {
        Some("") => home.to_path_buf(),
        Some(resto) if resto.starts_with('/') => home.join(&resto[1..]),
        _ => PathBuf::from(ruta),
    }
}

pub fn contraer(ruta: &Path, home: &Path) -> String {
    match ruta.strip_prefix(home) {
        Ok(resto) if resto.as_os_str().is_empty() => "~".into(),
        Ok(resto) => format!("~/{}", resto.display()),
        Err(_) => ruta.display().to_string(),
    }
}

pub fn archivo_por_defecto(home: &Path) -> PathBuf {
    home.join(".config/recetario/recetario.toml")
}

pub fn dir_estado(home: &Path) -> PathBuf {
    home.join(".local/state/recetario")
}

/// Fecha local no hace falta: `investigado` es informativo, y UTC evita depender de la zona.
pub fn hoy() -> String {
    let segundos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (a, m, d) = civil_desde_dias((segundos / 86_400) as i64);
    format!("{a:04}-{m:02}-{d:02}")
}

// Algoritmo days_from_civil inverso de Howard Hinnant.
fn civil_desde_dias(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let a = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (a, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tilde_ida_y_vuelta() {
        let home = Path::new("/home/x");
        assert_eq!(expandir("~/.claude", home), home.join(".claude"));
        assert_eq!(contraer(&home.join(".claude"), home), "~/.claude");
    }
    #[test]
    fn fecha_conocida() {
        assert_eq!(civil_desde_dias(20_733), (2026, 10, 7));
    }
}
