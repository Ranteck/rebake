use super::app::{App, Entrada, Modo, Pestana};
use crate::instalador::{self, Presencia, Resultado};
use crate::modelo::{Estado, Item, Modo as ModoPaso, Tipo, Via};
use crate::servicio::{self, Novedad};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

const TECLAS: &str = "s escanear  i investigar  I investigar todo  a aprobar  x excluir  l pegar link  e editar  r reinvestigar  P instalar  / filtrar  q salir";
const TECLAS_INSTALACION: &str =
    "↑↓ mover  espacio marcar  t todas/ninguna  c comprobar  P instalar  q salir";
const TECLAS_RESULTADOS: &str = "Esc volver al plan  q salir";

pub fn dibujar(f: &mut Frame, app: &App) {
    let [cabecera, cuerpo, pie] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(2),
    ])
    .areas(f.area());
    f.render_widget(Paragraph::new(texto_cabecera(app)), cabecera);
    match app.pestana {
        Pestana::Recetas => recetas(f, app, cuerpo),
        Pestana::Checklist => checklist(f, app, cuerpo),
        Pestana::Instalacion => instalacion(f, app, cuerpo),
    }
    f.render_widget(Paragraph::new(lineas_pie(app)), pie);
}

fn texto_cabecera(app: &App) -> String {
    let r = &app.recetario;
    let contar = |e: Estado| r.items.iter().filter(|i| i.estado == e).count();
    let perfiles: Vec<&str> = r.perfiles.iter().map(|p| p.nombre.as_str()).collect();
    // Fijo acá y no en los avisos: el próximo aviso lo pisaría y es lo que queda a mano.
    let sin_origen = match servicio::sin_origen(r).len() {
        0 => String::new(),
        n => format!(" ({n} sin origen)"),
    };
    let aviso = app
        .version_nueva
        .as_ref()
        .map_or_else(String::new, |nueva| {
            format!(
                " ⬆ rebake {nueva} disponible (tenés {}): U actualiza  ·",
                env!("CARGO_PKG_VERSION")
            )
        });
    // Adelante: en una terminal angosta se corta el final de la línea, no el aviso.
    format!(
        "{aviso} rebake · {}   ✓ {} aprobadas  ? {} por revisar  ○ {} pendientes{}  ✗ {} excluidas",
        perfiles.join(" + "),
        contar(Estado::Aprobada),
        contar(Estado::PorRevisar),
        contar(Estado::Pendiente),
        sin_origen,
        contar(Estado::Excluida)
    )
}

fn grupo(t: Tipo) -> &'static str {
    match t {
        Tipo::Claude => "Claude Code",
        Tipo::Marketplace => "Marketplaces",
        Tipo::Plugin => "Plugins",
        Tipo::Skill => "Skills",
        Tipo::Import => "Imports CLAUDE.md",
        Tipo::Hook => "Hooks",
        Tipo::Statusline => "Statusline",
        Tipo::Herramienta => "Herramientas",
    }
}

fn simbolo(e: Estado) -> &'static str {
    match e {
        Estado::Aprobada => "✓",
        Estado::PorRevisar => "?",
        Estado::Pendiente => "○",
        Estado::Excluida => "✗",
    }
}

fn nombre_estado(e: Estado) -> &'static str {
    match e {
        Estado::Aprobada => "aprobada",
        Estado::PorRevisar => "por revisar",
        Estado::Pendiente => "pendiente",
        Estado::Excluida => "excluida",
    }
}

fn nombre_via(v: Via) -> &'static str {
    match v {
        Via::Manual => "manual",
        Via::Metadatos => "metadatos",
        Via::Symlink => "symlink",
        Via::Busqueda => "origen buscado en la web",
    }
}

fn recetas(f: &mut Frame, app: &App, area: Rect) {
    let [izq, der] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(area);
    let mut lineas: Vec<Line> = Vec::new();
    let mut linea_sel = 0;
    let mut tipo_actual = None;
    for (n, item) in app.visibles().into_iter().enumerate() {
        if tipo_actual != Some(item.tipo) {
            tipo_actual = Some(item.tipo);
            lineas.push(Line::from(Span::styled(
                grupo(item.tipo),
                Style::default().add_modifier(Modifier::BOLD),
            )));
        }
        if n == app.seleccion {
            linea_sel = lineas.len();
        }
        lineas.push(linea_item(app, item, n == app.seleccion));
    }
    let alto = izq.height.saturating_sub(2) as usize;
    let desde = linea_sel.saturating_sub(alto.saturating_sub(1));
    f.render_widget(
        Paragraph::new(lineas)
            .scroll((desde as u16, 0))
            .block(Block::default().borders(Borders::ALL).title("Recetas")),
        izq,
    );
    let (titulo, detalle) = match app.seleccionado() {
        Some(item) => (item.id.clone(), lineas_detalle(app, item)),
        None => (
            "Detalle".into(),
            vec![Line::from("sin ítems (¿probaste escanear con s?)")],
        ),
    };
    f.render_widget(
        Paragraph::new(detalle)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title(titulo)),
        der,
    );
}

