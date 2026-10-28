mod comun;
use comun::*;
use rebake::escaner::{comandos::palabras, escanear};
use rebake::modelo::Tipo;
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
fn palabras_no_expande_home_literal_entre_comillas() {
    assert_eq!(
        palabras(
            "printf '$HOME' '~/.orca/hook.sh' \"~/.orca/hook.sh\"",
            Path::new("/home/falso")
        ),
        ["printf", "$HOME", "~/.orca/hook.sh", "~/.orca/hook.sh"]
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
fn claude_code_es_requisito_y_no_receta() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/settings.json", "{}");
    h.escribir(".claude/skills/propia/SKILL.md", "x");
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    assert!(e.hallazgos.iter().all(|x| x.tipo != Tipo::Claude));
    assert!(e.hallazgos.iter().any(|x| x.id == "skill:propia"));
    assert!(e.hallazgos.iter().all(|x| x.requiere.is_empty()));
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

#[test]
fn hook_con_script_resuelve_repo_y_usa_tilde() {
    let h = HomeFalso::nuevo();
    let sh = h.binario("bin/sh", b"#!/bin/sh\n");
    let repo = h.repo_git("Proyectos/sdlc", Some("https://github.com/ejemplo/sdlc"));
    h.escribir(
        "Proyectos/sdlc/skills/sdlc/scripts/cargar.sh",
        "#!/bin/sh\n",
    );
    h.enlazar(".claude/skills/sdlc", &repo.join("skills/sdlc"));
    let script = h.ruta().join(".claude/skills/sdlc/scripts/cargar.sh");
    let settings = format!(
        r#"{{"hooks": {{"SessionStart": [{{"hooks": [{{"type": "command", "command": "sh '{}'"}}]}}]}}}}"#,
        script.display()
    );
    h.escribir(".claude/settings.json", &settings);
    let pacman = PacmanFalso(vec![sh]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    let hook = e
        .hallazgos
        .iter()
        .find(|x| x.tipo == Tipo::Hook)
        .expect("falta el hook");
    assert_eq!(hook.id, "hook:sh '~/.claude/skills/sdlc/scripts/cargar.sh'");
    assert_eq!(
        hook.fuente.as_ref().unwrap().repo,
        "https://github.com/ejemplo/sdlc"
    );
    assert!(hook.pista.is_none());
}

#[test]
fn statusline_que_es_el_script_queda_como_statusline() {
    let h = HomeFalso::nuevo();
    h.binario(".claude/statusline.sh", b"#!/bin/bash\n");
    h.escribir(
        ".claude/settings.json",
        r#"{"statusLine": {"type": "command", "command": "~/.claude/statusline.sh"}}"#,
    );
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    let s = e
        .hallazgos
        .iter()
        .find(|x| x.id == "statusline:statusline.sh")
        .expect("falta la statusline");
    assert_eq!(s.perfiles, ["laburo"]);
    assert!(
        e.hallazgos.iter().all(|x| x.tipo != Tipo::Herramienta),
        "{:?}",
        e.hallazgos
    );
}

#[test]
fn hook_que_es_el_script_resuelve_su_repo() {
    let h = HomeFalso::nuevo();
    let repo = h.repo_git(
        "Proyectos/avisos",
        Some("https://github.com/ejemplo/avisos"),
    );
    h.binario("Proyectos/avisos/notify.sh", b"#!/bin/sh\n");
    h.enlazar(".claude/hooks/notify.sh", &repo.join("notify.sh"));
    h.escribir(".claude/settings.json", r#"{"hooks": {"Stop": [{"hooks": [{"type": "command", "command": "~/.claude/hooks/notify.sh"}]}]}}"#);
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    let hook = e.hallazgos.iter().find(|x| x.tipo == Tipo::Hook).unwrap();
    assert_eq!(
        hook.fuente.as_ref().unwrap().repo,
        "https://github.com/ejemplo/avisos"
    );
    assert!(
        e.hallazgos.iter().all(|x| x.tipo != Tipo::Herramienta),
        "{:?}",
        e.hallazgos
    );
}

#[test]
fn hook_con_script_local_no_es_herramienta() {
    let h = HomeFalso::nuevo();
    h.binario(".claude/hooks/local.sh", b"#!/bin/sh\necho hola\n");
    h.escribir(".claude/settings.json", r#"{"hooks": {"Stop": [{"hooks": [{"type": "command", "command": "~/.claude/hooks/local.sh"}]}]}}"#);
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    let hook = e.hallazgos.iter().find(|x| x.tipo == Tipo::Hook).unwrap();
    assert_eq!(
        hook.pista.as_deref(),
        Some("archivo sin origen conocido: ~/.claude/hooks/local.sh")
    );
    assert!(
        e.hallazgos.iter().all(|x| x.tipo != Tipo::Herramienta),
        "{:?}",
        e.hallazgos
    );
}
#[test]
fn vincula_hooks_por_ejecucion_y_conserva_scripts_propios() {
    use rebake::{modelo::Recetario, servicio};
    for (comando, tipo, nombre) in [
        ("TOKEN=secreto rtk hook claude", "item", "herramienta:rtk"),
        (
            "~/.claude/skills/impeccable/scripts/impeccable hook",
            "item",
            "skill:impeccable",
        ),
        (
            "[ -f ~/.orca/hook.sh ] || ~/.orca/hook.sh",
            "externo",
            "orca",
        ),
        (
            "if [ -f \"${HOME-}/.orca/hook.sh\" ]; then \"${HOME-}/.orca/hook.sh\"; fi",
            "externo",
            "orca",
        ),
        (
            "sh ~/.claude/skills/ai-native-sdlc/scripts/load-intent.sh",
            "",
            "",
        ),
        ("sh ~/.claude/sync-plugins.sh", "", ""),
        ("echo ~/.orca/hook.sh", "", ""),
        ("'~/.orca/hook.sh'", "", ""),
        ("'$HOME/.orca/hook.sh'", "", ""),
        (
            "~/.claude/skills/impeccable/../other/impeccable hook",
            "",
            "",
        ),
    ] {
        let h = HomeFalso::nuevo();
        h.binario("bin/rtk", b"#!/bin/sh\n");
        h.escribir(".claude/skills/impeccable/SKILL.md", "skill");
        h.escribir(".claude/skills/ai-native-sdlc/SKILL.md", "skill");
        h.escribir(".claude/settings.json", &serde_json::json!({"hooks":{"Stop":[{"hooks":[{"type":"command","command":comando}]}]}}).to_string());
        let pacman = PacmanFalso(vec![]);
        let mut r = Recetario::nuevo();
        servicio::escanear(&mut r, &entorno(&h, &pacman)).unwrap();
        let hook = r.items.iter().find(|i| i.tipo == Tipo::Hook).unwrap();
        let valor = serde_json::to_value(hook).unwrap();
        if tipo.is_empty() {
            assert!(valor.get("instalado_por").is_none(), "{comando}: {valor}");
        } else {
            assert_eq!(valor["instalado_por"]["tipo"], tipo, "{comando}");
            assert_eq!(valor["instalado_por"]["nombre"], nombre, "{comando}");
        }
    }
}

#[test]
fn ruta_absoluta_solo_vincula_al_ejecutable_del_path() {
    use rebake::{modelo::Recetario, servicio};
    for (ruta, vinculado) in [("bin/rtk", true), ("otro/rtk", false)] {
        let h = HomeFalso::nuevo();
        h.binario("bin/rtk", b"\x7fELF");
        let ejecutable = h.binario(ruta, b"\x7fELF");
        h.escribir(".claude/settings.json", &serde_json::json!({"hooks":{"Stop":[{"hooks":[{"command":format!("{} hook", ejecutable.display())}]}]}}).to_string());
        let pacman = PacmanFalso(vec![]);
        let mut r = Recetario::nuevo();
        servicio::escanear(&mut r, &entorno(&h, &pacman)).unwrap();
        let hook = r.items.iter().find(|i| i.tipo == Tipo::Hook).unwrap();
        assert_eq!(hook.instalado_por.is_some(), vinculado, "{ruta}");
    }
}
#[test]
fn nuevos_hooks_invalidan_receta_del_responsable_y_preservan_exclusiones() {
    use rebake::{modelo::*, servicio};
    let h = HomeFalso::nuevo();
    h.binario("bin/rtk", b"#!/bin/sh\n");
    let settings = r#"{"hooks":{"Stop":[{"hooks":[{"command":"rtk hook claude"}]}]}}"#;
    h.escribir(".claude/settings.json", settings);
    h.escribir(".claude-personal/settings.json", settings);
    let pacman = PacmanFalso(vec![]);
    let mut r = Recetario::nuevo();
    let mut tool = Item::nuevo("herramienta:rtk", Tipo::Herramienta);
    tool.estado = Estado::Aprobada;
    tool.pasos.push(Paso {
        cmd: "rtk-install".into(),
        modo: Modo::Auto,
        por_perfil: false,
        cita: None,
        nota: None,
    });
    r.items.push(tool);
    servicio::escanear(&mut r, &entorno(&h, &pacman)).unwrap();
    let tool = r.item("herramienta:rtk").unwrap();
    assert_eq!(tool.perfiles.len(), 2);
    assert_eq!(tool.estado, Estado::Pendiente);
    assert!(tool.pasos.is_empty());
    let hook = r.items.iter_mut().find(|i| i.tipo == Tipo::Hook).unwrap();
    let id = hook.id.clone();
    hook.estado = Estado::Excluida;
    hook.motivo = Some("decisión".into());
    servicio::escanear(&mut r, &entorno(&h, &pacman)).unwrap();
    assert_eq!(r.item(&id).unwrap().estado, Estado::Excluida);
    assert_eq!(r.item(&id).unwrap().motivo.as_deref(), Some("decisión"));
}
fn escanear_comando_real(
    nombre: &str,
) -> (
    HomeFalso,
    rebake::modelo::Recetario,
    rebake::servicio::ResumenEscaneo,
) {
    let h = HomeFalso::nuevo();
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/hooks-herramientas.json")).unwrap();
    let comando = fixtures[nombre]
        .as_str()
        .unwrap()
        .replace("__TEST_HOME__", h.ruta().to_str().unwrap());
    h.escribir(".claude/skills/impeccable/SKILL.md", "skill");
    h.escribir(
        ".claude/settings.json",
        &serde_json::json!({"hooks":{"Stop":[{"hooks":[{"command":comando}]}]}}).to_string(),
    );
    let pacman = PacmanFalso(vec![]);
    let mut r = rebake::modelo::Recetario::nuevo();
    let resumen = rebake::servicio::escanear(&mut r, &entorno(&h, &pacman)).unwrap();
    (h, r, resumen)
}

fn comprobar_marca_real(nombre: &str, esperado: rebake::modelo::InstaladorHook) {
    use ratatui::{Terminal, backend::TestBackend};
    let (_h, r, _) = escanear_comando_real(nombre);
    let hook = r.items.iter().find(|i| i.tipo == Tipo::Hook).unwrap();
    assert_eq!(hook.instalado_por.as_ref(), Some(&esperado), "{nombre}");
    let mut app = rebake::tui::app::App::nueva(r);
    app.filtro = "hook:".into();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal
        .draw(|f| rebake::tui::vista::dibujar(f, &app))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let pantalla: String = buffer.content.iter().map(|c| c.symbol()).collect();
    assert!(
        pantalla.contains(&format!("lo instala {}", esperado.nombre())),
        "{pantalla}"
    );
}

#[test]
fn comando_real_impeccable_con_ruta_absoluta_entre_comillas() {
    comprobar_marca_real(
        "impeccable",
        rebake::modelo::InstaladorHook::Item("skill:impeccable".into()),
    );
}

#[test]
fn comando_real_orca_corto_con_export_y_fallbacks() {
    comprobar_marca_real(
        "orca_corto",
        rebake::modelo::InstaladorHook::Externo("orca".into()),
    );
}

#[test]
fn comando_real_orca_largo_con_case_anidados() {
    comprobar_marca_real(
        "orca_largo",
        rebake::modelo::InstaladorHook::Externo("orca".into()),
    );
}

#[test]
fn palabras_reservadas_de_hooks_no_producen_avisos_de_binarios() {
    for nombre in ["impeccable", "orca_corto", "orca_largo"] {
        let (_, _, resumen) = escanear_comando_real(nombre);
        assert!(resumen.avisos.is_empty(), "{nombre}: {:?}", resumen.avisos);
    }
}

#[test]
fn segmentos_neutros_no_anulan_pero_pipes_y_responsables_distintos_si() {
    use rebake::{modelo::Recetario, servicio};
    for (comando, vinculado) in [
        ("export X=1; printf '{}'; exit 0; :; ~/.orca/hook.sh", true),
        ("printf '~/.otro/hook.sh'; ~/.orca/hook.sh", true),
        ("~/.orca/hook.sh | cat", false),
        (
            "case \"${OSTYPE-}\" in a*|b*) ~/.orca/hook.sh | cat ;; esac",
            false,
        ),
        (
            "case \"${OSTYPE-}\" in a*|b*) ~/.orca/hook.sh ;; *) ~/.otra/hook.sh ;; esac",
            false,
        ),
        ("printf '~/.orca/hook.sh'; export X=1; exit 0", false),
    ] {
        let h = HomeFalso::nuevo();
        h.escribir(
            ".claude/settings.json",
            &serde_json::json!({"hooks":{"Stop":[{"hooks":[{"command":comando}]}]}}).to_string(),
        );
        let mut r = Recetario::nuevo();
        let pacman = PacmanFalso(vec![]);
        servicio::escanear(&mut r, &entorno(&h, &pacman)).unwrap();
        assert_eq!(
            r.items
                .iter()
                .find(|i| i.tipo == Tipo::Hook)
                .unwrap()
                .instalado_por
                .is_some(),
            vinculado,
            "{comando}"
        );
    }
}

#[test]
fn scripts_propios_load_intent_sync_y_herdr_siguen_con_receta() {
    use rebake::{modelo::*, servicio};
    for ruta in [
        ".claude/skills/ai-native-sdlc/scripts/load-intent.sh",
        ".claude/sync-plugins.sh",
        ".claude/hooks/herdr-agent-state.sh",
    ] {
        let h = HomeFalso::nuevo();
        h.escribir(".claude/skills/ai-native-sdlc/SKILL.md", "skill");
        let script = h.escribir(ruta, "#!/bin/sh\n");
        h.escribir(".claude/settings.json", &serde_json::json!({"hooks":{"Stop":[{"hooks":[{"command":format!("sh '{}'", script.display())}]}]}}).to_string());
        let pacman = PacmanFalso(vec![]);
        let mut r = Recetario::nuevo();
        servicio::escanear(&mut r, &entorno(&h, &pacman)).unwrap();
        let hook = r.items.iter().find(|i| i.tipo == Tipo::Hook).unwrap();
        let id = hook.id.clone();
        assert!(hook.instalado_por.is_none(), "{ruta}");
        assert!(servicio::trabajos(&r, std::slice::from_ref(&id), h.ruta()).is_ok());
        r.item_mut(&id).unwrap().estado = Estado::PorRevisar;
        rebake::acciones::aprobar(&mut r, &id).unwrap();
        assert_eq!(r.item(&id).unwrap().estado, Estado::Aprobada);
    }
}
