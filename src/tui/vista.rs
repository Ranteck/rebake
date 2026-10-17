use super::app::{App, Entrada, Modo, Pestana};
use crate::instalador::{self, Presencia, Resultado};
use crate::investigador::PISTA_REQUISITO;
use crate::modelo::{Estado, Item, Modo as ModoPaso, Tipo, Via};
use crate::servicio::{self, Novedad};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const TECLAS_TODAS: [(&str, &str); 6] = [
    ("t", "investigar todas"),
    ("s", "escanear"),
    ("/", "filtrar"),
    ("p", "instalar"),
    ("?", "ayuda"),
    ("q", "salir"),
];
const TECLAS_INSTALACION: &str =
    "↑↓ mover  espacio marcar  t todas/ninguna  c comprobar  p instalar  ? ayuda  q salir";
const TECLAS_SIN_PLAN: &str = "1 volver a Recetas  q salir  ? ayuda";
const TECLAS_RESULTADOS: &str = "Esc volver al plan  ? ayuda  q salir";
const TECLAS_PENDIENTES: &str = "↑↓ mover  espacio marcar  ? ayuda  q salir";
const PASOS: [&str; 4] = ["escanear", "armar recetas", "aprobar", "instalar"];
/// Marca, nombre y estado (para el color) de cada entrada de la leyenda, en el orden del camino.
const LEYENDA: [(&str, &str, Estado); 5] = [
    ("✓", "aprobada", Estado::Aprobada),
    ("○", "por revisar", Estado::PorRevisar),
    ("·", "sin receta", Estado::Pendiente),
    ("?", "sin origen", Estado::Pendiente),
    ("✗", "excluida", Estado::Excluida),
];

pub fn dibujar(f: &mut Frame, app: &App) {
    let proposito = lineas_proposito(app, f.area().width);
    let [cabecera, camino, prop, cuerpo, pie] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(proposito.len() as u16),
        Constraint::Min(3),
        Constraint::Length(3),
    ])
    .areas(f.area());
    f.render_widget(
        Paragraph::new(texto_cabecera(app, cabecera.width)),
        cabecera,
    );
    f.render_widget(Paragraph::new(linea_camino(app)), camino);
    f.render_widget(Paragraph::new(proposito), prop);
    match app.pestana {
        Pestana::Recetas => recetas(f, app, cuerpo),
        Pestana::Checklist => checklist(f, app, cuerpo),
        Pestana::Instalacion => instalacion(f, app, cuerpo),
    }
    if app.modo == Modo::Ayuda {
        ayuda(f, app, cuerpo);
    }
    f.render_widget(Paragraph::new(lineas_pie(app)), pie);
}

fn estilo_estado(app: &App, e: Estado) -> Style {
    // Colores ANSI con nombre: la terminal los adapta a su paleta.
    match (app.color, e) {
        (false, _) => Style::default(),
        (true, Estado::Aprobada) => Style::default().fg(Color::Green),
        (true, Estado::PorRevisar) => Style::default().fg(Color::Yellow),
        (true, Estado::Pendiente | Estado::Excluida) => Style::default(),
    }
}

fn leyenda(app: &App) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (marca, nombre, e) in LEYENDA {
        spans.push(Span::styled(marca, estilo_estado(app, e)));
        spans.push(Span::raw(format!(" {nombre}  ")));
    }
    spans.push(Span::raw("… armando  "));
    spans.push(Span::styled(
        "(ausente) no está en esta PC",
        Style::default().add_modifier(Modifier::DIM),
    ));
    spans
}

