pub mod archivos;
pub mod comandos;
pub mod git;
pub mod mcp;
pub mod plugins;

use crate::modelo::{Ajuste, Fuente, Perfil, Tipo, TituloClaudeMd, Via};
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
        let path = std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default();
        Ok(Entorno {
            home: rutas::home()?,
            path,
            pacman,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hallazgo {
    pub id: String,
    pub tipo: Tipo,
    pub perfiles: Vec<String>,
    pub sincronizacion: std::collections::BTreeMap<String, bool>,
    pub fuente: Option<Fuente>,
    pub pista: Option<String>,
    pub descripcion: Option<String>,
    pub requiere: Vec<String>,
}

impl Hallazgo {
    pub fn nuevo(id: &str, tipo: Tipo, perfil: Option<&str>) -> Self {
        Hallazgo {
            id: id.into(),
            tipo,
            perfiles: perfil.into_iter().map(String::from).collect(),
            sincronizacion: Default::default(),
            fuente: None,
            pista: None,
            descripcion: None,
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
    /// Perfiles cuyos `settings.json` y `CLAUDE.md` se leyeron bien en este escaneo.
    pub perfiles_leidos: Vec<String>,
}

pub fn escanear(entorno: &Entorno, perfiles: &[Perfil]) -> Escaneo {
    let mut salida = Escaneo::default();
    for perfil in perfiles {
        let dir = rutas::expandir(&perfil.dir, &entorno.home);
        plugins::detectar(entorno, &perfil.nombre, &dir, &mut salida);
        archivos::detectar(entorno, &perfil.nombre, &dir, &mut salida);
        comandos::detectar(entorno, &perfil.nombre, &dir, &mut salida);
        mcp::detectar(entorno, &perfil.nombre, &dir, &mut salida);
        crate::checklist::detectar(entorno, &perfil.nombre, &dir, &mut salida);
    }
    salida.hallazgos = agrupar(std::mem::take(&mut salida.hallazgos));
    // Cada detector relee settings.json: un archivo inválido avisaría una vez por detector.
    let mut vistos = std::collections::HashSet::new();
    salida.avisos.retain(|a| vistos.insert(a.clone()));
    salida
}

fn agrupar(hallazgos: Vec<Hallazgo>) -> Vec<Hallazgo> {
    let mut unidos: Vec<Hallazgo> = Vec::new();
    for h in hallazgos {
        match unidos.iter_mut().find(|u| u.id == h.id) {
            Some(u) => {
                u.sincronizacion.extend(h.sincronizacion);
                for p in h.perfiles {
                    if !u.perfiles.contains(&p) {
                        u.perfiles.push(p);
                    }
                }
                for r in h.requiere {
                    if !u.requiere.contains(&r) {
                        u.requiere.push(r);
                    }
                }
                if u.fuente.is_none() {
                    u.fuente = h.fuente;
                }
                if u.descripcion.is_none() {
                    u.descripcion = h.descripcion;
                }
                if u.pista.is_none() {
                    u.pista = h.pista;
                }
                // Si algún perfil dio la fuente, la pista de otro perfil ya no aplica; la de un
                // servidor MCP es el comando que se registra, así que se queda.
                if u.fuente.is_some() && u.tipo != Tipo::Mcp {
                    u.pista = None;
                }
            }
            None => unidos.push(h),
        }
    }
    unidos
}

/// Saca credenciales incrustadas (`https://usuario:token@host/…`): el recetario se versiona.
pub fn normalizar_url(url: &str) -> String {
    let url = crate::secretos::sin_credenciales_url(url.trim());
    // Un repo no necesita query ni ancla, y ahí también puede viajar un token.
    let url = url.split(['?', '#']).next().unwrap_or_default();
    url.trim_end_matches('/')
        .trim_end_matches(".git")
        .to_string()
}

pub(super) fn descripcion_visible(texto: &str, entorno: &Entorno) -> Option<String> {
    let texto = texto.trim();
    (!texto.is_empty())
        .then(|| crate::secretos::ocultar(texto).replace(&*entorno.home.to_string_lossy(), "~"))
}

pub(super) fn descripcion_skill(
    ruta: &Path,
    entorno: &Entorno,
    avisos: &mut Vec<String>,
) -> Option<String> {
    let texto = match std::fs::read_to_string(ruta) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            avisos.push(format!(
                "no pude leer {}: {e}",
                rutas::contraer(ruta, &entorno.home)
            ));
            return None;
        }
    };
    let texto = texto.trim_start_matches('\u{feff}');
    let mut lineas = texto.lines();
    if lineas.next()? != "---" {
        return None;
    }
    let frente: Vec<_> = lineas.by_ref().take_while(|l| *l != "---").collect();
    // Sin cierre no es frontmatter: no extraer descripciones del cuerpo por accidente.
    if frente.len() + 1 == texto.lines().count() {
        return None;
    }
    let (indice, valor) = frente
        .iter()
        .enumerate()
        .find_map(|(i, l)| l.strip_prefix("description:").map(|v| (i, v.trim())))?;
    let continuacion = frente[indice + 1..]
        .iter()
        .take_while(|l| l.is_empty() || l.starts_with(char::is_whitespace))
        .map(|l| l.trim())
        .filter(|l| !l.is_empty());
    let descripcion = match valor.chars().next()? {
        '"' => serde_json::from_str::<String>(valor).ok()?,
        '\'' => {
            let interior = valor.strip_prefix('\'')?.strip_suffix('\'')?;
            if interior.replace("''", "").contains('\'') {
                return None;
            }
            interior.replace("''", "'")
        }
        '>' | '|'
            if matches!(
                &valor.as_bytes()[1..],
                [] | [b'+' | b'-']
                    | [b'1'..=b'9']
                    | [b'+' | b'-', b'1'..=b'9']
                    | [b'1'..=b'9', b'+' | b'-']
            ) =>
        {
            continuacion.collect::<Vec<_>>().join(" ")
        }
        '>' | '|' | '[' | '{' | '&' | '*' | '!' => return None,
        _ => std::iter::once(valor)
            .chain(continuacion)
            .collect::<Vec<_>>()
            .join(" "),
    };
    descripcion_visible(&descripcion, entorno)
}

pub fn leer_json(ruta: &Path, entorno: &Entorno, avisos: &mut Vec<String>) -> Option<Value> {
    let texto = match std::fs::read_to_string(ruta) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            avisos.push(format!(
                "no pude leer {}: {e}",
                rutas::contraer(ruta, &entorno.home)
            ));
            return None;
        }
    };
    match serde_json::from_str(&texto) {
        Ok(v) => Some(v),
        Err(e) => {
            avisos.push(format!(
                "no pude leer {}: {e}",
                rutas::contraer(ruta, &entorno.home)
            ));
            None
        }
    }
}

