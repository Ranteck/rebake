use crate::escaner::normalizar_url;
use crate::modelo::{Estado, Fuente, Item, Recetario, Via, validar};
use anyhow::{Context, Result, bail};

fn buscar<'a>(r: &'a mut Recetario, id: &str) -> Result<&'a mut Item> {
    r.item_mut(id)
        .with_context(|| format!("no existe el ítem {id}"))
}

pub fn aprobar(r: &mut Recetario, id: &str) -> Result<()> {
    let item = buscar(r, id)?;
    match item.estado {
        Estado::PorRevisar => item.estado = Estado::Aprobada,
        Estado::Aprobada => {}
        Estado::Pendiente => {
            bail!("{id} todavía no tiene receta: investigalo o editalo antes de aprobar")
        }
        Estado::Excluida => bail!("{id} está excluido: `x` lo devuelve"),
    }
    Ok(())
}

/// Aprueba una receta por revisar o devuelve a revisión una aprobada; dice cómo quedó.
pub fn alternar_aprobacion(r: &mut Recetario, id: &str) -> Result<Estado> {
    if buscar(r, id)?.estado == Estado::Aprobada {
        buscar(r, id)?.estado = Estado::PorRevisar;
    } else {
        aprobar(r, id)?;
    }
    Ok(buscar(r, id)?.estado)
}

/// Deshace una exclusión: vuelve a revisión si tiene pasos, o a sin receta si no.
pub fn incluir(r: &mut Recetario, id: &str) -> Result<Estado> {
    let item = buscar(r, id)?;
    if item.estado != Estado::Excluida {
        bail!("{id} no está excluido");
    }
    item.estado = if item.pasos.is_empty() {
        Estado::Pendiente
    } else {
        Estado::PorRevisar
    };
    item.motivo = None;
    Ok(item.estado)
}

pub fn excluir(r: &mut Recetario, id: &str, motivo: &str) -> Result<()> {
    let motivo = motivo.trim();
    if motivo.is_empty() {
        bail!("el motivo no puede estar vacío");
    }
    let item = buscar(r, id)?;
    item.estado = Estado::Excluida;
    item.motivo = Some(motivo.into());
    Ok(())
}

pub fn pegar_link(r: &mut Recetario, id: &str, url: &str) -> Result<()> {
    let url = url.trim();
    let resto = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));
    if resto.is_none_or(|r| r.split('/').next().unwrap_or("").is_empty()) {
        bail!("\"{url}\" no es un link http(s)");
    }
    let item = buscar(r, id)?;
    if item.estado == Estado::Excluida {
        bail!("{id} está excluido");
    }
    item.fuente = Some(Fuente {
        repo: normalizar_url(url),
        via: Via::Manual,
        doc: None,
    });
    item.estado = Estado::Pendiente;
    item.error = None;
    item.posible = None;
    Ok(())
}

/// El posible origen pasa a ser el origen, marcado como buscado en la web.
pub fn aceptar_posible(r: &mut Recetario, id: &str) -> Result<()> {
    let item = buscar(r, id)?;
    let Some(repo) = item.posible.as_ref().and_then(|p| p.pendiente()) else {
        bail!("{id} no tiene un posible origen para aceptar");
    };
    item.fuente = Some(Fuente {
        repo: repo.to_string(),
        via: Via::Busqueda,
        doc: None,
    });
    item.posible = None;
    item.error = None;
    Ok(())
}

/// Queda anotado para que `t` no lo vuelva a buscar.
pub fn descartar_posible(r: &mut Recetario, id: &str) -> Result<()> {
    let item = buscar(r, id)?;
    match item.posible.as_mut() {
        Some(p) if p.pendiente().is_some() => {
            p.descartado = true;
            Ok(())
        }
        _ => bail!("{id} no tiene un posible origen para descartar"),
    }
}

pub fn item_como_toml(item: &Item) -> Result<String> {
    toml::to_string_pretty(item).context("no pude convertir el ítem a TOML")
}

pub fn reemplazar_desde_toml(r: &mut Recetario, id: &str, texto: &str) -> Result<()> {
    let mut nuevo: Item = toml::from_str(texto)
        .map_err(|e| anyhow::anyhow!("la receta editada no es válida: {}", e.message()))?;
    if nuevo.id != id {
        bail!("no se puede cambiar el id ({id} → {})", nuevo.id);
    }
    // Aprobar es una acción aparte: una edición siempre vuelve a revisión.
    if nuevo.estado != Estado::Excluida {
        nuevo.estado = Estado::PorRevisar;
        nuevo.motivo = None;
    }
    let mut copia = r.clone();
    *buscar(&mut copia, id)? = nuevo;
    validar(&copia)?;
    *r = copia;
    Ok(())
}

pub fn renombrar_perfil(r: &mut Recetario, viejo: &str, nuevo: &str) -> Result<()> {
    let nuevo = nuevo.trim();
    if nuevo.is_empty() || nuevo.contains(char::is_whitespace) {
        bail!("el nombre del perfil no puede estar vacío ni tener espacios");
    }
    if r.perfiles.iter().any(|p| p.nombre == nuevo) {
        bail!("ya existe un perfil llamado {nuevo}");
    }
    let perfil = r
        .perfiles
        .iter_mut()
        .find(|p| p.nombre == viejo)
        .with_context(|| format!("no existe el perfil {viejo}"))?;
    perfil.nombre = nuevo.into();
    for item in &mut r.items {
        for p in item.perfiles.iter_mut().filter(|p| *p == viejo) {
            *p = nuevo.into();
        }
    }
    for a in r.checklist.ajustes.iter_mut().filter(|a| a.perfil == viejo) {
        a.perfil = nuevo.into();
    }
    for t in r.checklist.titulos.iter_mut().filter(|t| t.perfil == viejo) {
        t.perfil = nuevo.into();
    }
    Ok(())
}
