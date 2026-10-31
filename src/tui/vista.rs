use super::app::{App, ClaseFila, Entrada, Modo, Pestana};
use crate::instalador::{self, Presencia, Resultado};
use crate::investigador::{PISTA_REQUISITO, receta_directa};
use crate::modelo::{Estado, Item, Modo as ModoPaso, Tipo, Via};
use crate::servicio::{self, Novedad};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const TECLAS_INSTALACION: &str =
    "↑↓ mover  espacio marcar  t todas/ninguna  c comprobar  p instalar  ? ayuda  q salir";
const TECLAS_SIN_PLAN: &str = "1 volver a Armar  q salir  ? ayuda";
const TECLAS_RESULTADOS: &str = "Esc volver al plan  ? ayuda  q salir";
const TECLAS_CHECKLIST: &str = "espacio tilda  Enter ajustes  ? ayuda";
const PASOS: [&str; 4] = ["escanear", "armar recetas", "aprobar", "instalar"];
/// Marca, nombre y estado (para el color) de cada entrada de la leyenda, en el orden del camino.
const LEYENDA: [(&str, &str, Estado); 5] = [
    ("✓", "aprobada", Estado::Aprobada),
    ("□", "por revisar", Estado::PorRevisar),
    ("·", "sin receta", Estado::Pendiente),
    ("?", "sin origen", Estado::Pendiente),
    ("✗", "excluida", Estado::Excluida),
];

pub fn dibujar(f: &mut Frame, app: &App) {
    // Lo que no cambia de una sesión a otra (camino, leyenda, para qué es cada pestaña) vive en `?`.
    let [cabecera, cuerpo, pie] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(2),
    ])
    .areas(f.area());
    f.render_widget(
        Paragraph::new(texto_cabecera(app, cabecera.width)),
        cabecera,
    );
    match app.pestana {
        Pestana::Armar => recetas(f, app, cuerpo),
        Pestana::Instalacion => instalacion(f, app, cuerpo),
        Pestana::Checklist => checklist(f, app, cuerpo),
    }
    if app.modo == Modo::Ayuda {
        ayuda(f, app, cuerpo);
    }
    f.render_widget(Paragraph::new(lineas_pie(app, pie.width)), pie);
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

/// El primer paso del camino que todavía tiene trabajo.
fn paso_actual(app: &App) -> usize {
    let items = &app.recetario.items;
    let hay = |f: &dyn Fn(&Item) -> bool| {
        items
            .iter()
            .filter(|i| !i.es_sincronizado() && i.instalado_por.is_none())
            .any(f)
    };
    let (gratis, investigar, buscar) = app.por_armar();
    if items.is_empty() {
        0
    } else if !gratis.is_empty() || !investigar.is_empty() || !buscar.is_empty() {
        1
    } else if hay(&|i| i.estado == Estado::PorRevisar) {
        2
    } else {
        3
    }
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
    let contar = |e: Estado| {
        r.items
            .iter()
            .filter(|i| {
                !i.es_sincronizado()
                    && i.estado == e
                    && (e == Estado::Excluida || i.instalado_por.is_none())
            })
            .count()
    };
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
    // Los contadores en cero no dicen nada; los que quedan hacen de leyenda de las marcas.
    let contadores: Vec<String> = [
        ("✓", contar(Estado::Aprobada), "aprobadas"),
        ("□", contar(Estado::PorRevisar), "por revisar"),
        ("·", sin_receta, "sin receta"),
        ("?", sin_origen, "sin origen"),
        ("✗", contar(Estado::Excluida), "excluidas (o)"),
        ("…", app.investigando.len(), "investigando"),
        (spinner(app), app.buscando.len(), "buscando origen"),
    ]
    .into_iter()
    .filter(|(_, n, _)| *n > 0)
    .map(|(marca, n, nombre)| {
        if corta {
            if marca == "✗" {
                return format!("{marca}{n}(o)");
            }
            format!("{marca}{n}")
        } else {
            format!("{marca} {n} {nombre}")
        }
    })
    .collect();
    let separador = if corta { " " } else { "  " };
    // Adelante: en una terminal angosta se corta el final de la línea, no el aviso.
    format!(
        "{aviso} rebake · {}   {}",
        perfiles.join(" + "),
        contadores.join(separador)
    )
    .trim_end()
    .to_string()
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
    if i.instalado_por.is_some() {
        return "·";
    }
    match i.estado {
        Estado::Aprobada => "✓",
        Estado::PorRevisar => "□",
        Estado::Pendiente if i.fuente.is_none() => "?",
        Estado::Pendiente => "·",
        Estado::Excluida => "✗",
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
    let mut seccion_actual = None;
    let mut tipo_actual = None;
    for (n, item) in app.visibles().into_iter().enumerate() {
        let sec = item.seccion();
        if seccion_actual != Some(sec) {
            seccion_actual = Some(sec);
            tipo_actual = None;
            let titulo = match sec {
                crate::modelo::SeccionRecetas::PorResolver => "POR RESOLVER",
                crate::modelo::SeccionRecetas::Listas => "LISTAS",
                crate::modelo::SeccionRecetas::Excluidas => "EXCLUIDAS",
            };
            lineas.push(Line::from(Span::styled(
                titulo,
                Style::default().add_modifier(Modifier::BOLD),
            )));
        }
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
            .block(Block::default().borders(Borders::ALL).title("Armar")),
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
    let extra = item
        .instalado_por
        .as_ref()
        .map_or(extra, |r| format!("lo instala {}", r.nombre()));
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
            None => "Esta PC lo tiene, pero no sé de qué repo viene: Enter abre Resolver.",
        },
        Estado::Pendiente => "Esta PC lo tiene, pero todavía no hay receta: `i` la arma.",
        Estado::PorRevisar => "Hay receta: revisá qué va a ejecutar; Enter aprueba y avanza.",
        Estado::Aprobada => "Lista: en una PC nueva se instala con esto (pestaña 2).",
        Estado::Excluida => "No se instala.",
    }
}

