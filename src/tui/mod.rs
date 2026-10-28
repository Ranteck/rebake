pub mod app;
pub mod vista;

use crate::escaner::{Entorno, PacmanReal};
use crate::instalador::{self, Evento, Opciones, Presencia};
use crate::investigador::{self, Receta};
use crate::modelo::Posible;
use crate::servicio::{self, Novedad};
use crate::{acciones, actualizar, archivo, rutas};
use anyhow::{Context, Result, bail};
use app::{App, Efecto, Modo, Pestana};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::collections::HashMap;
use std::io::IsTerminal;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

type Investigado = (String, Result<Receta>);
type Buscado = (String, Result<Posible>);

#[derive(Default)]
pub struct Arranque {
    /// Abrir en la pestaña Instalación.
    pub instalacion: bool,
    pub novedades: HashMap<String, Novedad>,
    /// La revisión de versión que `main` lanzó al arrancar.
    pub revision: Option<mpsc::Receiver<Option<String>>>,
}

pub fn ejecutar(ruta: &Path, home: &Path, arranque: Arranque) -> Result<()> {
    let recetario = archivo::leer(ruta)?;
    let mut app = App::nueva(recetario).con_arranque(arranque.instalacion, arranque.novedades);
    app.color = std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty());
    // Sin esta guarda, `ratatui::init()` toma /dev/tty aunque la salida esté capturada.
    if !std::io::stdout().is_terminal() {
        bail!("no pude abrir la pantalla: la salida no es una terminal");
    }
    let mut terminal = ratatui::init();
    let resultado = bucle(&mut terminal, &mut app, ruta, home, arranque.revision);
    ratatui::restore();
    resultado
}

fn bucle(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    ruta: &Path,
    home: &Path,
    mut rx_version: Option<mpsc::Receiver<Option<String>>>,
) -> Result<()> {
    let (tx_inv, rx_inv) = mpsc::channel::<Investigado>();
    let (tx_bus, rx_bus) = mpsc::channel::<Buscado>();
    let mut rx_inst: Option<mpsc::Receiver<Evento>> = None;
    let mut rx_comp: Option<mpsc::Receiver<(String, Presencia)>> = None;
    loop {
        if let Some(rx) = &rx_version
            && let Ok(nueva) = rx.try_recv()
        {
            tracing::info!(nueva = ?nueva, "revisión de versión terminada");
            app.version_nueva = nueva;
            rx_version = None;
        }
        while let Ok((id, res)) = rx_inv.try_recv() {
            let resultado = investigador::resultado_receta(&app.recetario, &id, &res);
            let hooks_vinculados = app.recetario.hooks_de(&id).count();
            if resultado == "fallo" {
                tracing::error!(
                    accion = "investigar",
                    resultado,
                    causa = "investigacion",
                    hooks_vinculados
                );
            } else {
                tracing::info!(
                    accion = "investigar",
                    resultado,
                    causa = if resultado == "pendiente" {
                        "registro_sin_documentacion"
                    } else {
                        "ninguna"
                    },
                    hooks_vinculados
                );
            }
            app.registrar_investigacion(&id, res, &rutas::hoy());
            guardar(app, ruta);
        }
        while let Ok((id, res)) = rx_bus.try_recv() {
            tracing::info!(accion = "buscar_origen", ok = res.is_ok());
            app.registrar_busqueda(&id, res);
            guardar(app, ruta);
        }
        if let Some(rx) = &rx_inst {
            while let Ok(e) = rx.try_recv() {
                if let Evento::Fin(id, res) = &e {
                    let hooks_vinculados = app.recetario.hooks_de(id).count();
                    if matches!(
                        res,
                        crate::instalador::Resultado::Fallo(_)
                            | crate::instalador::Resultado::Bloqueado(_)
                    ) {
                        tracing::error!(
                            accion = "instalar",
                            resultado = res.categoria(),
                            hooks_vinculados
                        );
                    } else {
                        tracing::info!(
                            accion = "instalar",
                            resultado = res.categoria(),
                            hooks_vinculados
                        );
                    }
                }
                app.evento_instalacion(e);
            }
        }
        if let Some(rx) = &rx_comp {
            loop {
                match rx.try_recv() {
                    Ok((id, p)) => app.registrar_presencia(&id, p),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        app.terminar_comprobacion();
                        rx_comp = None;
                        break;
                    }
                }
            }
        }
        app.tic();
        terminal.draw(|f| vista::dibujar(f, app))?;
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        let Event::Key(tecla) = event::read()? else {
            continue;
        };
        if tecla.kind != KeyEventKind::Press {
            continue;
        }
        match tecla_registrada(app, tecla) {
            Efecto::Nada => {}
            Efecto::Salir => return Ok(()),
            Efecto::Guardar => guardar(app, ruta),
            Efecto::Escanear => escanear(app, ruta),
            Efecto::Investigar(ids) => investigar(app, ruta, home, ids, &tx_inv),
            Efecto::Buscar(ids) => buscar(app, home, ids, &tx_bus),
            Efecto::Lote(inv, bus) => {
                investigar(app, ruta, home, inv, &tx_inv);
                buscar(app, home, bus, &tx_bus);
            }
            Efecto::Editar(id) => editar(terminal, app, ruta, home, &id)?,
            Efecto::Actualizar(version) => actualizar(terminal, app, &version),
            Efecto::Comprobar => rx_comp = Some(comprobar(app, home)),
            Efecto::Instalar => rx_inst = instalar(app, home).or(rx_inst.take()),
        }
    }
}

