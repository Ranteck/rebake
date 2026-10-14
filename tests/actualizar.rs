use rebake::actualizar::{parsear, si_es_nueva};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const ASSET: &str = "rebake-x86_64-unknown-linux-musl.tar.gz";
const PROPIA: &str = env!("CARGO_PKG_VERSION");

/// Servidor mínimo: `/latest` redirige a `/tag/v<publicada>` y `/download/v*/<archivo>` sirve `raiz`.
struct Servidor {
    base: String,
    raiz: tempfile::TempDir,
    pedidos: Arc<Mutex<Vec<String>>>,
}

impl Servidor {
    fn nuevo(publicada: &str) -> Self {
        let raiz = tempfile::tempdir().unwrap();
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("install.sh"),
            raiz.path().join("install.sh"),
        )
        .unwrap();
        publicar(raiz.path(), publicada);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        let pedidos = Arc::new(Mutex::new(Vec::new()));
        let (r, p, v) = (
            raiz.path().to_path_buf(),
            pedidos.clone(),
            publicada.to_string(),
        );
        std::thread::spawn(move || {
            for conexion in listener.incoming().flatten() {
                atender(conexion, &r, &p, &v);
            }
        });
        Servidor {
            base,
            raiz,
            pedidos,
        }
    }

    fn pedidos(&self) -> Vec<String> {
        self.pedidos.lock().unwrap().clone()
    }
}

fn atender(mut conexion: TcpStream, raiz: &Path, pedidos: &Mutex<Vec<String>>, version: &str) {
    let mut linea = String::new();
    BufReader::new(&conexion).read_line(&mut linea).unwrap();
    let ruta = linea.split_whitespace().nth(1).unwrap_or("").to_string();
    pedidos.lock().unwrap().push(ruta.clone());
    let respuesta: Vec<u8> = if ruta == "/latest" {
        format!(
            "HTTP/1.1 302 Found\r\nLocation: /tag/v{version}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )
        .into_bytes()
    } else if ruta.starts_with("/tag/") {
        b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()
    } else if let Some(archivo) = ruta
        .strip_prefix("/download/")
        .and_then(|r| r.split_once('/'))
        .and_then(|(_, a)| fs::read(raiz.join(a)).ok())
    {
        let mut r = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            archivo.len()
        )
        .into_bytes();
        r.extend(archivo);
        r
    } else {
        b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()
    };
    let _ = conexion.write_all(&respuesta);
}

/// Deja en `dir` un release falso cuyo `rebake` imprime `rebake <version>`.
fn publicar(dir: &Path, version: &str) {
    let staging = tempfile::tempdir().unwrap();
    let bin = staging.path().join("rebake");
    fs::write(&bin, format!("#!/bin/sh\necho \"rebake {version}\"\n")).unwrap();
    fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
    let empaquetado = Command::new("tar")
        .arg("-czf")
        .arg(dir.join(ASSET))
        .arg("-C")
        .arg(staging.path())
        .arg("rebake")
        .status()
        .unwrap();
    assert!(empaquetado.success());
    let suma = Command::new("sha256sum")
        .arg(ASSET)
        .current_dir(dir)
        .output()
        .unwrap();
    fs::write(dir.join(format!("{ASSET}.sha256")), suma.stdout).unwrap();
}

/// Una copia del binario compilado, llamada `rebake`, que el test puede reemplazar sin riesgo.
struct Copia {
    dir: tempfile::TempDir,
    home: tempfile::TempDir,
}

impl Copia {
    fn nueva() -> Self {
        let dir = tempfile::tempdir().unwrap();
        // Con `cp` aparte: si el test abriera el archivo para escribir, otro test que hace fork en
        // paralelo heredaría ese descriptor y ejecutar la copia daría "Text file busy".
        let copiado = Command::new("cp")
            .arg(env!("CARGO_BIN_EXE_rebake"))
            .arg(dir.path().join("rebake"))
            .status()
            .unwrap();
        assert!(copiado.success());
        Copia {
            dir,
            home: tempfile::tempdir().unwrap(),
        }
    }

    fn bin(&self) -> PathBuf {
        self.dir.path().join("rebake")
    }

    fn contenido(&self) -> Vec<u8> {
        fs::read(self.bin()).unwrap()
    }

    fn correr(&self, base: &str, args: &[&str], stdin: Option<&str>) -> Output {
        let cookbook = self.home.path().join("cookbook.toml");
        let mut hijo = Command::new(self.bin())
            .args(args)
            .arg("--archivo")
            .arg(cookbook)
            .env("HOME", self.home.path())
            .env("PATH", "/usr/bin:/bin")
            .env("REBAKE_BASE_URL", base)
            .env_remove("CLAUDE_CONFIG_DIR")
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        if let Some(s) = stdin {
            hijo.stdin.take().unwrap().write_all(s.as_bytes()).unwrap();
        }
        hijo.wait_with_output().unwrap()
    }
}