/// La línea de para qué es la pestaña; en Recetas, con la leyenda al lado o debajo si no entra.
fn lineas_proposito(app: &App, ancho: u16) -> Vec<Line<'static>> {
    let texto = match app.pestana {
        Pestana::Recetas => "En esta PC: decidí cómo se instala cada cosa.",
        Pestana::Instalacion => "En una PC nueva: instalá las recetas aprobadas.",
        Pestana::Checklist => {
            "Después de instalar: lo que rebake no hace por vos (logins y ajustes)."
        }
    };
    let estilo = Style::default().add_modifier(Modifier::ITALIC);
    let proposito = Span::styled(format!(" {texto}"), estilo);
    if app.pestana != Pestana::Recetas {
        return vec![Line::from(proposito)];
    }
    let leyenda = leyenda(app);
    let ancho_leyenda: usize = leyenda.iter().map(|s| s.content.width()).sum();
    if proposito.content.width() + 4 + ancho_leyenda <= ancho as usize {
        let mut spans = vec![proposito, Span::raw("    ")];
        spans.extend(leyenda);
        vec![Line::from(spans)]
    } else {
        let mut spans = vec![Span::raw(" ")];
        spans.extend(leyenda);
        vec![Line::from(proposito), Line::from(spans)]
    }
}

/// El primer paso del camino que todavía tiene trabajo.
fn paso_actual(app: &App) -> usize {
    let items = &app.recetario.items;
    let hay = |f: &dyn Fn(&Item) -> bool| items.iter().any(f);
    if items.is_empty() {
        0
    } else if hay(&|i| armable(app, i)) {
        1
    } else if hay(&|i| i.estado == Estado::PorRevisar) {
        2
    } else {
        3
    }
}

/// Sin receta, con origen y sin investigación en curso: lo que `t` arma.
fn armable(app: &App, i: &Item) -> bool {
    i.estado == Estado::Pendiente && i.fuente.is_some() && !app.investigando.contains(&i.id)
}

fn linea_camino(app: &App) -> Line<'static> {
    let actual = paso_actual(app);
    let mut spans = vec![Span::raw(" Camino: ")];
    for (n, paso) in PASOS.iter().enumerate() {
        if n > 0 {
            spans.push(Span::raw(" › "));
        }
        // Los corchetes marcan el paso aunque no haya negrita ni color.
        spans.push(match n.cmp(&actual) {
            std::cmp::Ordering::Less => Span::raw(format!("✓ {paso}")),
            std::cmp::Ordering::Equal => Span::styled(
                format!("[{paso}]"),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            std::cmp::Ordering::Greater => Span::raw(*paso),
        });
    }
    Line::from(spans)
}

/// Si la cabecera no entra, abrevia los contadores y después el aviso, antes de que la
/// terminal los corte.
fn texto_cabecera(app: &App, ancho: u16) -> String {
    [(false, false), (false, true), (true, true)]
        .into_iter()
        .map(|(aviso_corto, corta)| cabecera(app, aviso_corto, corta))
        .find(|c| c.width() <= ancho as usize)
        .unwrap_or_else(|| cabecera(app, true, true))
}

fn cabecera(app: &App, aviso_corto: bool, corta: bool) -> String {
    let r = &app.recetario;
    let contar = |e: Estado| r.items.iter().filter(|i| i.estado == e).count();
    let perfiles: Vec<&str> = r.perfiles.iter().map(|p| p.nombre.as_str()).collect();
    let sin_origen = servicio::sin_origen(r).len();
    let sin_receta = contar(Estado::Pendiente) - sin_origen;
    let aviso = app
        .version_nueva
        .as_ref()
        .map_or_else(String::new, |nueva| {
            if aviso_corto {
                format!(" ⬆ {nueva}: u actualiza ·")
            } else {
                format!(
                    " ⬆ rebake {nueva} disponible (tenés {}): u actualiza  ·",
                    env!("CARGO_PKG_VERSION")
                )
            }
        });
    if corta {
        return format!(
            "{aviso} rebake · {}  ✓{} ○{} ·{sin_receta}{} ✗{}{}",
            perfiles.join(" + "),
            contar(Estado::Aprobada),
            contar(Estado::PorRevisar),
            match sin_origen {
                0 => String::new(),
                n => format!(" ?{n}"),
            },
            contar(Estado::Excluida),
            match app.investigando.len() {
                0 => String::new(),
                n => format!(" …{n}"),
            } + &match app.buscando.len() {
                0 => String::new(),
                n => format!(" {}{n}", spinner(app)),
            }
        );
    }
    // Adelante: en una terminal angosta se corta el final de la línea, no el aviso.
    format!(
        "{aviso} rebake · {}   ✓ {} aprobadas  ○ {} por revisar  · {sin_receta} sin receta{}  ✗ {} excluidas{}",
        perfiles.join(" + "),
        contar(Estado::Aprobada),
        contar(Estado::PorRevisar),
        // Fijo acá y no en los avisos: el próximo aviso lo pisaría y es lo que queda a mano.
        match sin_origen {
            0 => String::new(),
            n => format!("  ? {n} sin origen"),
        },
        contar(Estado::Excluida),
        match app.investigando.len() {
            0 => String::new(),
            n => format!("  … {n} investigando"),
        } + &match app.buscando.len() {
            0 => String::new(),
            n => format!("  {} {n} buscando origen", spinner(app)),
        }
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
        Tipo::Mcp => "Servidores MCP",
    }
}

