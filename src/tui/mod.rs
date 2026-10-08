pub mod app;
pub mod vista;

use crate::escaner::{Entorno, PacmanReal};
use crate::instalador::{self, Evento, Opciones};
use crate::investigador::{self, Receta};
use crate::{acciones, archivo, rutas, servicio};
use anyhow::{Context, Result};
use app::{App, Efecto, Pestana};
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

type Investigado = (String, Result<Receta>);

pub fn ejecutar(ruta: &Path, home: &Path) -> Result<()> {
    let recetario = archivo::leer(ruta)?;
    let mut app = App::nueva(recetario);
    let mut terminal = ratatui::init();
    let resultado = bucle(&mut terminal, &mut app, ruta, home);
    ratatui::restore();
    resultado
}

fn bucle(terminal: &mut DefaultTerminal, app: &mut App, ruta: &Path, home: &Path) -> Result<()> {
    let (tx_inv, rx_inv) = mpsc::channel::<Investigado>();
    let mut rx_inst: Option<mpsc::Receiver<Evento>> = None;
    loop {
        while let Ok((id, res)) = rx_inv.try_recv() {
            tracing::info!(accion = "investigar", id = %id, ok = res.is_ok());
            app.registrar_investigacion(&id, res, &rutas::hoy());
            guardar(app, ruta);
        }
        if let Some(rx) = &rx_inst {
            while let Ok(e) = rx.try_recv() { app.evento_instalacion(e); }
        }
        terminal.draw(|f| vista::dibujar(f, app))?;
        if !event::poll(Duration::from_millis(100))? { continue; }
        let Event::Key(tecla) = event::read()? else { continue };
        if tecla.kind != KeyEventKind::Press { continue; }
        match app.tecla(tecla) {
            Efecto::Nada => {}
            Efecto::Salir => return Ok(()),
            Efecto::Guardar => guardar(app, ruta),
            Efecto::Escanear => escanear(app, ruta),
            Efecto::Investigar(ids) => investigar(app, ruta, home, ids, &tx_inv),
            Efecto::Editar(id) => editar(terminal, app, ruta, &id)?,
            Efecto::Instalar => rx_inst = instalar(app, home).or(rx_inst.take()),
        }
    }
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
            let avisos = if res.avisos.is_empty() { String::new() } else { format!(" · aviso: {}", res.avisos.join(" · ")) };
            app.mensaje = Some(format!("escaneo: {} nuevos{avisos}", res.nuevos));
            guardar(app, ruta);
        }
        Err(e) => app.mensaje = Some(format!("no pude escanear: {e:#}")),
    }
}

fn investigar(app: &mut App, ruta: &Path, home: &Path, ids: Vec<String>, tx: &mpsc::Sender<Investigado>) {
    guardar(app, ruta);
    match servicio::trabajos(&app.recetario, &ids, home) {
        Ok(trabajos) => {
            let rx = investigador::en_paralelo(trabajos, investigador::binario_claude(), investigador::TOPE, investigador::PARALELO);
            let tx = tx.clone();
            thread::spawn(move || {
                for m in rx {
                    if tx.send(m).is_err() { break; }
                }
            });
        }
        Err(e) => {
            for id in &ids { app.investigando.remove(id); }
            app.mensaje = Some(format!("{e:#}"));
        }
    }
}

fn instalar(app: &mut App, home: &Path) -> Option<mpsc::Receiver<Evento>> {
    let orden = match instalador::planificar(&app.recetario) {
        Ok(o) => o,
        Err(e) => {
            app.mensaje = Some(format!("{e:#}"));
            return None;
        }
    };
    app.instalacion = orden.iter().map(|id| (id.clone(), None)).collect();
    app.pestana = Pestana::Instalacion;
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
            // Si la TUI se cerró, nadie escucha: la instalación sigue y queda en el log.
            let _ = tx.send(e);
        });
    });
    Some(rx)
}

fn editar(terminal: &mut DefaultTerminal, app: &mut App, ruta: &Path, id: &str) -> Result<()> {
    let item = app.recetario.item(id).context("el ítem a editar ya no existe")?;
    let temporal = std::env::temp_dir().join(format!("recetario-{}-editar.toml", std::process::id()));
    std::fs::write(&temporal, acciones::item_como_toml(item)?)?;
    ratatui::restore();
    let editor = std::env::var("VISUAL").or_else(|_| std::env::var("EDITOR")).unwrap_or_else(|_| "vi".into());
    // Vía `sh` para aceptar editores con argumentos (p. ej. "code --wait").
    let estado = Command::new("sh").arg("-c").arg(format!("{editor} \"$1\"")).arg("sh").arg(&temporal).status();
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
