use std::io::{BufRead, BufReader, Read};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const MARGEN_DRENADO: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct Salida {
    pub codigo: Option<i32>,
    pub texto: String,
    pub vencido: bool,
}

impl Salida {
    pub fn exito(&self) -> bool {
        !self.vencido && self.codigo == Some(0)
    }
}

pub fn ejecutar(
    cmd: &mut Command,
    tope: Duration,
    al_leer: &mut dyn FnMut(&str),
) -> std::io::Result<Salida> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    let mut hijo = lanzar(cmd)?;
    let (tx, rx) = mpsc::channel::<String>();
    let mut fuentes: Vec<Box<dyn Read + Send>> = Vec::new();
    if let Some(s) = hijo.stdout.take() {
        fuentes.push(Box::new(s));
    }
    if let Some(s) = hijo.stderr.take() {
        fuentes.push(Box::new(s));
    }
    let lectores: Vec<_> = fuentes
        .into_iter()
        .map(|fuente| {
            let tx = tx.clone();
            thread::spawn(move || {
                for linea in BufReader::new(fuente).lines() {
                    let Ok(linea) = linea else { break };
                    if tx.send(linea).is_err() {
                        break;
                    }
                }
            })
        })
        .collect();
    drop(tx);

    let mut texto = String::new();
    let mut recibir = |l: String, texto: &mut String| {
        al_leer(&l);
        texto.push_str(&l);
        texto.push('\n');
    };
    let inicio = Instant::now();
    let mut vencido = false;
    let estado = loop {
        if let Some(estado) = hijo.try_wait()? {
            break estado;
        }
        if inicio.elapsed() >= tope {
            vencido = true;
            matar_grupo(hijo.id());
            break hijo.wait()?;
        }
        if let Ok(l) = rx.recv_timeout(Duration::from_millis(50)) {
            recibir(l, &mut texto);
        }
    };
    // Un nieto en segundo plano puede dejar la salida abierta y bloquear a los lectores.
    matar_grupo(hijo.id());
    // Un proceso que escapó a otra sesión (setsid) puede mantener la salida abierta para
    // siempre: se drena lo que quede durante un margen corto y los lectores se sueltan.
    let limite = Instant::now() + MARGEN_DRENADO;
    loop {
        let resto = limite.saturating_duration_since(Instant::now());
        match rx.recv_timeout(resto) {
            Ok(l) => recibir(l, &mut texto),
            Err(_) => break,
        }
    }
    drop(lectores);
    Ok(Salida {
        codigo: estado.code(),
        texto,
        vencido,
    })
}

/// ETXTBSY es transitorio: otro hilo hizo fork mientras se escribía el ejecutable y ese hijo
/// todavía no llegó a exec (rust-lang/rust#114554). Pasa al ejecutar un binario recién escrito.
fn lanzar(cmd: &mut Command) -> std::io::Result<Child> {
    let mut espera = Duration::from_millis(10);
    for _ in 0..8 {
        match cmd.spawn() {
            Err(e) if e.kind() == std::io::ErrorKind::ExecutableFileBusy => {
                thread::sleep(espera);
                espera *= 2;
            }
            otro => return otro,
        }
    }
    cmd.spawn()
}

fn matar_grupo(pid: u32) {
    // PID negativo = el grupo creado con process_group(0). Si el grupo ya no existe, `kill`
    // falla con "No such process", que es el caso normal y no hay nada que hacer.
    let _ = Command::new("kill")
        .args(["-KILL", "--", &format!("-{pid}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