fn texto(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

#[test]
fn compara_versiones_por_numero() {
    assert_eq!(si_es_nueva("0.5.9", "0.5.10"), Some("0.5.10".into()));
    assert_eq!(si_es_nueva("0.5.10", "0.5.9"), None);
    assert_eq!(si_es_nueva("0.5.3", "0.5.3"), None);
    assert_eq!(si_es_nueva("0.5.3", "v0.6.0"), Some("0.6.0".into()));
    assert_eq!(si_es_nueva("0.5.3", "latest"), None);
    assert_eq!(parsear("v0.6.0"), Some((0, 6, 0)));
    assert_eq!(parsear("latest"), None);
    assert_eq!(parsear("1.2"), None);
    assert_eq!(parsear("1.2.3.4"), None);
}

#[test]
fn publicada_igual_no_avisa() {
    let s = Servidor::nuevo(PROPIA);
    let c = Copia::nueva();
    let o = c.correr(&s.base, &["escanear"], None);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(!texto(&o).contains("hay rebake"), "{}", texto(&o));
}

#[test]
fn subcomando_avisa_y_sigue() {
    let s = Servidor::nuevo("99.0.0");
    let c = Copia::nueva();
    let o = c.correr(&s.base, &["escanear"], None);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        texto(&o).contains(&format!(
            "hay rebake 99.0.0 (tenés {PROPIA}): abrí `rebake` para actualizar"
        )),
        "{}",
        texto(&o)
    );
}

#[test]
fn pantalla_con_s_actualiza_y_abre_la_nueva() {
    let s = Servidor::nuevo("99.0.0");
    let c = Copia::nueva();
    let antes = c.contenido();
    let o = c.correr(&s.base, &[], Some("s\n"));
    assert!(texto(&o).contains("rebake 99.0.0"), "{}", texto(&o));
    assert_ne!(c.contenido(), antes);
}

#[test]
fn pantalla_con_enter_no_cambia_nada() {
    let s = Servidor::nuevo("99.0.0");
    let c = Copia::nueva();
    let antes = c.contenido();
    let o = c.correr(&s.base, &[], Some("\n"));
    assert!(
        texto(&o).contains(&format!(
            "Hay rebake 99.0.0 (tenés {PROPIA}). ¿Actualizar? [s/N]"
        )),
        "{}",
        texto(&o)
    );
    assert_eq!(c.contenido(), antes);
}

#[test]
fn checksum_malo_no_actualiza() {
    let s = Servidor::nuevo("99.0.0");
    fs::write(
        s.raiz.path().join(format!("{ASSET}.sha256")),
        format!("{}  {ASSET}\n", "0".repeat(64)),
    )
    .unwrap();
    let c = Copia::nueva();
    let antes = c.contenido();
    let o = c.correr(&s.base, &[], Some("s\n"));
    assert!(texto(&o).contains("no se actualizó"), "{}", texto(&o));
    assert!(texto(&o).contains("checksum"), "{}", texto(&o));
    assert_eq!(c.contenido(), antes);
}

#[test]
fn carpeta_sin_permiso_no_actualiza() {
    // Como root el chmod no impide escribir: el caso no se puede probar.
    if Command::new("id").arg("-u").output().unwrap().stdout == b"0\n" {
        return;
    }
    let s = Servidor::nuevo("99.0.0");
    let c = Copia::nueva();
    let antes = c.contenido();
    fs::set_permissions(c.dir.path(), fs::Permissions::from_mode(0o555)).unwrap();
    let o = c.correr(&s.base, &[], Some("s\n"));
    fs::set_permissions(c.dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(texto(&o).contains("no se actualizó"), "{}", texto(&o));
    assert!(
        texto(&o).contains(&c.dir.path().display().to_string()),
        "{}",
        texto(&o)
    );
    assert_eq!(c.contenido(), antes);
}

#[test]
fn puerto_cerrado_no_avisa() {
    let puerto = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let c = Copia::nueva();
    let o = c.correr(&format!("http://127.0.0.1:{puerto}"), &["escanear"], None);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(!texto(&o).contains("hay rebake"), "{}", texto(&o));
}

#[test]
fn servidor_que_no_responde_no_frena() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    std::thread::spawn(move || {
        // Acepta y calla: las conexiones quedan abiertas hasta que termine el test.
        let mut abiertas = Vec::new();
        for c in listener.incoming().flatten() {
            abiertas.push(c);
        }
    });
    let c = Copia::nueva();
    let t = Instant::now();
    let o = c.correr(&base, &["escanear"], None);
    assert!(t.elapsed() < Duration::from_secs(3), "{:?}", t.elapsed());
    assert!(o.status.success(), "{}", texto(&o));
    assert!(!texto(&o).contains("hay rebake"), "{}", texto(&o));
}

#[test]
fn version_no_consulta_la_red() {
    let s = Servidor::nuevo("99.0.0");
    let c = Copia::nueva();
    let o = Command::new(c.bin())
        .arg("--version")
        .env("HOME", c.home.path())
        .env("REBAKE_BASE_URL", &s.base)
        .output()
        .unwrap();
    assert!(o.status.success());
    assert!(s.pedidos().is_empty(), "{:?}", s.pedidos());
}
