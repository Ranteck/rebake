mod comun;
use comun::*;
use rebake::checklist;
use rebake::escaner::escanear;
use rebake::modelo::*;
use serde_json::json;

fn valor(a: &[Ajuste], clave: &str) -> Option<String> {
    a.iter().find(|x| x.clave == clave).map(|x| x.valor.clone())
}

#[test]
fn ajustes_sin_claves_de_instaladores() {
    let s = json!({
        "enabledPlugins": {"codex@openai-codex": true},
        "extraKnownMarketplaces": {},
        "hooks": {"PreToolUse": []},
        "statusLine": {"type": "command", "command": "x"},
        "modelSettings": {"claude-opus-5-5": {"effortLevel": "xhigh"}},
        "voice": {"enabled": true, "mode": "hold"},
        "autoMode": {"allow": ["$defaults", "Bash(npx vitest:*)"]}
    });
    let a = checklist::ajustes("laburo", &s);
    assert_eq!(
        valor(&a, "modelSettings.claude-opus-5-5.effortLevel").as_deref(),
        Some("xhigh")
    );
    assert_eq!(valor(&a, "voice.enabled").as_deref(), Some("true"));
    assert_eq!(
        valor(&a, "autoMode.allow[1]").as_deref(),
        Some("Bash(npx vitest:*)")
    );
    for prohibida in [
        "enabledPlugins",
        "extraKnownMarketplaces",
        "hooks",
        "statusLine",
    ] {
        assert!(
            a.iter().all(|x| !x.clave.starts_with(prohibida)),
            "{prohibida}"
        );
    }
    assert!(a.iter().all(|x| x.perfil == "laburo"));
}

#[test]
fn env_y_claves_sensibles_ocultas() {
    let s = json!({"env": {"FOO": "bar"}, "apiKeyHelper": "/bin/x", "github": {"Token": "abc"}, "theme": "dark"});
    let a = checklist::ajustes("laburo", &s);
    assert_eq!(valor(&a, "env.FOO").as_deref(), Some(OCULTO));
    assert_eq!(valor(&a, "apiKeyHelper").as_deref(), Some(OCULTO));
    assert_eq!(valor(&a, "github.Token").as_deref(), Some(OCULTO));
    assert_eq!(valor(&a, "theme").as_deref(), Some("dark"));
}

#[test]
fn titulos_sin_imports() {
    let t = checklist::titulos(
        "laburo",
        "@RTK.md\n@HOUSE-RULES.md\n\n## Skills de proceso\ntexto\n### Correcciones de inglés\n",
    );
    let textos: Vec<_> = t.iter().map(|x| x.texto.as_str()).collect();
    assert_eq!(textos, ["Skills de proceso", "Correcciones de inglés"]);
}

#[test]
fn logins_y_pasos_manuales() {
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        Perfil {
            nombre: "laburo".into(),
            dir: "~/.claude".into(),
        },
        Perfil {
            nombre: "personal".into(),
            dir: "~/.claude-personal".into(),
        },
    ];
    let manual = Paso {
        cmd: "/codex:setup".into(),
        modo: Modo::Manual,
        por_perfil: false,
        cita: None,
        nota: None,
    };
    let mut codex = Item::nuevo("plugin:codex@openai-codex", Tipo::Plugin);
    codex.estado = Estado::Aprobada;
    codex.pasos = vec![manual.clone()];
    let mut otro = Item::nuevo("plugin:otro@x", Tipo::Plugin);
    otro.pasos = vec![manual.clone()];
    r.items = vec![codex, otro];

    let logins = checklist::logins(&r);
    assert!(logins.iter().any(|l| l.contains("gh auth login")));
    assert!(
        logins
            .iter()
            .any(|l| l.contains("CLAUDE_CONFIG_DIR=~/.claude-personal"))
    );
    assert_eq!(logins.len(), 3);
    assert_eq!(
        checklist::pasos_manuales(&r),
        vec![("plugin:codex@openai-codex".to_string(), manual)]
    );
}

#[test]
fn no_filtra_secretos() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/.credentials.json", r#"{"token": "SECRETO-XYZ"}"#);
    h.escribir(".claude.json", r#"{"oauthAccount": "SECRETO-XYZ"}"#);
    h.escribir(".codex/auth.json", r#"{"key": "SECRETO-XYZ"}"#);
    h.escribir(
        ".claude/settings.json",
        r#"{"env": {"API_TOKEN": "SECRETO-XYZ"}, "github": {"token": "SECRETO-XYZ"}}"#,
    );
    h.escribir(".claude/CLAUDE.md", "## Propio\n");
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    assert!(!format!("{e:?}").contains("SECRETO-XYZ"));
    assert_eq!(valor(&e.ajustes, "env.API_TOKEN").as_deref(), Some(OCULTO));
    assert_eq!(e.titulos.len(), 1);
}

#[test]
fn valores_con_forma_de_secreto_ocultos() {
    let s = json!({"awsCredentialExport": "/bin/export", "model": "sk-ant-api03-abcdefghijkl", "otelHeadersHelper": "/bin/h", "theme": "dark"});
    let a = checklist::ajustes("laburo", &s);
    for clave in ["awsCredentialExport", "model", "otelHeadersHelper"] {
        assert_eq!(valor(&a, clave).as_deref(), Some(OCULTO), "{clave}");
    }
    assert_eq!(valor(&a, "theme").as_deref(), Some("dark"));
}