fn linea_item(app: &App, item: &Item, seleccionado: bool) -> Line<'static> {
    let nombre = item
        .id
        .split_once(':')
        .map(|(_, n)| n)
        .unwrap_or(&item.id)
        .to_string();
    let marca = if app.investigando.contains(&item.id) {
        "⟳"
    } else {
        simbolo(item.estado)
    };
    let extra = match (item.estado, &item.motivo, &item.pista, &item.error) {
        (Estado::Excluida, Some(m), _, _) => format!("  {m}"),
        (_, _, _, Some(_)) => "  error".into(),
        (Estado::Pendiente, _, Some(p), _) if p.starts_with("repo local sin remoto") => {
            "  sin remoto".into()
        }
        (Estado::Pendiente, _, Some(p), _) if p.contains("sin origen") => "  sin origen".into(),
        _ => String::new(),
    };
    let ausente = if item.ausente { "  (ausente)" } else { "" };
    let mut estilo = Style::default();
    if item.ausente {
        estilo = estilo.add_modifier(Modifier::DIM);
    }
    if seleccionado {
        estilo = estilo.add_modifier(Modifier::REVERSED);
    }
    let flecha = if seleccionado { "▶" } else { " " };
    Line::from(Span::styled(
        format!("{flecha}{marca} {nombre}{extra}{ausente}"),
        estilo,
    ))
}

