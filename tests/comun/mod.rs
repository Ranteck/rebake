#![allow(dead_code)]
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct HomeFalso {
    pub dir: tempfile::TempDir,
}

impl HomeFalso {
    pub fn nuevo() -> Self {
        HomeFalso {
            dir: tempfile::tempdir().unwrap(),
        }
    }
    pub fn ruta(&self) -> &Path {
        self.dir.path()
    }
    pub fn escribir(&self, rel: &str, contenido: &str) -> PathBuf {
        let p = self.ruta().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, contenido).unwrap();
        p
    }
    pub fn enlazar(&self, rel_enlace: &str, destino: &Path) {
        let p = self.ruta().join(rel_enlace);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(destino, p).unwrap();
    }
    pub fn repo_git(&self, rel: &str, remoto: Option<&str>) -> PathBuf {
        let p = self.ruta().join(rel);
        fs::create_dir_all(&p).unwrap();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .arg("-C")
                    .arg(&p)
                    .args(args)
                    .output()
                    .unwrap()
                    .status
                    .success()
            )
        };
        git(&["init", "-q"]);
        if let Some(url) = remoto {
            git(&["remote", "add", "origin", url]);
        }
        p
    }
    pub fn binario(&self, rel: &str, contenido: &[u8]) -> PathBuf {
        let p = self.ruta().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, contenido).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
        p
    }
}

pub struct PacmanFalso(pub Vec<PathBuf>);

impl rebake::escaner::Pacman for PacmanFalso {
    fn es_del_sistema(&self, binario: &Path) -> bool {
        self.0.iter().any(|p| p == binario)
    }
}

pub fn entorno<'a>(h: &HomeFalso, pacman: &'a PacmanFalso) -> rebake::escaner::Entorno<'a> {
    rebake::escaner::Entorno {
        home: h.ruta().to_path_buf(),
        path: vec![h.ruta().join("bin")],
        pacman,
    }
}

pub fn perfil(nombre: &str, dir: &str) -> rebake::modelo::Perfil {
    rebake::modelo::Perfil {
        nombre: nombre.into(),
        dir: dir.into(),
    }
}

/// Ejecuta el caso completo, incluidos sus git, sin configuración del usuario.
pub fn aislado(nombre: &str) -> bool {
    if std::env::var("REBAKE_TEST_AISLADO").as_deref() == Ok(nombre) {
        return true;
    }
    let h = HomeFalso::nuevo();
    h.escribir(".gitconfig", "CONFIGURACION INVALIDA\n");
    h.escribir("xdg/git/config", "CONFIGURACION XDG INVALIDA\n");
    h.binario(
        "bin/git",
        br#"#!/bin/sh
test -z "${CLAUDE_CONFIG_DIR+x}${GIT_CONFIG_COUNT+x}" || exit 90
test "$GIT_CONFIG_NOSYSTEM" = 1 || exit 91
test "$GIT_CONFIG_GLOBAL" = /dev/null || exit 92
test "$GIT_CONFIG_SYSTEM" = /dev/null || exit 93
printf 'git aislado\n' >> "$HOME/git-aislado"
exec /usr/bin/git "$@"
"#,
    );
    let salida = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", nombre, "--nocapture"])
        .env_clear()
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", h.ruta().join("bin").display()),
        )
        .env("HOME", h.ruta())
        .env("XDG_CONFIG_HOME", h.ruta().join("xdg"))
        .env("XDG_DATA_HOME", h.ruta().join("data"))
        .env("XDG_STATE_HOME", h.ruta().join("state"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("REBAKE_TEST_AISLADO", nombre)
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&salida.stdout),
        String::from_utf8_lossy(&salida.stderr)
    );
    false
}

struct Captura(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
impl std::io::Write for Captura {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn capturar_logs(
    salida: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
) -> impl tracing::Subscriber + Send + Sync {
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
