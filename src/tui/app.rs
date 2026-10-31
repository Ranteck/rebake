use crate::acciones;
use crate::checklist;
use crate::instalador::{self, Evento, Presencia, Resultado};
use crate::investigador::{Receta, aplicar, aplicar_posible, receta_directa};
use crate::modelo::{Ajuste, Estado, Fuente, Item, Posible, Recetario, SeccionRecetas, Via};
use crate::servicio::Novedad;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use std::collections::{HashMap, HashSet};

const MAX_SALIDA: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pestana {
    Armar,
    Instalacion,
    Checklist,
}

#[derive(Debug, Clone, PartialEq)]
/// El link guarda el ítem al empezar a escribir: una investigación en segundo plano
/// puede reordenar la lista mientras tanto.
pub enum Entrada {
    Link { id: String },
    Filtro,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modo {
    Normal,
    Escribiendo { para: Entrada, texto: String },
    ConfirmarInstalacion,
    ConfirmarAprobacion { ids: Vec<String> },
    Resolver { id: String, opcion: usize },
    ConfirmarTanda { accion: AccionClaude, tanda: bool },
    ConfirmarSalida,
    Ayuda,
}

/// La confirmación conserva la decisión y su estado: un resultado tardío la invalida.
#[derive(Debug, Clone, PartialEq)]
pub enum AccionClaude {
    Armar(Vec<Item>),
    Buscar(Box<Item>),
    AceptarPosible(Box<Item>),
}

impl AccionClaude {
    pub fn corridas(&self) -> usize {
        match self {
            Self::Armar(items) => items.len(),
            Self::Buscar(_) | Self::AceptarPosible(_) => 1,
        }
    }
}

/// Una fila de Checklist: el texto corto de la lista y el completo para el panel de detalle.
#[derive(Debug, Clone, PartialEq)]
pub struct Fila {
    pub texto: String,
    pub detalle: String,
    pub clase: ClaseFila,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClaseFila {
    /// Título de sección: se ve pero no se elige.
    Titulo,
    /// Algo para hacer; espacio lo tilda.
    Marcable,
    /// El bloque de ajustes; Enter lo despliega o lo colapsa.
    Plegable { abierto: bool },
}

impl Fila {
    fn nueva(texto: String, detalle: String, clase: ClaseFila) -> Self {
        Fila {
            texto,
            detalle,
            clase,
        }
    }
    fn marcable(texto: String) -> Self {
        Fila::nueva(texto.clone(), texto, ClaseFila::Marcable)
    }
    fn titulo(texto: &str) -> Self {
        Fila::nueva(texto.into(), String::new(), ClaseFila::Titulo)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Efecto {
    Nada,
    Guardar,
    Escanear,
    Investigar(Vec<String>),
    /// Buscar en la web el posible origen de los que no lo tienen.
    Buscar(Vec<String>),
    Editar(String),
    Instalar,
    Comprobar,
    Actualizar(String),
    Salir,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Corrida {
    Investigar(String),
    Buscar(String),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TandaActiva {
    pub miembros: HashSet<String>,
    pub por_revisar: usize,
    pub posibles: usize,
    pub errores: usize,
    pub ultimo_error: Option<String>,
}

pub struct App {
    pub recetario: Recetario,
    // Aprobar cambia la marca; abrir o escanear vuelve a agrupar las filas.
    secciones: HashMap<String, SeccionRecetas>,
    pub pestana: Pestana,
    pub seleccion: usize,
    pub filtro: String,
    pub mostrar_excluidas: bool,
    pub modo: Modo,
    pub mensaje: Option<String>,
    pub aviso: Option<String>,
    pub error_guardado: Option<String>,
    pub archivo: String,
    pub repo: Option<String>,
    pub sincroniza: bool,
    /// Una corrida tardía solo ocupa el pie si todavía es la última acción.
    accion: u64,
    acciones_corridas: HashMap<Corrida, u64>,
    accion_instalacion: Option<u64>,
    pub investigando: HashSet<String>,
    pub buscando: HashSet<String>,
    /// `investigando` incluye recetas directas en hilos; estas no gastan Claude.
    pub claude_en_curso: HashSet<Corrida>,
    /// Avanza en cada vuelta del bucle: el cuadro del spinner.
    pub cuadro: usize,
    pub instalacion: Vec<(String, Option<Resultado>)>,
    pub en_curso: Option<String>,
    pub salida: Vec<String>,
    /// Se guarda el texto completo y no la posición: desplegar los ajustes corre las filas.
    pub hechos: HashSet<String>,
    /// Índice entre las filas que se pueden elegir (los títulos no cuentan).
    pub seleccion_checklist: usize,
    pub ajustes_abiertos: bool,
    pub instalando: bool,
    /// Se guardan las desmarcadas: una receta aprobada después aparece marcada sin refrescar.
    pub desmarcadas: HashSet<String>,
    pub presencia: HashMap<String, Presencia>,
    pub novedades: HashMap<String, Novedad>,
    pub seleccion_plan: usize,
    pub comprobando: bool,
    /// Versión publicada más nueva que la propia, si la revisión ya llegó.
    pub version_nueva: Option<String>,
    /// Falso con `NO_COLOR`: los estados se distinguen solo por marca y texto.
    pub color: bool,
    pub tanda_activa: Option<TandaActiva>,
}

impl App {
    pub fn nueva(recetario: Recetario) -> Self {
        App {
            secciones: recetario
                .items
                .iter()
                .map(|i| (i.id.clone(), i.seccion()))
                .collect(),
            recetario,
            pestana: Pestana::Armar,
            seleccion: 0,
            filtro: String::new(),
            mostrar_excluidas: false,
            modo: Modo::Normal,
            mensaje: None,
            aviso: None,
            error_guardado: None,
            archivo: "cookbook.toml".into(),
            repo: None,
            sincroniza: false,
            accion: 0,
            acciones_corridas: HashMap::new(),
            accion_instalacion: None,
            investigando: HashSet::new(),
            buscando: HashSet::new(),
            claude_en_curso: HashSet::new(),
            cuadro: 0,
            instalacion: vec![],
            en_curso: None,
            salida: vec![],
            hechos: HashSet::new(),
            seleccion_checklist: 0,
            ajustes_abiertos: false,
            instalando: false,
            desmarcadas: HashSet::new(),
            presencia: HashMap::new(),
            novedades: HashMap::new(),
            seleccion_plan: 0,
            comprobando: false,
            version_nueva: None,
            color: true,
            tanda_activa: None,
        }
    }

    pub fn con_arranque(mut self, instalacion: bool, novedades: HashMap<String, Novedad>) -> Self {
        if instalacion {
            self.pestana = Pestana::Instalacion;
        }
        self.novedades = novedades;
        self
    }

    /// Aprobadas en el orden en que se instalan; vacío si `planificar` falla.
    pub fn plan(&self) -> Vec<String> {
        instalador::planificar(&self.recetario).unwrap_or_default()
    }

    /// Lo que `p` instala: del plan, lo marcado que no está comprobado como instalado.
    pub fn a_instalar(&self) -> Vec<String> {
        self.plan()
            .into_iter()
            .filter(|id| self.marcada(id))
            .collect()
    }

    pub fn marcada(&self, id: &str) -> bool {
        !self.desmarcadas.contains(id) && !self.instalada(id)
    }

    pub fn instalada(&self, id: &str) -> bool {
        self.presencia.get(id) == Some(&Presencia::Instalada)
            && self
                .recetario
                .hooks_de(id)
                .all(|h| self.presencia.get(&h.id) == Some(&Presencia::Instalada))
    }

    pub fn registrar_presencia(&mut self, id: &str, p: Presencia) {
        self.presencia.insert(id.to_string(), p);
    }

    pub fn terminar_comprobacion(&mut self) {
        self.comprobando = false;
        let instaladas = self
            .presencia
            .values()
            .filter(|p| **p == Presencia::Instalada)
            .count();
        self.aviso = Some(format!(
            "comprobación: {instaladas} instaladas de {} comprobadas",
            self.presencia.len()
        ));
    }

    /// Hay plan a la vista: ni instalación en curso ni resultados de una anterior.
    pub fn viendo_plan(&self) -> bool {
        self.pestana == Pestana::Instalacion && self.instalacion.is_empty()
    }

    pub fn visibles(&self) -> Vec<&Item> {
        let filtro = self.filtro.to_lowercase();
        let mut items: Vec<&Item> = self
            .recetario
            .items
            .iter()
            .filter(|i| {
                if i.es_sincronizado() {
                    return false;
                }
                if i.estado == Estado::Excluida {
                    self.mostrar_excluidas
                } else {
                    i.instalado_por.is_none()
                }
            })
            .filter(|i| i.id.to_lowercase().contains(&filtro))
            .collect();
        items.sort_by(|a, b| {
            (self.seccion_en_lista(a), a.tipo, &a.id).cmp(&(
                self.seccion_en_lista(b),
                b.tipo,
                &b.id,
            ))
        });
        items
    }

    pub fn reordenar_recetas(&mut self) {
        self.secciones = self
            .recetario
            .items
            .iter()
            .map(|i| (i.id.clone(), i.seccion()))
            .collect();
    }

    pub fn seccion_en_lista(&self, item: &Item) -> SeccionRecetas {
        if item.estado == Estado::Excluida {
            return SeccionRecetas::Excluidas;
        }
        self.secciones
            .get(&item.id)
            .copied()
            .filter(|s| *s != SeccionRecetas::Excluidas)
            .unwrap_or(SeccionRecetas::PorResolver)
    }

    pub fn por_resolver(&self) -> Vec<&Item> {
        self.recetario
            .items
            .iter()
            .filter(|i| {
                !i.es_sincronizado()
                    && i.instalado_por.is_none()
                    && i.seccion() == crate::modelo::SeccionRecetas::PorResolver
            })
            .collect()
    }

    pub fn esta_listo(&self) -> bool {
        self.por_resolver().is_empty()
            && self.investigando.is_empty()
            && self.buscando.is_empty()
            && self.claude_en_curso.is_empty()
            && !self.instalando
            && !self.comprobando
            && self.error_guardado.is_none()
            && self
                .recetario
                .items
                .iter()
                .any(|i| !i.es_sincronizado() && i.instalado_por.is_none())
    }

    pub fn seleccionado(&self) -> Option<&Item> {
        self.visibles().get(self.seleccion).copied()
    }

    /// Gratis, investigaciones y búsquedas: alcance global, orden de la lista.
    pub fn por_armar(&self) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut items: Vec<&Item> = self
            .recetario
            .items
            .iter()
            .filter(|i| {
                !i.es_sincronizado()
                    && i.estado == Estado::Pendiente
                    && i.instalado_por.is_none()
                    && !self.investigando.contains(&i.id)
                    && !self.buscando.contains(&i.id)
            })
            .collect();
        items.sort_by(|a, b| (a.tipo, &a.id).cmp(&(b.tipo, &b.id)));
        let (mut gratis, mut investigar, mut buscar) = (vec![], vec![], vec![]);
        for item in items {
            if item.posible.is_some() {
                continue;
            }
            if item.fuente.is_some() && receta_directa(&self.recetario, item).is_some() {
                gratis.push(item.id.clone());
            } else if item.fuente.is_some() {
                investigar.push(item.id.clone());
            } else if item.posible.is_none() {
                buscar.push(item.id.clone());
            }
        }
        (gratis, investigar, buscar)
    }

    pub fn restaurar_seleccion(&mut self, elegido: Option<&str>) {
        let visibles = self.visibles();
        let siguiente = elegido.and_then(|id| visibles.iter().position(|i| i.id == id));
        self.seleccion =
            siguiente.unwrap_or_else(|| self.seleccion.min(visibles.len().saturating_sub(1)));
    }

    fn id_seleccionado(&self) -> Option<String> {
        self.seleccionado().map(|i| i.id.clone())
    }

    pub fn tecla(&mut self, k: KeyEvent) -> Efecto {
        let navegando = !matches!(
            self.modo,
            Modo::Escribiendo { .. }
                | Modo::ConfirmarTanda { .. }
                | Modo::ConfirmarAprobacion { .. }
                | Modo::ConfirmarInstalacion
                | Modo::ConfirmarSalida
        ) && matches!(
            k.code,
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Char('j' | 'k')
        );
        if !navegando {
            self.accion = self.accion.wrapping_add(1);
        }
        match self.modo.clone() {
            Modo::Escribiendo { para, texto } => self.escribiendo(k.code, para, texto),
            Modo::Resolver { id, opcion } => self.resolver(k.code, id, opcion),
            Modo::ConfirmarTanda { accion, tanda } => {
                self.modo = Modo::Normal;
                if k.code != KeyCode::Char('s') {
                    self.avisar("acción cancelada; no se gastó nada");
                    return Efecto::Nada;
                }
                let (ids, buscar) = match accion {
                    AccionClaude::Armar(items) => (
                        items
                            .into_iter()
                            .filter(|i| self.vigente(i) && i.fuente.is_some())
                            .map(|i| i.id)
                            .collect::<Vec<_>>(),
                        false,
                    ),
                    AccionClaude::Buscar(item) => (
                        if self.vigente(&item) && item.fuente.is_none() {
                            vec![item.id]
                        } else {
                            vec![]
                        },
                        true,
                    ),
                    AccionClaude::AceptarPosible(item) => {
                        if !self.vigente(&item) || !self.posible_pendiente(&item.id) {
                            self.avisar("el ítem cambió; volvé a elegir la acción");
                            return Efecto::Nada;
                        }
                        if let Err(e) = acciones::aceptar_posible(&mut self.recetario, &item.id) {
                            self.avisar(&format!("{e:#}"));
                            return Efecto::Nada;
                        }
                        (vec![item.id], false)
                    }
                };
                if ids.is_empty() {
                    self.avisar("el ítem cambió; no se gastó nada");
                    return Efecto::Nada;
                }
                if tanda {
                    self.tanda_activa = Some(TandaActiva {
                        miembros: ids.iter().cloned().collect(),
                        ..TandaActiva::default()
                    });
                }
                if buscar {
                    self.buscar(ids)
                } else {
                    self.investigar(ids)
                }
            }
            Modo::ConfirmarInstalacion => {
                self.modo = Modo::Normal;
                if k.code == KeyCode::Char('s') {
                    Efecto::Instalar
                } else {
                    self.avisar("instalación cancelada");
                    Efecto::Nada
                }
            }
            Modo::ConfirmarAprobacion { ids } => {
                let elegido = self.id_seleccionado();
                self.modo = Modo::Normal;
                if k.code != KeyCode::Char('s') {
                    self.avisar("aprobación cancelada; no se aprobó nada");
                    return Efecto::Nada;
                }
                let mut aprobadas = 0;
                for id in ids {
                    if acciones::aprobar(&mut self.recetario, &id).is_ok() {
                        aprobadas += 1;
                    }
                }
                self.restaurar_seleccion(elegido.as_deref());
                if aprobadas > 0 {
                    self.avisar(&format!(
                        "{aprobadas} recetas aprobadas (queda guardado en el cookbook)"
                    ));
                    Efecto::Guardar
                } else {
                    Efecto::Nada
                }
            }
            Modo::ConfirmarSalida => {
                self.modo = Modo::Normal;
                if k.code == KeyCode::Char('s') {
                    Efecto::Salir
                } else {
                    self.avisar("la instalación sigue");
                    Efecto::Nada
                }
            }
            Modo::Ayuda => match k.code {
                KeyCode::Char('?') | KeyCode::Esc => {
                    self.modo = Modo::Normal;
                    Efecto::Nada
                }
                KeyCode::Char('q') => {
                    self.modo = Modo::Normal;
                    self.normal(k.code)
                }
                _ => Efecto::Nada,
            },
            Modo::Normal => self.normal(k.code),
        }
    }

    fn normal(&mut self, codigo: KeyCode) -> Efecto {
        match codigo {
            KeyCode::Char('q') if self.instalando => {
                self.avisar("Hay una instalación en curso y salir la corta. ¿Salir igual? s/n");
                self.modo = Modo::ConfirmarSalida;
            }
            KeyCode::Char('q') => return Efecto::Salir,
            KeyCode::Char('?') => self.modo = Modo::Ayuda,
            KeyCode::Char('p' | 'P') if self.instalando => {
                self.avisar("ya hay una instalación en curso")
            }
            KeyCode::Char('u' | 'U') if self.instalando => {
                self.avisar("esperá que termine la instalación para actualizar")
            }
            KeyCode::Char('u' | 'U') => match self.version_nueva.clone() {
                Some(version) => return Efecto::Actualizar(version),
                None => self.avisar("rebake está al día"),
            },
            KeyCode::Char('1') => self.pestana = Pestana::Armar,
            KeyCode::Char('2') => self.pestana = Pestana::Instalacion,
            KeyCode::Char('3') => self.pestana = Pestana::Checklist,
            KeyCode::Char('s') => return Efecto::Escanear,
            // Sin depender de Shift: la mayúscula queda por costumbre, el pie muestra la minúscula.
            KeyCode::Char('p' | 'P') => {
                let n = self.a_instalar().len();
                if let Err(e) = instalador::planificar(&self.recetario) {
                    self.avisar(&format!("{e:#}"));
                } else if n == 0 {
                    self.avisar("no hay nada marcado para instalar");
                } else {
                    let m = self.plan().len();
                    self.mensaje = Some(format!("¿Instalar {n} de {m} recetas? s/n"));
                    self.modo = Modo::ConfirmarInstalacion;
                }
            }
            KeyCode::Esc if self.pestana == Pestana::Instalacion && !self.instalando => {
                self.instalacion.clear();
                self.salida.clear();
            }
            _ if self.viendo_plan() => return self.plan_instalacion(codigo),
            _ if self.pestana == Pestana::Checklist => self.checklist(codigo),
            _ if self.pestana == Pestana::Armar => return self.recetas(codigo),
            _ => {}
        }
        Efecto::Nada
    }

    fn plan_instalacion(&mut self, codigo: KeyCode) -> Efecto {
        let plan = self.plan();
        match codigo {
            KeyCode::Down | KeyCode::Char('j') if self.seleccion_plan + 1 < plan.len() => {
                self.seleccion_plan += 1
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.seleccion_plan = self.seleccion_plan.saturating_sub(1)
            }
            KeyCode::Char(' ') => {
                if let Some(id) = plan
                    .get(self.seleccion_plan)
                    .filter(|id| !self.instalada(id))
                {
                    self.alternar(id, &plan);
                    self.avisar(&format!(
                        "{} recetas marcadas para instalar",
                        self.a_instalar().len()
                    ));
                }
            }
            KeyCode::Char('t') => {
                if self.a_instalar().is_empty() {
                    self.desmarcadas.clear();
                } else {
                    self.desmarcadas.extend(plan);
                }
                self.avisar(&format!(
                    "{} recetas marcadas para instalar",
                    self.a_instalar().len()
                ));
            }
            KeyCode::Char('c') if !self.comprobando => {
                self.comprobando = true;
                self.avisar("comprobando las recetas aprobadas…");
                return Efecto::Comprobar;
            }
            _ => {}
        }
        Efecto::Nada
    }

    /// Marcar arrastra lo que la receta requiere; desmarcar, lo que la requiere a ella.
    fn alternar(&mut self, id: &str, plan: &[String]) {
        let marcar = !self.marcada(id);
        let mut afectadas: HashSet<String> = HashSet::from([id.to_string()]);
        loop {
            let antes = afectadas.len();
            for otro in plan {
                let Some(item) = self.recetario.item(otro) else {
                    continue;
                };
                if marcar && afectadas.contains(otro) {
                    afectadas.extend(item.requiere.iter().cloned());
                } else if !marcar && item.requiere.iter().any(|r| afectadas.contains(r)) {
                    afectadas.insert(otro.clone());
                }
            }
            if afectadas.len() == antes {
                break;
            }
        }
        for a in afectadas {
            if marcar {
                self.desmarcadas.remove(&a);
            } else {
                self.desmarcadas.insert(a);
            }
        }
    }

    fn checklist(&mut self, codigo: KeyCode) {
        let filas = self.filas_checklist();
        let elegibles: Vec<&Fila> = filas
            .iter()
            .filter(|f| f.clase != ClaseFila::Titulo)
            .collect();
        let actual = elegibles.get(self.seleccion_checklist).copied();
        match codigo {
            KeyCode::Down | KeyCode::Char('j')
                if self.seleccion_checklist + 1 < elegibles.len() =>
            {
                self.seleccion_checklist += 1
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.seleccion_checklist = self.seleccion_checklist.saturating_sub(1)
            }
            KeyCode::Char(' ') => {
                if let Some(f) = actual.filter(|f| f.clase == ClaseFila::Marcable)
                    && !self.hechos.remove(&f.detalle)
                {
                    self.hechos.insert(f.detalle.clone());
                }
                self.avisar(&format!(
                    "{} pendientes tildados en Checklist",
                    self.hechos.len()
                ));
            }
            // Lo de antes del bloque no cambia: la selección queda sobre él.
            KeyCode::Enter
                if actual.is_some_and(|f| matches!(f.clase, ClaseFila::Plegable { .. })) =>
            {
                self.ajustes_abiertos = !self.ajustes_abiertos
            }
            _ => {}
        }
    }

    /// La fila elegida en Checklist, si hay alguna.
    pub fn fila_elegida(&self) -> Option<Fila> {
        self.filas_checklist()
            .into_iter()
            .filter(|f| f.clase != ClaseFila::Titulo)
            .nth(self.seleccion_checklist)
    }

    fn recetas(&mut self, codigo: KeyCode) -> Efecto {
        let total = self.visibles().len();
        self.seleccion = self.seleccion.min(total.saturating_sub(1));
        let id = self.id_seleccionado();
        if let Some(responsable) = self
            .seleccionado()
            .filter(|i| i.estado != Estado::Excluida || codigo != KeyCode::Enter)
            .and_then(|i| i.instalado_por.as_ref())
            && matches!(
                codigo,
                KeyCode::Char('a' | ' ' | 'i' | 'r' | 'l' | 'n') | KeyCode::Enter
            )
        {
            self.avisar(&format!("lo instala {}", responsable.nombre()));
            return Efecto::Nada;
        }
        match (codigo, id) {
            (KeyCode::Down | KeyCode::Char('j'), _) if self.seleccion + 1 < total => {
                self.seleccion += 1
            }
            (KeyCode::Up | KeyCode::Char('k'), _) => {
                self.seleccion = self.seleccion.saturating_sub(1)
            }
            (KeyCode::Char('o'), elegido) => {
                self.mostrar_excluidas = !self.mostrar_excluidas;
                self.restaurar_seleccion(elegido.as_deref());
                self.avisar(if self.mostrar_excluidas {
                    "excluidas visibles"
                } else {
                    "excluidas ocultas"
                });
            }
            (KeyCode::Char('/'), _) => {
                self.modo = Modo::Escribiendo {
                    para: Entrada::Filtro,
                    texto: self.filtro.clone(),
                }
            }
            (KeyCode::Char('x'), Some(id))
                if self.recetario.item(&id).map(|i| i.estado) == Some(Estado::Excluida) =>
            {
                let resultado = acciones::incluir(&mut self.recetario, &id);
                return self.guardar_cambio(&id, resultado);
            }
            (KeyCode::Char('x'), Some(id)) => {
                let resultado =
                    acciones::excluir(&mut self.recetario, &id, "").map(|()| Estado::Excluida);
                return self.guardar_cambio(&id, resultado);
            }
            (KeyCode::Char('l'), Some(id)) => {
                if self.recetario.item(&id).is_some_and(|i| self.disponible(i)) {
                    self.pedir_link(id);
                }
            }
            (KeyCode::Char('e'), Some(id)) => return Efecto::Editar(id),
            // Espacio marca en las otras pestañas; acá lo marcado es la aprobación.
            (KeyCode::Char('a' | ' '), Some(id)) => {
                let resultado = acciones::alternar_aprobacion(&mut self.recetario, &id);
                return self.guardar_cambio(&id, resultado);
            }
            (KeyCode::Char('i' | 'r'), Some(id)) => {
                let Some(item) = self.recetario.item(&id).cloned() else {
                    return Efecto::Nada;
                };
                if !self.disponible(&item) {
                    self.avisar("el ítem no está disponible para armar o buscar");
                } else if item.fuente.is_some() {
                    return self.armar_item(item);
                } else if item.posible.is_none() {
                    self.preguntar(AccionClaude::Buscar(Box::new(item)), false);
                } else {
                    self.pedir_link(id);
                }
            }
            (KeyCode::Enter, Some(id)) => return self.siguiente_accion(&id),
            (KeyCode::Char('n'), Some(id)) if self.posible_pendiente(&id) => {
                return match acciones::descartar_posible(&mut self.recetario, &id) {
                    Ok(()) => {
                        self.avisar(&format!("{id}: descartado; Enter abre Resolver"));
                        Efecto::Guardar
                    }
                    Err(e) => {
                        self.avisar(&format!("{e:#}"));
                        Efecto::Nada
                    }
                };
            }
            (KeyCode::Char('v'), _) => {
                let ids: Vec<String> = self
                    .recetario
                    .items
                    .iter()
                    .filter(|i| {
                        !i.es_sincronizado()
                            && i.instalado_por.is_none()
                            && i.estado == Estado::PorRevisar
                    })
                    .map(|i| i.id.clone())
                    .collect();
                if ids.is_empty() {
                    self.avisar("no hay recetas por revisar para aprobar");
                } else {
                    self.modo = Modo::ConfirmarAprobacion { ids };
                }
                return Efecto::Nada;
            }
            (KeyCode::Char('t' | 'I'), _) => {
                let (gratis, mut investigar, _) = self.por_armar();
                let hay_gratis = !gratis.is_empty();
                let efecto = self.investigar(gratis);
                if !self.claude_en_curso.is_empty() {
                    self.avisar("esperá que terminen las corridas de Claude");
                } else if !investigar.is_empty() {
                    investigar.truncate(5);
                    let items = investigar
                        .iter()
                        .filter_map(|id| self.recetario.item(id).cloned())
                        .collect();
                    self.preguntar(AccionClaude::Armar(items), true);
                } else if !hay_gratis {
                    self.avisar(
                        "sin pendientes con origen; Enter en un ítem sin origen abre Resolver",
                    );
                }
                return efecto;
            }
            _ => {}
        }
        Efecto::Nada
    }

    fn disponible(&self, item: &Item) -> bool {
        !item.es_sincronizado()
            && item.instalado_por.is_none()
            && item.estado != Estado::Excluida
            && !self.investigando.contains(&item.id)
            && !self.buscando.contains(&item.id)
    }

    fn vigente(&self, item: &Item) -> bool {
        self.recetario.item(&item.id) == Some(item) && self.disponible(item)
    }

    fn preguntar(&mut self, accion: AccionClaude, tanda: bool) {
        self.modo = Modo::ConfirmarTanda { accion, tanda };
    }

    fn pedir_link(&mut self, id: String) {
        self.modo = Modo::Escribiendo {
            para: Entrada::Link { id },
            texto: String::new(),
        };
    }

    fn armar_item(&mut self, item: Item) -> Efecto {
        if receta_directa(&self.recetario, &item).is_some() {
            self.investigar(vec![item.id])
        } else {
            self.preguntar(AccionClaude::Armar(vec![item]), false);
            Efecto::Nada
        }
    }

    pub fn posible_gratis(&self, item: &Item) -> bool {
        let Some(repo) = item.posible.as_ref().and_then(|p| p.pendiente()) else {
            return false;
        };
        let mut aceptado = item.clone();
        aceptado.fuente = Some(Fuente {
            repo: repo.into(),
            via: Via::Busqueda,
            doc: None,
        });
        receta_directa(&self.recetario, &aceptado).is_some()
    }

    fn siguiente_accion(&mut self, id: &str) -> Efecto {
        let Some(item) = self.recetario.item(id).cloned() else {
            return Efecto::Nada;
        };
        if self.investigando.contains(id) {
            self.avisar("ya se está armando la receta");
            return Efecto::Nada;
        }
        if self.buscando.contains(id) {
            self.avisar("ya se está buscando el origen");
            return Efecto::Nada;
        }
        match item.estado {
            Estado::Excluida => self.avisar("x reincluir"),
            Estado::Aprobada => {
                let resultado = acciones::alternar_aprobacion(&mut self.recetario, id);
                return self.guardar_cambio(id, resultado);
            }
            Estado::PorRevisar => {
                let pendientes: Vec<String> = self
                    .visibles()
                    .into_iter()
                    .filter(|i| i.seccion() == crate::modelo::SeccionRecetas::PorResolver)
                    .map(|i| i.id.clone())
                    .collect();
                let pos = pendientes.iter().position(|i| i == id).unwrap_or(0);
                let siguiente = pendientes
                    .iter()
                    .skip(pos + 1)
                    .chain(pendientes.iter().take(pos))
                    .next()
                    .cloned()
                    .unwrap_or_else(|| id.into());
                let resultado =
                    acciones::aprobar(&mut self.recetario, id).map(|()| Estado::Aprobada);
                let efecto = self.guardar_cambio(id, resultado);
                self.restaurar_seleccion(Some(&siguiente));
                return efecto;
            }
            Estado::Pendiente if item.fuente.is_some() => return self.armar_item(item),
            Estado::Pendiente if self.posible_pendiente(id) => {
                if self.posible_gratis(&item) {
                    return match acciones::aceptar_posible(&mut self.recetario, id) {
                        Ok(()) => self.investigar(vec![id.into()]),
                        Err(e) => {
                            self.avisar(&format!("{e:#}"));
                            Efecto::Nada
                        }
                    };
                }
                self.preguntar(AccionClaude::AceptarPosible(Box::new(item)), false);
            }
            Estado::Pendiente => {
                self.modo = Modo::Resolver {
                    id: id.into(),
                    opcion: 0,
                }
            }
        }
        Efecto::Nada
    }

    fn resolver(&mut self, codigo: KeyCode, id: String, opcion: usize) -> Efecto {
        if codigo == KeyCode::Esc {
            self.modo = Modo::Normal;
            self.restaurar_seleccion(Some(&id));
            self.avisar("Resolver cancelado; no se cambió nada");
            return Efecto::Nada;
        }
        if codigo != KeyCode::Enter {
            let opcion = match codigo {
                KeyCode::Up | KeyCode::Left | KeyCode::Char('k') => opcion.saturating_sub(1),
                KeyCode::Down | KeyCode::Right | KeyCode::Char('j') => (opcion + 1).min(2),
                _ => opcion,
            };
            self.modo = Modo::Resolver { id, opcion };
            return Efecto::Nada;
        }
        let Some(item) = self.recetario.item(&id).cloned().filter(|i| {
            self.disponible(i)
                && i.estado == Estado::Pendiente
                && i.fuente.is_none()
                && !self.posible_pendiente(&id)
        }) else {
            self.modo = Modo::Normal;
            self.restaurar_seleccion(Some(&id));
            self.avisar("el ítem cambió; volvé a elegir la acción");
            return Efecto::Nada;
        };
        self.modo = Modo::Normal;
        self.restaurar_seleccion(Some(&id));
        match opcion {
            0 => self.pedir_link(id),
            1 => self.preguntar(AccionClaude::Buscar(Box::new(item)), false),
            _ => {
                let resultado =
                    acciones::excluir(&mut self.recetario, &id, "").map(|()| Estado::Excluida);
                return self.guardar_cambio(&id, resultado);
            }
        }
        Efecto::Nada
    }

    pub fn pegar(&mut self, texto: &str) {
        if let Modo::Escribiendo { texto: entrada, .. } = &mut self.modo {
            entrada.extend(
                texto
                    .chars()
                    .filter(|c| !c.is_control() && !matches!(c, '\u{2028}' | '\u{2029}')),
            );
        }
    }

    fn guardar_cambio(&mut self, id: &str, resultado: anyhow::Result<Estado>) -> Efecto {
        self.restaurar_seleccion(Some(id));
        match resultado {
            Ok(Estado::Aprobada) => {
                self.avisar(&format!("{id}: aprobada (queda guardado en el cookbook)"));
                Efecto::Guardar
            }
            Ok(Estado::Excluida) => {
                self.avisar("excluida; o muestra las excluidas para volver a incluir");
                Efecto::Guardar
            }
            Ok(Estado::PorRevisar) => {
                self.avisar(&format!("{id}: vuelve a por revisar"));
                Efecto::Guardar
            }
            Ok(_) => {
                self.avisar(&format!("{id}: vuelve a sin receta"));
                Efecto::Guardar
            }
            Err(e) => {
                self.avisar(&format!("{e:#}"));
                Efecto::Nada
            }
        }
    }

    fn fin_tanda(&mut self, tanda: TandaActiva, accion: Option<u64>) {
        tracing::info!(
            accion = "tanda_terminada",
            por_revisar = tanda.por_revisar,
            posibles = tanda.posibles,
            errores = tanda.errores,
        );
        let mut partes = Vec::new();
        if tanda.por_revisar > 0 {
            partes.push(format!("{} por revisar", tanda.por_revisar));
        }
        if tanda.posibles > 0 {
            let etiqueta = if tanda.posibles == 1 {
                "posible origen"
            } else {
                "posibles orígenes"
            };
            partes.push(format!("{} {etiqueta}", tanda.posibles));
        }
        if tanda.errores == 1 {
            if let Some(err) = tanda.ultimo_error {
                partes.push(format!("1 error: {err}"));
            } else {
                partes.push("1 error".into());
            }
        } else if tanda.errores > 1 {
            partes.push(format!("{} errores", tanda.errores));
        }
        let balance = if partes.is_empty() {
            "sin cambios".into()
        } else {
            partes.join(", ")
        };

        let (gratis, investigar, _) = self.por_armar();
        let quedan_pendientes = !gratis.is_empty() || !investigar.is_empty();
        let hay_por_revisar = tanda.por_revisar > 0
            || self
                .recetario
                .items
                .iter()
                .any(|i| i.estado == Estado::PorRevisar);

        let sugerencia = match (hay_por_revisar, quedan_pendientes) {
            (true, true) => "; `v` aprueba o `t` arma otra tanda",
            (true, false) => "; `v` aprueba",
            (false, true) => "; `t` arma otra tanda",
            (false, false) => "",
        };

        self.completar(format!("tanda lista: {balance}{sugerencia}"), accion);
    }

    pub fn posible_pendiente(&self, id: &str) -> bool {
        self.recetario
            .item(id)
            .and_then(|i| i.posible.as_ref())
            .is_some_and(|p| p.pendiente().is_some())
    }

    fn buscar(&mut self, ids: Vec<String>) -> Efecto {
        let ids: Vec<String> = ids
            .into_iter()
            .filter(|id| !self.buscando.contains(id) && !self.investigando.contains(id))
            .filter(|id| {
                self.recetario.item(id).is_some_and(|i| {
                    !i.es_sincronizado()
                        && i.instalado_por.is_none()
                        && i.estado != Estado::Excluida
                        && i.fuente.is_none()
                })
            })
            .collect();
        if ids.is_empty() {
            return Efecto::Nada;
        }
        self.buscando.extend(ids.iter().cloned());
        self.acciones_corridas.extend(
            ids.iter()
                .cloned()
                .map(|id| (Corrida::Buscar(id), self.accion)),
        );
        self.claude_en_curso
            .extend(ids.iter().cloned().map(Corrida::Buscar));
        self.avisar(&format!("buscando de dónde vienen {} ítems…", ids.len()));
        Efecto::Buscar(ids)
    }

    pub fn registrar_busqueda(&mut self, id: &str, resultado: anyhow::Result<Posible>) {
        let accion = self.acciones_corridas.remove(&Corrida::Buscar(id.into()));
        self.buscando.remove(id);
        self.claude_en_curso.remove(&Corrida::Buscar(id.into()));
        let texto = match &resultado {
            Ok(p) if p.repo.is_some() => {
                format!("{id}: posible origen, mirá el panel de la derecha")
            }
            Ok(_) => format!("{id}: no encontré de dónde viene; Enter abre Resolver"),
            Err(e) => format!("{id} → error: {e:#}"),
        };
        let mut tanda_fin = None;
        if let Some(t) = &mut self.tanda_activa
            && t.miembros.remove(id)
        {
            match &resultado {
                Ok(p) if p.repo.is_some() => t.posibles += 1,
                Ok(_) => {}
                Err(e) => {
                    t.errores += 1;
                    t.ultimo_error = Some(format!("{e:#}"));
                }
            }
            if t.miembros.is_empty() {
                tanda_fin = self.tanda_activa.take();
            }
        }
        let elegido = self.id_seleccionado();
        aplicar_posible(&mut self.recetario, id, resultado);
        self.restaurar_seleccion(elegido.as_deref());
        if let Some(tanda) = tanda_fin {
            self.fin_tanda(tanda, accion);
        } else {
            self.completar(texto, accion);
        }
    }

    pub fn tic(&mut self) {
        self.cuadro = self.cuadro.wrapping_add(1);
    }

    fn investigar(&mut self, ids: Vec<String>) -> Efecto {
        let ids: Vec<String> = ids
            .into_iter()
            .filter(|id| !self.investigando.contains(id))
            .filter(|id| {
                self.recetario.item(id).is_some_and(|i| {
                    !i.es_sincronizado()
                        && i.instalado_por.is_none()
                        && i.estado != Estado::Excluida
                })
            })
            .collect();
        if ids.is_empty() {
            self.avisar("no hay nada para investigar");
            return Efecto::Nada;
        }
        self.investigando.extend(ids.iter().cloned());
        self.acciones_corridas.extend(
            ids.iter()
                .cloned()
                .map(|id| (Corrida::Investigar(id), self.accion)),
        );
        self.claude_en_curso.extend(
            ids.iter()
                .filter(|id| {
                    self.recetario
                        .item(id)
                        .is_some_and(|i| receta_directa(&self.recetario, i).is_none())
                })
                .cloned()
                .map(Corrida::Investigar),
        );
        self.avisar(&format!("investigando {} ítems…", ids.len()));
        Efecto::Investigar(ids)
    }

    fn escribiendo(&mut self, codigo: KeyCode, para: Entrada, mut texto: String) -> Efecto {
        match codigo {
            KeyCode::Esc => {
                let elegido = self.id_seleccionado();
                self.modo = Modo::Normal;
                match para {
                    Entrada::Filtro => {
                        self.filtro.clear();
                        self.restaurar_seleccion(elegido.as_deref());
                    }
                    Entrada::Link { id } => self.restaurar_seleccion(Some(&id)),
                }
                self.avisar("entrada cancelada; no se cambió el origen");
            }
            KeyCode::Backspace => {
                texto.pop();
                self.modo = Modo::Escribiendo { para, texto };
            }
            KeyCode::Char(c) if !c.is_control() => {
                texto.push(c);
                self.modo = Modo::Escribiendo { para, texto };
            }
            KeyCode::Enter => {
                self.modo = Modo::Normal;
                // El filtro se aplica aunque hoy no haya ningún ítem seleccionado.
                if para == Entrada::Filtro {
                    let elegido = self.id_seleccionado();
                    self.filtro = texto;
                    self.restaurar_seleccion(elegido.as_deref());
                    self.avisar(&format!(
                        "filtro aplicado: {} ítems visibles",
                        self.visibles().len()
                    ));
                    return Efecto::Nada;
                }
                let Entrada::Link { id } = &para else {
                    return Efecto::Nada;
                };
                let id = id.clone();
                self.restaurar_seleccion(Some(&id));
                let Some(item) = self
                    .recetario
                    .item(&id)
                    .cloned()
                    .filter(|i| self.disponible(i))
                else {
                    self.avisar("el ítem cambió; volvé a elegir la acción");
                    return Efecto::Nada;
                };
                if texto.trim().is_empty() {
                    if item.fuente.is_none() {
                        self.preguntar(AccionClaude::Buscar(Box::new(item)), false);
                    } else {
                        self.avisar("el ítem ya tiene origen; Enter arma la receta");
                    }
                    return Efecto::Nada;
                }
                return match acciones::pegar_link(&mut self.recetario, &id, &texto) {
                    Ok(()) => {
                        self.restaurar_seleccion(Some(&id));
                        if self
                            .recetario
                            .item(&id)
                            .is_some_and(|i| receta_directa(&self.recetario, i).is_some())
                        {
                            self.investigar(vec![id])
                        } else {
                            self.avisar("origen guardado; Enter arma la receta (1 corrida)");
                            Efecto::Guardar
                        }
                    }
                    Err(e) => {
                        self.avisar(&format!("{e:#}"));
                        Efecto::Nada
                    }
                };
            }
            _ => self.modo = Modo::Escribiendo { para, texto },
        }
        Efecto::Nada
    }

    fn avisar(&mut self, texto: &str) {
        self.mensaje = Some(texto.to_string());
    }

    fn completar(&mut self, texto: String, accion: Option<u64>) {
        if accion.is_none_or(|a| a == self.accion) {
            self.mensaje = Some(texto);
        } else {
            self.aviso = Some(texto);
        }
    }

    pub fn iniciar_instalacion(&mut self) {
        self.accion_instalacion = Some(self.accion);
        self.avisar(&format!("instalando {} recetas…", self.instalacion.len()));
    }

    pub fn filas_checklist(&self) -> Vec<Fila> {
        let r = &self.recetario;
        let mut f = vec![Fila::titulo("Logins")];
        f.extend(checklist::logins(r).into_iter().map(Fila::marcable));
        let externos = checklist::hooks_externos(r);
        if !externos.is_empty() {
            f.push(Fila::titulo("Hooks de herramientas externas"));
            f.extend(externos.into_iter().map(Fila::marcable));
        }
        let a_mano = checklist::a_mano(r);
        if !a_mano.is_empty() {
            f.push(Fila::titulo("Pasos a mano (rebake no los corre)"));
            for (titulo, pasos) in a_mano {
                // Elegir el bloque muestra todos sus pasos enteros en el detalle.
                let detalle = std::iter::once(titulo.clone())
                    .chain(pasos.iter().map(|p| format!("  {p}")))
                    .collect::<Vec<_>>()
                    .join("\n");
                f.push(Fila::nueva(titulo, detalle, ClaseFila::Marcable));
                f.extend(
                    pasos
                        .into_iter()
                        .map(|p| Fila::nueva(format!("    {p}"), p, ClaseFila::Marcable)),
                );
            }
        }
        let ajustes = agrupar_ajustes(&r.checklist.ajustes);
        if !ajustes.is_empty() {
            let abierto = self.ajustes_abiertos;
            let accion = if abierto { "colapsa" } else { "despliega" };
            f.push(Fila::nueva(
                format!("Ajustes ({}) · Enter {accion}", ajustes.len()),
                "Ajustes de settings.json que rebake no copia: revisalos y ponelos a mano si los querés.".into(),
                ClaseFila::Plegable { abierto },
            ));
            if abierto {
                f.extend(ajustes);
            }
        }
        if !r.checklist.sistema.is_empty() {
            f.push(Fila::titulo("Paquetes del sistema"));
            f.extend(r.checklist.sistema.iter().map(|p| {
                Fila::marcable(format!("sudo pacman -S {} (para {})", p.paquete, p.para))
            }));
        }
        if !r.checklist.titulos.is_empty() {
            f.push(Fila::titulo("Tu texto propio en CLAUDE.md"));
            f.extend(
                r.checklist
                    .titulos
                    .iter()
                    .map(|t| Fila::marcable(format!("[{}] {}", t.perfil, t.texto))),
            );
        }
        f
    }

    pub fn registrar_investigacion(
        &mut self,
        id: &str,
        resultado: anyhow::Result<Receta>,
        hoy: &str,
    ) {
        let accion = self
            .acciones_corridas
            .remove(&Corrida::Investigar(id.into()));
        self.investigando.remove(id);
        self.claude_en_curso.remove(&Corrida::Investigar(id.into()));
        let estado = crate::investigador::resultado_receta(&self.recetario, id, &resultado);
        let texto = match &resultado {
            Ok(_) if estado == "pendiente" => {
                "registro de hooks pendiente: falta documentación del repositorio".into()
            }
            Ok(_) => format!("{id} → por revisar"),
            Err(e) => format!("{id} → error: {e:#}"),
        };
        let err_inicial = resultado.as_ref().err().map(|e| format!("{e:#}"));
        let mut tanda_fin = None;
        let esta_en_tanda = self
            .tanda_activa
            .as_mut()
            .is_some_and(|t| t.miembros.remove(id));
        let elegido = self.id_seleccionado();
        aplicar(&mut self.recetario, id, resultado, hoy);
        self.restaurar_seleccion(elegido.as_deref());
        if esta_en_tanda && let Some(t) = &mut self.tanda_activa {
            if let Some(item) = self.recetario.item(id) {
                if item.estado == Estado::PorRevisar {
                    t.por_revisar += 1;
                } else if item.error.is_some() {
                    t.errores += 1;
                    t.ultimo_error = item.error.clone();
                }
            } else {
                t.errores += 1;
                t.ultimo_error = err_inicial;
            }
            if t.miembros.is_empty() {
                tanda_fin = self.tanda_activa.take();
            }
        }
        if let Some(tanda) = tanda_fin {
            self.fin_tanda(tanda, accion);
        } else {
            self.completar(texto, accion);
        }
    }

    pub fn evento_instalacion(&mut self, e: Evento) {
        match e {
            Evento::Inicio(id) => {
                self.en_curso = Some(id);
                self.salida.clear();
            }
            Evento::Linea(_, linea) => {
                self.salida.push(linea);
                if self.salida.len() > MAX_SALIDA {
                    self.salida.remove(0);
                }
            }
            Evento::Fin(id, res) => {
                // Lo comprobado antes de instalar ya no vale: salteada es que la comprobación
                // pasó; del resto no se sabe hasta volver a comprobar.
                if res == Resultado::Salteado {
                    self.registrar_presencia(&id, Presencia::Instalada);
                } else {
                    self.presencia.remove(&id);
                }
                if let Some(fila) = self.instalacion.iter_mut().find(|(x, _)| *x == id) {
                    fila.1 = Some(res);
                }
                self.en_curso = None;
                if self.instalacion.iter().all(|(_, r)| r.is_some()) {
                    self.terminar_instalacion();
                }
            }
        }
    }

    fn terminar_instalacion(&mut self) {
        self.instalando = false;
        let contar = |f: fn(&Resultado) -> bool| {
            self.instalacion
                .iter()
                .filter(|(_, r)| r.as_ref().is_some_and(f))
                .count()
        };
        let ok = contar(|r| matches!(r, Resultado::Ok));
        let salteadas = contar(|r| matches!(r, Resultado::Salteado));
        let error = contar(|r| matches!(r, Resultado::Fallo(_)));
        let bloqueadas = contar(|r| matches!(r, Resultado::Bloqueado(_)));
        let a_mano = self
            .instalacion
            .iter()
            .filter(|(id, r)| {
                matches!(r, Some(Resultado::Ok | Resultado::Salteado))
                    && self
                        .recetario
                        .item(id)
                        .is_some_and(checklist::tiene_pasos_a_mano)
            })
            .count();
        let a_mano = match a_mano {
            0 => String::new(),
            n => format!(" · {n} con pasos a mano: mirá Checklist"),
        };
        self.completar(
            format!(
                "Instalación terminada: {ok} instaladas, {salteadas} salteadas, {error} con error, \
             {bloqueadas} bloqueadas{a_mano} · detalle en la pestaña 2 y en \
             ~/.local/state/rebake/ultima-instalacion.log"
            ),
            self.accion_instalacion,
        );
        if self.accion_instalacion.is_none_or(|a| a == self.accion) {
            self.pestana = Pestana::Checklist;
        }
    }
}

/// `clave[0]`, `clave[1]`… de un mismo perfil son una lista: una fila, con todo en el detalle.
fn agrupar_ajustes(ajustes: &[Ajuste]) -> Vec<Fila> {
    let mut filas: Vec<Fila> = Vec::new();
    let mut lista: Option<(String, String, Vec<String>)> = None;
    let cerrar = |lista: &mut Option<(String, String, Vec<String>)>, filas: &mut Vec<Fila>| {
        if let Some((perfil, base, valores)) = lista.take() {
            filas.push(Fila::nueva(
                format!("[{perfil}] {base} ({} líneas)", valores.len()),
                format!("[{perfil}] {base}:\n{}", valores.join("\n")),
                ClaseFila::Marcable,
            ));
        }
    };
    for a in ajustes {
        let base = a
            .clave
            .strip_suffix(']')
            .and_then(|c| c.rsplit_once('['))
            .filter(|(_, n)| n.chars().all(|c| c.is_ascii_digit()))
            .map(|(b, _)| b.to_string());
        match (&mut lista, base) {
            (Some((p, b, valores)), Some(base)) if *p == a.perfil && *b == base => {
                valores.push(a.valor.clone())
            }
            (_, Some(base)) => {
                cerrar(&mut lista, &mut filas);
                lista = Some((a.perfil.clone(), base, vec![a.valor.clone()]));
            }
            (_, None) => {
                cerrar(&mut lista, &mut filas);
                filas.push(Fila::marcable(format!(
                    "[{}] {} = {}",
                    a.perfil, a.clave, a.valor
                )));
            }
        }
    }
    cerrar(&mut lista, &mut filas);
    filas
}
