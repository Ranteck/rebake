use crate::escaner::{Entorno, Escaneo, leer_json};
use crate::modelo::{
    Ajuste, Estado, Item, Modo, OCULTO, Paso, Perfil, Recetario, Tipo, TituloClaudeMd,
};
use crate::rutas::contraer;
use crate::secretos;
use serde_json::Value;
use std::path::Path;

// Estas claves las reconstruyen los instaladores; repetirlas en la checklist confunde.
const CUBIERTAS: [&str; 4] = [
    "enabledPlugins",
    "extraKnownMarketplaces",
    "hooks",
    "statusLine",
];

pub fn detectar(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    let settings = leer_json(&dir.join("settings.json"), entorno, &mut salida.avisos);
    if let Some(settings) = &settings {
        salida.ajustes.extend(ajustes(perfil, settings));
    }
    let ruta = dir.join("CLAUDE.md");
    let claude_md_ok = match std::fs::read_to_string(&ruta) {
        Ok(texto) => {
            salida.titulos.extend(titulos(perfil, &texto));
            true
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
        Err(e) => {
            salida.avisos.push(format!(
                "no pude leer {}: {e}",
                contraer(&ruta, &entorno.home)
            ));
            false
        }
    };
    // Solo un perfil leído entero reemplaza su parte de la checklist en la fusión.
    if settings.is_some() && claude_md_ok {
        salida.perfiles_leidos.push(perfil.to_string());
    }
}

pub fn ajustes(perfil: &str, settings: &Value) -> Vec<Ajuste> {
    let mut salida = Vec::new();
    if let Some(mapa) = settings.as_object() {
        for (clave, valor) in mapa {
            if CUBIERTAS.contains(&clave.as_str()) {
                continue;
            }
            aplanar(perfil, clave, valor, clave == "env", &mut salida);
        }
    }
    salida
}

fn aplanar(perfil: &str, clave: &str, valor: &Value, oculto: bool, salida: &mut Vec<Ajuste>) {
    let oculto = oculto || secretos::clave_sensible(clave);
    match valor {
        Value::Object(mapa) => {
            for (k, v) in mapa {
                aplanar(perfil, &format!("{clave}.{k}"), v, oculto, salida);
            }
        }
        Value::Array(lista) => {
            for (i, v) in lista.iter().enumerate() {
                aplanar(perfil, &format!("{clave}[{i}]"), v, oculto, salida);
            }
        }
        hoja => {
            let texto = match hoja {
                Value::String(s) => s.clone(),
                otro => otro.to_string(),
            };
            salida.push(Ajuste {
                perfil: perfil.into(),
                clave: clave.into(),
                valor: if oculto || secretos::parece_secreto(&texto) {
                    OCULTO.into()
                } else {
                    secretos::ocultar(&texto)
                },
            });
        }
    }
}

pub fn titulos(perfil: &str, claude_md: &str) -> Vec<TituloClaudeMd> {
    claude_md
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('#'))
        .map(|l| l.trim_start_matches('#').trim())
        .filter(|t| !t.is_empty())
        .map(|t| TituloClaudeMd {
            perfil: perfil.into(),
            texto: t.into(),
        })
        .collect()
}

pub fn logins(r: &Recetario) -> Vec<String> {
    let mut salida: Vec<String> = r
        .perfiles
        .iter()
        .map(|p| format!("Perfil {}: abrí {} y corré /login", p.nombre, abrir(p)))
        .collect();
    salida.push("GitHub: `gh auth login`".into());
    salida
}

