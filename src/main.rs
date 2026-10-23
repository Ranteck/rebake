use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use rebake::actualizar;
use rebake::escaner::{Entorno, PacmanReal};
use rebake::instalador::{self, Evento, Opciones, Resultado};
use rebake::investigador::{self, aplicar, en_paralelo};
use rebake::modelo::Tipo;
use rebake::secretos::sin_credenciales_url;
use rebake::servicio::Novedad;
use rebake::{acciones, archivo, checklist, rutas, servicio, sincro};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc::Receiver;

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
    /// Alias o función de zsh/bash cuya cuenta usan las corridas de claude -p
    #[arg(long, global = true)]
    profile: Option<String>,
    /// URL del repo de tu cookbook: lo clona la primera vez y abre la pantalla en Instalación
    url: Option<String>,
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

impl Comando {
    fn nombre(&self) -> &'static str {
        match self {
            Comando::Clonar { .. } => "clonar",
            Comando::Escanear => "escanear",
            Comando::Investigar { .. } => "investigar",
            Comando::Instalar { .. } => "instalar",
            Comando::RenombrarPerfil { .. } => "renombrar-perfil",
        }
    }
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
    let propia = env!("CARGO_PKG_VERSION");
    let mut revision = Some(actualizar::en_segundo_plano(
        propia.to_string(),
        actualizar::base(),
    ));
    let resultado = correr(cli, &home, &mut revision);
    // Sigue acá solo con subcomando (la pantalla se lleva el receptor): avisa si ya llegó.
    if let Some(nueva) = revision.and_then(|rx| rx.try_recv().ok().flatten()) {
        tracing::info!(propia, nueva = %nueva, "hay una versión nueva");
        eprintln!("hay rebake {nueva} (tenés {propia}): abrí `rebake` para actualizar");
    }
    match resultado {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            if let Some(cuenta) = e.downcast_ref::<investigador::ErrorCuenta>() {
                if cuenta.operativo {
                    tracing::error!(
                        accion = "resolver_cuenta",
                        shell = cuenta.shell,
                        resultado = "fallo",
                        causa = cuenta.causa
                    );
                } else {
                    tracing::info!(
                        accion = "resolver_cuenta",
                        shell = cuenta.shell,
                        resultado = "rechazado",
                        causa = cuenta.causa
                    );
                }
            } else {
                tracing::error!(error = %format!("{e:#}"), "el comando falló");
            }
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
fn correr(cli: Cli, home: &Path, revision: &mut Option<Receiver<Option<String>>>) -> Result<bool> {
    if matches!(cli.comando, None | Some(Comando::Investigar { .. }))
        && let Some(nombre) = &cli.profile
    {
        let shell = investigador::configurar_cuenta(nombre)?;
        tracing::info!(
            accion = "resolver_cuenta",
            shell,
            resultado = "ok",
            causa = "ninguna"
        );
    }
    let ruta = cli
        .archivo
        .unwrap_or_else(|| rutas::archivo_por_defecto(home));
    // `clonar` crea el clon: no hay nada que traer antes ni que publicar después.
    let mut sincroniza =
        !matches!(cli.comando, Some(Comando::Clonar { .. })) && sincronizado(&ruta);
    let mut recien_clonado = false;
    if let Some(url) = &cli.url {
        if cli.comando.is_some() {
            bail!("pasá la URL o un comando, no los dos");
        }
        // Una palabra suelta es un comando mal escrito, no un repo para clonar.
        if !url.contains(['/', ':']) {
            bail!("`{url}` no es un comando ni la URL de un repo: mirá `rebake --help`");
        }
        if sincroniza {
            let origen = sincro::origen(&ruta)?;
            if !sincro::misma_url(&origen, url) {
                bail!(
                    "ya hay un clon de {} en {}: movelo o usá --archivo",
                    sin_credenciales_url(&origen),
                    ruta.parent().unwrap_or(&ruta).display()
                );
            }
        } else {
            let r = sincro::clonar(url, &ruta)?;
            tracing::info!(comando = "clonar", items = r.items.len());
            sincroniza = true;
            recien_clonado = true;
        }
    }
    // El primer clonado no marca nada (REQ-12): no hay un "antes" con qué comparar. Si el
    // cookbook local no se puede leer tampoco hay marcas; el error lo da la lectura de abajo.
    let previo = (sincroniza && !recien_clonado)
        .then(|| archivo::leer(&ruta).ok())
        .flatten();
    if sincroniza && let Err(e) = sincro::traer(&ruta) {
        avisar(&e);
    }
    let antes = if sincroniza {
        sincro::exigir_archivo(&ruta)?;
        Some(archivo::leer(&ruta)?)
    } else {
        None
    };
    let novedades = match (&previo, &antes) {
        (Some(previo), Some(antes)) => servicio::novedades(previo, antes),
        _ => Default::default(),
    };
    let nombre = cli.comando.as_ref().map(Comando::nombre);
    let resultado = ejecutar_comando(
        cli.comando,
        &ruta,
        home,
        novedades,
        cli.url.is_some(),
        revision,
    );
    // También si el comando falló: lo que llegó a guardar no se pierde.
    if let Some(antes) = &antes {
        publicar(&ruta, nombre, antes);
    }
    resultado
}

/// Si no se puede saber si el clon se sincroniza, el comando sigue sin tocar git.
fn sincronizado(ruta: &Path) -> bool {
    sincro::sincronizado(ruta).unwrap_or_else(|e| {
        avisar(&e.context("no pude ver si el cookbook está en un clon de rebake"));
        false
    })
}

fn publicar(ruta: &Path, comando: Option<&str>, antes: &rebake::modelo::Recetario) {
    let despues = match archivo::leer(ruta) {
        Ok(r) => r,
        Err(e) => {
            avisar(&e.context("no pude leer el cookbook para publicarlo"));
            return;
        }
    };
    match sincro::publicar(ruta, &servicio::mensaje_commit(comando, antes, &despues)) {
        Ok(true) => {
            tracing::info!(accion = "publicar", "cookbook publicado en el repo");
            eprintln!("cookbook publicado en el repo");
        }
        Ok(false) => {}
        Err(e) => avisar(&e),
    }
}

/// Un fallo de git no frena el comando: el cookbook ya quedó en disco.
fn avisar(e: &anyhow::Error) {
    tracing::warn!(error = %format!("{e:#}"), "sincronización con el repo");
    eprintln!("aviso: {e:#}");
}

fn ejecutar_comando(
    comando: Option<Comando>,
    ruta: &Path,
    home: &Path,
    novedades: HashMap<String, Novedad>,
    hubo_url: bool,
    revision: &mut Option<Receiver<Option<String>>>,
) -> Result<bool> {
    let Some(comando) = comando else {
        let arranque = rebake::tui::Arranque {
            instalacion: hubo_url,
            novedades,
            revision: revision.take(),
        };
        rebake::tui::ejecutar(ruta, home, arranque)?;
        return Ok(true);
    };
    match comando {
        Comando::Clonar { url } => clonar(&url, ruta, home),
        Comando::Escanear => {
            let mut r = archivo::leer(ruta)?;
            let pacman = PacmanReal;
            let entorno = Entorno::real(&pacman)?;
            let resumen = servicio::escanear(&mut r, &entorno)?;
            archivo::guardar(ruta, &r)?;
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
        Comando::Investigar { ids } => investigar(&mut archivo::leer(ruta)?, ruta, &ids, home),
        Comando::Instalar { dry_run, si } => {
            instalar(&archivo::leer(ruta)?, home, dry_run, si, &novedades)
        }
        Comando::RenombrarPerfil { viejo, nuevo } => {
            let mut r = archivo::leer(ruta)?;
            acciones::renombrar_perfil(&mut r, &viejo, &nuevo)?;
            archivo::guardar(ruta, &r)?;
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
    if ids.is_empty() {
        let sin_origen = servicio::sin_origen(r);
        if !sin_origen.is_empty() {
            println!(
                "{} sin origen: investigalas con `i` en la pantalla o con `rebake investigar <id>`",
                sin_origen.len()
            );
            for id in sin_origen {
                println!("  {id}");
            }
        }
    }
    Ok(fallidos == 0)
}

fn instalar(
    r: &rebake::modelo::Recetario,
    home: &Path,
    dry_run: bool,
    si: bool,
    novedades: &HashMap<String, Novedad>,
) -> Result<bool> {
    let orden = instalador::planificar(r)?;
    if orden.is_empty() {
        println!("no hay recetas aprobadas");
        return Ok(true);
    }
    eprintln!("Va a instalar:");
    for id in &orden {
        let marca = match novedades.get(id) {
            Some(Novedad::Nueva) => "  (nueva)",
            Some(Novedad::Cambio) => "  (cambió)",
            None => "",
        };
        eprintln!("  {id}{marca}");
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
        let item = r.item(id);
        let texto = match res {
            Resultado::Ok if item.is_some_and(|i| i.tipo == Tipo::Mcp) => {
                "a mano: mirá la checklist".to_string()
            }
            Resultado::Ok if item.is_some_and(checklist::tiene_pasos_a_mano) => {
                "instalado; faltan pasos a mano (checklist)".to_string()
            }
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
