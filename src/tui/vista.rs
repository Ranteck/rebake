use super::app::{App, Entrada, Modo, Pestana};
use crate::instalador::Resultado;
use crate::modelo::{Estado, Item, Modo as ModoPaso, Tipo, Via};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

const TECLAS: &str = "s escanear  i investigar  I investigar todo  a aprobar  x excluir  l pegar link  e editar  r reinvestigar  P instalar  / filtrar  q salir";

pub fn dibujar(f: &mut Frame, app: &App) {
    let [cabecera, cuerpo, pie] = Layout::vertical([Constraint::Length(1), Constraint::Min(3), Constraint::Length(2)]).areas(f.area());
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
    format!(
        " recetario · {}   ✓ {} aprobadas  ? {} por revisar  ○ {} pendientes  ✗ {} excluidas",
        perfiles.join(" + "),
        contar(Estado::Aprobada), contar(Estado::PorRevisar), contar(Estado::Pendiente), contar(Estado::Excluida)
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
    match e { Estado::Aprobada => "✓", Estado::PorRevisar => "?", Estado::Pendiente => "○", Estado::Excluida => "✗" }
}

fn nombre_estado(e: Estado) -> &'static str {
    match e { Estado::Aprobada => "aprobada", Estado::PorRevisar => "por revisar", Estado::Pendiente => "pendiente", Estado::Excluida => "excluida" }
}

fn nombre_via(v: Via) -> &'static str {
    match v { Via::Manual => "manual", Via::Metadatos => "metadatos", Via::Symlink => "symlink", Via::Busqueda => "búsqueda" }
}

fn recetas(f: &mut Frame, app: &App, area: Rect) {
    let [izq, der] = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(area);
    let mut lineas: Vec<Line> = Vec::new();
    let mut linea_sel = 0;
    let mut tipo_actual = None;
    for (n, item) in app.visibles().into_iter().enumerate() {
        if tipo_actual != Some(item.tipo) {
            tipo_actual = Some(item.tipo);
            lineas.push(Line::from(Span::styled(grupo(item.tipo), Style::default().add_modifier(Modifier::BOLD))));
        }
        if n == app.seleccion { linea_sel = lineas.len(); }
        lineas.push(linea_item(app, item, n == app.seleccion));
    }
    let alto = izq.height.saturating_sub(2) as usize;
    let desde = linea_sel.saturating_sub(alto.saturating_sub(1));
    f.render_widget(
        Paragraph::new(lineas).scroll((desde as u16, 0)).block(Block::default().borders(Borders::ALL).title("Recetas")),
        izq,
    );
    let (titulo, detalle) = match app.seleccionado() {
        Some(item) => (item.id.clone(), lineas_detalle(app, item)),
        None => ("Detalle".into(), vec![Line::from("sin ítems (¿probaste escanear con s?)")]),
    };
    f.render_widget(
        Paragraph::new(detalle).wrap(Wrap { trim: false }).block(Block::default().borders(Borders::ALL).title(titulo)),
        der,
    );
}

fn linea_item(app: &App, item: &Item, seleccionado: bool) -> Line<'static> {
    let nombre = item.id.split_once(':').map(|(_, n)| n).unwrap_or(&item.id).to_string();
    let marca = if app.investigando.contains(&item.id) { "⟳" } else { simbolo(item.estado) };
    let extra = match (item.estado, &item.motivo, &item.pista, &item.error) {
        (Estado::Excluida, Some(m), _, _) => format!("  {m}"),
        (_, _, _, Some(_)) => "  error".into(),
        (Estado::Pendiente, _, Some(p), _) if p.starts_with("repo local sin remoto") => "  sin remoto".into(),
        (Estado::Pendiente, _, Some(p), _) if p.contains("sin origen") => "  sin origen".into(),
        _ => String::new(),
    };
    let ausente = if item.ausente { "  (ausente)" } else { "" };
    let mut estilo = Style::default();
    if item.ausente { estilo = estilo.add_modifier(Modifier::DIM); }
    if seleccionado { estilo = estilo.add_modifier(Modifier::REVERSED); }
    let flecha = if seleccionado { "▶" } else { " " };
    Line::from(Span::styled(format!("{flecha}{marca} {nombre}{extra}{ausente}"), estilo))
}

