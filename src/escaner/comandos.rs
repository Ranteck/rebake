use super::{Entorno, Escaneo, Hallazgo, asignar_origen, git, leer_json};
use crate::modelo::{Fuente, Tipo, Via};
use crate::rutas::contraer;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn detectar(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    let Some(settings) = leer_json(&dir.join("settings.json"), entorno, &mut salida.avisos) else {
        return;
    };
    for comando in comandos_de_hooks(&settings) {
        let visible = crate::secretos::ocultar(&comando);
        let mut h = Hallazgo::nuevo(&format!("hook:{visible}"), Tipo::Hook, Some(perfil));
        h.requiere = vec!["claude".into()];
        // Las asignaciones iniciales (`VAR=valor cmd`) no son el binario y pueden traer secretos.
        let binario = palabras(&comando, &entorno.home)
            .into_iter()
            .find(|p| !es_asignacion(p));
        match binario.and_then(|b| herramienta(entorno, &b, "hook", salida)) {
            Some(tool) => {
                h.requiere.push(tool.id.clone());
                h.fuente = tool.fuente.clone();
                if h.fuente.is_none() {
                    h.pista = Some(format!("comando de hook: {visible}"));
                }
                salida.hallazgos.push(tool);
            }
            None => h.pista = Some(format!("comando de hook: {visible}")),
        }
        salida.hallazgos.push(h);
    }
    if let Some(comando) = settings
        .pointer("/statusLine/command")
        .and_then(Value::as_str)
    {
        statusline(entorno, perfil, comando, salida);
    }
}

fn comandos_de_hooks(settings: &Value) -> Vec<String> {
    let mut comandos = Vec::new();
    for grupos in settings
        .get("hooks")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|m| m.values())
    {
        for grupo in grupos.as_array().into_iter().flatten() {
            for hook in grupo
                .get("hooks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Some(c) = hook.get("command").and_then(Value::as_str)
                    && !comandos.iter().any(|x: &String| x == c)
                {
                    comandos.push(c.to_string());
                }
            }
        }
    }
    comandos
}

fn statusline(entorno: &Entorno, perfil: &str, comando: &str, salida: &mut Escaneo) {
    let palabras = palabras(comando, &entorno.home);
    let tool = palabras
        .first()
        .and_then(|b| herramienta(entorno, b, "statusline", salida));
    for palabra in &palabras {
        let ruta = PathBuf::from(palabra);
        if !palabra.contains('/') || !ruta.is_file() || Some(palabra) == palabras.first() {
            continue;
        }
        let nombre = ruta
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut h = Hallazgo::nuevo(
            &format!("statusline:{nombre}"),
            Tipo::Statusline,
            Some(perfil),
        );
        h.requiere = vec!["claude".into()];
        if let Some(t) = &tool {
            h.requiere.push(t.id.clone());
        }
        if !asignar_origen(&mut h, &ruta, entorno) {
            h.pista = Some(format!(
                "archivo sin origen conocido: {}",
                contraer(&ruta, &entorno.home)
            ));
        }
        salida.hallazgos.push(h);
    }
    if let Some(t) = tool {
        salida.hallazgos.push(t);
    }
}

/// `None` si el binario es de pacman (es del sistema) o no está en el PATH (con aviso).
fn herramienta(
    entorno: &Entorno,
    palabra: &str,
    origen: &str,
    salida: &mut Escaneo,
) -> Option<Hallazgo> {
    let Some(ruta) = buscar(entorno, palabra) else {
        salida.avisos.push(format!(
            "no encontré en el PATH el binario {palabra} de la {origen}"
        ));
        return None;
    };
    if entorno.pacman.es_del_sistema(&ruta) {
        return None;
    }
    let nombre = ruta.file_name()?.to_string_lossy().into_owned();
    let mut h = Hallazgo::nuevo(&format!("herramienta:{nombre}"), Tipo::Herramienta, None);
    let real = std::fs::canonicalize(&ruta).unwrap_or(ruta.clone());
    match git::url_en_binario(&real) {
        Some(repo) => {
            h.fuente = Some(Fuente {
                repo,
                via: Via::Metadatos,
                doc: None,
            })
        }
        None => {
            h.pista = Some(format!(
                "binario sin origen conocido: {}",
                contraer(&ruta, &entorno.home)
            ))
        }
    }
    Some(h)
}

fn buscar(entorno: &Entorno, palabra: &str) -> Option<PathBuf> {
    if palabra.contains('/') {
        let p = PathBuf::from(palabra);
        return p.is_file().then_some(p);
    }
    entorno
        .path
        .iter()
        .map(|d| d.join(palabra))
        .find(|p| p.is_file())
}

pub fn palabras(comando: &str, home: &Path) -> Vec<String> {
    let home = home.display().to_string();
    let mut salida = Vec::new();
    let mut actual = String::new();
    let mut hay_palabra = false;
    let mut comilla: Option<char> = None;
    for c in comando.chars() {
        match (comilla, c) {
            (Some(q), c) if c == q => comilla = None,
            (Some(_), c) => actual.push(c),
            (None, '"' | '\'') => {
                comilla = Some(c);
                hay_palabra = true;
            }
            (None, c) if c.is_whitespace() => {
                if hay_palabra {
                    salida.push(std::mem::take(&mut actual));
                    hay_palabra = false;
                }
            }
            (None, c) => {
                actual.push(c);
                hay_palabra = true;
            }
        }
    }
    if hay_palabra {
        salida.push(actual);
    }
    salida
        .into_iter()
        .map(|p| {
            let p = p.replace("${HOME}", &home).replace("$HOME", &home);
            match p.strip_prefix('~') {
                Some(resto) if resto.is_empty() || resto.starts_with('/') => {
                    format!("{home}{resto}")
                }
                _ => p,
            }
        })
        .collect()
}

fn es_asignacion(palabra: &str) -> bool {
    palabra.split_once('=').is_some_and(|(n, _)| {
        !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}
