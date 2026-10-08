mod comun;
use comun::*;
use recetario::escaner::{comandos::palabras, escanear};
use recetario::modelo::Tipo;
use std::path::Path;

#[test]
fn palabras_respeta_comillas_y_home() {
    let home = Path::new("/home/x");
    assert_eq!(
        palabras("bash \"$HOME/.claude/statusline.sh\" 'a b'", home),
        ["bash", "/home/x/.claude/statusline.sh", "a b"]
    );
    assert_eq!(
        palabras("~/bin/tool ${HOME}/y", home),
        ["/home/x/bin/tool", "/home/x/y"]
    );
}

#[test]
fn binario_de_pacman_no_aparece() {
    let h = HomeFalso::nuevo();
    let jq = h.binario("bin/jq", b"#!/bin/sh\n");
    h.escribir(".claude/settings.json", r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "jq --version"}]}]}}"#);
    let pacman = PacmanFalso(vec![jq]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    assert!(e.hallazgos.iter().all(|x| x.id != "herramienta:jq"));
    assert!(e.hallazgos.iter().any(|x| x.id == "hook:jq --version"));
}

#[test]
fn statusline_con_comillas_y_home() {
    let h = HomeFalso::nuevo();
    let bash = h.binario("bin/bash", b"#!/bin/sh\n");
    h.escribir(".claude/statusline.sh", "#!/bin/bash\n");
    h.escribir(
        ".claude/settings.json",
        r#"{"statusLine": {"type": "command", "command": "bash \"$HOME/.claude/statusline.sh\""}}"#,
    );
    let pacman = PacmanFalso(vec![bash]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    let s = e
        .hallazgos
        .iter()
        .find(|x| x.id == "statusline:statusline.sh")
        .expect("falta la statusline");
    assert_eq!(s.tipo, Tipo::Statusline);
    assert_eq!(
        s.pista.as_deref(),
        Some("archivo sin origen conocido: ~/.claude/statusline.sh")
    );
    assert!(e.hallazgos.iter().all(|x| x.id != "herramienta:bash"));
}

#[test]
fn hook_crea_herramienta_con_url_del_binario() {
    let h = HomeFalso::nuevo();
    h.binario("bin/rtk", b"\x7fELF...https://github.com/ejemplo/rtk ... https://github.com/ejemplo/rtk/issues ... https://github.com/clap-rs/clap\0");
    h.escribir(".claude/settings.json", r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "rtk hook claude"}]}]}}"#);
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    let rtk = e
        .hallazgos
        .iter()
        .find(|x| x.id == "herramienta:rtk")
        .expect("falta rtk");
    assert_eq!(
        rtk.fuente.as_ref().unwrap().repo,
        "https://github.com/ejemplo/rtk"
    );
    assert!(rtk.perfiles.is_empty());
    let hook = e
        .hallazgos
        .iter()
        .find(|x| x.id == "hook:rtk hook claude")
        .unwrap();
    assert!(hook.requiere.contains(&"herramienta:rtk".to_string()));
    assert_eq!(hook.perfiles, ["laburo"]);
}

#[test]
fn claude_siempre_esta() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/settings.json", "{}");
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    let c = e.hallazgos.iter().find(|x| x.id == "claude").unwrap();
    assert_eq!(c.tipo, Tipo::Claude);
}

#[test]
fn hook_con_secreto_no_se_guarda() {
    let h = HomeFalso::nuevo();
    h.binario("bin/rtk", b"\x7fELF https://github.com/ejemplo/rtk\0");
    h.escribir(".claude/settings.json", r#"{"hooks": {"PreToolUse": [{"hooks": [{"type": "command", "command": "API_TOKEN=supersecreto rtk hook claude"}]}]}}"#);
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    assert!(!format!("{e:?}").contains("supersecreto"), "{e:?}");
    assert!(e.hallazgos.iter().any(|x| x.id == "herramienta:rtk"));
}