/// `?` solo cuando rebake no sabe de qué repo viene; sin receta pero con origen, `t` la arma.
fn marca(i: &Item) -> &'static str {
    match i.estado {
        Estado::Aprobada => "✓",
        Estado::PorRevisar => "○",
        Estado::Pendiente if i.fuente.is_none() => "?",
        Estado::Pendiente => "·",
        Estado::Excluida => "✗",
    }
}

fn nombre_estado(e: Estado) -> &'static str {
    match e {
        Estado::Aprobada => "aprobada",
        Estado::PorRevisar => "por revisar",
        Estado::Pendiente => "sin receta",
        Estado::Excluida => "excluida",
    }
}

fn nombre_via(v: Via) -> &'static str {
    match v {
        Via::Manual => "lo pasaste vos",
        Via::Metadatos => "lo registró su instalador",
        Via::Symlink => "por el enlace simbólico",
        Via::Busqueda => "buscado en la web",
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
        lineas.push(linea_item(
            app,
            item,
            n == app.seleccion,
            izq.width.saturating_sub(2),
        ));
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
        Some(item) => (
            cortar(&item.id, der.width.saturating_sub(2) as usize),
            lineas_detalle(app, item),
        ),
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

/// Corta al ancho en celdas de la terminal, con `…` si no entra.
// shortcut: mide por carácter, no por grafema; un emoji compuesto (ZWJ) mide de más y se corta antes.
fn cortar(texto: &str, ancho: usize) -> String {
    if texto.width() <= ancho {
        return texto.to_string();
    }
    let mut corto = String::new();
    let mut usado = 0;
    for c in texto.chars() {
        let w = c.width().unwrap_or(0);
        if usado + w + 1 > ancho {
            break;
        }
        corto.push(c);
        usado += w;
    }
    if ancho > 0 {
        corto.push('…');
    }
    corto
}

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn spinner(app: &App) -> &'static str {
    SPINNER[app.cuadro % SPINNER.len()]
}

fn linea_item(app: &App, item: &Item, seleccionado: bool, ancho: u16) -> Line<'static> {
    let nombre = item.id.split_once(':').map_or(item.id.as_str(), |(_, n)| n);
    // Casi todos los plugins son del oficial: repetirlo no dice nada y empuja el nombre fuera.
    let nombre = nombre
        .strip_suffix("@claude-plugins-official")
        .unwrap_or(nombre);
    let (marca, estilo_marca) = if app.investigando.contains(&item.id) {
        ("…", Style::default())
    } else if app.buscando.contains(&item.id) {
        (spinner(app), Style::default())
    } else {
        (marca(item), estilo_estado(app, item.estado))
    };
    let extra = match (item.estado, &item.motivo, &item.pista, &item.error) {
        _ if app.buscando.contains(&item.id) => "buscando origen".into(),
        (Estado::Excluida, Some(m), _, _) => m.clone(),
        (_, _, _, Some(_)) => "error".into(),
        (Estado::Pendiente, ..) if app.posible_pendiente(&item.id) => "posible origen".into(),
        (Estado::Pendiente, _, Some(p), _) if p.starts_with("repo local sin remoto") => {
            "sin remoto".into()
        }
        (Estado::Pendiente, _, Some(p), _)
            if p.contains("sin origen") || (item.tipo == Tipo::Mcp && item.fuente.is_none()) =>
        {
            "sin origen".into()
        }
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
    let derecha = format!("{extra}{ausente}").trim_start().to_string();
    // Contra el borde se lee como una columna de estado: el nombre se corta antes que ella.
    let ancho = ancho as usize;
    let fijo = 3;
    let lugar_derecha = if derecha.is_empty() {
        0
    } else {
        derecha.width() + 2
    };
    let nombre = cortar(nombre, ancho.saturating_sub(fijo + lugar_derecha));
    let relleno = if derecha.is_empty() {
        0
    } else {
        ancho
            .saturating_sub(fijo + nombre.width() + derecha.width())
            .max(2)
    };
    let estilo_derecha = if item.error.is_some() && app.color {
        Style::default().fg(Color::Red)
    } else {
        Style::default()
    };
    Line::from(vec![
        Span::raw(flecha),
        Span::styled(marca, estilo_marca),
        Span::raw(format!(" {nombre}{}", " ".repeat(relleno))),
        Span::styled(derecha, estilo_derecha),
    ])
    .style(estilo)
}

