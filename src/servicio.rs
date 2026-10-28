use crate::checklist;
use crate::escaner::{self, Entorno};
use crate::fusion;
use crate::investigador::{Trabajo, trabajo_para};
use crate::modelo::{Estado, Recetario};
use crate::perfiles;
use anyhow::{Result, bail};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::path::Path;

pub struct ResumenEscaneo {
    pub nuevos: usize,
    pub avisos: Vec<String>,
}

pub fn escanear(r: &mut Recetario, entorno: &Entorno) -> Result<ResumenEscaneo> {
    let detectados = perfiles::detectar(&entorno.home)?;
    fusion::asegurar_perfiles(r, &detectados, &entorno.home);
    let antes: HashSet<String> = r.items.iter().map(|i| i.id.clone()).collect();
    let escaneo = escaner::escanear(entorno, &r.perfiles);
    let avisos = escaneo.avisos.clone();
    let existe = |nombre: &str| entorno.path.iter().any(|d| d.join(nombre).is_file());
    fusion::fusionar(r, escaneo, &existe);
    escaner::comandos::vincular(r, &entorno.home);
    let nuevos = r.items.iter().filter(|i| !antes.contains(&i.id)).count();
    Ok(ResumenEscaneo { nuevos, avisos })
}

pub fn trabajos(r: &Recetario, ids: &[String], home: &Path) -> Result<Vec<Trabajo>> {
    if ids.is_empty() {
        return Ok(r
            .items
            .iter()
            .filter(|i| {
                i.instalado_por.is_none() && i.estado == Estado::Pendiente && i.fuente.is_some()
            })
            .map(|i| trabajo_para(r, i, home))
            .collect());
    }
    ids.iter()
        .map(|id| match r.item(id) {
            None => bail!("no existe el ítem {id}"),
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
            i.instalado_por.is_none() && i.estado == Estado::Pendiente && i.fuente.is_none()
        })
        .map(|i| i.id.clone())
        .collect()
}

pub fn texto_checklist(r: &Recetario) -> String {
    let mut s = String::from("\n== Checklist ==\n\nLogins\n");
    for l in checklist::logins(r) {
        let _ = writeln!(s, "  [ ] {l}");
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
