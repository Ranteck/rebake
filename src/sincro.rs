//! El cookbook en un repo git: `clonar` lo trae y marca el clon, y solo ese clon se sincroniza.
use crate::archivo;
use crate::modelo::Recetario;
use crate::proceso::{Salida, ejecutar};
use crate::secretos::{ocultar, sin_credenciales_url};
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

pub const TOPE: Duration = Duration::from_secs(60);
const MARCA: &str = "rebake.sincronizar";

/// Clona `url` en la carpeta del cookbook y la marca para sincronizar; devuelve el cookbook.
pub fn clonar(url: &str, ruta: &Path) -> Result<Recetario> {
    let (destino, nombre) = partes(ruta)?;
    if destino.exists()
        && fs::read_dir(destino)
            .with_context(|| format!("no pude leer {}", destino.display()))?
            .next()
            .is_some()
    {
        bail!(
            "{} ya existe y no está vacía: movela o usá --archivo",
            destino.display()
        );
    }
    let padre = destino
        .parent()
        .context("la carpeta del cookbook no tiene carpeta padre")?;
    fs::create_dir_all(padre).with_context(|| format!("no pude crear {}", padre.display()))?;
    // Se clona al lado y se mueve al final: un fallo no deja un clon a medias en el destino.
    let carpeta = format!(".rebake-clon.{}", std::process::id());
    let temporal = padre.join(&carpeta);
    let resultado = clonar_en(url, padre, &carpeta, nombre).and_then(|r| {
        fs::rename(&temporal, destino)
            .with_context(|| format!("no pude mover el clon a {}", destino.display()))?;
        Ok(r)
    });
    if resultado.is_err()
        && temporal.exists()
        && let Err(e) = fs::remove_dir_all(&temporal)
    {
        // El error que importa es el del clonado; este solo deja una carpeta oculta de más.
        tracing::warn!(ruta = %temporal.display(), error = %e, "no pude borrar el clon temporal");
    }
    resultado
}

fn clonar_en(url: &str, padre: &Path, carpeta: &str, nombre: &str) -> Result<Recetario> {
    git(padre, &["clone", "--quiet", "--", url, carpeta]).with_context(|| {
        format!(
            "no pude clonar {}; si el repo es privado, autenticá git con `gh auth login` y \
             `gh auth setup-git`, o con una clave ssh",
            sin_credenciales_url(url)
        )
    })?;
    let clon = padre.join(carpeta);
    let ruta = clon.join(nombre);
    // Un repo recién creado no tiene cookbook: se arranca vacío y el primer guardado lo sube.
    let r = if ruta.exists() {
        let texto = fs::read_to_string(&ruta)
            .with_context(|| format!("no pude leer {}", ruta.display()))?;
        archivo::parsear(&texto, nombre).context("el cookbook del repo no es válido")?
    } else {
        Recetario::nuevo()
    };
    git(&clon, &["config", MARCA, "true"])?;
    Ok(r)
}

/// La carpeta del cookbook (la del clon) y el nombre del archivo dentro de ella.
fn partes(ruta: &Path) -> Result<(&Path, &str)> {
    let dir = ruta
        .parent()
        .context("la ruta del cookbook no tiene carpeta")?;
    let nombre = ruta
        .file_name()
        .and_then(|n| n.to_str())
        .context("el nombre del cookbook no es texto válido")?;
    Ok((dir, nombre))
}

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let salida = correr_git(dir, args)?;
    if !salida.exito() {
        bail!("git {} falló: {}", args[0], limpiar(&salida.texto));
    }
    Ok(salida.texto)
}

/// Sin terminal: git falla en vez de pedir usuario o contraseña, y el tope corta lo que se cuelgue.
fn correr_git(dir: &Path, args: &[&str]) -> Result<Salida> {
    let mut c = Command::new("git");
    c.arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0");
    let salida = ejecutar(&mut c, TOPE, &mut |_| {})
        .with_context(|| format!("no pude ejecutar git {}", args[0]))?;
    if salida.vencido {
        bail!("git {} superó {} segundos", args[0], TOPE.as_secs());
    }
    Ok(salida)
}

/// La salida de git puede repetir la URL del remoto con usuario y token: se quitan antes de
/// `ocultar`, que no reconoce una URL seguida de `:`.
fn limpiar(texto: &str) -> String {
    texto
        .lines()
        .map(|linea| {
            let sin_usuario: Vec<String> = linea
                .split(' ')
                .map(|p| {
                    if p.contains("://") {
                        sin_credenciales_url(p)
                    } else {
                        p.to_string()
                    }
                })
                .collect();
            ocultar(&sin_usuario.join(" "))
        })
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" / ")
}

/// Si el cookbook está en la raíz de un clon hecho por `clonar`. Sin `.git` en esa carpeta no
/// se ejecuta git: un cookbook suelto o enlazado desde otro repo no se toca.
pub fn sincronizado(ruta: &Path) -> Result<bool> {
    let (dir, _) = partes(ruta)?;
    if !dir.join(".git").is_dir() {
        return Ok(false);
    }
    let salida = correr_git(dir, &["config", "--local", "--get", MARCA])?;
    Ok(salida.exito() && salida.texto.trim() == "true")
}

/// Commitea solo el cookbook si cambió y pushea lo que la rama tenga sin subir; `Ok(true)` si
/// subió algo.
pub fn publicar(ruta: &Path, mensaje: &str) -> Result<bool> {
    let (dir, nombre) = partes(ruta)?;
    if rama(dir, nombre)?.cookbook_cambiado {
        git(dir, &["add", "--", nombre])?;
        // Con la ruta, el commit lleva solo el cookbook aunque haya otras cosas en el índice.
        git(dir, &["commit", "--quiet", "-m", mensaje, "--", nombre]).context(
            "no pude commitear el cookbook; quedó guardado en disco y se commitea en la próxima \
             corrida",
        )?;
    }
    let estado = rama(dir, nombre)?;
    if !estado.con_commits || estado.adelante == Some(0) {
        return Ok(false);
    }
    git(
        dir,
        &["push", "--quiet", "--set-upstream", "origin", "HEAD"],
    )
    .context("el cookbook quedó commiteado en local y se pushea en la próxima corrida")?;
    Ok(true)
}

/// Lo que dice `git status` de la rama y del cookbook, sin tocar la red.
struct Rama {
    con_commits: bool,
    /// Commits sin subir; `None` si la rama todavía no tiene rama remota (repo recién creado o
    /// un primer push que no salió), y entonces todo commit está pendiente.
    adelante: Option<u32>,
    cookbook_cambiado: bool,
}

fn rama(dir: &Path, nombre: &str) -> Result<Rama> {
    let texto = git(dir, &["status", "--porcelain=v2", "--branch", "--", nombre])?;
    let mut estado = Rama {
        con_commits: true,
        adelante: None,
        cookbook_cambiado: false,
    };
    for linea in texto.lines() {
        if linea == "# branch.oid (initial)" {
            estado.con_commits = false;
        } else if let Some(ab) = linea.strip_prefix("# branch.ab +") {
            let n = ab.split_once(' ').map_or(ab, |(n, _)| n);
            estado.adelante = Some(
                n.parse()
                    .with_context(|| format!("git status devolvió algo inesperado: {linea}"))?,
            );
        } else if ["1 ", "2 ", "u ", "? "]
            .iter()
            .any(|p| linea.starts_with(*p))
        {
            estado.cookbook_cambiado = true;
        }
    }
    Ok(estado)
}
