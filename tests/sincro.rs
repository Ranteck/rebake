mod comun;
use comun::HomeFalso;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const COOKBOOK: &str = r#"version = 1

[[perfil]]
nombre = "laburo"
dir = "~/.claude"

[[item]]
id = "plugin:a@b"
tipo = "plugin"
perfiles = ["laburo"]
estado = "pendiente"
"#;

/// Git aislado de la config del usuario, con identidad y rama por defecto fijas.
fn aislar<'a>(c: &'a mut Command, h: &HomeFalso) -> &'a mut Command {
    c.env("HOME", h.ruta())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "init.defaultBranch")
        .env("GIT_CONFIG_VALUE_0", "main")
        .env("GIT_AUTHOR_NAME", "Prueba")
        .env("GIT_AUTHOR_EMAIL", "prueba@example.com")
        .env("GIT_COMMITTER_NAME", "Prueba")
        .env("GIT_COMMITTER_EMAIL", "prueba@example.com")
}

fn git(h: &HomeFalso, dir: &Path, args: &[&str]) -> String {
    let mut c = Command::new("git");
    c.arg("-C").arg(dir).args(args);
    let o = aislar(&mut c, h).output().unwrap();
    assert!(
        o.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

fn rebake_con(h: &HomeFalso, args: &[&str], entorno: &[(&str, &str)]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_rebake"));
    aislar(&mut c, h)
        .args(args)
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", h.ruta().join("bin").display()),
        )
        .env_remove("CLAUDE_CONFIG_DIR")
        .envs(entorno.iter().copied());
    c.output().unwrap()
}

fn rebake(h: &HomeFalso, args: &[&str]) -> Output {
    rebake_con(h, args, &[])
}

fn texto(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// Repo bare que hace de GitHub; con `cookbook` ya tiene un commit con ese archivo.
fn remoto(h: &HomeFalso, cookbook: Option<&str>) -> PathBuf {
    let bare = h.ruta().join("remoto.git");
    git(h, h.ruta(), &["init", "-q", "--bare", url(&bare)]);
    if let Some(contenido) = cookbook {
        let semilla = h.ruta().join("semilla");
        git(h, h.ruta(), &["clone", "-q", url(&bare), url(&semilla)]);
        fs::write(semilla.join("cookbook.toml"), contenido).unwrap();
        git(h, &semilla, &["add", "cookbook.toml"]);
        git(h, &semilla, &["commit", "-q", "-m", "semilla"]);
        git(h, &semilla, &["push", "-q", "origin", "HEAD"]);
    }
    bare
}

fn carpeta(h: &HomeFalso) -> PathBuf {
    h.ruta().join("Documents/rebake")
}

fn url(ruta: &Path) -> &str {
    ruta.to_str().unwrap()
}

#[test]
fn clonar_en_carpeta_vacia_clona_y_cuenta_recetas() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    fs::create_dir_all(carpeta(&h)).unwrap();
    let o = rebake(&h, &["clonar", url(&bare)]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("clonado en ~/Documents/rebake (1 receta)"),
        "{}",
        texto(&o)
    );
    assert_eq!(
        fs::read_to_string(carpeta(&h).join("cookbook.toml")).unwrap(),
        COOKBOOK
    );
}

#[test]
fn clonar_no_pisa_una_carpeta_con_contenido() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    let propio = h.escribir("Documents/rebake/cookbook.toml", "version = 1\n");
    let o = rebake(&h, &["clonar", url(&bare)]);
    assert!(!o.status.success());
    assert!(texto(&o).contains("no está vacía"), "{}", texto(&o));
    assert_eq!(fs::read_to_string(propio).unwrap(), "version = 1\n");
}

#[test]
fn clonar_un_cookbook_invalido_no_deja_nada() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some("version = 1\ncampo_raro = 1\n"));
    let o = rebake(&h, &["clonar", url(&bare)]);
    assert!(!o.status.success());
    assert!(texto(&o).contains("cookbook.toml:2:1"), "{}", texto(&o));
    let documentos: Vec<_> = fs::read_dir(h.ruta().join("Documents")).unwrap().collect();
    assert!(documentos.is_empty(), "{documentos:?}");
}

#[test]
fn clonar_un_repo_vacio_arranca_sin_recetas() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, None);
    let o = rebake(&h, &["clonar", url(&bare)]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("(0 recetas)"),
        "{}",
        texto(&o)
    );
}

#[test]
fn clonar_no_muestra_credenciales_de_la_url() {
    let h = HomeFalso::nuevo();
    let o = rebake(
        &h,
        &["clonar", "https://usuario:secreto@127.0.0.1:1/repo.git"],
    );
    assert!(!o.status.success());
    let salida = texto(&o);
    assert!(salida.contains("gh auth login"), "{salida}");
    assert!(!salida.contains("secreto"), "{salida}");
    let log = fs::read_to_string(h.ruta().join(".local/state/rebake/rebake.log")).unwrap();
    assert!(!log.contains("secreto"), "{log}");
}
