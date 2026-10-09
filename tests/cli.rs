mod comun;
use comun::HomeFalso;
use rebake::{archivo, modelo::Estado};
use std::process::{Command, Output};

fn recetario(h: &HomeFalso, args: &[&str], claude: Option<&str>) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_rebake"));
    c.args(args)
        .env("HOME", h.ruta())
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", h.ruta().join("bin").display()),
        )
        .env_remove("CLAUDE_CONFIG_DIR");
    if let Some(cl) = claude {
        c.env("REBAKE_CLAUDE", cl);
    }
    c.output().unwrap()
}

fn texto(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

#[test]
fn escanear_crea_recetario() {
    let h = HomeFalso::nuevo();
    h.escribir(
        ".claude/settings.json",
        r#"{"enabledPlugins": {"codex@openai-codex": true}, "theme": "dark"}"#,
    );
    let o = recetario(&h, &["escanear"], None);
    assert!(o.status.success(), "{}", texto(&o));
    let r = archivo::leer(&h.ruta().join("Documents/rebake/cookbook.toml")).unwrap();
    assert_eq!(r.perfiles[0].dir, "~/.claude");
    assert!(r.item("plugin:codex@openai-codex").is_some());
    assert!(r.checklist.ajustes.iter().any(|a| a.clave == "theme"));
}

const RESPUESTA: &str = r#"{"type":"result","subtype":"success","is_error":false,"structured_output":{"repo":"https://github.com/ejemplo/house-rules","doc":"https://github.com/ejemplo/house-rules#install","verificar":"test -f \"$CLAUDE_CONFIG_DIR/instalado\"","pasos":[{"cmd":"touch \"$CLAUDE_CONFIG_DIR/instalado\"","modo":"auto","por_perfil":true,"cita":"install.sh"},{"cmd":"/algo:setup","modo":"manual","por_perfil":false,"cita":"/algo:setup"}],"requiere":[]}}"#;

#[test]
fn de_punta_a_punta_con_claude_falso() {
    let h = HomeFalso::nuevo();
    let repo = h.repo_git(
        "Proyectos/house-rules",
        Some("https://github.com/ejemplo/house-rules"),
    );
    h.escribir("Proyectos/house-rules/HOUSE-RULES.md", "# reglas\n");
    h.enlazar(".claude/HOUSE-RULES.md", &repo.join("HOUSE-RULES.md"));
    h.escribir(".claude/CLAUDE.md", "@HOUSE-RULES.md\n");
    h.escribir(".claude/settings.json", "{}");
    let claude = h.binario(
        "bin/claude-falso",
        format!("#!/bin/sh\ncat <<'FIN'\n{RESPUESTA}\nFIN\n").as_bytes(),
    );
    let claude = claude.to_str().unwrap();
    let ruta = h.ruta().join("Documents/rebake/cookbook.toml");
    let id = "import:HOUSE-RULES.md";

    assert!(recetario(&h, &["escanear"], None).status.success());
    let o = recetario(&h, &["investigar", id], Some(claude));
    assert!(o.status.success(), "{}", texto(&o));
    let mut r = archivo::leer(&ruta).unwrap();
    assert_eq!(r.item(id).unwrap().estado, Estado::PorRevisar);

    rebake::acciones::aprobar(&mut r, id).unwrap();
    archivo::guardar(&ruta, &r).unwrap();

    let o = recetario(&h, &["instalar", "--dry-run"], None);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(!h.ruta().join(".claude/instalado").exists());

    let o = recetario(&h, &["instalar", "--si"], None);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(h.ruta().join(".claude/instalado").exists());
    let salida = String::from_utf8_lossy(&o.stdout);
    assert!(salida.contains("/algo:setup"), "{salida}");
    assert!(salida.contains("gh auth login"), "{salida}");

    let o = recetario(&h, &["instalar", "--si"], None);
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("salteado"),
        "{}",
        texto(&o)
    );
}
