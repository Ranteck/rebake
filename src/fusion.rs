use crate::escaner::Escaneo;
use crate::modelo::{Estado, Item, Perfil, Recetario, Tipo};
use crate::perfiles::PerfilDetectado;
use crate::rutas::contraer;
use std::collections::HashSet;
use std::path::Path;

pub fn asegurar_perfiles(r: &mut Recetario, detectados: &[PerfilDetectado], home: &Path) {
    for d in detectados {
        let dir = contraer(&d.dir, home);
        if r.perfiles.iter().any(|p| p.dir == dir) { continue; }
        let mut nombre = d.nombre_sugerido.clone();
        let mut n = 2;
        while r.perfiles.iter().any(|p| p.nombre == nombre) {
            nombre = format!("{}-{n}", d.nombre_sugerido);
            n += 1;
        }
        r.perfiles.push(Perfil { nombre, dir });
    }
}

pub fn fusionar(r: &mut Recetario, escaneo: Escaneo, existe_herramienta: &dyn Fn(&str) -> bool) {
    let vistos: HashSet<String> = escaneo.hallazgos.iter().map(|h| h.id.clone()).collect();
    for h in escaneo.hallazgos {
        match r.item_mut(&h.id) {
            Some(item) => {
                item.perfiles = h.perfiles;
                item.ausente = false;
                item.pista = h.pista;
                for req in h.requiere {
                    if !item.requiere.contains(&req) { item.requiere.push(req); }
                }
                // Un pendiente puede ganar fuente (p. ej. el repo recibió un remoto); lo revisado no se toca.
                if item.estado == Estado::Pendiente && item.fuente.is_none() {
                    item.fuente = h.fuente;
                }
            }
            None => {
                let mut item = Item::nuevo(&h.id, h.tipo);
                item.perfiles = h.perfiles;
                item.fuente = h.fuente;
                item.pista = h.pista;
                item.requiere = h.requiere;
                r.items.push(item);
            }
        }
    }
    for item in r.items.iter_mut().filter(|i| !vistos.contains(&i.id)) {
        // Las herramientas que agrega la investigación no salen en el escaneo: se mira el PATH.
        item.ausente = match item.tipo {
            Tipo::Herramienta => !existe_herramienta(item.id.trim_start_matches("herramienta:")),
            _ => true,
        };
    }
    r.checklist.ajustes = escaneo.ajustes;
    r.checklist.titulos = escaneo.titulos;
}
