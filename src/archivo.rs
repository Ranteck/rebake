use crate::modelo::{Recetario, validar};
use anyhow::{Context, Result, anyhow};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn leer(ruta: &Path) -> Result<Recetario> {
    if !ruta.exists() {
        return Ok(Recetario::nuevo());
    }
    let texto =
        fs::read_to_string(ruta).with_context(|| format!("no pude leer {}", ruta.display()))?;
    parsear(&texto, &ruta.display().to_string())
}

pub fn parsear(texto: &str, nombre: &str) -> Result<Recetario> {
    let r: Recetario = toml::from_str(texto).map_err(|e| {
        let (linea, columna) = e.span().map(|s| posicion(texto, s.start)).unwrap_or((1, 1));
        anyhow!("{nombre}:{linea}:{columna}: {}", e.message())
    })?;
    validar(&r).with_context(|| format!("{nombre} no es válido"))?;
    Ok(r)
}

fn posicion(texto: &str, offset: usize) -> (usize, usize) {
    let antes = &texto[..offset.min(texto.len())];
    let linea = antes.matches('\n').count() + 1;
    let columna = antes
        .rsplit('\n')
        .next()
        .map(|l| l.chars().count())
        .unwrap_or(0)
        + 1;
    (linea, columna)
}

/// Escribe en el destino real del symlink para no reemplazar el enlace por un archivo.
pub fn guardar(ruta: &Path, r: &Recetario) -> Result<()> {
    validar(r)?;
    let destino = destino_real(ruta)?;
    let dir = destino
        .parent()
        .context("la ruta del recetario no tiene carpeta")?;
    fs::create_dir_all(dir).with_context(|| format!("no pude crear {}", dir.display()))?;
    let texto = toml::to_string_pretty(r).context("no pude serializar el recetario")?;
    let temporal = dir.join(format!(".recetario.toml.{}.tmp", std::process::id()));
    let mut f = fs::File::create(&temporal)
        .with_context(|| format!("no pude crear {}", temporal.display()))?;
    f.write_all(texto.as_bytes())?;
    f.sync_all()?;
    fs::rename(&temporal, &destino)
        .with_context(|| format!("no pude reemplazar {}", destino.display()))?;
    Ok(())
}

fn destino_real(ruta: &Path) -> Result<PathBuf> {
    match fs::symlink_metadata(ruta) {
        Ok(m) if m.file_type().is_symlink() => {
            let objetivo = fs::read_link(ruta)?;
            Ok(if objetivo.is_absolute() {
                objetivo
            } else {
                ruta.parent().unwrap_or(Path::new(".")).join(objetivo)
            })
        }
        _ => Ok(ruta.to_path_buf()),
    }
}

/// Crea un archivo solo legible por el usuario y falla si la ruta ya existe: así un symlink
/// plantado por otro usuario no se sigue (`O_EXCL`).
pub fn crear_privado(ruta: &Path, contenido: &str) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(ruta)
        .with_context(|| format!("no pude crear {}", ruta.display()))?;
    f.write_all(contenido.as_bytes())?;
    Ok(())
}