fn tecla_registrada(app: &mut App, tecla: KeyEvent) -> Efecto {
    let confirmando = matches!(app.modo, Modo::ConfirmarTanda { .. });
    let armando = app.modo == Modo::Normal
        && app.pestana == Pestana::Recetas
        && matches!(tecla.code, KeyCode::Char('t' | 'I'));
    let efecto = app.tecla(tecla);
    if armando || confirmando {
        let (inv, bus) = match &efecto {
            Efecto::Investigar(ids) => (ids.len(), 0),
            Efecto::Buscar(ids) => (0, ids.len()),
            Efecto::Lote(inv, bus) => (inv.len(), bus.len()),
            _ => (0, 0),
        };
        let decision = if confirmando {
            if tecla.code == KeyCode::Char('s') {
                "confirmar"
            } else {
                "cancelar"
            }
        } else if matches!(app.modo, Modo::ConfirmarTanda { .. }) {
            "preguntar"
        } else if !app.claude_en_curso.is_empty() {
            "esperar"
        } else {
            "sin_claude"
        };
        let corridas = match &app.modo {
            Modo::ConfirmarTanda { investigar, buscar } => investigar.len() + buscar.len(),
            _ if confirmando => inv + bus,
            _ => 0,
        };
        tracing::info!(
            accion = "armar_todas",
            decision,
            gratis = if confirmando { 0 } else { inv },
            corridas,
            en_curso = app.claude_en_curso.len(),
        );
    }
    efecto
}

fn guardar(app: &mut App, ruta: &Path) {
    if let Err(e) = archivo::guardar(ruta, &app.recetario) {
        tracing::error!(error = %format!("{e:#}"), "no pude guardar el recetario");
        app.mensaje = Some(format!("no pude guardar: {e:#}"));
    }
}

fn escanear(app: &mut App, ruta: &Path) {
    let pacman = PacmanReal;
    match Entorno::real(&pacman).and_then(|e| servicio::escanear(&mut app.recetario, &e)) {
        Ok(res) => {
            tracing::info!(
                accion = "escanear",
                resultado = "ok",
                nuevos = res.nuevos,
                hooks_vinculados = res.hooks_vinculados,
                externos = res.externos,
                avisos = res.avisos.len()
            );
            let avisos = if res.avisos.is_empty() {
                String::new()
            } else {
                format!(" · aviso: {}", res.avisos.join(" · "))
            };
            app.mensaje = Some(format!("escaneo: {} nuevos{avisos}", res.nuevos));
            guardar(app, ruta);
        }
        Err(e) => {
            tracing::error!(accion = "escanear", resultado = "fallo", causa = "lectura");
            app.mensaje = Some(format!("no pude escanear: {e:#}"));
        }
    }
}

fn investigar(
    app: &mut App,
    ruta: &Path,
    home: &Path,
    ids: Vec<String>,
    tx: &mpsc::Sender<Investigado>,
) {
    // `trabajos` interpreta IDs vacíos como todas las pendientes de la CLI.
    if ids.is_empty() {
        return;
    }
    guardar(app, ruta);
    match servicio::trabajos(&app.recetario, &ids, home) {
        Ok(trabajos) => {
            let rx = investigador::en_paralelo(
                trabajos,
                investigador::binario_claude(),
                investigador::TOPE,
                investigador::PARALELO,
            );
            let tx = tx.clone();
            thread::spawn(move || {
                for m in rx {
                    if tx.send(m).is_err() {
                        break;
                    }
                }
            });
        }
        Err(e) => {
            tracing::error!(
                accion = "investigar",
                cantidad = ids.len(),
                "no pude preparar las investigaciones"
            );
            for id in &ids {
                app.registrar_investigacion(id, Err(anyhow::anyhow!("{e:#}")), &rutas::hoy());
            }
        }
    }
}

