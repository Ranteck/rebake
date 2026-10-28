use super::{Entorno, Escaneo, Hallazgo, asignar_origen, git, leer_json};
use crate::modelo::{Estado, Fuente, InstaladorHook, Recetario, Tipo, Via};
use crate::rutas::contraer;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn detectar(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    let Some(settings) = leer_json(&dir.join("settings.json"), entorno, &mut salida.avisos) else {
        return;
    };
    for comando in comandos_de_hooks(&settings) {
        let visible =
            crate::secretos::ocultar(&comando).replace(&*entorno.home.to_string_lossy(), "~");
        let mut h = Hallazgo::nuevo(&format!("hook:{visible}"), Tipo::Hook, Some(perfil));
        // Las asignaciones iniciales (`VAR=valor cmd`) no son el binario y pueden traer secretos.
        let binario = palabras(&comando, &entorno.home)
            .into_iter()
            .find(|p| !es_asignacion(p));
        // Si el comando es una ruta, es un script del usuario: primero se busca su repo.
        let es_script = binario.as_ref().is_some_and(|b| {
            let ruta = Path::new(b);
            if !b.contains('/') || !ruta.is_file() {
                return false;
            }
            if asignar_origen(&mut h, ruta, entorno) {
                return true;
            }
            // Un script local (con shebang) no es una herramienta que se instale aparte.
            let con_shebang = std::fs::read(ruta).is_ok_and(|b| b.starts_with(b"#!"));
            if con_shebang {
                h.pista = Some(format!(
                    "archivo sin origen conocido: {}",
                    contraer(ruta, &entorno.home)
                ));
            }
            con_shebang
        });
        let binario = binario.filter(|_| !es_script);
        match binario.and_then(|b| herramienta(entorno, &b, "hook", salida)) {
            Some(tool) => {
                h.requiere.push(tool.id.clone());
                h.fuente = tool.fuente.clone();
                if h.fuente.is_none() {
                    h.pista = Some(format!("comando de hook: {visible}"));
                }
                salida.hallazgos.push(tool);
            }
            None if es_script => {}
            None => h.pista = Some(format!("comando de hook: {visible}")),
        }
        // Un hook como `sh script.sh` viene del repo del script, no del intérprete.
        if h.fuente.is_none() {
            let scripts = palabras(&comando, &entorno.home)
                .into_iter()
                .filter(|p| p.contains('/'))
                .map(PathBuf::from)
                .filter(|p| p.is_file());
            for script in scripts {
                if asignar_origen(&mut h, &script, entorno) {
                    if h.fuente.is_some() {
                        h.pista = None;
                    }
                    break;
                }
            }
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
    // Un intérprete por nombre (`bash script.sh`) es una herramienta; una ruta es el script.
    let tool = palabras
        .first()
        .filter(|b| !b.contains('/'))
        .and_then(|b| herramienta(entorno, b, "statusline", salida));
    for palabra in &palabras {
        let ruta = PathBuf::from(palabra);
        if !palabra.contains('/') || !ruta.is_file() {
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
pub(super) fn herramienta(
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
            let p = p
                .replace("${HOME-}", &home)
                .replace("${HOME}", &home)
                .replace("$HOME", &home);
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

pub fn vincular(r: &mut Recetario, home: &Path) {
    let vinculos: Vec<_> = r
        .items
        .iter()
        .filter(|i| i.tipo == Tipo::Hook && !i.ausente)
        .map(|i| {
            (
                i.id.clone(),
                responsable(r, i.id.strip_prefix("hook:").unwrap_or(""), home),
            )
        })
        .collect();
    for (id, responsable) in vinculos {
        let i = r.item_mut(&id).expect("ítem del mismo recetario");
        let nuevo = i.instalado_por != responsable;
        let perfiles = i.perfiles.clone();
        let duenio = responsable.clone();
        i.instalado_por = responsable;
        if i.instalado_por.is_some() {
            i.pasos.clear();
            i.verificar = None;
            i.posible = None;
            i.error = None;
            if i.estado != Estado::Excluida {
                i.estado = Estado::Pendiente;
            }
        }
        if let Some(InstaladorHook::Item(id)) = duenio {
            let tool = r.item_mut(&id).expect("responsable del mismo recetario");
            let mut cambiado = nuevo;
            for perfil in perfiles {
                if !tool.perfiles.contains(&perfil) {
                    tool.perfiles.push(perfil);
                    cambiado = true;
                }
            }
            // La receta anterior todavía no documenta los hooks recién vinculados.
            if cambiado && tool.estado != Estado::Excluida {
                tool.pasos.clear();
                tool.estado = Estado::Pendiente;
            }
        }
    }
}

fn responsable(r: &Recetario, comando: &str, home: &Path) -> Option<InstaladorHook> {
    let mut candidatos = Vec::new();
    for segmento in segmentos(comando)? {
        let palabras = palabras(&segmento, home);
        let mut p = palabras.iter().map(String::as_str).peekable();
        while p
            .peek()
            .is_some_and(|s| es_asignacion(s) || matches!(*s, "if" | "then" | "else" | "exec"))
        {
            p.next();
        }
        let Some(mut ejecutable) = p.next() else {
            continue;
        };
        if matches!(ejecutable, "[" | "test" | "fi" | ":") {
            continue;
        }
        if matches!(
            ejecutable,
            "sh" | "bash" | "zsh" | "python" | "python3" | "node"
        ) {
            ejecutable = p.next().filter(|s| !s.starts_with('-'))?;
        } else if matches!(ejecutable, "echo" | "printf" | "command" | "env") {
            return None;
        }
        candidatos.push(responsable_de(r, ejecutable, home)?);
    }
    let primero = candidatos.first()?.clone();
    candidatos.iter().all(|c| c == &primero).then_some(primero)
}

fn responsable_de(r: &Recetario, ejecutable: &str, home: &Path) -> Option<InstaladorHook> {
    let ruta = Path::new(ejecutable);
    if ruta
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return None;
    }
    let nombre = ruta.file_name()?.to_str()?;
    let mut candidatos = Vec::new();
    for item in r.items.iter().filter(|i| i.tipo != Tipo::Hook) {
        let slug = item.id.split_once(':').map_or(item.id.as_str(), |(_, n)| n);
        if slug != nombre {
            continue;
        }
        let coincide = if !ejecutable.contains('/') {
            true
        } else if item.tipo == Tipo::Herramienta {
            false
        } else {
            item.perfiles
                .iter()
                .filter_map(|n| r.perfiles.iter().find(|p| &p.nombre == n))
                .any(|p| {
                    let base = crate::rutas::expandir(&p.dir, home)
                        .join("skills")
                        .join(slug);
                    item.tipo == Tipo::Skill && ruta.starts_with(base)
                })
                || item
                    .pista
                    .as_ref()
                    .and_then(|p| p.rsplit_once(": "))
                    .is_some_and(|(_, p)| {
                        let base = crate::rutas::expandir(p, home);
                        base.is_dir() && ruta.starts_with(base)
                    })
        };
        if coincide {
            candidatos.push(InstaladorHook::Item(item.id.clone()));
        }
    }
    match candidatos.len() {
        1 => return candidatos.pop(),
        0 => {}
        _ => return None,
    }
    if r.perfiles
        .iter()
        .any(|p| ruta.starts_with(crate::rutas::expandir(&p.dir, home)))
    {
        return None;
    }
    let relativo = ruta.strip_prefix(home).ok()?;
    let carpeta = relativo.components().next()?.as_os_str().to_str()?;
    let externo = carpeta.strip_prefix('.')?;
    if externo.is_empty()
        || !externo
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        || r.items
            .iter()
            .any(|i| i.id.split_once(':').is_some_and(|(_, n)| n == externo))
    {
        return None;
    }
    Some(InstaladorHook::Externo(externo.into()))
}

// Solo reconocemos separadores fuera de comillas; no interpretamos programas de shell.
fn segmentos(comando: &str) -> Option<Vec<String>> {
    if comando.contains("$(") || comando.contains('`') || comando.contains('\\') {
        return None;
    }
    let mut partes = vec![];
    let mut actual = String::new();
    let mut comilla = None;
    let mut chars = comando.chars().peekable();
    while let Some(c) = chars.next() {
        if let Some(q) = comilla {
            actual.push(c);
            if c == q {
                comilla = None;
            }
        } else if matches!(c, '\'' | '"') {
            comilla = Some(c);
            actual.push(c);
        } else if matches!(c, ';' | '&' | '|') {
            if c != ';' {
                if chars.next() != Some(c) {
                    return None;
                }
            }
            partes.push(std::mem::take(&mut actual));
        } else {
            actual.push(c);
        }
    }
    if comilla.is_some() {
        return None;
    }
    partes.push(actual);
    Some(partes)
}
