use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const OCULTO: &str = "‹oculto›";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recetario {
    pub version: u32,
    #[serde(default, rename = "perfil", skip_serializing_if = "Vec::is_empty")]
    pub perfiles: Vec<Perfil>,
    #[serde(default, rename = "item", skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<Item>,
    #[serde(default, skip_serializing_if = "Checklist::vacia")]
    pub checklist: Checklist,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Perfil {
    pub nombre: String,
    pub dir: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tipo { Claude, Marketplace, Plugin, Skill, Import, Hook, Statusline, Herramienta }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Estado { Pendiente, PorRevisar, Aprobada, Excluida }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Via { Manual, Metadatos, Symlink, Busqueda }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modo { Auto, Manual }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fuente {
    pub repo: String,
    pub via: Via,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Paso {
    pub cmd: String,
    pub modo: Modo,
    #[serde(default)]
    pub por_perfil: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cita: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nota: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub tipo: Tipo,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub perfiles: Vec<String>,
    pub estado: Estado,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fuente: Option<Fuente>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pista: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requiere: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verificar: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub investigado: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motivo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "es_falso")]
    pub ausente: bool,
    #[serde(default, rename = "paso", skip_serializing_if = "Vec::is_empty")]
    pub pasos: Vec<Paso>,
}

fn es_falso(b: &bool) -> bool { !*b }

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checklist {
    #[serde(default, rename = "ajuste", skip_serializing_if = "Vec::is_empty")]
    pub ajustes: Vec<Ajuste>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sistema: Vec<PaqueteSistema>,
    #[serde(default, rename = "titulo", skip_serializing_if = "Vec::is_empty")]
    pub titulos: Vec<TituloClaudeMd>,
}

impl Checklist {
    pub fn vacia(&self) -> bool {
        self.ajustes.is_empty() && self.sistema.is_empty() && self.titulos.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ajuste { pub perfil: String, pub clave: String, pub valor: String }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaqueteSistema { pub paquete: String, pub para: String }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TituloClaudeMd { pub perfil: String, pub texto: String }

impl Recetario {
    pub fn nuevo() -> Self {
        Recetario { version: 1, perfiles: vec![], items: vec![], checklist: Checklist::default() }
    }
    pub fn item(&self, id: &str) -> Option<&Item> { self.items.iter().find(|i| i.id == id) }
    pub fn item_mut(&mut self, id: &str) -> Option<&mut Item> { self.items.iter_mut().find(|i| i.id == id) }
}

impl Item {
    pub fn nuevo(id: &str, tipo: Tipo) -> Self {
        Item {
            id: id.into(), tipo, perfiles: vec![], estado: Estado::Pendiente, fuente: None,
            pista: None, requiere: vec![], verificar: None, investigado: None, motivo: None,
            error: None, ausente: false, pasos: vec![],
        }
    }
}

/// Reglas que serde no puede expresar; corren al leer y antes de guardar.
pub fn validar(r: &Recetario) -> Result<()> {
    if r.version != 1 {
        bail!("versión de recetario no soportada: {}", r.version);
    }
    let perfiles: HashSet<&str> = r.perfiles.iter().map(|p| p.nombre.as_str()).collect();
    let mut ids = HashSet::new();
    for item in &r.items {
        if !ids.insert(item.id.as_str()) {
            bail!("el ítem {} está repetido", item.id);
        }
        if let Some(p) = item.perfiles.iter().find(|p| !perfiles.contains(p.as_str())) {
            bail!("el ítem {} usa el perfil {p}, que no existe", item.id);
        }
        if item.motivo.is_some() && item.estado != Estado::Excluida {
            bail!("el ítem {} tiene motivo pero no está excluido", item.id);
        }
    }
    Ok(())
}
