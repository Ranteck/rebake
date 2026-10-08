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
