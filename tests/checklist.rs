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
fn logins_y_pasos_a_mano() {
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
    // Solo la aprobada; un comando de Claude sin perfiles dice cómo abrirlo.
    assert_eq!(
        checklist::a_mano(&r),
        vec![(
            "revisá codex@openai-codex (plugin): instalada, pero falta esto".to_string(),
            vec!["abrí `claude` y corré /codex:setup".to_string()]
        )]
    );
}

#[test]
fn un_comando_de_claude_por_perfil_dice_como_abrir_cada_perfil() {
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        Perfil {
            nombre: "claude".into(),
            dir: "~/.claude".into(),
        },
        Perfil {
            nombre: "personal".into(),
            dir: "~/.claude-personal".into(),
        },
    ];
    let mut ge = Item::nuevo("skill:graph-engineer", Tipo::Skill);
    ge.estado = Estado::Aprobada;
    ge.perfiles = vec!["claude".into(), "personal".into()];
    ge.pasos = vec![
        Paso {
            cmd: "npx skills add x".into(),
            modo: Modo::Auto,
            por_perfil: true,
            cita: None,
            nota: None,
        },
        Paso {
            cmd: "/codex:setup".into(),
            modo: Modo::Manual,
            por_perfil: true,
            cita: None,
            nota: None,
        },
    ];
    r.items = vec![ge];
    assert_eq!(
        checklist::a_mano(&r),
        vec![(
            "revisá graph-engineer (skill): instalada, pero falta esto".to_string(),
            vec![
                "[claude] abrí `claude` y corré /codex:setup".to_string(),
                "[personal] abrí `CLAUDE_CONFIG_DIR=~/.claude-personal claude` y corré /codex:setup"
                    .to_string(),
            ]
        )]
    );
    let texto = rebake::servicio::texto_checklist(&r);
    assert!(
        texto.contains("Pasos a mano (rebake no los corre)"),
        "{texto}"
    );
    assert!(!texto.contains("npx skills add"), "{texto}");
}

#[test]
fn no_filtra_secretos() {
    if !comun::aislado("no_filtra_secretos") {
        return;
    }
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

#[test]
fn los_servidores_mcp_aprobados_van_a_pendientes() {
    let mut r = Recetario::nuevo();
    let mut con_pasos = Item::nuevo("mcp:serena", Tipo::Mcp);
    con_pasos.estado = Estado::Aprobada;
    con_pasos.pasos = vec![Paso {
        cmd: "uv tool install -p 3.13 serena-agent".into(),
        modo: Modo::Manual,
        por_perfil: false,
        cita: None,
        nota: None,
    }];
    let mut sin_pasos = Item::nuevo("mcp:remoto", Tipo::Mcp);
    sin_pasos.estado = Estado::Aprobada;
    sin_pasos.pista = Some("servidor MCP: https://mcp.ejemplo.com".into());
    let mut pendiente = Item::nuevo("mcp:otro", Tipo::Mcp);
    pendiente.pista = Some("servidor MCP: otro".into());
    r.items = vec![con_pasos, sin_pasos, pendiente];

    assert_eq!(
        checklist::a_mano(&r),
        vec![
            (
                "revisá serena (servidor MCP)".to_string(),
                vec!["uv tool install -p 3.13 serena-agent".to_string()]
            ),
            (
                "revisá remoto (servidor MCP)".to_string(),
                vec!["servidor MCP: https://mcp.ejemplo.com".to_string()]
            ),
        ]
    );
    let texto = rebake::servicio::texto_checklist(&r);
    assert!(texto.contains("Pasos a mano (rebake no los corre)"));
    assert_eq!(texto.matches("uv tool install").count(), 1, "{texto}");
    assert!(texto.contains("[ ] revisá serena (servidor MCP)"));
    assert!(texto.contains("uv tool install -p 3.13 serena-agent"));
    assert!(!texto.contains("otro"));
}

#[test]
fn el_registro_de_un_servidor_mcp_sale_una_vez_por_perfil() {
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        Perfil {
            nombre: "claude".into(),
            dir: "~/.claude".into(),
        },
        Perfil {
            nombre: "personal".into(),
            dir: "~/.claude-personal".into(),
        },
    ];
    let paso = |cmd: &str, por_perfil| Paso {
        cmd: cmd.into(),
        modo: Modo::Manual,
        por_perfil,
        cita: None,
        nota: None,
    };
    let mut serena = Item::nuevo("mcp:serena", Tipo::Mcp);
    serena.estado = Estado::Aprobada;
    serena.perfiles = vec!["claude".into(), "personal".into()];
    serena.pasos = vec![
        paso("uv tool install serena-agent", false),
        paso(
            "claude mcp get serena || claude mcp add --scope user serena -- serena",
            true,
        ),
    ];
    r.items = vec![serena];
    assert_eq!(
        checklist::a_mano(&r)[0].1,
        [
            "uv tool install serena-agent",
            // Sin la variable: así Claude Code usa `~/.claude.json`.
            "[claude] claude mcp get serena || claude mcp add --scope user serena -- serena",
            "[personal] (export CLAUDE_CONFIG_DIR=~/.claude-personal; claude mcp get serena || claude mcp add --scope user serena -- serena)",
        ]
    );
}
#[test]
fn hooks_externos_se_agrupan_sin_comandos_ni_aprobacion() {
    use rebake::modelo::{InstaladorHook, Item, Recetario, Tipo};
    let mut r = Recetario::nuevo();
    for n in 0..15 {
        let mut hook = Item::nuevo(&format!("hook:comando-privado-{n}"), Tipo::Hook);
        hook.instalado_por = Some(InstaladorHook::Externo("orca".into()));
        hook.perfiles = vec!["laburo".into(), "personal".into()];
        r.items.push(hook);
    }
    let texto = rebake::servicio::texto_checklist(&r);
    let filas: Vec<_> = texto.lines().filter(|s| s.contains("15 hooks")).collect();
    assert_eq!(filas.len(), 1, "{texto}");
    for palabra in ["orca", "laburo", "personal", "instalar o abrir"] {
        assert!(filas[0].contains(palabra), "{texto}");
    }
    assert!(!texto.contains("comando-privado"));
    let app = rebake::tui::app::App::nueva(r);
    assert_eq!(
        app.filas_checklist()
            .iter()
            .filter(|f| f.texto.contains("15 hooks"))
            .count(),
        1
    );
}

#[test]
fn pasos_manuales_omiten_sincronizados_y_destinos_mixtos_de_cuenta() {
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        perfil("laburo", "~/.claude"),
        perfil("personal", "~/.claude-personal"),
    ];
    let mut i = Item::nuevo("plugin:mixto@m", Tipo::Plugin);
    i.perfiles = vec!["laburo".into(), "personal".into()];
    i.sincronizado_en = vec!["laburo".into()];
    i.estado = Estado::Aprobada;
    for (cmd, por_perfil) in [("/setup", false), ("echo ajuste", true)] {
        i.pasos = vec![Paso {
            cmd: cmd.into(),
            por_perfil,
            modo: Modo::Manual,
            cita: None,
            nota: None,
        }];
        r.items = vec![i.clone()];
        let filas = checklist::a_mano(&r);
        assert_eq!(filas.len(), 1);
        assert_eq!(filas[0].1.len(), 1);
        assert!(filas[0].1[0].contains("personal"), "{filas:?}");
        assert!(!filas[0].1[0].contains("laburo"), "{filas:?}");
    }
    r.items[0].sincronizado_en.push("personal".into());
    assert!(checklist::a_mano(&r).is_empty());
}