fn que_falta(app: &App, item: &Item) -> &'static str {
    if app.investigando.contains(&item.id) {
        return "Armando la receta…";
    }
    if app.buscando.contains(&item.id) {
        return "Buscando en la web de dónde viene…";
    }
    match item.estado {
        Estado::Pendiente if item.fuente.is_none() => match &item.posible {
            Some(p) if p.pendiente().is_some() => {
                "Encontré de dónde podría venir: Enter lo acepta, `n` lo descarta."
            }
            Some(p) if p.descartado => "Descartaste el posible origen: `i` te pide el link.",
            Some(_) => "No encontré de qué repo viene: `i` te pide el link.",
            None => "Esta PC lo tiene, pero no sé de qué repo viene: `i` lo busca en la web.",
        },
        Estado::Pendiente => "Esta PC lo tiene, pero todavía no hay receta: `i` la arma.",
        Estado::PorRevisar => "Hay receta: revisá qué va a ejecutar y `a` la aprueba.",
        Estado::Aprobada => "Lista: en una PC nueva se instala con esto (pestaña 3).",
        Estado::Excluida => "No se instala.",
    }
}

fn lineas_detalle(app: &App, item: &Item) -> Vec<Line<'static>> {
    let mut primera = vec![Span::styled(
        que_falta(app, item),
        Style::default().add_modifier(Modifier::BOLD),
    )];
    if app.buscando.contains(&item.id) {
        primera.insert(0, Span::raw(format!("{} ", spinner(app))));
    }
    let mut l = vec![Line::from(primera)];
    l.push(Line::from(format!(
        "Estado: {}   Perfiles: {}",
        nombre_estado(item.estado),
        if item.perfiles.is_empty() {
            "—".to_string()
        } else {
            item.perfiles.join(", ")
        }
    )));
    let doc = item.fuente.as_ref().and_then(|f| f.doc.as_deref());
    match &item.fuente {
        Some(f) => {
            l.push(Line::from(format!(
                "Viene de: {} ({})",
                f.repo,
                nombre_via(f.via)
            )));
            match doc.map(|d| d.strip_prefix(DIRECTA).ok_or(d)) {
                Some(Ok(instalador)) => l.push(Line::from(format!(
                    "Cómo se arma: {instalador}, sin Claude"
                ))),
                Some(Err(d)) => l.push(Line::from(format!("Documentación: {d}"))),
                None => {}
            }
        }
        None => l.push(Line::from("Viene de: — (no se sabe de qué repo)")),
    }
    if item.fuente.is_none()
        && let Some(p) = &item.posible
    {
        if let Some(repo) = &p.repo {
            let estado = if p.descartado { " (descartado)" } else { "" };
            l.push(Line::from(Span::styled(
                format!("Posible origen: {repo}{estado}"),
                Style::default().add_modifier(Modifier::BOLD),
            )));
        }
        if !p.motivo.is_empty() {
            l.push(Line::from(format!("Por qué: {}", p.motivo)));
        }
        if p.pendiente().is_some() {
            l.push(Line::from(teclas(&[
                ("Enter", "aceptar"),
                ("n", "descartar"),
                ("i", "pegar otro link"),
            ])));
        }
    }
    if !item.requiere.is_empty() {
        l.push(Line::from(format!(
            "Necesita antes: {}",
            item.requiere.join(", ")
        )));
    }
    if let Some(v) = &item.verificar {
        l.push(Line::from(format!("Para saber si ya está: {v}")));
    }
    // La pista dice por qué está en el cookbook (un requisito) o por qué falta el origen;
    // lo segundo deja de importar cuando ya hay receta.
    match item.pista.as_deref() {
        Some(p) if p.starts_with(PISTA_REQUISITO) => l.push(Line::from(format!(
            "Lo pide: {}",
            &p[PISTA_REQUISITO.len()..]
        ))),
        Some(p) if item.estado == Estado::Pendiente => {
            l.push(Line::from(format!("Por qué falta: {p}")))
        }
        _ => {}
    }
    for (etiqueta, valor) in [
        ("Motivo", &item.motivo),
        ("Error", &item.error),
        ("Receta armada el", &item.investigado),
    ] {
        if let Some(v) = valor {
            l.push(Line::from(format!("{etiqueta}: {v}")));
        }
    }
    if !item.pasos.is_empty() {
        l.push(Line::from(""));
        l.push(Line::from(Span::styled(
            "Qué va a ejecutar",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for (n, p) in item.pasos.iter().enumerate() {
            let perfil = if p.por_perfil {
                "en cada perfil"
            } else {
                "una sola vez"
            };
            let modo = if p.modo == ModoPaso::Auto {
                "automático"
            } else {
                "lo hacés vos (va a Pendientes)"
            };
            l.push(Line::from(format!("{}. {}", n + 1, p.cmd)));
            l.push(Line::from(format!("   {perfil}, {modo}")));
            if let Some(c) = &p.cita {
                l.push(Line::from(format!("   según el repo: {c}")));
            }
            // La nota de una receta directa repite "Cómo se arma".
            if let Some(nota) = p.nota.as_deref().filter(|n| Some(*n) != doc) {
                l.push(Line::from(format!("   {nota}")));
            }
        }
    }
    l
}

const DIRECTA: &str = "receta directa: ";

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
                .title("Pendientes (espacio marca)"),
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

fn teclas_de_receta(item: &Item) -> Vec<(&'static str, &'static str)> {
    match item.estado {
        Estado::Pendiente if item.fuente.is_none() => match &item.posible {
            Some(p) if p.pendiente().is_some() => vec![
                ("Enter", "aceptar origen"),
                ("n", "descartar"),
                ("i", "pegar otro link"),
                ("x", "excluir"),
            ],
            Some(_) => vec![("i", "pegar el link"), ("x", "excluir")],
            None => vec![("i", "buscar origen"), ("x", "excluir")],
        },
        Estado::Pendiente => vec![("i", "armar receta"), ("x", "excluir")],
        Estado::PorRevisar => vec![
            ("espacio/a", "aprobar"),
            ("e", "editar"),
            ("r", "reinvestigar"),
            ("x", "excluir"),
        ],
        Estado::Aprobada => vec![
            ("espacio/a", "desaprobar"),
            ("e", "editar"),
            ("r", "reinvestigar"),
            ("x", "excluir"),
        ],
        // Excluida no se investiga: se vuelve a incluir o se edita.
        Estado::Excluida => vec![("x", "volver a incluir"), ("e", "editar")],
    }
}

/// La tecla en video inverso se lee como un botón, separada de lo que hace.
fn teclas(lista: &[(&'static str, &'static str)]) -> Vec<Span<'static>> {
    let resaltada = Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD);
    let mut spans = Vec::new();
    for (tecla, accion) in lista {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(format!(" {tecla} "), resaltada));
        spans.push(Span::raw(format!(" {accion} ")));
    }
    spans
}

