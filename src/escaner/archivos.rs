use super::{Entorno, Escaneo, Hallazgo, asignar_origen, leer_json, normalizar_url};
use crate::modelo::{Fuente, Tipo, Via};
use crate::rutas::contraer;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn detectar(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    skills(entorno, perfil, dir, salida);
    imports(entorno, perfil, dir, salida);
}

fn skills(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    let carpeta = dir.join("skills");
    let entradas = match fs::read_dir(&carpeta) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
        Err(e) => {
            salida.avisos.push(format!(
                "no pude listar {}: {e}",
                contraer(&carpeta, &entorno.home)
            ));
            return;
        }
    };
    let lock = leer_json(
        &entorno.home.join(".local/state/skills/.skill-lock.json"),
        entorno,
        &mut salida.avisos,
    );
    let mut nombres: Vec<String> = entradas
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    nombres.sort();
    for nombre in nombres {
        // `synced/` lo trae la cuenta de claude.ai; no hay nada que instalar.
        if nombre == "synced" || nombre.starts_with('.') {
            continue;
        }
        let ruta = carpeta.join(&nombre);
        let mut h = Hallazgo::nuevo(&format!("skill:{nombre}"), Tipo::Skill, Some(perfil));
        h.requiere = vec!["claude".into()];
        if !asignar_origen(&mut h, &ruta, entorno) {
            match fuente_del_lock(lock.as_ref(), &nombre) {
                Some(repo) => {
                    h.fuente = Some(Fuente {
                        repo,
                        via: Via::Metadatos,
                        doc: None,
                    })
                }
                None => {
                    h.pista = Some(format!(
                        "skill sin origen conocido: {}",
                        contraer(&ruta, &entorno.home)
                    ))
                }
            }
        }
        salida.hallazgos.push(h);
    }
}

fn fuente_del_lock(lock: Option<&Value>, nombre: &str) -> Option<String> {
    let entrada = lock?.get("skills")?.get(nombre)?;
    if let Some(url) = entrada.get("sourceUrl").and_then(Value::as_str) {
        return Some(normalizar_url(url));
    }
    match entrada.get("sourceType").and_then(Value::as_str) {
        Some("github") => entrada
            .get("source")
            .and_then(Value::as_str)
            .map(|s| format!("https://github.com/{s}")),
        _ => None,
    }
}

fn imports(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    let ruta = dir.join("CLAUDE.md");
    let texto = match fs::read_to_string(&ruta) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
        Err(e) => {
            salida.avisos.push(format!(
                "no pude leer {}: {e}",
                contraer(&ruta, &entorno.home)
            ));
            return;
        }
    };
    for linea in texto.lines().map(str::trim) {
        let Some(nombre) = linea.strip_prefix('@') else {
            continue;
        };
        if nombre.is_empty() || nombre.contains(char::is_whitespace) {
            continue;
        }
        let mut h = Hallazgo::nuevo(&format!("import:{nombre}"), Tipo::Import, Some(perfil));
        h.requiere = vec!["claude".into()];
        let archivo = dir.join(nombre);
        if !asignar_origen(&mut h, &archivo, entorno) {
            h.pista = Some(format!(
                "archivo sin origen conocido: {}",
                contraer(&archivo, &entorno.home)
            ));
        }
        salida.hallazgos.push(h);
    }
}
