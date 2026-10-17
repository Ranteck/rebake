use crate::escaner::{Entorno, Escaneo, leer_json};
use crate::modelo::{Ajuste, Estado, Modo, OCULTO, Paso, Recetario, Tipo, TituloClaudeMd};
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
        .map(|p| {
            if p.dir == "~/.claude" {
                format!("Perfil {}: abrí `claude` y corré /login", p.nombre)
            } else {
                format!(
                    "Perfil {}: abrí `CLAUDE_CONFIG_DIR={} claude` y corré /login",
                    p.nombre, p.dir
                )
            }
        })
        .collect();
    salida.push("GitHub: `gh auth login`".into());
    salida
}

pub fn pasos_manuales(r: &Recetario) -> Vec<(String, Paso)> {
    r.items
        .iter()
        .filter(|i| i.estado == Estado::Aprobada && i.tipo != Tipo::Mcp)
        .flat_map(|i| {
            i.pasos
                .iter()
                .filter(|p| p.modo == Modo::Manual)
                .map(|p| (i.id.clone(), p.clone()))
        })
        .collect()
}

/// Servidores MCP aprobados: "revisá X" y debajo sus pasos, o el comando detectado si no tiene.
pub fn servidores_mcp(r: &Recetario) -> Vec<(String, Vec<String>)> {
    r.items
        .iter()
        .filter(|i| i.estado == Estado::Aprobada && i.tipo == Tipo::Mcp)
        .map(|i| {
            let nombre = i.id.trim_start_matches("mcp:");
            let pasos = if i.pasos.is_empty() {
                i.pista.iter().cloned().collect()
            } else {
                i.pasos
                    .iter()
                    .flat_map(|p| {
                        if !p.por_perfil || i.perfiles.is_empty() {
                            return vec![p.cmd.clone()];
                        }
                        i.perfiles.iter().map(|n| en_perfil(r, n, &p.cmd)).collect()
                    })
                    .collect()
            };
            (format!("revisá {nombre} (servidor MCP)"), pasos)
        })
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
