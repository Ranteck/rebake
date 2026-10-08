use crate::acciones;
use crate::checklist;
use crate::instalador::{Evento, Resultado};
use crate::investigador::{Receta, aplicar};
use crate::modelo::{Estado, Item, Recetario};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashSet;

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
        }
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
            KeyCode::Char('1') => self.pestana = Pestana::Recetas,
            KeyCode::Char('2') => self.pestana = Pestana::Checklist,
            KeyCode::Char('3') => self.pestana = Pestana::Instalacion,
            KeyCode::Char('s') => return Efecto::Escanear,
            KeyCode::Char('P') => {
                let n = self
                    .recetario
                    .items
                    .iter()
                    .filter(|i| i.estado == Estado::Aprobada)
                    .count();
                self.mensaje = Some(format!("¿Instalar {n} recetas aprobadas? s/n"));
                self.modo = Modo::ConfirmarInstalacion;
            }
            _ if self.pestana == Pestana::Checklist => self.checklist(codigo),
            _ if self.pestana == Pestana::Recetas => return self.recetas(codigo),
            _ => {}
        }
        Efecto::Nada
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
            (KeyCode::Char('i' | 'r'), Some(id)) => return self.investigar(vec![id]),
            (KeyCode::Char('I'), _) => {
                let ids = self
                    .recetario
                    .items
                    .iter()
                    .filter(|i| i.estado == Estado::Pendiente)
                    .map(|i| i.id.clone())
                    .collect();
                return self.investigar(ids);
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
             ~/.local/state/recetario/ultima-instalacion.log"
        ));
        self.pestana = Pestana::Checklist;
    }
}
