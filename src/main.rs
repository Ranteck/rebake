use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use rebake::escaner::{Entorno, PacmanReal};
use rebake::instalador::{self, Evento, Opciones, Resultado};
use rebake::investigador::{self, aplicar, en_paralelo};
use rebake::{acciones, archivo, rutas, servicio, sincro};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "rebake",
    version,
    about = "Vuelve a hornear tu setup de Claude Code desde un cookbook de recetas de instalación"
)]
struct Cli {
    /// Cookbook a usar (por defecto ~/Documentos/rebake/cookbook.toml, la carpeta Documentos de tu sistema)
    #[arg(long, global = true)]
    archivo: Option<PathBuf>,
    #[command(subcommand)]
    comando: Option<Comando>,
}

#[derive(Subcommand)]
enum Comando {
    /// Clona el repo git de tu cookbook en su carpeta; desde ahí cada comando lo sincroniza
    Clonar { url: String },
    /// Escanea esta PC y agrega lo nuevo al cookbook
    Escanear,
    /// Investiga con claude -p los ítems pendientes (o los ids indicados)
    Investigar { ids: Vec<String> },
    /// Instala las recetas aprobadas
    Instalar {
        /// Muestra los comandos sin ejecutar nada
        #[arg(long)]
        dry_run: bool,
        /// No pide confirmación
        #[arg(long)]
        si: bool,
    },
    /// Cambia el nombre de un perfil en todo el cookbook
    RenombrarPerfil { viejo: String, nuevo: String },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let home = match rutas::home() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("rebake: {e:#}");
            return ExitCode::FAILURE;
        }
    };
    let _guardia = iniciar_logs(&home);
    match correr(cli, &home) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "el comando falló");
            eprintln!("rebake: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn iniciar_logs(home: &Path) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let dir = rutas::dir_estado(home);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("rebake: sin logs, no pude crear {}: {e}", dir.display());
        return None;
    }
    let (escritor, guardia) =
        tracing_appender::non_blocking(tracing_appender::rolling::never(&dir, "rebake.log"));
    tracing_subscriber::fmt()
        .with_writer(escritor)
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .init();
    Some(guardia)
}

/// `Ok(false)` = terminó, pero algo no salió (p. ej. una receta falló).
fn correr(cli: Cli, home: &Path) -> Result<bool> {
    let ruta = cli
        .archivo
        .unwrap_or_else(|| rutas::archivo_por_defecto(home));
    let Some(comando) = cli.comando else {
        rebake::tui::ejecutar(&ruta, home)?;
        return Ok(true);
    };
    match comando {
        Comando::Clonar { url } => clonar(&url, &ruta, home),
        Comando::Escanear => {
            let mut r = archivo::leer(&ruta)?;
            let pacman = PacmanReal;
            let entorno = Entorno::real(&pacman)?;
            let resumen = servicio::escanear(&mut r, &entorno)?;
            archivo::guardar(&ruta, &r)?;
            for a in &resumen.avisos {
                eprintln!("aviso: {a}");
            }
            tracing::info!(
                comando = "escanear",
                items = r.items.len(),
                nuevos = resumen.nuevos,
                avisos = resumen.avisos.len()
            );
            println!(
                "{} ítems ({} nuevos) en {}",
                r.items.len(),
                resumen.nuevos,
                ruta.display()
            );
            Ok(true)
        }
        Comando::Investigar { ids } => investigar(&mut archivo::leer(&ruta)?, &ruta, &ids, home),
        Comando::Instalar { dry_run, si } => instalar(&archivo::leer(&ruta)?, home, dry_run, si),
        Comando::RenombrarPerfil { viejo, nuevo } => {
            let mut r = archivo::leer(&ruta)?;
            acciones::renombrar_perfil(&mut r, &viejo, &nuevo)?;
            archivo::guardar(&ruta, &r)?;
            println!("perfil {viejo} → {nuevo}");
            Ok(true)
        }
    }
}