/// La próxima tecla útil según el paso del camino.
fn siguiente_paso(app: &App) -> String {
    let r = &app.recetario;
    let contar = |f: &dyn Fn(&Item) -> bool| r.items.iter().filter(|i| f(i)).count();
    match paso_actual(app) {
        0 => "Siguiente: `s` escanea esta PC".into(),
        1 => format!(
            "Siguiente: `t` arma {} recetas (las que tienen origen)",
            contar(&|i| armable(app, i))
        ),
        2 => format!(
            "Siguiente: {} recetas por revisar: elegí una y `a` la aprueba",
            contar(&|i| i.estado == Estado::PorRevisar)
        ),
        _ => match (
            contar(&|i| i.estado == Estado::Aprobada),
            servicio::sin_origen(r).len(),
        ) {
            (0, 0) => String::new(),
            (0, n) => format!("Siguiente: {n} sin origen: elegí una y `i` te pide el link"),
            (n, _) => format!("Siguiente: en una PC nueva, `3` y `p` instalan las {n} aprobadas"),
        },
    }
}

fn ayuda(f: &mut Frame, app: &App, area: Rect) {
    let titulo = |t: &'static str| {
        Line::from(Span::styled(
            t,
            Style::default().add_modifier(Modifier::BOLD),
        ))
    };
    let lineas = vec![
        Line::from(
            "rebake guarda en un cookbook cómo se instala cada cosa de tu Claude Code, para rehacerlo en una PC nueva.",
        ),
        Line::from(""),
        titulo("Camino"),
        Line::from("1. escanear (s): busca lo que tiene esta PC."),
        Line::from("2. armar recetas (i, t): de qué repo viene cada cosa y cómo se instala."),
        Line::from(
            "3. aprobar (espacio o a): aprueba y desaprueba, y queda guardado en el cookbook; x excluye y vuelve a incluir, e edita.",
        ),
        Line::from(
            "4. instalar (3, p): en una PC nueva instala lo aprobado; después, 2 Pendientes.",
        ),
        Line::from(""),
        titulo("Marcas"),
        Line::from(leyenda(app)),
        Line::from(""),
        titulo("Pestañas"),
        Line::from("1 Recetas: en esta PC, armar y aprobar recetas."),
        Line::from("2 Pendientes: logins y ajustes que rebake no hace por vos."),
        Line::from(
            "3 Instalación: en una PC nueva, comprobar (c), marcar (espacio, solo para esa instalación) e instalar (p).",
        ),
        Line::from(""),
        titulo("Otras teclas"),
        Line::from(
            "↑↓ o j k mover  / filtrar  l pegar el link  r reinvestigar  u actualizar rebake  q salir",
        ),
    ];
    f.render_widget(Clear, area);
    f.render_widget(
        Paragraph::new(lineas).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Ayuda (? o Esc cierra)"),
        ),
        area,
    );
}

