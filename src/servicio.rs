use crate::checklist;
use crate::escaner::{self, Entorno};
use crate::fusion;
use crate::investigador::{self, Trabajo, trabajo_para};
use crate::modelo::{Estado, Recetario};
use crate::perfiles;
use anyhow::{Result, bail};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::path::Path;

pub struct ResumenEscaneo {
    pub nuevos: usize,
    pub sincronizados: usize,
    pub directas: usize,
    pub avisos: Vec<String>,
    pub hooks_vinculados: usize,
    pub externos: usize,
    pub perfiles_quitados: usize,
    pub ajustes_quitados: usize,
    pub titulos_quitados: usize,
    pub referencias_quitadas: usize,
}

pub fn escanear(r: &mut Recetario, entorno: &Entorno) -> Result<ResumenEscaneo> {
    let detectados = perfiles::detectar(&entorno.home)?;
    let mut avisos = Vec::new();
    let mut quitar = HashSet::new();
    for perfil in &r.perfiles {
        let dir = crate::rutas::expandir(&perfil.dir, &entorno.home);
        let comprobacion = match std::fs::metadata(&dir) {
            Ok(m) if m.is_dir() => perfiles::es_perfil(&dir, &entorno.home),
            Ok(_) => continue,
            // Un perfil de otra PC puede ser un destino de reinstalación.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => Err(e),
        };
        match comprobacion {
            Ok(true) => {}
            Ok(false) => {
                let ajustes = r
                    .checklist
                    .ajustes
                    .iter()
                    .filter(|a| a.perfil == perfil.nombre)
                    .count();
                avisos.push(format!(
                    "{} no es un perfil de Claude Code: quitado con {ajustes} ajustes",
                    perfil.nombre
                ));
                quitar.insert(perfil.nombre.clone());
            }
            Err(e) => avisos.push(format!(
                "no pude verificar el perfil {}: {e}; se conserva",
                perfil.nombre
            )),
        }
    }
    r.perfiles.retain(|p| !quitar.contains(&p.nombre));
    let ajustes_antes = r.checklist.ajustes.len();
    let titulos_antes = r.checklist.titulos.len();
    r.checklist.ajustes.retain(|a| !quitar.contains(&a.perfil));
    r.checklist.titulos.retain(|t| !quitar.contains(&t.perfil));
    let ajustes_quitados = ajustes_antes - r.checklist.ajustes.len();
    let titulos_quitados = titulos_antes - r.checklist.titulos.len();
    let mut referencias_quitadas = 0;
    for item in &mut r.items {
        let antes = item.perfiles.len();
        item.perfiles.retain(|p| !quitar.contains(p));
        referencias_quitadas += antes - item.perfiles.len();
        if antes > 0 && item.perfiles.is_empty() && item.estado != Estado::Excluida {
            // Sin perfiles, el instalador trataría sus pasos como comandos globales.
            item.estado = Estado::Pendiente;
            item.error = Some("sin perfiles válidos: revisá los perfiles antes de aprobar".into());
        }
    }
    fusion::asegurar_perfiles(r, &detectados, &entorno.home);
    let antes: HashSet<String> = r.items.iter().map(|i| i.id.clone()).collect();
    let escaneo = escaner::escanear(entorno, &r.perfiles);
    avisos.extend(escaneo.avisos.clone());
    let existe = |nombre: &str| entorno.path.iter().any(|d| d.join(nombre).is_file());
    fusion::fusionar(r, escaneo, &existe);
    avisos.extend(escaner::comandos::vincular(r, entorno));
    let mut vistos = HashSet::new();
    avisos.retain(|aviso| vistos.insert(aviso.clone()));
    let hoy = crate::rutas::hoy();
    let mut directas = 0;
    let candidatas: Vec<String> = r
        .items
        .iter()
        .filter(|i| !antes.contains(&i.id) && !i.es_sincronizado() && i.estado == Estado::Pendiente)
        .filter(|i| investigador::receta_directa(r, i).is_some())
        .map(|i| i.id.clone())
        .collect();
    for id in candidatas {
        if let Some(receta) = r.item(&id).and_then(|i| investigador::receta_directa(r, i)) {
            investigador::aplicar(r, &id, Ok(receta), &hoy);
            directas += 1;
        }
    }
    let nuevos = r
        .items
        .iter()
        .filter(|i| !i.es_sincronizado() && !antes.contains(&i.id))
        .count();
    let sincronizados = r.items.iter().filter(|i| i.es_sincronizado()).count();
    let hooks_vinculados = r
        .items
        .iter()
        .filter(|i| i.instalado_por.is_some() && i.estado != Estado::Excluida)
        .count();
    let externos = checklist::hooks_externos(r).len();
    Ok(ResumenEscaneo {
        nuevos,
        sincronizados,
        directas,
        avisos,
        hooks_vinculados,
        externos,
        perfiles_quitados: quitar.len(),
        ajustes_quitados,
        titulos_quitados,
        referencias_quitadas,
    })
}