fn clonar(url: &str, ruta: &Path, home: &Path) -> Result<bool> {
    let r = sincro::clonar(url, ruta)?;
    let carpeta = ruta.parent().unwrap_or(ruta);
    tracing::info!(comando = "clonar", items = r.items.len());
    println!(
        "clonado en {} ({})",
        rutas::contraer(carpeta, home),
        servicio::cantidad(r.items.len(), "receta", "recetas")
    );
    Ok(true)
}

fn investigar(
    r: &mut rebake::modelo::Recetario,
    ruta: &Path,
    ids: &[String],
    home: &Path,
) -> Result<bool> {
    let trabajos = servicio::trabajos(r, ids, home)?;
    let total = trabajos.len();
    let rx = en_paralelo(
        trabajos,
        investigador::binario_claude(),
        investigador::TOPE,
        investigador::PARALELO,
    );
    let (mut ok, mut fallidos) = (0, 0);
    for (n, (id, resultado)) in rx.iter().enumerate() {
        match &resultado {
            Ok(_) => {
                ok += 1;
                eprintln!("[{}/{total}] {id} → por revisar", n + 1);
            }
            Err(e) => {
                fallidos += 1;
                eprintln!("[{}/{total}] {id} → error: {e:#}", n + 1);
            }
        }
        tracing::info!(comando = "investigar", id = %id, ok = resultado.is_ok());
        aplicar(r, &id, resultado, &rutas::hoy());
        // Se guarda tras cada ítem para no perder lo investigado si se corta.
        archivo::guardar(ruta, r)?;
    }
    println!("{ok} por revisar, {fallidos} con error");
    Ok(fallidos == 0)
}

fn instalar(r: &rebake::modelo::Recetario, home: &Path, dry_run: bool, si: bool) -> Result<bool> {
    let orden = instalador::planificar(r)?;
    if orden.is_empty() {
        println!("no hay recetas aprobadas");
        return Ok(true);
    }
    let perfiles: Vec<&str> = r.perfiles.iter().map(|p| p.nombre.as_str()).collect();
    if !dry_run
        && !si
        && !confirmar(&format!(
            "¿Instalar {} recetas en {}? [s/N] ",
            orden.len(),
            perfiles.join(", ")
        ))?
    {
        bail!("instalación cancelada");
    }
    let opciones = Opciones {
        dry_run,
        tope: instalador::TOPE,
        home: home.to_path_buf(),
        log: (!dry_run).then(|| rutas::dir_estado(home).join("ultima-instalacion.log")),
    };
    let resultados = instalador::instalar(r, &orden, &opciones, &mut |e| match e {
        Evento::Inicio(id) => eprintln!("→ {id}"),
        Evento::Linea(_, l) => eprintln!("    {l}"),
        Evento::Fin(..) => {}
    });
    let mut todo_bien = true;
    for (id, res) in &resultados {
        let texto = match res {
            Resultado::Ok => "instalado".to_string(),
            Resultado::Salteado => "salteado (ya estaba)".to_string(),
            Resultado::Simulado => "simulado".to_string(),
            Resultado::Fallo(m) => {
                todo_bien = false;
                format!("falló: {m}")
            }
            Resultado::Bloqueado(m) => {
                todo_bien = false;
                format!("bloqueado: {m}")
            }
        };
        tracing::info!(comando = "instalar", id = %id, resultado = %texto);
        println!("{id}: {texto}");
    }
    if !dry_run {
        print!("{}", servicio::texto_checklist(r));
    }
    Ok(todo_bien)
}

fn confirmar(pregunta: &str) -> Result<bool> {
    eprint!("{pregunta}");
    std::io::stderr().flush()?;
    let mut linea = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut linea)
        .context("no pude leer la respuesta")?;
    Ok(matches!(linea.trim(), "s" | "S" | "si" | "sí"))
}
