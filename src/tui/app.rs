use crate::acciones;
use crate::checklist;
use crate::instalador::{self, Evento, Presencia, Resultado};
use crate::investigador::{Receta, aplicar};
use crate::modelo::{Estado, Item, Recetario};
use crate::servicio::{self, Novedad};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use std::collections::{HashMap, HashSet};

const MAX_SALIDA: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pestana {
    Recetas,
    Checklist,
    Instalacion,
}

#[derive(Debug, Clone, PartialEq)]
/// Motivo y link guardan el ítem al empezar a escribir: una investigación en segundo plano
/// puede reordenar la lista mientras tanto.
pub enum Entrada {
    Link { id: String },
    Motivo { id: String },
    Filtro,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modo {
    Normal,
    Escribiendo { para: Entrada, texto: String },
    ConfirmarInstalacion,
    ConfirmarSalida,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Efecto {
    Nada,
    Guardar,
    Escanear,
    Investigar(Vec<String>),
    Editar(String),
    Instalar,
    Comprobar,
    Actualizar(String),
    Salir,
}

pub struct App {
    pub recetario: Recetario,
    pub pestana: Pestana,
    pub seleccion: usize,
    pub filtro: String,
    pub modo: Modo,
    pub mensaje: Option<String>,
    pub investigando: HashSet<String>,
    pub instalacion: Vec<(String, Option<Resultado>)>,
    pub en_curso: Option<String>,
    pub salida: Vec<String>,
    pub hechos: HashSet<usize>,
    pub seleccion_checklist: usize,
    pub instalando: bool,
    /// Se guardan las desmarcadas: una receta aprobada después aparece marcada sin refrescar.
    pub desmarcadas: HashSet<String>,
    pub presencia: HashMap<String, Presencia>,
    pub novedades: HashMap<String, Novedad>,
    pub seleccion_plan: usize,
    pub comprobando: bool,
    /// Versión publicada más nueva que la propia, si la revisión ya llegó.
    pub version_nueva: Option<String>,
}

impl App {
    pub fn nueva(recetario: Recetario) -> Self {
        App {
            recetario,
            pestana: Pestana::Recetas,
            seleccion: 0,
            filtro: String::new(),
            modo: Modo::Normal,
            mensaje: None,
            investigando: HashSet::new(),
            instalacion: vec![],
            en_curso: None,
            salida: vec![],
            hechos: HashSet::new(),
            seleccion_checklist: 0,
            instalando: false,
            desmarcadas: HashSet::new(),
            presencia: HashMap::new(),
            novedades: HashMap::new(),
            seleccion_plan: 0,
            comprobando: false,
            version_nueva: None,
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

    /// Lo que `P` instala: del plan, lo marcado que no está comprobado como instalado.
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
    }

    pub fn registrar_presencia(&mut self, id: &str, p: Presencia) {
        self.presencia.insert(id.to_string(), p);
    }

    pub fn terminar_comprobacion(&mut self) {
        self.comprobando = false;
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
            .filter(|i| i.id.to_lowercase().contains(&filtro))
            .collect();
        items.sort_by(|a, b| (a.tipo, &a.id).cmp(&(b.tipo, &b.id)));
        items
    }

    pub fn seleccionado(&self) -> Option<&Item> {
        self.visibles().get(self.seleccion).copied()
    }

    fn id_seleccionado(&self) -> Option<String> {
        self.seleccionado().map(|i| i.id.clone())
    }

    pub fn tecla(&mut self, k: KeyEvent) -> Efecto {
        match self.modo.clone() {
            Modo::Escribiendo { para, texto } => self.escribiendo(k.code, para, texto),
            Modo::ConfirmarInstalacion => {
                self.modo = Modo::Normal;
                if k.code == KeyCode::Char('s') {
                    Efecto::Instalar
                } else {
                    self.avisar("instalación cancelada");
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
            KeyCode::Char('P') if self.instalando => self.avisar("ya hay una instalación en curso"),
            KeyCode::Char('u' | 'U') if self.instalando => {
                self.avisar("esperá que termine la instalación para actualizar")
            }
            KeyCode::Char('u' | 'U') => match self.version_nueva.clone() {
                Some(version) => return Efecto::Actualizar(version),
                None => self.avisar("rebake está al día"),
            },
            KeyCode::Char('1') => self.pestana = Pestana::Recetas,
            KeyCode::Char('2') => self.pestana = Pestana::Checklist,
            KeyCode::Char('3') => self.pestana = Pestana::Instalacion,
            KeyCode::Char('s') => return Efecto::Escanear,
            KeyCode::Char('P') => {
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
            _ if self.pestana == Pestana::Recetas => return self.recetas(codigo),
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
                }
            }
            KeyCode::Char('t') => {
                if self.a_instalar().is_empty() {
                    self.desmarcadas.clear();
                } else {
                    self.desmarcadas.extend(plan);
                }
            }
            KeyCode::Char('c') if !self.comprobando => {
                self.comprobando = true;
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
        let total = self.lineas_checklist().len();
        match codigo {
            KeyCode::Down | KeyCode::Char('j') if self.seleccion_checklist + 1 < total => {
                self.seleccion_checklist += 1
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.seleccion_checklist = self.seleccion_checklist.saturating_sub(1)
            }
            KeyCode::Char(' ') => {
                let n = self.seleccion_checklist;
                if self.hechos.contains(&n) {
                    self.hechos.remove(&n);
                } else {
                    self.hechos.insert(n);
                }
            }
            _ => {}
        }
    }

    fn recetas(&mut self, codigo: KeyCode) -> Efecto {
        let total = self.visibles().len();
        let id = self.id_seleccionado();
        match (codigo, id) {
            (KeyCode::Down | KeyCode::Char('j'), _) if self.seleccion + 1 < total => {
                self.seleccion += 1
            }
            (KeyCode::Up | KeyCode::Char('k'), _) => {
                self.seleccion = self.seleccion.saturating_sub(1)
            }
            (KeyCode::Char('/'), _) => {
                self.modo = Modo::Escribiendo {
                    para: Entrada::Filtro,
                    texto: self.filtro.clone(),
                }
            }
            (KeyCode::Char('x'), Some(id)) => {
                self.modo = Modo::Escribiendo {
                    para: Entrada::Motivo { id },
                    texto: String::new(),
                }
            }
            (KeyCode::Char('l'), Some(id)) => {
                self.modo = Modo::Escribiendo {
                    para: Entrada::Link { id },
                    texto: String::new(),
                }
            }
            (KeyCode::Char('e'), Some(id)) => return Efecto::Editar(id),
            (KeyCode::Char('a'), Some(id)) => {
                return match acciones::aprobar(&mut self.recetario, &id) {
                    Ok(()) => {
                        self.avisar(&format!("{id} aprobado"));
                        Efecto::Guardar
                    }
                    Err(e) => {
                        self.avisar(&format!("{e:#}"));
                        Efecto::Nada
                    }
                };
            }
            (KeyCode::Char('i' | 'r'), Some(id)) => {
                if self.recetario.item(&id).is_some_and(|i| i.fuente.is_none()) {
                    self.modo = Modo::Escribiendo {
                        para: Entrada::Link { id },
                        texto: String::new(),
                    };
                } else {
                    return self.investigar(vec![id]);
                }
            }
            (KeyCode::Char('I'), _) => {
                let ids = self
                    .recetario
                    .items
                    .iter()
                    .filter(|i| i.estado == Estado::Pendiente && i.fuente.is_some())
                    .map(|i| i.id.clone())
                    .collect();
                let efecto = self.investigar(ids);
                let sin_origen = servicio::sin_origen(&self.recetario).len();
                if sin_origen > 0 {
                    // Pisa el aviso de investigar: es lo único que el usuario tiene que hacer a mano.
                    self.avisar(&format!("{sin_origen} sin origen: `i` para pasar el link"));
                }
                return efecto;
            }
            _ => {}
        }
        Efecto::Nada
    }

    fn investigar(&mut self, ids: Vec<String>) -> Efecto {
        let ids: Vec<String> = ids
            .into_iter()
            .filter(|id| !self.investigando.contains(id))
            .filter(|id| {
                self.recetario
                    .item(id)
                    .is_some_and(|i| i.estado != Estado::Excluida)
            })
            .collect();
        if ids.is_empty() {
            self.avisar("no hay nada para investigar");
            return Efecto::Nada;
        }
        self.investigando.extend(ids.iter().cloned());
        self.avisar(&format!("investigando {} ítems…", ids.len()));
        Efecto::Investigar(ids)
    }

    fn escribiendo(&mut self, codigo: KeyCode, para: Entrada, mut texto: String) -> Efecto {
        match codigo {
            KeyCode::Esc => {
                self.modo = Modo::Normal;
                if para == Entrada::Filtro {
                    self.filtro.clear();
                    self.seleccion = 0;
                }
            }
            KeyCode::Backspace => {
                texto.pop();
                self.modo = Modo::Escribiendo { para, texto };
            }
            KeyCode::Char(c) => {
                texto.push(c);
                self.modo = Modo::Escribiendo { para, texto };
            }
            KeyCode::Enter => {
                self.modo = Modo::Normal;
                // El filtro se aplica aunque hoy no haya ningún ítem seleccionado.
                if para == Entrada::Filtro {
                    self.filtro = texto;
                    self.seleccion = 0;
                    return Efecto::Nada;
                }
                let (Entrada::Motivo { id } | Entrada::Link { id }) = &para else {
                    return Efecto::Nada;
                };
                let id = id.clone();
                if matches!(para, Entrada::Link { .. }) && texto.trim().is_empty() {
                    return self.investigar(vec![id]);
                }
                let resultado = if matches!(para, Entrada::Motivo { .. }) {
                    acciones::excluir(&mut self.recetario, &id, &texto).map(|_| Efecto::Guardar)
                } else {
                    acciones::pegar_link(&mut self.recetario, &id, &texto)
                        .map(|_| Efecto::Investigar(vec![id.clone()]))
                };
                return match resultado {
                    Ok(Efecto::Investigar(ids)) => self.investigar(ids),
                    Ok(efecto) => {
                        self.avisar(&format!("{id} actualizado"));
                        efecto
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

    pub fn lineas_checklist(&self) -> Vec<String> {
        let r = &self.recetario;
        let mut l: Vec<String> = checklist::logins(r);
        l.extend(
            checklist::pasos_manuales(r)
                .into_iter()
                .map(|(id, p)| format!("{id}: {}", p.cmd)),
        );
        l.extend(
            r.checklist
                .ajustes
                .iter()
                .map(|a| format!("[{}] {} = {}", a.perfil, a.clave, a.valor)),
        );
        l.extend(
            r.checklist
                .sistema
                .iter()
                .map(|p| format!("sudo pacman -S {} (para {})", p.paquete, p.para)),
        );
        l.extend(
            r.checklist
                .titulos
                .iter()
                .map(|t| format!("[{}] CLAUDE.md: {}", t.perfil, t.texto)),
        );
        l
    }

    pub fn registrar_investigacion(
        &mut self,
        id: &str,
        resultado: anyhow::Result<Receta>,
        hoy: &str,
    ) {
        self.investigando.remove(id);
        let texto = match &resultado {
            Ok(_) => format!("{id} → por revisar"),
            Err(e) => format!("{id} → error: {e:#}"),
        };
        let elegido = self.id_seleccionado();
        aplicar(&mut self.recetario, id, resultado, hoy);
        // Un requisito nuevo reordena la lista: la selección sigue al mismo ítem.
        if let Some(pos) = elegido.and_then(|e| self.visibles().iter().position(|i| i.id == e)) {
            self.seleccion = pos;
        }
        self.avisar(&texto);
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
        self.avisar(&format!(
            "Instalación terminada: {ok} instaladas, {salteadas} salteadas, {error} con error, \
             {bloqueadas} bloqueadas · detalle en la pestaña 3 y en \
             ~/.local/state/rebake/ultima-instalacion.log"
        ));
        self.pestana = Pestana::Checklist;
    }
}