fn buscar(app: &App, home: &Path, ids: Vec<String>, tx: &mpsc::Sender<Buscado>) {
    if ids.is_empty() {
        return;
    }
    let trabajos = ids
        .iter()
        .filter_map(|id| app.recetario.item(id))
        .map(|i| (i.id.clone(), investigador::prompt_origen(i, home)))
        .collect();
    let rx = investigador::buscar_origenes(
        trabajos,
        investigador::binario_claude(),
        investigador::TOPE,
        investigador::PARALELO,
    );
    let tx = tx.clone();
    thread::spawn(move || {
        for m in rx {
            if tx.send(m).is_err() {
                break;
            }
        }
    });
}

fn comprobar(app: &App, home: &Path) -> mpsc::Receiver<(String, Presencia)> {
    let recetario = app.recetario.clone();
    let mut ids = app.plan();
    ids.extend(
        recetario
            .items
            .iter()
            .filter(|i| i.instalado_por.is_some() && i.estado != crate::modelo::Estado::Excluida)
            .map(|i| i.id.clone()),
    );
    let home = home.to_path_buf();
    let (tx, rx) = mpsc::channel();
    let dispatch = tracing::dispatcher::get_default(Clone::clone);
    thread::spawn(move || {
        tracing::dispatcher::with_default(&dispatch, || {
            let (mut comprobados, mut faltantes, mut fallos) = (0, 0, 0);
            instalador::comprobar(
                &recetario,
                &ids,
                &home,
                instalador::TOPE_COMPROBACION,
                &mut |id, p| {
                    comprobados += 1;
                    match &p {
                        Presencia::Falta(perfiles) => faltantes += perfiles.len().max(1),
                        Presencia::NoSePudo(_) => fallos += 1,
                        _ => {}
                    }
                    // Solo falla si la TUI ya terminó; no queda nadie a quien avisar.
                    let _ = tx.send((id, p));
                },
            );
            if fallos > 0 {
                tracing::error!(
                    accion = "comprobar",
                    resultado = "fallo",
                    causa = "lectura",
                    comprobados,
                    faltantes,
                    fallos
                );
            } else {
                tracing::info!(
                    accion = "comprobar",
                    resultado = "ok",
                    comprobados,
                    faltantes,
                    fallos
                );
            }
        })
    });
    rx
}

fn instalar(app: &mut App, home: &Path) -> Option<mpsc::Receiver<Evento>> {
    let orden = app.a_instalar();
    if orden.is_empty() {
        app.mensaje = Some("no hay nada marcado para instalar".into());
        return None;
    }
    app.instalacion = orden.iter().map(|id| (id.clone(), None)).collect();
    app.pestana = Pestana::Instalacion;
    app.instalando = true;
    let recetario = app.recetario.clone();
    let opciones = Opciones {
        dry_run: false,
        tope: instalador::TOPE,
        home: home.to_path_buf(),
        log: Some(rutas::dir_estado(home).join("ultima-instalacion.log")),
    };
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        instalador::instalar(&recetario, &orden, &opciones, &mut |e| {
            // Solo falla si la TUI ya terminó; no queda nadie a quien avisar.
            let _ = tx.send(e);
        });
    });
    Some(rx)
}

/// Como `editar`, suelta la terminal mientras corre el instalador; con éxito no vuelve.
fn actualizar(terminal: &mut DefaultTerminal, app: &mut App, version: &str) {
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => {
            app.mensaje = Some(format!(
                "no se actualizó: no pude ver el binario en uso: {e}"
            ));
            return;
        }
    };
    ratatui::restore();
    eprintln!("actualizando a rebake {version}…");
    let error = match actualizar::instalar(version, &actualizar::base(), &exe) {
        Ok(()) => {
            tracing::info!(version, "rebake actualizado; se reabre");
            actualizar::reabrir(&exe)
        }
        Err(e) => e,
    };
    tracing::warn!(version, error = %format!("{error:#}"), "no se actualizó");
    *terminal = ratatui::init();
    app.mensaje = Some(format!("no se actualizó: {error:#}"));
}

