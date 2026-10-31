use super::{
    Entorno, Escaneo, Hallazgo, clasificar_sincronizacion, descripcion_visible, leer_json,
    normalizar_url,
};
use crate::modelo::{Fuente, Tipo, Via};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

pub fn detectar(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    let instalados = leer_json(
        &dir.join("plugins/installed_plugins.json"),
        entorno,
        &mut salida.avisos,
    );
    let mut repos: HashMap<String, Option<String>> = HashMap::new();
    if let Some(Value::Object(mapa)) = leer_json(
        &dir.join("plugins/known_marketplaces.json"),
        entorno,
        &mut salida.avisos,
    ) {
        for (nombre, datos) in mapa {
            let repo = repo_de_source(datos.get("source"));
            let mut h = Hallazgo::nuevo(
                &format!("marketplace:{nombre}"),
                Tipo::Marketplace,
                Some(perfil),
            );
            let ruta = datos
                .get("installLocation")
                .and_then(Value::as_str)
                .map(|p| crate::rutas::expandir(p, &entorno.home))
                .unwrap_or_else(|| dir.join("plugins/marketplaces").join(&nombre));
            clasificar_sincronizacion(&mut h, &ruta, perfil, dir, entorno, &mut salida.avisos);
            h.descripcion = leer_json(
                &ruta.join(".claude-plugin/marketplace.json"),
                entorno,
                &mut salida.avisos,
            )
            .and_then(|v| {
                v.get("description")
                    .and_then(Value::as_str)
                    .and_then(|s| descripcion_visible(s, entorno))
                    .or_else(|| {
                        v.pointer("/metadata/description")
                            .and_then(Value::as_str)
                            .and_then(|s| descripcion_visible(s, entorno))
                    })
            });
            asignar(
                &mut h,
                repo.clone(),
                "marketplace sin fuente conocida".into(),
            );
            repos.insert(nombre, repo);
            salida.hallazgos.push(h);
        }
    }
    let Some(settings) = leer_json(&dir.join("settings.json"), entorno, &mut salida.avisos) else {
        return;
    };
    let Some(habilitados) = settings.get("enabledPlugins").and_then(Value::as_object) else {
        return;
    };
    for (clave, valor) in habilitados {
        if valor != &Value::Bool(true) {
            continue;
        }
        let Some((nombre, mkt)) = clave.split_once('@') else {
            salida
                .avisos
                .push(format!("plugin con nombre inesperado en {perfil}: {clave}"));
            continue;
        };
        let mut h = Hallazgo::nuevo(&format!("plugin:{clave}"), Tipo::Plugin, Some(perfil));
        let rutas: Vec<_> = instalados
            .as_ref()
            .and_then(|v| v.get("plugins"))
            .and_then(|v| v.get(clave))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|v| v.get("installPath").and_then(Value::as_str))
            .map(|p| crate::rutas::expandir(p, &entorno.home))
            .collect();
        if rutas.is_empty() {
            clasificar_sincronizacion(
                &mut h,
                &dir.join("plugins/marketplaces").join(mkt),
                perfil,
                dir,
                entorno,
                &mut salida.avisos,
            );
        } else {
            for ruta in &rutas {
                clasificar_sincronizacion(&mut h, ruta, perfil, dir, entorno, &mut salida.avisos);
            }
        }
        if h.sincronizacion.get(perfil) == Some(&true) {
            salida.hallazgos.push(h);
            continue;
        }
        for ruta in rutas {
            h.descripcion = leer_json(
                &ruta.join(".claude-plugin/plugin.json"),
                entorno,
                &mut salida.avisos,
            )
            .and_then(|v| {
                v.get("description")
                    .and_then(Value::as_str)
                    .and_then(|s| descripcion_visible(s, entorno))
            });
            if h.descripcion.is_some() {
                break;
            }
        }
        h.requiere = vec![format!("marketplace:{mkt}")];
        let repo_mkt = repos.get(mkt).cloned().flatten();
        let (repo, descripcion) =
            datos_del_plugin(entorno, dir, nombre, mkt, repo_mkt, &mut salida.avisos)
                .unwrap_or_default();
        h.descripcion = h.descripcion.or(descripcion);
        asignar(
            &mut h,
            repo,
            format!("plugin sin entrada en el catálogo de {mkt}"),
        );
        salida.hallazgos.push(h);
    }
}

fn asignar(h: &mut Hallazgo, repo: Option<String>, pista: String) {
    match repo {
        Some(repo) => {
            h.fuente = Some(Fuente {
                repo,
                via: Via::Metadatos,
                doc: None,
            })
        }
        None => h.pista = Some(pista),
    }
}

fn repo_de_source(source: Option<&Value>) -> Option<String> {
    let s = source?;
    match s.get("source").and_then(Value::as_str) {
        Some("github") => s
            .get("repo")
            .and_then(Value::as_str)
            .map(|r| format!("https://github.com/{r}")),
        _ => s.get("url").and_then(Value::as_str).map(normalizar_url),
    }
}

fn datos_del_plugin(
    entorno: &Entorno,
    dir: &Path,
    nombre: &str,
    mkt: &str,
    repo_mkt: Option<String>,
    avisos: &mut Vec<String>,
) -> Option<(Option<String>, Option<String>)> {
    let ruta = dir.join(format!(
        "plugins/marketplaces/{mkt}/.claude-plugin/marketplace.json"
    ));
    let catalogo = leer_json(&ruta, entorno, avisos)?;
    let entrada = catalogo
        .get("plugins")?
        .as_array()?
        .iter()
        .find(|p| p.get("name").and_then(Value::as_str) == Some(nombre))?;
    let repo = match entrada.get("source") {
        // Una ruta relativa significa que el plugin vive dentro del repo del marketplace.
        Some(Value::String(rel)) if rel.starts_with("./") => repo_mkt,
        Some(obj @ Value::Object(_)) => repo_de_source(Some(obj)),
        _ => entrada
            .get("homepage")
            .and_then(Value::as_str)
            .map(normalizar_url),
    };
    let descripcion = entrada
        .get("description")
        .and_then(Value::as_str)
        .and_then(|s| descripcion_visible(s, entorno));
    Some((repo, descripcion))
}