fn lineas_detalle(app: &App, item: &Item) -> Vec<Line<'static>> {
    let mut l = vec![Line::from(format!(
        "Estado: {}   Perfiles: {}",
        nombre_estado(item.estado),
        if item.perfiles.is_empty() { "—".to_string() } else { item.perfiles.join(", ") }
    ))];
    match &item.fuente {
        Some(f) => {
            l.push(Line::from(format!("Fuente: {} ({})", f.repo, nombre_via(f.via))));
            if let Some(doc) = &f.doc { l.push(Line::from(format!("Doc: {doc}"))); }
        }
        None => l.push(Line::from("Fuente: —")),
    }
    if !item.requiere.is_empty() { l.push(Line::from(format!("Requiere: {}", item.requiere.join(", ")))); }
    if let Some(v) = &item.verificar { l.push(Line::from(format!("Verificar: {v}"))); }
    for (etiqueta, valor) in [("Pista", &item.pista), ("Motivo", &item.motivo), ("Error", &item.error), ("Investigado", &item.investigado)] {
        if let Some(v) = valor { l.push(Line::from(format!("{etiqueta}: {v}"))); }
    }
    if app.investigando.contains(&item.id) { l.push(Line::from("Investigando…")); }
    if !item.pasos.is_empty() {
        l.push(Line::from(""));
        l.push(Line::from(Span::styled("Pasos", Style::default().add_modifier(Modifier::BOLD))));
        for (n, p) in item.pasos.iter().enumerate() {
            let modo = if p.modo == ModoPaso::Auto { "auto  " } else { "manual" };
            let perfil = if p.por_perfil { "por perfil " } else { "una vez    " };
            l.push(Line::from(format!("{} {modo} {perfil} {}", n + 1, p.cmd)));
            if let Some(c) = &p.cita { l.push(Line::from(format!("   cita: {c}"))); }
            if let Some(nota) = &p.nota { l.push(Line::from(format!("   nota: {nota}"))); }
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
            let marca = if app.hechos.contains(&n) { "[x]" } else { "[ ]" };
            let estilo = if n == app.seleccion_checklist { Style::default().add_modifier(Modifier::REVERSED) } else { Style::default() };
            Line::from(Span::styled(format!("{marca} {texto}"), estilo))
        })
        .collect();
    let alto = area.height.saturating_sub(2) as usize;
    let desde = app.seleccion_checklist.saturating_sub(alto.saturating_sub(1));
    f.render_widget(
        Paragraph::new(lineas).scroll((desde as u16, 0)).block(Block::default().borders(Borders::ALL).title("Checklist (espacio marca)")),
        area,
    );
}

fn instalacion(f: &mut Frame, app: &App, area: Rect) {
    let [izq, der] = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(area);
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
            Line::from(format!("{s} {id}"))
        })
        .collect();
    f.render_widget(Paragraph::new(lineas).block(Block::default().borders(Borders::ALL).title("Instalación")), izq);
    let alto = der.height.saturating_sub(2) as usize;
    let salida: Vec<Line> = app.salida.iter().skip(app.salida.len().saturating_sub(alto)).map(|l| Line::from(l.clone())).collect();
    let titulo = app.en_curso.clone().unwrap_or_else(|| "Salida".into());
    f.render_widget(Paragraph::new(salida).block(Block::default().borders(Borders::ALL).title(titulo)), der);
}

fn lineas_pie(app: &App) -> Vec<Line<'static>> {
    let pestanas = [(Pestana::Recetas, "[1]Recetas"), (Pestana::Checklist, "[2]Checklist"), (Pestana::Instalacion, "[3]Instalación")];
    let mut spans: Vec<Span> = Vec::new();
    for (p, texto) in pestanas {
        let estilo = if p == app.pestana { Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED) } else { Style::default() };
        spans.push(Span::styled(format!(" {texto} "), estilo));
    }
    spans.push(Span::raw(format!("  {TECLAS}")));
    let segunda = match &app.modo {
        Modo::Escribiendo { para, texto } => {
            let etiqueta = match para { Entrada::Link => "Link del repo", Entrada::Motivo => "Motivo para excluir", Entrada::Filtro => "Filtro" };
            format!(" {etiqueta}: {texto}_   (Enter confirma, Esc cancela)")
        }
        _ => format!(" {}", app.mensaje.clone().unwrap_or_default()),
    };
    vec![Line::from(spans), Line::from(segunda)]
}