pub fn trabajos(r: &Recetario, ids: &[String], home: &Path) -> Result<Vec<Trabajo>> {
    if ids.is_empty() {
        return Ok(r
            .items
            .iter()
            .filter(|i| {
                !i.es_sincronizado()
                    && i.instalado_por.is_none()
                    && i.estado == Estado::Pendiente
                    && i.fuente.is_some()
            })
            .map(|i| trabajo_para(r, i, home))
            .collect());
    }
    ids.iter()
        .map(|id| match r.item(id) {
            None => bail!("no existe el ítem {id}"),
            Some(i) if i.es_sincronizado() => Err(ErrorSincronizado.into()),
            Some(i) if i.instalado_por.is_some() => bail!("el hook lo instala su herramienta"),
            Some(i) if i.estado == Estado::Excluida => bail!("{id} está excluido"),
            Some(i) => Ok(trabajo_para(r, i, home)),
        })
        .collect()
}

/// Pendientes sin fuente: investigarlos en lote sería buscar a ciegas, así que se piden a mano.
pub fn sin_origen(r: &Recetario) -> Vec<String> {
    r.items
        .iter()
        .filter(|i| {
            !i.es_sincronizado()
                && i.instalado_por.is_none()
                && i.estado == Estado::Pendiente
                && i.fuente.is_none()
        })
        .map(|i| i.id.clone())
        .collect()
}

pub fn texto_checklist(r: &Recetario) -> String {
    let mut s = String::from("\n== Checklist ==\n\nLogins\n");
    for l in checklist::logins(r) {
        let _ = writeln!(s, "  [ ] {l}");
    }
    let externos = checklist::hooks_externos(r);
    if !externos.is_empty() {
        s.push_str("\nHooks de herramientas externas\n");
        for linea in externos {
            let _ = writeln!(s, "  [ ] {linea}");
        }
    }
    let a_mano = checklist::a_mano(r);
    if !a_mano.is_empty() {
        s.push_str("\nPasos a mano (rebake no los corre)\n");
        for (titulo, pasos) in a_mano {
            let _ = writeln!(s, "  [ ] {titulo}");
            for p in pasos {
                let _ = writeln!(s, "        {p}");
            }
        }
    }
    if !r.checklist.ajustes.is_empty() {
        s.push_str("\nAjustes\n");
        for a in &r.checklist.ajustes {
            let _ = writeln!(s, "  [ ] [{}] {} = {}", a.perfil, a.clave, a.valor);
        }
    }
    if !r.checklist.sistema.is_empty() {
        s.push_str("\nPaquetes del sistema\n");
        for p in &r.checklist.sistema {
            let _ = writeln!(s, "  [ ] sudo pacman -S {} (para {})", p.paquete, p.para);
        }
    }
    if !r.checklist.titulos.is_empty() {
        s.push_str("\nTu texto propio en CLAUDE.md\n");
        for t in &r.checklist.titulos {
            let _ = writeln!(s, "  [ ] [{}] {}", t.perfil, t.texto);
        }
    }
    s
}

/// Resumen para el commit del repo del cookbook: ítems nuevos y cuántos pasaron a cada estado.
pub fn mensaje_commit(comando: Option<&str>, antes: &Recetario, despues: &Recetario) -> String {
    let nuevos = despues
        .items
        .iter()
        .filter(|i| antes.item(&i.id).is_none())
        .count();
    let mut partes = Vec::new();
    if nuevos > 0 {
        partes.push(cantidad(nuevos, "nuevo", "nuevos"));
    }
    for (estado, singular, plural) in [
        (Estado::PorRevisar, "por revisar", "por revisar"),
        (Estado::Aprobada, "aprobada", "aprobadas"),
        (Estado::Excluida, "excluida", "excluidas"),
        (Estado::Pendiente, "pendiente", "pendientes"),
    ] {
        let n = despues
            .items
            .iter()
            .filter(|i| i.estado == estado && antes.item(&i.id).is_some_and(|a| a.estado != estado))
            .count();
        if n > 0 {
            partes.push(cantidad(n, singular, plural));
        }
    }
    let prefijo = match comando {
        Some(c) => format!("rebake {c}"),
        None => "rebake".to_string(),
    };
    if partes.is_empty() {
        format!("{prefijo}: actualiza el cookbook")
    } else {
        format!("{prefijo}: {}", partes.join(", "))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Novedad {
    Nueva,
    Cambio,
}

/// Recetas aprobadas de `despues` que no estaban aprobadas en `antes`, o que cambiaron.
pub fn novedades(antes: &Recetario, despues: &Recetario) -> HashMap<String, Novedad> {
    despues
        .items
        .iter()
        .filter(|i| i.estado == Estado::Aprobada)
        .filter_map(|i| {
            let novedad = match antes.item(&i.id) {
                Some(a) if a.estado == Estado::Aprobada => {
                    (a.pasos != i.pasos || a.verificar != i.verificar).then_some(Novedad::Cambio)?
                }
                _ => Novedad::Nueva,
            };
            Some((i.id.clone(), novedad))
        })
        .collect()
}

pub fn cantidad(n: usize, singular: &str, plural: &str) -> String {
    format!("{n} {}", if n == 1 { singular } else { plural })
}

#[derive(Debug)]
pub struct ErrorSincronizado;
impl std::fmt::Display for ErrorSincronizado {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Claude sincroniza este ítem desde tu cuenta; no requiere receta. Volvé a escanear si cambiaste su origen")
    }
}
impl std::error::Error for ErrorSincronizado {}
