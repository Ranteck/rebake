use crate::modelo::{Estado, Item, Modo, Recetario};
use crate::proceso::ejecutar;
use crate::rutas::expandir;
use anyhow::{bail, Result};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

pub const TOPE: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Clone, PartialEq)]
pub enum Resultado { Ok, Salteado, Simulado, Fallo(String), Bloqueado(String) }

#[derive(Debug, Clone)]
pub enum Evento { Inicio(String), Linea(String, String), Fin(String, Resultado) }

pub struct Opciones {
    pub dry_run: bool,
    pub tope: Duration,
    pub home: PathBuf,
    pub log: Option<PathBuf>,
}

pub fn planificar(r: &Recetario) -> Result<Vec<String>> {
    let aprobadas: HashSet<&str> = r.items.iter().filter(|i| i.estado == Estado::Aprobada).map(|i| i.id.as_str()).collect();
    let mut hecho = HashSet::new();
    let mut pila: Vec<String> = Vec::new();
    let mut orden = Vec::new();
    for item in r.items.iter().filter(|i| aprobadas.contains(i.id.as_str())) {
        visitar(r, &item.id, &aprobadas, &mut hecho, &mut pila, &mut orden)?;
    }
    Ok(orden)
}

fn visitar(r: &Recetario, id: &str, aprobadas: &HashSet<&str>, hecho: &mut HashSet<String>, pila: &mut Vec<String>, orden: &mut Vec<String>) -> Result<()> {
    if hecho.contains(id) { return Ok(()); }
    if let Some(i) = pila.iter().position(|x| x == id) {
        bail!("ciclo en requiere: {} → {id}", pila[i..].join(" → "));
    }
    pila.push(id.to_string());
    if let Some(item) = r.item(id) {
        for dep in item.requiere.iter().filter(|d| aprobadas.contains(d.as_str())) {
            visitar(r, dep, aprobadas, hecho, pila, orden)?;
        }
    }
    pila.pop();
    hecho.insert(id.to_string());
    orden.push(id.to_string());
    Ok(())
}

pub fn instalar(r: &Recetario, orden: &[String], op: &Opciones, emitir: &mut dyn FnMut(Evento)) -> Vec<(String, Resultado)> {
    let mut log = abrir_log(op);
    let mut resultados: Vec<(String, Resultado)> = Vec::new();
    for id in orden {
        let Some(item) = r.item(id) else { continue };
        emitir(Evento::Inicio(id.clone()));
        let fallida = item.requiere.iter().find(|d| {
            resultados.iter().any(|(x, res)| x == *d && matches!(res, Resultado::Fallo(_) | Resultado::Bloqueado(_)))
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
        if let Some(dir) = ruta.parent() { fs::create_dir_all(dir)?; }
        OpenOptions::new().create(true).write(true).truncate(true).open(ruta)
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

fn instalar_item(r: &Recetario, item: &Item, op: &Opciones, log: &mut Option<File>, emitir: &mut dyn FnMut(Evento)) -> Resultado {
    let perfiles: Vec<(String, PathBuf)> = item
        .perfiles
        .iter()
        .filter_map(|n| r.perfiles.iter().find(|p| &p.nombre == n))
        .map(|p| (p.nombre.clone(), expandir(&p.dir, &op.home)))
        .collect();
    let autos: Vec<_> = item.pasos.iter().filter(|p| p.modo == Modo::Auto).collect();
    if op.dry_run {
        for paso in &autos {
            if paso.por_perfil && !perfiles.is_empty() {
                for (nombre, _) in &perfiles { emitir(Evento::Linea(item.id.clone(), format!("[{nombre}] {}", paso.cmd))); }
            } else {
                emitir(Evento::Linea(item.id.clone(), paso.cmd.clone()));
            }
        }
        return Resultado::Simulado;
    }
    let mut correr = |cmd: &str, dir: Option<&PathBuf>| -> Result<(), String> {
        let mut c = Command::new("sh");
        c.arg("-c").arg(cmd);
        match dir {
            Some(d) => { c.env("CLAUDE_CONFIG_DIR", d); }
            None => { c.env_remove("CLAUDE_CONFIG_DIR"); }
        }
        if let Some(f) = log.as_mut() {
            // Un log que no se puede escribir no frena la instalación.
            let _ = writeln!(f, "== {} $ {cmd}", item.id);
        }
        let salida = ejecutar(&mut c, op.tope, &mut |l| {
            if let Some(f) = log.as_mut() { let _ = writeln!(f, "{l}"); }
            emitir(Evento::Linea(item.id.clone(), l.to_string()));
        })
        .map_err(|e| format!("no pude ejecutar `{cmd}`: {e}"))?;
        if salida.vencido { return Err(format!("`{cmd}` superó {} segundos", op.tope.as_secs())); }
        if !salida.exito() { return Err(format!("`{cmd}` terminó con código {:?}", salida.codigo)); }
        Ok(())
    };
    let pendientes: Vec<&(String, PathBuf)> = match &item.verificar {
        None => perfiles.iter().collect(),
        Some(v) => perfiles.iter().filter(|(_, d)| correr(v, Some(d)).is_err()).collect(),
    };
    let listo = match &item.verificar {
        Some(v) if perfiles.is_empty() => correr(v, None).is_ok(),
        Some(_) => pendientes.is_empty(),
        None => false,
    };
    if listo { return Resultado::Salteado; }
    for paso in autos {
        let res = if paso.por_perfil && !perfiles.is_empty() {
            pendientes.iter().try_for_each(|(_, d)| correr(&paso.cmd, Some(d)))
        } else {
            correr(&paso.cmd, None)
        };
        if let Err(m) = res { return Resultado::Fallo(m); }
    }
    Resultado::Ok
}
