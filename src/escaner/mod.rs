pub mod plugins;

use crate::modelo::{Ajuste, Fuente, Perfil, Tipo, TituloClaudeMd};
use crate::rutas;
use anyhow::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub trait Pacman {
    fn es_del_sistema(&self, binario: &Path) -> bool;
}

pub struct PacmanReal;

impl Pacman for PacmanReal {
    fn es_del_sistema(&self, binario: &Path) -> bool {
        // Sin pacman (otra distro) nada cuenta como del sistema: el ítem aparece y Denis decide.
        Command::new("pacman")
            .arg("-Qo")
            .arg(binario)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

pub struct Entorno<'a> {
    pub home: PathBuf,
    pub path: Vec<PathBuf>,
    pub pacman: &'a dyn Pacman,
}

impl<'a> Entorno<'a> {
    pub fn real(pacman: &'a dyn Pacman) -> Result<Self> {
        let path = std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
        Ok(Entorno { home: rutas::home()?, path, pacman })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hallazgo {
    pub id: String,
    pub tipo: Tipo,
    pub perfiles: Vec<String>,
    pub fuente: Option<Fuente>,
    pub pista: Option<String>,
    pub requiere: Vec<String>,
}

impl Hallazgo {
    pub fn nuevo(id: &str, tipo: Tipo, perfil: Option<&str>) -> Self {
        Hallazgo {
            id: id.into(),
            tipo,
            perfiles: perfil.into_iter().map(String::from).collect(),
            fuente: None,
            pista: None,
            requiere: vec![],
        }
    }
}

#[derive(Debug, Default)]
pub struct Escaneo {
    pub hallazgos: Vec<Hallazgo>,
    pub avisos: Vec<String>,
    pub ajustes: Vec<Ajuste>,
    pub titulos: Vec<TituloClaudeMd>,
}

pub fn escanear(entorno: &Entorno, perfiles: &[Perfil]) -> Escaneo {
    let mut salida = Escaneo::default();
    for perfil in perfiles {
        let dir = rutas::expandir(&perfil.dir, &entorno.home);
        plugins::detectar(entorno, &perfil.nombre, &dir, &mut salida);
    }
    salida.hallazgos = agrupar(std::mem::take(&mut salida.hallazgos));
    salida
}

fn agrupar(hallazgos: Vec<Hallazgo>) -> Vec<Hallazgo> {
    let mut unidos: Vec<Hallazgo> = Vec::new();
    for h in hallazgos {
        match unidos.iter_mut().find(|u| u.id == h.id) {
            Some(u) => {
                for p in h.perfiles {
                    if !u.perfiles.contains(&p) { u.perfiles.push(p); }
                }
                for r in h.requiere {
                    if !u.requiere.contains(&r) { u.requiere.push(r); }
                }
                if u.fuente.is_none() { u.fuente = h.fuente; }
                if u.pista.is_none() { u.pista = h.pista; }
            }
            None => unidos.push(h),
        }
    }
    unidos
}

pub fn normalizar_url(url: &str) -> String {
    url.trim().trim_end_matches('/').trim_end_matches(".git").to_string()
}

pub fn leer_json(ruta: &Path, entorno: &Entorno, avisos: &mut Vec<String>) -> Option<Value> {
    let texto = match std::fs::read_to_string(ruta) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            avisos.push(format!("no pude leer {}: {e}", rutas::contraer(ruta, &entorno.home)));
            return None;
        }
    };
    match serde_json::from_str(&texto) {
        Ok(v) => Some(v),
        Err(e) => {
            avisos.push(format!("no pude leer {}: {e}", rutas::contraer(ruta, &entorno.home)));
            None
        }
    }
}