fn lineas_detalle(app: &App, item: &Item) -> Vec<Line<'static>> {
    let mut l = vec![Line::from(format!(
        "Estado: {}   Perfiles: {}",
        nombre_estado(item.estado),
        if item.perfiles.is_empty() {
            "—".to_string()
        } else {
            item.perfiles.join(", ")
        }
    ))];
    match &item.fuente {
        Some(f) => {
            l.push(Line::from(format!(
                "Fuente: {} ({})",
                f.repo,
                nombre_via(f.via)
            )));
            if let Some(doc) = &f.doc {
                l.push(Line::from(format!("Doc: {doc}")));
            }
        }
        None => l.push(Line::from("Fuente: —")),
    }
    if !item.requiere.is_empty() {
        l.push(Line::from(format!(
            "Requiere: {}",
            item.requiere.join(", ")
        )));
    }
    if let Some(v) = &item.verificar {
        l.push(Line::from(format!("Verificar: {v}")));
    }
    for (etiqueta, valor) in [
        ("Pista", &item.pista),
        ("Motivo", &item.motivo),
        ("Error", &item.error),
        ("Investigado", &item.investigado),
    ] {
        if let Some(v) = valor {
            l.push(Line::from(format!("{etiqueta}: {v}")));
        }
    }
    if app.investigando.contains(&item.id) {
        l.push(Line::from("Investigando…"));
    }
    if !item.pasos.is_empty() {
        l.push(Line::from(""));
        l.push(Line::from(Span::styled(
            "Pasos",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for (n, p) in item.pasos.iter().enumerate() {
            let modo = if p.modo == ModoPaso::Auto {
                "auto  "
            } else {
                "manual"
            };
            let perfil = if p.por_perfil {
                "por perfil "
            } else {
                "una vez    "
            };
            l.push(Line::from(format!("{} {modo} {perfil} {}", n + 1, p.cmd)));
            if let Some(c) = &p.cita {
                l.push(Line::from(format!("   cita: {c}")));
            }
            if let Some(nota) = &p.nota {
                l.push(Line::from(format!("   nota: {nota}")));
            }
        }
    }
    l
}

fn checklist(f: &mut Frame, app: &App, area: Rect) {
    let lineas: Vec<Line> = app
        .lineas_checklist()
        .into_iter()
        .enumerate()
        .map(|(n, texto)| {
            let marca = if app.hechos.contains(&n) {
                "[x]"
            } else {
                "[ ]"
            };
            let estilo = if n == app.seleccion_checklist {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            Line::from(Span::styled(format!("{marca} {texto}"), estilo))
        })
        .collect();
    let alto = area.height.saturating_sub(2) as usize;
    let desde = app
        .seleccion_checklist
        .saturating_sub(alto.saturating_sub(1));
    f.render_widget(
        Paragraph::new(lineas).scroll((desde as u16, 0)).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Checklist (espacio marca)"),
        ),
        area,
    );
}

fn texto_presencia(app: &App, id: &str) -> String {
    match app.presencia.get(id) {
        None if app.comprobando => "comprobando…".into(),
        None => "sin comprobar".into(),
        Some(Presencia::Instalada) => "instalada".into(),
        Some(Presencia::Falta(faltan)) => {
            let todos = app.recetario.item(id).map_or(&[][..], |i| &i.perfiles[..]);
            if faltan.is_empty() || faltan.len() >= todos.len() {
                "falta".into()
            } else {
                format!("falta en {}", faltan.join(", "))
            }
        }
        Some(Presencia::SinComprobacion) => "no se puede saber".into(),
        Some(Presencia::NoSePudo(_)) => "no se pudo comprobar".into(),
    }
}

fn plan_instalacion(f: &mut Frame, app: &App, area: Rect) {
    let [izq, der] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(area);
    let plan = match instalador::planificar(&app.recetario) {
        Ok(plan) if plan.is_empty() => {
            let aviso = "No hay recetas aprobadas: investigá (i) y aprobá (a) en la pestaña 1.";
            f.render_widget(
                Paragraph::new(aviso)
                    .block(Block::default().borders(Borders::ALL).title("Instalación")),
                area,
            );
            return;
        }
        Ok(plan) => plan,
        Err(e) => {
            f.render_widget(
                Paragraph::new(format!("{e:#}"))
                    .wrap(Wrap { trim: false })
                    .block(Block::default().borders(Borders::ALL).title("Instalación")),
                area,
            );
            return;
        }
    };
    let lineas: Vec<Line> = plan
        .iter()
        .enumerate()
        .map(|(n, id)| {
            let casilla = if app.instalada(id) {
                " ✓ "
            } else if app.marcada(id) {
                "[x]"
            } else {
                "[ ]"
            };
            let novedad = match app.novedades.get(id) {
                Some(Novedad::Nueva) => "  nueva",
                Some(Novedad::Cambio) => "  cambió",
                None => "",
            };
            let estilo = if n == app.seleccion_plan {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            Line::from(Span::styled(
                format!("{casilla} {id}  {}{novedad}", texto_presencia(app, id)),
                estilo,
            ))
        })
        .collect();
    let alto = izq.height.saturating_sub(2) as usize;
    let desde = app.seleccion_plan.saturating_sub(alto.saturating_sub(1));
    f.render_widget(
        Paragraph::new(lineas)
            .scroll((desde as u16, 0))
            .block(Block::default().borders(Borders::ALL).title("Instalación")),
        izq,
    );
    let (titulo, detalle) = match plan
        .get(app.seleccion_plan)
        .and_then(|id| app.recetario.item(id))
    {
        Some(item) => (item.id.clone(), lineas_detalle(app, item)),
        None => ("Detalle".into(), vec![]),
    };
    f.render_widget(
        Paragraph::new(detalle)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title(titulo)),
        der,
    );
}

fn instalacion(f: &mut Frame, app: &App, area: Rect) {
    if app.viendo_plan() {
        return plan_instalacion(f, app, area);
    }
    let [izq, der] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(area);
    let lineas: Vec<Line> = app
        .instalacion
        .iter()
        .map(|(id, res)| {
            let s = match res {
                None if app.en_curso.as_deref() == Some(id) => "⏳",
                None => " ",
                Some(Resultado::Ok) => "✓",
                Some(Resultado::Salteado) => "⤼",
                Some(Resultado::Simulado) => "~",
                Some(Resultado::Fallo(_)) => "✗",
                Some(Resultado::Bloqueado(_)) => "⊘",
            };
            let motivo = match res {
                Some(Resultado::Fallo(m) | Resultado::Bloqueado(m)) => format!(" — {m}"),
                _ => String::new(),
            };
            Line::from(format!("{s} {id}{motivo}"))
        })
        .collect();
    f.render_widget(
        Paragraph::new(lineas)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("Instalación")),
        izq,
    );
    let alto = der.height.saturating_sub(2) as usize;
    let salida: Vec<Line> = app
        .salida
        .iter()
        .skip(app.salida.len().saturating_sub(alto))
        .map(|l| Line::from(l.clone()))
        .collect();
    let titulo = app.en_curso.clone().unwrap_or_else(|| "Salida".into());
    f.render_widget(
        Paragraph::new(salida).block(Block::default().borders(Borders::ALL).title(titulo)),
        der,
    );
}

fn lineas_pie(app: &App) -> Vec<Line<'static>> {
    let pestanas = [
        (Pestana::Recetas, "[1]Recetas"),
        (Pestana::Checklist, "[2]Checklist"),
        (Pestana::Instalacion, "[3]Instalación"),
    ];
    let mut spans: Vec<Span> = Vec::new();
    for (p, texto) in pestanas {
        let estilo = if p == app.pestana {
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default()
        };
        spans.push(Span::styled(format!(" {texto} "), estilo));
    }
    let teclas = if app.viendo_plan() {
        TECLAS_INSTALACION
    } else if app.pestana == Pestana::Instalacion && !app.instalando {
        TECLAS_RESULTADOS
    } else {
        TECLAS
    };
    spans.push(Span::raw(format!("  {teclas}")));
    let segunda = match &app.modo {
        Modo::Escribiendo { para, texto } => {
            let etiqueta = match para {
                Entrada::Link { .. } => "Link del repo",
                Entrada::Motivo { .. } => "Motivo para excluir",
                Entrada::Filtro => "Filtro",
            };
            let ayuda = match para {
                Entrada::Link { .. } => "Enter vacío lo busca en la web, Esc cancela",
                _ => "Enter confirma, Esc cancela",
            };
            format!(" {etiqueta}: {texto}_   ({ayuda})")
        }
        _ => format!(" {}", app.mensaje.clone().unwrap_or_default()),
    };
    vec![Line::from(spans), Line::from(segunda)]
}
