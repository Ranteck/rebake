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
        Estado::Excluida => bail!("{id} está excluido"),
    }
    Ok(())
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
    Ok(())
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