fn lineas_detalle(app: &App, item: &Item) -> Vec<Line<'static>> {
    if let Some(r) = &item.instalado_por {
        return vec![
            Line::from(format!("lo instala {}", r.nombre())),
            Line::from(format!(
                "Perfiles: {}",
                item.perfiles_propios()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            Line::from("Comprobar revisa si el hook está en la configuración de cada perfil."),
            Line::from("No necesita receta ni aprobación independiente."),
        ];
    }
    let mut primera = vec![Span::styled(
        que_falta(app, item),
        Style::default().add_modifier(Modifier::BOLD),
    )];
    if app.buscando.contains(&item.id) {
        primera.insert(0, Span::raw(format!("{} ", spinner(app))));
    }
    let mut l = vec![Line::from(primera)];
    // La primera línea ya dice el estado; los perfiles importan solo si no son todos.
    let propios = item.perfiles_propios().count();
    if propios > 0 && propios < app.recetario.perfiles.len() {
        l.push(Line::from(format!(
            "Solo en: {}",
            item.perfiles_propios()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    let hooks_vinculados: Vec<&Item> = app.recetario.hooks_de(&item.id).collect();
    if !hooks_vinculados.is_empty() {
        let nombres: Vec<&str> = hooks_vinculados
            .iter()
            .map(|h| h.id.strip_prefix("hook:").unwrap_or(&h.id))
            .collect();
        l.push(Line::from(Span::styled(
            format!("Hooks vinculados: {}", nombres.join(", ")),
            Style::default().add_modifier(Modifier::BOLD),
        )));
    }
    let doc = item.fuente.as_ref().and_then(|f| f.doc.as_deref());
    match &item.fuente {
        Some(f) => {
            l.push(Line::from(format!(
                "Viene de: {} ({})",
                f.repo,
                nombre_via(f.via)
            )));
            match doc.map(|d| d.strip_prefix(DIRECTA).ok_or(d)) {
                Some(Ok(instalador)) => {
                    l.push(Line::from(format!("          {instalador}, sin Claude")))
                }
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
            l.push(Line::from(if app.posible_gratis(item) {
                "Aceptar: gratis, sin Claude"
            } else {
                "Aceptar: 1 corrida de Claude"
            }));
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
    for (etiqueta, valor) in [("Motivo", &item.motivo), ("Error", &item.error)] {
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
                "lo hacés vos (va a Checklist)"
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
    // Como Armar: la lista corta a la izquierda y lo elegido entero a la derecha.
    let [izq, der] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(area);
    let ancho = izq.width.saturating_sub(2) as usize;
    let mut lineas: Vec<Line> = Vec::new();
    let mut linea_sel = 0;
    let mut elegible = 0;
    for fila in app.filas_checklist() {
        if fila.clase == ClaseFila::Titulo {
            lineas.push(Line::from(Span::styled(
                cortar(&fila.texto, ancho),
                Style::default().add_modifier(Modifier::BOLD),
            )));
            continue;
        }
        let marca = match fila.clase {
            ClaseFila::Plegable { abierto: true } => "▾",
            ClaseFila::Plegable { abierto: false } => "▸",
            _ if app.hechos.contains(&fila.detalle) => "[x]",
            _ => "[ ]",
        };
        let mut estilo = Style::default();
        if matches!(fila.clase, ClaseFila::Plegable { .. }) {
            estilo = estilo.add_modifier(Modifier::BOLD);
        }
        if elegible == app.seleccion_checklist {
            estilo = estilo.add_modifier(Modifier::REVERSED);
            linea_sel = lineas.len();
        }
        lineas.push(Line::from(Span::styled(
            cortar(&format!("{marca} {}", fila.texto), ancho),
            estilo,
        )));
        elegible += 1;
    }
    let alto = izq.height.saturating_sub(2) as usize;
    let desde = linea_sel.saturating_sub(alto.saturating_sub(1));
    f.render_widget(
        Paragraph::new(lineas).scroll((desde as u16, 0)).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Checklist (espacio marca)"),
        ),
        izq,
    );
    let detalle: Vec<Line> = app
        .fila_elegida()
        .map(|f| {
            f.detalle
                .lines()
                .map(|l| Line::from(l.to_string()))
                .collect()
        })
        .unwrap_or_default();
    f.render_widget(
        Paragraph::new(detalle)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("Detalle")),
        der,
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
                    .block(Block::default().borders(Borders::ALL).title("Instalar")),
                area,
            );
            return;
        }
        Ok(plan) => plan,
        Err(e) => {
            f.render_widget(
                Paragraph::new(format!("{e:#}"))
                    .wrap(Wrap { trim: false })
                    .block(Block::default().borders(Borders::ALL).title("Instalar")),
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
            .block(Block::default().borders(Borders::ALL).title("Instalar")),
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
            .block(Block::default().borders(Borders::ALL).title("Instalar")),
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

fn teclas_de_receta(app: &App, item: &Item) -> Vec<(&'static str, &'static str)> {
    if item.instalado_por.is_some() && item.estado != Estado::Excluida {
        return vec![("x", "excluir")];
    }
    if app.investigando.contains(&item.id) {
        return vec![("Enter", "armando")];
    }
    if app.buscando.contains(&item.id) {
        return vec![("Enter", "buscando")];
    }
    match item.estado {
        Estado::Pendiente if item.fuente.is_none() => match &item.posible {
            Some(p) if p.pendiente().is_some() => vec![
                (
                    "Enter",
                    if app.posible_gratis(item) {
                        "aceptar gratis"
                    } else {
                        "aceptar (1 corrida)"
                    },
                ),
                ("n", "descartar"),
                ("i", "pegar otro link"),
                ("x", "excluir"),
            ],
            _ => vec![("Enter", "resolver"), ("l", "pegar link"), ("x", "excluir")],
        },
        Estado::Pendiente => vec![
            (
                "Enter",
                if receta_directa(&app.recetario, item).is_some() {
                    "armar gratis"
                } else {
                    "armar (1 corrida)"
                },
            ),
            ("x", "excluir"),
        ],
        Estado::PorRevisar => vec![
            ("Enter", "aprobar"),
            ("e", "editar"),
            ("r", "rearmar"),
            ("x", "excluir"),
        ],
        Estado::Aprobada => vec![
            ("Enter", "2 Instalar"),
            ("a", "desaprobar"),
            ("e", "editar"),
            ("r", "rearmar"),
        ],
        Estado::Excluida => vec![("x", "reincluir"), ("e", "editar")],
    }
}

/// La tecla en video inverso se lee como un botón, separada de lo que hace.
fn teclas(lista: &[(&str, &str)]) -> Vec<Span<'static>> {
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
    let contar = |f: &dyn Fn(&Item) -> bool| {
        r.items
            .iter()
            .filter(|i| !i.es_sincronizado() && i.instalado_por.is_none() && f(i))
            .count()
    };
    if app.esta_listo() {
        let n = contar(&|i| i.estado == Estado::Aprobada);
        return if n > 0 {
            format!("COOKBOOK LISTO · en una PC nueva, `2` y `p` instalan las {n} aprobadas")
        } else {
            "COOKBOOK LISTO · no hay recetas aprobadas para instalar".into()
        };
    }
    if let Some(item) = app.seleccionado().filter(|i| app.posible_pendiente(&i.id)) {
        return format!(
            "Siguiente: Enter acepta y arma ({})",
            if app.posible_gratis(item) {
                "gratis"
            } else {
                "1 corrida; pide s"
            }
        );
    }
    let (gratis, investigar, _) = app.por_armar();
    let (gratis, claude) = (gratis.len(), investigar.len());
    match paso_actual(app) {
        0 => "Siguiente: `s` escanea esta PC".into(),
        1 => match (gratis, claude) {
            (0, 0) => "Enter en un ítem sin origen abre Resolver".into(),
            (n, 0) => format!("Siguiente: `t` arma {n} recetas gratis"),
            (0, n) => format!(
                "Siguiente: `t` pregunta por hasta {} de {n} corridas de Claude",
                n.min(5)
            ),
            (g, c) => format!(
                "Siguiente: `t` arma {g} gratis; Claude: {} de {c}, con `s`",
                c.min(5)
            ),
        },
        2 => format!(
            "Siguiente: {} recetas por revisar: Enter aprueba y avanza",
            contar(&|i| i.estado == Estado::PorRevisar)
        ),
        _ => match (
            contar(&|i| i.estado == Estado::Aprobada),
            servicio::sin_origen(r).len(),
        ) {
            (0, 0) => String::new(),
            (0, n) => format!("Siguiente: {n} sin origen: Enter abre Resolver"),
            (n, _) => format!("Siguiente: en una PC nueva, `2` y `p` instalan las {n} aprobadas"),
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
        Line::from("rebake: recetas de tu setup para rehacerlo en una PC nueva."),
        linea_camino(app),
        Line::from("Enter arma o abre Resolver; posible: Enter acepta, n descarta."),
        Line::from("Resolver: ↑↓/j k mueven, Enter elige, Esc cancela; link gratis."),
        Line::from("Buscar cuesta 1 corrida, sin armar. Pegar texto nunca ejecuta."),
        Line::from("t armar todas con origen: gratis o hasta 5 corridas; nunca busca."),
        Line::from("Claude (Enter, i/r, t, buscar): s confirma; otra tecla cancela."),
        Line::from("Enter aprueba y avanza, nunca desaprueba; a/espacio alternan."),
        Line::from("Aprobar queda guardado en el cookbook; v aprueba todas; e edita."),
        Line::from("x excluye sin motivo; o muestra/oculta excluidas; x reincluye."),
        Line::from("l pega link: Enter guarda; vacío busca con s. / filtra; u actualiza."),
        Line::from("1 Armar. En esta PC: decidí cómo se instala cada cosa."),
        Line::from("2 Instalar. En una PC nueva: instalá las recetas aprobadas."),
        Line::from("c comprueba; espacio marca solo para esa instalación; p instalar."),
        Line::from("3 Checklist. Después de instalar: lo que rebake no hace por vos."),
        Line::from("espacio tilda; Enter despliega ajustes; s escanea; q salir."),
        titulo("Marcas"),
        Line::from(leyenda(app)),
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

fn lineas_pie(app: &App, ancho: u16) -> Vec<Line<'static>> {
    let (gratis, investigar, _) = app.por_armar();
    let (gratis, claude) = (gratis.len(), investigar.len());
    let por_revisar = app
        .recetario
        .items
        .iter()
        .filter(|i| {
            !i.es_sincronizado() && i.instalado_por.is_none() && i.estado == Estado::PorRevisar
        })
        .count();
    let compactar = app.pestana == Pestana::Armar && gratis + claude > 0 && por_revisar > 0;

    let pestanas = if compactar {
        [
            (Pestana::Armar, "1 Armar"),
            (Pestana::Instalacion, "2 Inst"),
            (Pestana::Checklist, "3 Check"),
        ]
    } else {
        [
            (Pestana::Armar, "1 Armar"),
            (Pestana::Instalacion, "2 Instalar"),
            (Pestana::Checklist, "3 Checklist"),
        ]
    };
    let mut spans: Vec<Span> = Vec::new();
    for (p, texto) in pestanas {
        let estilo = if p == app.pestana {
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default()
        };
        spans.push(Span::styled(format!(" {texto} "), estilo));
    }
    if app.pestana == Pestana::Armar {
        // `?` va primero: en una terminal angosta se cortan las otras, no la que explica todo.
        spans.push(Span::raw(" "));
        spans.extend(teclas(&[("?", "ayuda")]));
        match paso_actual(app) {
            0 => spans.extend(teclas(&[("s", "escanear")])),
            _ if gratis + claude > 0 => {
                let accion = if compactar {
                    match (gratis, claude) {
                        (n, 0) => format!("{n}"),
                        (0, n) => format!("{n} Claude"),
                        (g, c) => format!("{g} + {c} Claude"),
                    }
                } else {
                    match (gratis, claude) {
                        (n, 0) => format!("armar {n}"),
                        (0, n) => format!("armar {n} Claude"),
                        (g, c) => format!("armar {g} + {c} Claude"),
                    }
                };
                spans.extend(teclas(&[("t", &accion)]));
            }
            3 if app
                .recetario
                .items
                .iter()
                .any(|i| i.estado == Estado::Aprobada) =>
            {
                spans.extend(teclas(&[("2", "ir a instalar")]));
            }
            _ => {}
        }
        if por_revisar > 0 {
            let accion_v = format!("aprobar {por_revisar}");
            spans.extend(teclas(&[("v", &accion_v)]));
        }
        if let Some(item) = app.seleccionado() {
            spans.extend(teclas(&teclas_de_receta(app, item)));
        }
    } else if app.pestana == Pestana::Checklist {
        spans.push(Span::raw(format!("  {TECLAS_CHECKLIST}")));
    } else {
        let texto = if app.viendo_plan() && app.plan().is_empty() {
            TECLAS_SIN_PLAN
        } else if app.viendo_plan() {
            TECLAS_INSTALACION
        } else if app.pestana == Pestana::Instalacion && !app.instalando {
            TECLAS_RESULTADOS
        } else {
            ""
        };
        spans.push(Span::raw(format!("  {texto}")));
    }
    if app.pestana == Pestana::Armar && ancho <= 100 {
        let mut acciones = Vec::new();
        if let Some(item) = app.seleccionado() {
            let lista = teclas_de_receta(app, item);
            if let Some((tecla, accion)) = lista.first() {
                acciones.push(format!("{tecla} {accion}"));
            }
        } else if app.recetario.items.is_empty() {
            acciones.push("s escanear".into());
        }
        if gratis + claude > 0 {
            acciones.push("t armar".into());
        }
        if por_revisar > 0 {
            acciones.push("v aprobar".into());
        }
        let acciones = acciones.join("  ");
        let completa = format!("1 Armar  2 Instalar  3 Checklist  {acciones}  ?  ayuda q salir");
        let texto = if completa.width() <= ancho as usize {
            completa
        } else {
            format!("1 Armar 2 Inst 3 Check  {acciones}  ? q salir")
        };
        spans = vec![Span::raw(texto)];
    }
    let segunda = match &app.modo {
        Modo::ConfirmarAprobacion { ids } => format!(
            " ¿Aprobar {} recetas? s confirma; otra tecla cancela",
            ids.len()
        ),
        Modo::ConfirmarTanda { accion, .. } => {
            let n = accion.corridas();
            format!(
                " ¿Gastar {n} {} de Claude? s confirma; otra tecla cancela",
                if n == 1 { "corrida" } else { "corridas" }
            )
        }
        Modo::Resolver { opcion, .. } => {
            let opciones = [
                "pegar link (gratis)",
                "buscar en la web (1 corrida)",
                "excluir",
            ];
            format!(
                " Resolver: {}",
                opciones
                    .iter()
                    .enumerate()
                    .map(|(n, texto)| format!("{}{texto}", if n == *opcion { ">" } else { " " }))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        }
        Modo::Escribiendo { para, texto } => {
            let etiqueta = match para {
                Entrada::Link { .. } => "Link del repo",
                Entrada::Filtro => "Filtro",
            };
            let ayuda = match para {
                Entrada::Link { .. } => "Enter guarda; vacío busca con s; Esc cancela",
                _ => "Enter confirma, Esc cancela",
            };
            format!(" {etiqueta}: {texto}_   ({ayuda})")
        }
        _ => format!(
            " {}",
            app.mensaje.clone().unwrap_or_else(|| siguiente_paso(app))
        ),
    };
    if matches!(app.modo, Modo::Resolver { .. }) {
        spans = vec![Span::raw(
            " ↑↓/j k mover · Enter elige · Esc cancela; luego q salir",
        )];
    } else if matches!(app.modo, Modo::Escribiendo { .. }) {
        spans = vec![Span::raw(" Enter confirma · Esc cancela; luego q salir")];
    } else if matches!(app.modo, Modo::ConfirmarTanda { .. }) {
        spans = vec![Span::raw(" s confirma · otra tecla cancela; luego q salir")];
    }
    vec![Line::from(spans), Line::from(segunda)]
}
