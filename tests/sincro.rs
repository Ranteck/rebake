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

/// Clona `bare` con rebake y devuelve la carpeta del clon.
fn clonado(h: &HomeFalso, bare: &Path) -> PathBuf {
    let o = rebake(h, &["clonar", url(bare)]);
    assert!(o.status.success(), "{}", texto(&o));
    carpeta(h)
}

fn commits(h: &HomeFalso, repo: &Path) -> String {
    git(h, repo, &["rev-list", "--count", "--all"])
}

fn ultimo_mensaje(h: &HomeFalso, bare: &Path) -> String {
    git(h, bare, &["log", "-1", "--format=%s", "main"])
}

#[test]
fn un_cambio_se_commitea_solo_y_llega_al_repo() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    let clon = clonado(&h, &bare);
    fs::write(clon.join("notas.md"), "algo\n").unwrap();
    git(&h, &clon, &["add", "notas.md"]);
    let o = rebake(&h, &["renombrar-perfil", "laburo", "casa"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert_eq!(
        ultimo_mensaje(&h, &bare),
        "rebake renombrar-perfil: actualiza el cookbook"
    );
    assert_eq!(
        git(&h, &bare, &["show", "--name-only", "--format=", "main"]),
        "cookbook.toml"
    );
}

#[test]
fn sin_cambios_no_hay_commit() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    clonado(&h, &bare);
    let o = rebake(&h, &["instalar", "--dry-run"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert_eq!(commits(&h, &bare), "1");
}

#[test]
fn un_cookbook_enlazado_a_otro_repo_no_se_toca() {
    let h = HomeFalso::nuevo();
    let dotfiles = h.ruta().join("dotfiles");
    fs::create_dir_all(&dotfiles).unwrap();
    git(&h, &dotfiles, &["init", "-q"]);
    let real = h.escribir("dotfiles/cookbook.toml", COOKBOOK);
    h.enlazar("Documents/rebake/cookbook.toml", &real);
    let o = rebake(&h, &["renombrar-perfil", "laburo", "casa"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(fs::read_to_string(&real).unwrap().contains("casa"));
    assert_eq!(commits(&h, &dotfiles), "0");
}

#[test]
fn un_repo_vacio_recibe_el_cookbook_en_el_primer_guardado() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, None);
    clonado(&h, &bare);
    h.escribir(".claude/settings.json", "{}");
    let o = rebake(&h, &["escanear"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        ultimo_mensaje(&h, &bare).starts_with("rebake escanear"),
        "{}",
        texto(&o)
    );
    git(&h, &bare, &["cat-file", "-e", "main:cookbook.toml"]);
}

#[test]
fn lo_guardado_se_publica_aunque_el_comando_falle() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    clonado(&h, &bare);
    let claude = h.binario("bin/claude-roto", b"#!/bin/sh\nexit 1\n");
    let o = rebake_con(
        &h,
        &["investigar"],
        &[("REBAKE_CLAUDE", claude.to_str().unwrap())],
    );
    assert!(!o.status.success(), "{}", texto(&o));
    assert_eq!(commits(&h, &bare), "2");
}

#[test]
fn sin_remoto_queda_commiteado_y_sale_en_la_corrida_siguiente() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    let clon = clonado(&h, &bare);
    let apagado = h.ruta().join("apagado.git");
    fs::rename(&bare, &apagado).unwrap();
    let o = rebake(&h, &["renombrar-perfil", "laburo", "casa"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        texto(&o).contains("se pushea en la próxima corrida"),
        "{}",
        texto(&o)
    );
    assert!(git(&h, &clon, &["log", "-1", "--format=%s"]).starts_with("rebake renombrar-perfil"));
    fs::rename(&apagado, &bare).unwrap();
    let o = rebake(&h, &["instalar", "--dry-run"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(ultimo_mensaje(&h, &bare).starts_with("rebake renombrar-perfil"));
}

#[test]
fn un_commit_fallido_se_hace_en_la_corrida_siguiente() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    let clon = clonado(&h, &bare);
    let o = rebake_con(
        &h,
        &["renombrar-perfil", "laburo", "casa"],
        &[("GIT_AUTHOR_NAME", ""), ("GIT_COMMITTER_NAME", "")],
    );
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        texto(&o).contains("se commitea en la próxima corrida"),
        "{}",
        texto(&o)
    );
    assert!(
        fs::read_to_string(clon.join("cookbook.toml"))
            .unwrap()
            .contains("casa")
    );
    assert_eq!(commits(&h, &bare), "1");
    let o = rebake(&h, &["instalar", "--dry-run"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert_eq!(commits(&h, &bare), "2");
}

#[test]
fn trae_lo_que_otro_equipo_subio() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    clonado(&h, &bare);
    let otro = h.ruta().join("otro");
    git(&h, h.ruta(), &["clone", "-q", url(&bare), url(&otro)]);
    let aprobado = COOKBOOK.replace("estado = \"pendiente\"", "estado = \"aprobada\"");
    fs::write(otro.join("cookbook.toml"), aprobado).unwrap();
    git(&h, &otro, &["commit", "-q", "-am", "otro equipo"]);
    git(&h, &otro, &["push", "-q", "origin", "HEAD"]);
    let o = rebake(&h, &["instalar", "--dry-run"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("plugin:a@b: simulado"),
        "{}",
        texto(&o)
    );
}

#[test]
fn sin_red_avisa_y_sigue_con_la_copia_local() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    clonado(&h, &bare);
    fs::rename(&bare, h.ruta().join("apagado.git")).unwrap();
    let o = rebake(&h, &["instalar", "--dry-run"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        texto(&o).contains("sigo con la copia local"),
        "{}",
        texto(&o)
    );
}
