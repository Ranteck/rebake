use crate::modelo::{Estado, Item, Modo, Recetario, Tipo};
use crate::proceso::ejecutar;
use crate::rutas::expandir;
use anyhow::{Result, bail};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub const TOPE: Duration = Duration::from_secs(10 * 60);
pub const TOPE_COMPROBACION: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq)]
pub enum Resultado {
    Ok,
    Salteado,
    Simulado,
    Fallo(String),
    Bloqueado(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Presencia {
    Instalada,
    /// Perfiles donde falta; vacío si la receta no va por perfil.
    Falta(Vec<String>),
    SinComprobacion,
    NoSePudo(String),
}

#[derive(Debug, Clone)]
pub enum Evento {
    Inicio(String),
    Linea(String, String),
    Fin(String, Resultado),
}

pub struct Opciones {
    pub dry_run: bool,
    pub tope: Duration,
    pub home: PathBuf,
    pub log: Option<PathBuf>,
}

pub fn planificar(r: &Recetario) -> Result<Vec<String>> {
    let aprobadas: HashSet<&str> = r
        .items
        .iter()
        .filter(|i| i.estado == Estado::Aprobada && i.instalado_por.is_none())
        .map(|i| i.id.as_str())
        .collect();
    let mut hecho = HashSet::new();
    let mut pila: Vec<String> = Vec::new();
    let mut orden = Vec::new();
    for item in r.items.iter().filter(|i| aprobadas.contains(i.id.as_str())) {
        visitar(r, &item.id, &aprobadas, &mut hecho, &mut pila, &mut orden)?;
    }
    Ok(orden)
}

fn visitar(
    r: &Recetario,
    id: &str,
    aprobadas: &HashSet<&str>,
    hecho: &mut HashSet<String>,
    pila: &mut Vec<String>,
    orden: &mut Vec<String>,
) -> Result<()> {
    if hecho.contains(id) {
        return Ok(());
    }
    if let Some(i) = pila.iter().position(|x| x == id) {
        bail!("ciclo en requiere: {} → {id}", pila[i..].join(" → "));
    }
    pila.push(id.to_string());
    if let Some(item) = r.item(id) {
        for dep in item
            .requiere
            .iter()
            .filter(|d| aprobadas.contains(d.as_str()))
        {
            visitar(r, dep, aprobadas, hecho, pila, orden)?;
        }
    }
    pila.pop();
    hecho.insert(id.to_string());
    orden.push(id.to_string());
    Ok(())
}

pub fn instalar(
    r: &Recetario,
    orden: &[String],
    op: &Opciones,
    emitir: &mut dyn FnMut(Evento),
) -> Vec<(String, Resultado)> {
    let mut log = abrir_log(op);
    let mut resultados: Vec<(String, Resultado)> = Vec::new();
    for id in orden {
        let Some(item) = r.item(id) else { continue };
        if item.instalado_por.is_some() {
            continue;
        }
        emitir(Evento::Inicio(id.clone()));
        let fallida = item.requiere.iter().find(|d| {
            resultados.iter().any(|(x, res)| {
                x == *d && matches!(res, Resultado::Fallo(_) | Resultado::Bloqueado(_))
            })
        });
        let resultado = match fallida {
            Some(dep) => Resultado::Bloqueado(format!("depende de {dep}, que no se instaló")),
            None => instalar_item(r, item, op, &mut log, emitir),
        };
        emitir(Evento::Fin(id.clone(), resultado.clone()));
        resultados.push((id.clone(), resultado));
    }
    resultados
}

fn abrir_log(op: &Opciones) -> Option<File> {
    let ruta = op.log.as_ref()?;
    let abrir = || -> std::io::Result<File> {
        if let Some(dir) = ruta.parent() {
            fs::create_dir_all(dir)?;
        }
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(ruta)
    };
    match abrir() {
        Ok(f) => Some(f),
        Err(e) => {
            // Sin log la instalación sigue: la salida igual se ve en vivo.
            tracing::warn!(ruta = %ruta.display(), error = %e, "no pude abrir el log de instalación");
            None
        }
    }
}

fn perfiles_de(r: &Recetario, item: &Item, home: &Path) -> Vec<(String, PathBuf)> {
    item.perfiles
        .iter()
        .filter_map(|n| r.perfiles.iter().find(|p| &p.nombre == n))
        .map(|p| (p.nombre.clone(), expandir(&p.dir, home)))
        .collect()
}

fn comando(cmd: &str, dir: Option<&PathBuf>, home: &Path) -> Command {
    let mut c = Command::new("sh");
    c.env_remove("CLAUDE_CONFIG_DIR");
    match dir {
        // Claude Code lee `~/.claude.json` sin la variable y `~/.claude/.claude.json` con ella:
        // el perfil por defecto la ve solo la receta, no los programas que corre.
        Some(d) if *d == home.join(".claude") => {
            let ruta = d.display().to_string().replace('\'', "'\\''");
            c.arg("-c")
                .arg(format!("CLAUDE_CONFIG_DIR='{ruta}'\n{cmd}"));
        }
        Some(d) => {
            c.arg("-c").arg(cmd).env("CLAUDE_CONFIG_DIR", d);
        }
        None => {
            c.arg("-c").arg(cmd);
        }
    }
    c
}

/// Corre `verificar` de cada id, por perfil, sin instalar. Llama a `emitir` una vez por id.
pub fn comprobar(
    r: &Recetario,
    ids: &[String],
    home: &Path,
    tope: Duration,
    emitir: &mut dyn FnMut(String, Presencia),
) {
    for id in ids {
        let Some(item) = r.item(id) else { continue };
        let presencia = match &item.verificar {
            None => Presencia::SinComprobacion,
            Some(v) => presencia_de(v, &perfiles_de(r, item, home), home, tope),
        };
        emitir(id.clone(), presencia);
    }
}

fn presencia_de(
    verificar: &str,
    perfiles: &[(String, PathBuf)],
    home: &Path,
    tope: Duration,
) -> Presencia {
    let esta = |dir: Option<&PathBuf>| -> Result<bool, String> {
        let salida = ejecutar(&mut comando(verificar, dir, home), tope, &mut |_| {})
            .map_err(|e| format!("no pude ejecutar `{verificar}`: {e}"))?;
        if salida.vencido {
            return Err(format!("`{verificar}` superó {} segundos", tope.as_secs()));
        }
        Ok(salida.exito())
    };
    let mut faltan = Vec::new();
    if perfiles.is_empty() {
        match esta(None) {
            Ok(true) => {}
            Ok(false) => return Presencia::Falta(faltan),
            Err(m) => return Presencia::NoSePudo(m),
        }
    }
    for (nombre, dir) in perfiles {
        match esta(Some(dir)) {
            Ok(true) => {}
            Ok(false) => faltan.push(nombre.clone()),
            Err(m) => return Presencia::NoSePudo(m),
        }
    }
    if faltan.is_empty() {
        Presencia::Instalada
    } else {
        Presencia::Falta(faltan)
    }
}

fn instalar_item(
    r: &Recetario,
    item: &Item,
    op: &Opciones,
    log: &mut Option<File>,
    emitir: &mut dyn FnMut(Evento),
) -> Resultado {
    let perfiles = perfiles_de(r, item, &op.home);
    // Un servidor MCP nunca se instala solo, aunque una edición marque un paso como automático.
    let autos: Vec<_> = item
        .pasos
        .iter()
        .filter(|p| p.modo == Modo::Auto && item.tipo != Tipo::Mcp)
        .collect();
    if op.dry_run {
        for paso in &autos {
            if paso.por_perfil && !perfiles.is_empty() {
                for (nombre, _) in &perfiles {
                    emitir(Evento::Linea(
                        item.id.clone(),
                        format!("[{nombre}] {}", paso.cmd),
                    ));
                }
            } else {
                emitir(Evento::Linea(item.id.clone(), paso.cmd.clone()));
            }
        }
        return Resultado::Simulado;
    }
    let mut correr = |cmd: &str, dir: Option<&PathBuf>| -> Result<(), String> {
        let mut c = comando(cmd, dir, &op.home);
        if let Some(f) = log.as_mut() {
            // Un log que no se puede escribir no frena la instalación.
            let _ = writeln!(f, "== {} $ {cmd}", item.id);
        }
        let salida = ejecutar(&mut c, op.tope, &mut |l| {
            if let Some(f) = log.as_mut() {
                let _ = writeln!(f, "{l}");
            }
            emitir(Evento::Linea(item.id.clone(), l.to_string()));
        })
        .map_err(|e| format!("no pude ejecutar `{cmd}`: {e}"))?;
        if salida.vencido {
            return Err(format!("`{cmd}` superó {} segundos", op.tope.as_secs()));
        }
        if !salida.exito() {
            return Err(format!("`{cmd}` terminó con código {:?}", salida.codigo));
        }
        Ok(())
    };
    let pendientes: Vec<&(String, PathBuf)> = match &item.verificar {
        None => perfiles.iter().collect(),
        Some(v) => perfiles
            .iter()
            .filter(|(_, d)| correr(v, Some(d)).is_err())
            .collect(),
    };
    let listo = match &item.verificar {
        Some(v) if perfiles.is_empty() => correr(v, None).is_ok(),
        Some(_) => pendientes.is_empty(),
        None => false,
    };
    if listo {
        return Resultado::Salteado;
    }
    for paso in autos {
        let res = if paso.por_perfil && !perfiles.is_empty() {
            pendientes
                .iter()
                .try_for_each(|(_, d)| correr(&paso.cmd, Some(d)))
        } else {
            correr(&paso.cmd, None)
        };
        if let Err(m) = res {
            return Resultado::Fallo(m);
        }
    }
    Resultado::Ok
}