fn lineas_pie(app: &App) -> Vec<Line<'static>> {
    let pestanas = [
        (Pestana::Recetas, "[1]Recetas"),
        (Pestana::Checklist, "[2]Pendientes"),
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
    let mut todas: Vec<Span> = Vec::new();
    if app.pestana == Pestana::Recetas {
        // Solo lo que hace algo sobre la receta elegida, en el orden en que conviene hacerlo.
        if let Some(item) = app.seleccionado()
            && !app.investigando.contains(&item.id)
            && !app.buscando.contains(&item.id)
        {
            spans.push(Span::raw("  Esta receta:"));
            spans.extend(teclas(&teclas_de_receta(item)));
        }
        todas.push(Span::raw(" Todas:      "));
        todas.extend(teclas(&TECLAS_TODAS));
    } else {
        let texto = if app.viendo_plan() && app.plan().is_empty() {
            TECLAS_SIN_PLAN
        } else if app.viendo_plan() {
            TECLAS_INSTALACION
        } else if app.pestana == Pestana::Instalacion && !app.instalando {
            TECLAS_RESULTADOS
        } else if app.pestana == Pestana::Checklist {
            TECLAS_PENDIENTES
        } else {
            ""
        };
        spans.push(Span::raw(format!("  {texto}")));
    }
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
        _ => format!(
            " {}",
            app.mensaje.clone().unwrap_or_else(|| siguiente_paso(app))
        ),
    };
    vec![Line::from(spans), Line::from(todas), Line::from(segunda)]
}