pub fn hooks_externos(r: &Recetario) -> Vec<String> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut grupos: BTreeMap<&str, (usize, BTreeSet<&str>)> = BTreeMap::new();
    for hook in r.items.iter().filter(|i| i.estado != Estado::Excluida) {
        if let Some(crate::modelo::InstaladorHook::Externo(nombre)) = &hook.instalado_por {
            let (cuenta, perfiles) = grupos.entry(nombre).or_default();
            *cuenta += 1;
            perfiles.extend(hook.perfiles.iter().map(String::as_str));
        }
    }
    grupos
        .into_iter()
        .map(|(nombre, (n, perfiles))| {
            let unidad = if n == 1 { "hook" } else { "hooks" };
            format!(
                "{nombre}: {n} {unidad} [{}]; aparecen al instalar o abrir {nombre}",
                perfiles.into_iter().collect::<Vec<_>>().join(", ")
            )
        })
        .collect()
}

/// Cómo abrir Claude Code en un perfil; el por defecto sin la variable (ver `en_perfil`).
fn abrir(p: &Perfil) -> String {
    if p.dir == "~/.claude" {
        "`claude`".into()
    } else {
        format!("`CLAUDE_CONFIG_DIR={} claude`", p.dir)
    }
}

/// Lo que rebake instala pero no termina solo: un servidor MCP o una receta con pasos manuales.
pub fn tiene_pasos_a_mano(i: &Item) -> bool {
    i.tipo == Tipo::Mcp || i.pasos.iter().any(|p| p.modo == Modo::Manual)
}

/// Recetas aprobadas que dejan algo a mano: "revisá X" y debajo cada paso, una línea por perfil.
pub fn a_mano(r: &Recetario) -> Vec<(String, Vec<String>)> {
    r.items
        .iter()
        .filter(|i| !i.es_sincronizado() && i.estado == Estado::Aprobada && tiene_pasos_a_mano(i))
        .map(|i| {
            let nombre = i.id.split_once(':').map_or(i.id.as_str(), |(_, n)| n);
            let mcp = i.tipo == Tipo::Mcp;
            let titulo = if mcp {
                format!("revisá {nombre} (servidor MCP)")
            } else {
                format!(
                    "revisá {nombre} ({}): instalada, pero falta esto",
                    format!("{:?}", i.tipo).to_lowercase()
                )
            };
            let pasos: Vec<String> = i
                .pasos
                .iter()
                .filter(|p| mcp || p.modo == Modo::Manual)
                .flat_map(|p| lineas_de(r, i, p))
                .collect();
            // Un servidor sin receta muestra al menos el comando que tenía en esta PC.
            let pasos = if pasos.is_empty() {
                i.pista.iter().cloned().collect()
            } else {
                pasos
            };
            (titulo, pasos)
        })
        .collect()
}

fn lineas_de(r: &Recetario, i: &Item, p: &Paso) -> Vec<String> {
    // `/x` existe solo dentro de Claude: pegarlo en la terminal no hace nada.
    if p.cmd.starts_with('/') {
        let perfiles: Vec<&Perfil> = i
            .perfiles_propios()
            .filter_map(|n| r.perfiles.iter().find(|q| &q.nombre == n))
            .collect();
        if perfiles.is_empty() {
            return vec![format!("abrí `claude` y corré {}", p.cmd)];
        }
        return perfiles
            .iter()
            .map(|q| format!("[{}] abrí {} y corré {}", q.nombre, abrir(q), p.cmd))
            .collect();
    }
    if !p.por_perfil || i.perfiles.is_empty() {
        return vec![p.cmd.clone()];
    }
    i.perfiles_propios()
        .map(|n| en_perfil(r, n, &p.cmd))
        .collect()
}

/// El perfil por defecto va sin la variable: con ella Claude Code leería `~/.claude/.claude.json`
/// en vez de `~/.claude.json`. El `export` cubre también lo que viene después de un `||`.
fn en_perfil(r: &Recetario, nombre: &str, cmd: &str) -> String {
    match r.perfiles.iter().find(|p| p.nombre == nombre) {
        Some(p) if p.dir != "~/.claude" => {
            format!("[{nombre}] (export CLAUDE_CONFIG_DIR={}; {cmd})", p.dir)
        }
        _ => format!("[{nombre}] {cmd}"),
    }
}