fn editar(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    ruta: &Path,
    home: &Path,
    id: &str,
) -> Result<()> {
    let item = app
        .recetario
        .item(id)
        .context("el ítem a editar ya no existe")?;
    // En la carpeta de estado del usuario y no en /tmp, que es compartido con otros usuarios.
    let dir = rutas::dir_estado(home);
    std::fs::create_dir_all(&dir).with_context(|| format!("no pude crear {}", dir.display()))?;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or_default();
    let temporal = dir.join(format!("editar-{}-{nanos}.toml", std::process::id()));
    archivo::crear_privado(&temporal, &acciones::item_como_toml(item)?)?;
    ratatui::restore();
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".into());
    // Vía `sh` para aceptar editores con argumentos (p. ej. "code --wait").
    let estado = Command::new("sh")
        .arg("-c")
        .arg(format!("{editor} \"$1\""))
        .arg("sh")
        .arg(&temporal)
        .status();
    *terminal = ratatui::init();
    app.mensaje = Some(match estado {
        Ok(s) if s.success() => {
            let texto = std::fs::read_to_string(&temporal)?;
            match acciones::reemplazar_desde_toml(&mut app.recetario, id, &texto) {
                Ok(()) => {
                    guardar(app, ruta);
                    format!("{id} editado; queda por revisar")
                }
                Err(e) => format!("no se cambió nada: {e:#}"),
            }
        }
        Ok(s) => format!("el editor terminó con {s}; no se cambió nada"),
        Err(e) => format!("no pude abrir {editor}: {e}"),
    });
    if let Err(e) = std::fs::remove_file(&temporal) {
        tracing::warn!(ruta = %temporal.display(), error = %e, "no pude borrar el temporal de edición");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn comprobar_incluye_hooks_sin_aprobar() {
        use crate::modelo::{InstaladorHook, Item, Perfil, Recetario, Tipo};
        let home = tempfile::tempdir().unwrap();
        let mut r = Recetario::nuevo();
        r.perfiles.push(Perfil {
            nombre: "personal".into(),
            dir: "~/.claude-personal".into(),
        });
        let mut hook = Item::nuevo("hook:tool hook", Tipo::Hook);
        hook.perfiles.push("personal".into());
        hook.instalado_por = Some(InstaladorHook::Externo("tool".into()));
        r.items.push(hook);
        let app = super::App::nueva(r);
        let resultados: Vec<_> = super::comprobar(&app, home.path()).iter().collect();
        assert_eq!(
            resultados,
            vec![(
                "hook:tool hook".into(),
                crate::instalador::Presencia::Falta(vec!["personal".into()])
            )]
        );
    }
    use super::*;
    use crate::modelo::{Fuente, Item, Recetario, Tipo, Via};
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    fn pendiente() -> App {
        let mut r = Recetario::nuevo();
        let mut i = Item::nuevo("mcp:privado", Tipo::Mcp);
        i.fuente = Some(Fuente {
            repo: "https://github.com/a/b".into(),
            via: Via::Manual,
            doc: None,
        });
        r.items.push(i);
        App::nueva(r)
    }

    #[test]
    fn error_al_preparar_la_tanda_libera_todos_sus_ids() {
        let mut app = pendiente();
        app.tecla(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE));
        let Efecto::Investigar(ids) =
            app.tecla(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE))
        else {
            panic!()
        };
        app.recetario.items.clear();
        let dir = tempfile::tempdir().unwrap();
        let (tx, rx) = mpsc::channel();
        investigar(
            &mut app,
            &dir.path().join("cookbook.toml"),
            dir.path(),
            ids,
            &tx,
        );
        assert!(app.investigando.is_empty());
        assert!(app.claude_en_curso.is_empty());
        assert!(
            app.mensaje
                .as_deref()
                .unwrap()
                .contains("no existe el ítem")
        );
        assert!(rx.try_recv().is_err());
        app.recetario = pendiente().recetario;
        assert_eq!(
            app.tecla(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE)),
            Efecto::Nada
        );
        assert_eq!(
            app.modo,
            Modo::ConfirmarTanda {
                investigar: vec!["mcp:privado".into()],
                buscar: vec![],
            }
        );
        assert_eq!(
            app.tecla(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE)),
            Efecto::Investigar(vec!["mcp:privado".into()])
        );
    }

    struct Captura(Arc<Mutex<Vec<u8>>>);
    impl Write for Captura {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn capturar_logs(salida: Arc<Mutex<Vec<u8>>>) -> impl tracing::Subscriber + Send + Sync {
        static INICIAR: std::sync::Once = std::sync::Once::new();
        INICIAR.call_once(|| {
            // Evita cachear Interest::never si otro hilo estrena un evento sin captura.
            tracing::subscriber::set_global_default(tracing_subscriber::registry()).unwrap();
        });
        tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_writer(move || Captura(salida.clone()))
            .finish()
    }

    #[test]
    fn comprobacion_registra_conteos_y_fallos_sin_comandos() {
        use crate::modelo::{InstaladorHook, Perfil};
        let salida = Arc::new(Mutex::new(Vec::new()));
        let subscriber = capturar_logs(salida.clone());
        tracing::subscriber::with_default(subscriber, || {
            let home = tempfile::tempdir().unwrap();
            std::fs::create_dir_all(home.path().join(".claude")).unwrap();
            let mut r = Recetario::nuevo();
            r.perfiles.push(Perfil {
                nombre: "perfil".into(),
                dir: "~/.claude".into(),
            });
            let mut hook = Item::nuevo("hook:tool", Tipo::Hook);
            hook.perfiles.push("perfil".into());
            hook.instalado_por = Some(InstaladorHook::Externo("tool".into()));
            r.items.push(hook);
            let app = App::nueva(r);
            // Otro test puede usar primero el mismo evento sin un subscriber local.
            thread::scope(|hilos| {
                hilos
                    .spawn(|| {
                        let _: Vec<_> = comprobar(&app, home.path()).iter().collect();
                    })
                    .join()
                    .unwrap();
            });
            for contenido in [
                r#"{"hooks":{"Stop":[{"hooks":[{"command":"tool"}]}]}}"#,
                "{}",
                "API_TOKEN=secreto-sintetico",
            ] {
                std::fs::write(home.path().join(".claude/settings.json"), contenido).unwrap();
                let _: Vec<_> = comprobar(&app, home.path()).iter().collect();
            }
        });
        let log = String::from_utf8(salida.lock().unwrap().clone()).unwrap();
        assert_eq!(log.lines().count(), 3, "{log}");
        for parte in [
            "resultado=\"ok\"",
            "resultado=\"fallo\"",
            "faltantes=1",
            "comprobados=1",
        ] {
            assert!(log.contains(parte), "{log}");
        }
        for (linea, (resultado, faltantes, fallos)) in
            log.lines()
                .zip([("ok", 0, 0), ("ok", 1, 0), ("fallo", 0, 1)])
        {
            for campo in [
                format!("resultado=\"{resultado}\""),
                "comprobados=1".into(),
                format!("faltantes={faltantes}"),
                format!("fallos={fallos}"),
            ] {
                assert!(linea.contains(&campo), "{linea}");
            }
        }
        assert!(
            !log.contains("hook:tool")
                && !log.contains("secreto-sintetico")
                && !log.contains("API_TOKEN"),
            "{log}"
        );
    }

    #[test]
    fn las_decisiones_de_cuota_registran_contadores_sin_datos_del_item() {
        let salida = Arc::new(Mutex::new(Vec::new()));
        let subscriber = capturar_logs(salida.clone());
        tracing::subscriber::with_default(subscriber, || {
            let mut app = pendiente();
            for c in ['t', 'i', 't', 's'] {
                tecla_registrada(
                    &mut app,
                    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE),
                );
            }
            let mut gratis = Item::nuevo("plugin:privado@x", Tipo::Plugin);
            gratis.fuente = Some(Fuente {
                repo: "https://github.com/a/b".into(),
                via: Via::Metadatos,
                doc: None,
            });
            app.recetario.items.push(gratis);
            tecla_registrada(
                &mut app,
                KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE),
            );
        });
        let log = String::from_utf8(salida.lock().unwrap().clone()).unwrap();
        assert_eq!(log.lines().count(), 5, "{log}");
        for decision in ["preguntar", "cancelar", "confirmar", "esperar"] {
            assert!(log.contains(&format!("decision=\"{decision}\"")), "{log}");
        }
        assert!(log.contains("corridas=1"), "{log}");
        assert!(log.contains("gratis=0"), "{log}");
        assert!(log.contains("gratis=1"), "{log}");
        assert!(!log.contains("privado") && !log.contains("github"), "{log}");
    }
}