pub fn asignar_origen(h: &mut Hallazgo, ruta: &Path, entorno: &Entorno) -> bool {
    match git::origen_de_archivo(ruta, &entorno.home) {
        git::Origen::Repo(repo) => {
            h.fuente = Some(Fuente {
                repo,
                via: Via::Symlink,
                doc: None,
            });
            true
        }
        git::Origen::RepoSinRemoto(raiz) => {
            h.pista = Some(format!(
                "repo local sin remoto: {}",
                rutas::contraer(&raiz, &entorno.home)
            ));
            true
        }
        git::Origen::Desconocido => false,
    }
}

/// Una ruta ausente no desmiente lo recordado por un escaneo anterior.
pub fn clasificar_sincronizacion(
    h: &mut Hallazgo,
    ruta: &Path,
    perfil: &str,
    dir: &Path,
    entorno: &Entorno,
    avisos: &mut Vec<String>,
) -> bool {
    let real = match std::fs::canonicalize(ruta) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return false,
        Err(e) => {
            avisos.push(format!(
                "no pude clasificar {}: {e}",
                rutas::contraer(ruta, &entorno.home)
            ));
            return false;
        }
    };
    let raices = [
        entorno.home.join(".agents/skills/synced"),
        dir.join("skills/synced"),
        entorno.home.join(".claude/plugins/synced"),
        dir.join("plugins/synced"),
    ];
    let mut sincronizado = false;
    for raiz in raices {
        match std::fs::canonicalize(&raiz) {
            Ok(r) => sincronizado |= real.starts_with(r),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                avisos.push(format!(
                    "no pude clasificar {}: {e}",
                    rutas::contraer(&raiz, &entorno.home)
                ));
                return false;
            }
        }
    }
    h.sincronizacion
        .entry(perfil.to_string())
        .and_modify(|previo| *previo &= sincronizado)
        .or_insert(sincronizado);
    h.sincronizacion[perfil]
}
