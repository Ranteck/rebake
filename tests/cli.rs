mod comun;
use comun::HomeFalso;
use rebake::{archivo, modelo::Estado};
use std::process::{Command, Output};

fn recetario(h: &HomeFalso, args: &[&str], claude: Option<&str>) -> Output {
    comando(h, args, claude).output().unwrap()
}

fn comando(h: &HomeFalso, args: &[&str], claude: Option<&str>) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_rebake"));
    c.args(args)
        .env("HOME", h.ruta())
        .env("REBAKE_BASE_URL", format!("file://{}", h.ruta().display()))
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", h.ruta().join("bin").display()),
        )
        .env_remove("CLAUDE_CONFIG_DIR");
    if let Some(cl) = claude {
        c.env("REBAKE_CLAUDE", cl);
    }
    c
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

#[test]
fn investigar_en_lote_salta_las_sin_origen_y_las_lista() {
    let h = HomeFalso::nuevo();
    let ruta = h.ruta().join("Documents/rebake/cookbook.toml");
    let mut r = rebake::modelo::Recetario::nuevo();
    for (id, fuente) in [("a", true), ("b", true), ("huerfana", false)] {
        let mut i = rebake::modelo::Item::nuevo(id, rebake::modelo::Tipo::Plugin);
        if fuente {
            i.fuente = Some(rebake::modelo::Fuente {
                repo: "https://github.com/ejemplo/repo".into(),
                via: rebake::modelo::Via::Metadatos,
                doc: None,
            });
        }
        r.items.push(i);
    }
    std::fs::create_dir_all(ruta.parent().unwrap()).unwrap();
    archivo::guardar(&ruta, &r).unwrap();
    let llamadas = h.ruta().join("llamadas");
    let claude = h.binario(
        "bin/claude-falso",
        format!(
            "#!/bin/sh\necho x >> {}\ncat <<'FIN'\n{RESPUESTA}\nFIN\n",
            llamadas.display()
        )
        .as_bytes(),
    );
    let o = recetario(&h, &["investigar"], Some(claude.to_str().unwrap()));
    assert!(o.status.success(), "{}", texto(&o));
    assert_eq!(
        std::fs::read_to_string(&llamadas).unwrap().lines().count(),
        2
    );
    let salida = String::from_utf8_lossy(&o.stdout);
    assert!(salida.contains("1 sin origen"), "{salida}");
    assert!(salida.contains("  huerfana"), "{salida}");
}

#[test]
fn profile_es_global_para_investigar_y_la_entrada_tui() {
    let h = HomeFalso::nuevo();
    let ruta = h.ruta().join("cookbook.toml");
    archivo::guardar(&ruta, &rebake::modelo::Recetario::nuevo()).unwrap();
    let shell = h.binario("bin/bash", b"#!/bin/sh\nprintf 'REBAKE_PROFILE:\\n'\n");
    for args in [
        vec!["--profile", "clp", "investigar"],
        vec!["investigar", "--profile", "clp"],
    ] {
        let o = comando(&h, &args, None)
            .args(["--archivo", ruta.to_str().unwrap()])
            .env("SHELL", &shell)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", texto(&o));
    }
    let o = comando(&h, &["--profile", "clp"], None)
        .args(["--archivo", ruta.to_str().unwrap()])
        .env("SHELL", &shell)
        .output()
        .unwrap();
    assert!(
        texto(&o).contains("la salida no es una terminal"),
        "{}",
        texto(&o)
    );
}

#[test]
fn profile_se_ignora_fuera_de_las_corridas_de_claude() {
    let h = HomeFalso::nuevo();
    let ruta = h.ruta().join("cookbook.toml");
    let mut r = rebake::modelo::Recetario::nuevo();
    r.perfiles.push(comun::perfil("viejo", "~/.claude"));
    archivo::guardar(&ruta, &r).unwrap();
    for args in [
        vec!["escanear"],
        vec!["instalar", "--dry-run"],
        vec!["renombrar-perfil", "viejo", "nuevo"],
    ] {
        let o = comando(&h, &args, None)
            .args([
                "--profile",
                "invalido;",
                "--archivo",
                ruta.to_str().unwrap(),
            ])
            .env("SHELL", "/no/existe/fish")
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", texto(&o));
    }
    let o = comando(&h, &["clonar", "no-existe"], None)
        .args([
            "--profile",
            "invalido;",
            "--archivo",
            h.ruta().join("clon/cookbook.toml").to_str().unwrap(),
        ])
        .env("SHELL", "")
        .output()
        .unwrap();
    assert!(texto(&o).contains("no pude clonar"), "{}", texto(&o));
    assert!(!texto(&o).contains("--profile"), "{}", texto(&o));
}

fn home_profile() -> HomeFalso {
    let h = HomeFalso::nuevo();
    archivo::guardar(
        &h.ruta().join("cookbook.toml"),
        &rebake::modelo::Recetario::nuevo(),
    )
    .unwrap();
    h
}

fn con_profile(h: &HomeFalso, nombre: &str, shell: &std::path::Path) -> Command {
    let mut c = comando(h, &["investigar"], None);
    c.arg(format!("--profile={nombre}"))
        .args([
            "--archivo",
            h.ruta().join("cookbook.toml").to_str().unwrap(),
        ])
        .env("SHELL", shell)
        .env("ZDOTDIR", h.ruta())
        .env_remove("BASH_ENV")
        .env_remove("ENV");
    c
}

fn shell_profile(h: &HomeFalso, cuerpo: &str) -> std::path::PathBuf {
    h.binario(
        "bin/bash",
        format!("#!/bin/sh\nprintf 'x\\n' >> \"$HOME/consultas\"\n{cuerpo}\n").as_bytes(),
    )
}

#[test]
fn profile_rechaza_nombres_invalidos_sin_consultar_shell() {
    let h = home_profile();
    let shell = shell_profile(&h, "printf 'REBAKE_PROFILE:\\n'");
    for nombre in [
        "",
        "-clp",
        "dos palabras",
        "clp;reboot",
        "clp/otro",
        "ál",
        "clp\n",
    ] {
        let o = con_profile(&h, nombre, &shell).output().unwrap();
        assert!(!o.status.success(), "{nombre:?}: {}", texto(&o));
        assert!(texto(&o).contains("nombre"), "{}", texto(&o));
    }
    assert!(!h.ruta().join("consultas").exists());
}

#[test]
fn profile_rechaza_shell_ausente_incompatible_o_inaccesible() {
    let h = home_profile();
    for (shell, mensaje) in [
        ("", "SHELL"),
        ("/usr/bin/fish", "soportada"),
        ("/no/existe/bash", "iniciar"),
    ] {
        let o = con_profile(&h, "clp", std::path::Path::new(shell))
            .output()
            .unwrap();
        assert!(!o.status.success());
        assert!(texto(&o).contains(mensaje), "{}", texto(&o));
    }
    let o = con_profile(&h, "clp", std::path::Path::new(""))
        .env_remove("SHELL")
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(texto(&o).contains("SHELL"), "{}", texto(&o));
}

#[test]
fn profile_exige_marcador_unico_de_stdout_y_shell_exitosa() {
    for (cuerpo, mensaje) in [
        ("printf 'REBAKE_PROFILE:privado\\n' >&2", "respuesta"),
        ("printf 'ruido\\n'", "respuesta"),
        (
            "printf 'REBAKE_PROFILE:a\\nREBAKE_PROFILE:b\\n'",
            "respuesta",
        ),
        ("printf 'REBAKE_PROFILE:mal\\000dato\\n'", "respuesta"),
        ("printf 'REBAKE_PROFILE:privado\\n'; exit 1", "falló"),
        ("exit 64", "programa"),
        ("exit 65", "no existe"),
        ("exit 66", "llama a claude"),
    ] {
        let h = home_profile();
        let shell = shell_profile(&h, cuerpo);
        let o = con_profile(&h, "clp", &shell).output().unwrap();
        assert!(!o.status.success(), "{cuerpo}: {}", texto(&o));
        assert!(texto(&o).contains(mensaje), "{}", texto(&o));
        assert!(!texto(&o).contains("privado"), "{}", texto(&o));
        assert_eq!(
            std::fs::read_to_string(h.ruta().join("consultas")).unwrap(),
            "x\n"
        );
    }
    let h = home_profile();
    let shell = shell_profile(
        &h,
        "printf 'ruido\\nREBAKE_PROFILE:\\n'; printf 'REBAKE_PROFILE:no-leer\\n' >&2",
    );
    let o = con_profile(&h, "clp", &shell).output().unwrap();
    assert!(o.status.success(), "{}", texto(&o));
}

#[test]
fn profile_resuelve_alias_y_funciones_en_zsh_y_bash_reales_aisladas() {
    for shell in ["/usr/bin/zsh", "/usr/bin/bash"] {
        let h = home_profile();
        let inicio = "printf 'inicio\\n' >> \"$HOME/inicios\"\nalias clp='CLAUDE_CONFIG_DIR=\"$HOME/personal\" claude'\nalias cl='claude'\nalias cadena='clp'\nfuncion() { CLAUDE_CONFIG_DIR=\"$HOME/funcion\" claude; }\nalias otro=':'\n";
        h.escribir(".bashrc", inicio);
        h.escribir(".zshrc", inicio);
        h.binario(
            "bin/claude",
            b"#!/bin/sh\ntouch \"$HOME/claude-real\"\nexit 99\n",
        );
        h.binario(
            "bin/reboot",
            b"#!/bin/sh\ntouch \"$HOME/programa\"\nexit 99\n",
        );
        for nombre in ["clp", "cl", "cadena", "funcion"] {
            let o = con_profile(&h, nombre, std::path::Path::new(shell))
                .env("CLAUDE_CONFIG_DIR", "heredada")
                .output()
                .unwrap();
            assert!(o.status.success(), "{shell} {nombre}: {}", texto(&o));
        }
        for (nombre, mensaje) in [
            ("reboot", "programa"),
            ("desconocido", "no existe"),
            ("otro", "llama a claude"),
        ] {
            let o = con_profile(&h, nombre, std::path::Path::new(shell))
                .output()
                .unwrap();
            assert!(!o.status.success());
            assert!(texto(&o).contains(mensaje), "{shell}: {}", texto(&o));
        }
        assert_eq!(
            std::fs::read_to_string(h.ruta().join("inicios"))
                .unwrap()
                .lines()
                .count(),
            7
        );
        assert!(!h.ruta().join("programa").exists());
        assert!(!h.ruta().join("claude-real").exists());
    }
}

#[test]
fn profile_resuelve_en_terminal_con_zsh_real() {
    profile_en_terminal("/usr/bin/zsh");
}

#[test]
fn profile_resuelve_en_terminal_con_bash_real() {
    profile_en_terminal("/usr/bin/bash");
}

fn profile_en_terminal(shell: &str) {
    let h = home_profile();
    let inicio = "alias clp='CLAUDE_CONFIG_DIR=\"$HOME/personal\" claude'\n";
    h.escribir(".bashrc", inicio);
    h.escribir(".zshrc", inicio);
    let o = Command::new("script")
        .args([
            "-qec",
            "exec \"$REBAKE_BINARIO\" --profile clp --archivo \"$HOME/cookbook.toml\" investigar",
            "/dev/null",
        ])
        .env("REBAKE_BINARIO", env!("CARGO_BIN_EXE_rebake"))
        .env("REBAKE_BASE_URL", format!("file://{}", h.ruta().display()))
        .env("HOME", h.ruta())
        .env("ZDOTDIR", h.ruta())
        .env("SHELL", shell)
        .env("PATH", "/usr/bin:/bin")
        .env_remove("CLAUDE_CONFIG_DIR")
        .env_remove("BASH_ENV")
        .env_remove("ENV")
        .output()
        .unwrap();
    assert!(o.status.success(), "{shell}: {}", texto(&o));
}

#[test]
fn profile_loguea_una_resolucion_sin_rutas_alias_ni_salida_cruda() {
    for (nombre, cuerpo, nivel, causa) in [
        (
            "alias_privado",
            "printf 'REBAKE_PROFILE:/ruta/privada\\n'; printf 'secreto-de-shell\\n' >&2",
            "INFO",
            "ninguna",
        ),
        ("alias;privado", "", "INFO", "nombre_invalido"),
        ("alias_privado", "exit 64", "INFO", "programa"),
        ("alias_privado", "exit 65", "INFO", "inexistente"),
        ("alias_privado", "exit 66", "INFO", "sin_claude"),
        (
            "alias_privado",
            "printf 'secreto-de-shell\\n' >&2; exit 1",
            "ERROR",
            "shell_fallo",
        ),
        (
            "alias_privado",
            "printf 'REBAKE_PROFILE:a\\nREBAKE_PROFILE:b\\n'",
            "ERROR",
            "respuesta_invalida",
        ),
    ] {
        let h = home_profile();
        let shell = shell_profile(&h, cuerpo);
        let o = con_profile(&h, nombre, &shell).output().unwrap();
        assert_eq!(o.status.success(), causa == "ninguna", "{}", texto(&o));
        let log = std::fs::read_to_string(h.ruta().join(".local/state/rebake/rebake.log")).unwrap();
        let eventos: Vec<_> = log
            .lines()
            .filter(|l| l.contains("resolver_cuenta") || l.contains("el comando falló"))
            .collect();
        assert_eq!(eventos.len(), 1, "{log}");
        let evento = eventos[0];
        assert!(evento.contains("accion=\"resolver_cuenta\""), "{evento}");
        assert!(evento.contains(nivel), "{evento}");
        assert!(evento.contains(&format!("causa=\"{causa}\"")), "{evento}");
        for privado in [
            "alias_privado",
            "alias;privado",
            "/ruta/privada",
            "secreto-de-shell",
            h.ruta().to_str().unwrap(),
        ] {
            assert!(!log.contains(privado), "{log}");
        }
    }
}

#[test]
fn sin_profile_no_consulta_shell_ni_registra_resolucion() {
    let h = home_profile();
    let shell = shell_profile(&h, "exit 1");
    let o = comando(&h, &["investigar"], None)
        .args([
            "--archivo",
            h.ruta().join("cookbook.toml").to_str().unwrap(),
        ])
        .env("SHELL", shell)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", texto(&o));
    assert!(!h.ruta().join("consultas").exists());
    let log = std::fs::read_to_string(h.ruta().join(".local/state/rebake/rebake.log")).unwrap();
    assert!(!log.contains("resolver_cuenta"), "{log}");
}
#[test]
fn instalar_sin_recetas_igual_muestra_hooks_externos() {
    use rebake::modelo::{InstaladorHook, Item, Perfil, Recetario, Tipo};
    let h = HomeFalso::nuevo();
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil {
        nombre: "personal".into(),
        dir: "~/.claude-personal".into(),
    });
    let mut hook = Item::nuevo("hook:tool hook", Tipo::Hook);
    hook.perfiles.push("personal".into());
    hook.instalado_por = Some(InstaladorHook::Externo("tool".into()));
    r.items.push(hook);
    let ruta = h.ruta().join("cookbook.toml");
    archivo::guardar(&ruta, &r).unwrap();
    let o = recetario(
        &h,
        &["--archivo", ruta.to_str().unwrap(), "instalar", "--si"],
        None,
    );
    assert!(o.status.success(), "{}", texto(&o));
    let salida = texto(&o);
    assert!(
        salida.contains("1 hook [personal]") && salida.contains("instalar o abrir"),
        "{salida}"
    );
    let log = std::fs::read_to_string(h.ruta().join(".local/state/rebake/rebake.log")).unwrap();
    assert!(
        log.contains("resultado=\"sin_recetas\"") && log.contains("externos=1"),
        "{log}"
    );
}

#[test]
fn investigacion_con_hooks_registra_ok_pendiente_y_fallo_sin_secretos() {
    use rebake::modelo::{Fuente, Via};
    for resultado in ["ok", "pendiente", "fallo"] {
        let h = HomeFalso::nuevo();
        h.binario("bin/pacman", b"#!/bin/sh\nexit 1\n");
        h.binario("bin/tool", b"#!/bin/sh\nexit 0\n");
        h.escribir(".claude/settings.json", r#"{"hooks":{"Stop":[{"hooks":[{"command":"API_TOKEN=secreto-sintetico tool hook"}]}]}}"#);
        assert!(recetario(&h, &["escanear"], None).status.success());
        let ruta = h.ruta().join("Documents/rebake/cookbook.toml");
        let mut r = archivo::leer(&ruta).unwrap();
        r.item_mut("herramienta:tool").unwrap().fuente = Some(Fuente {
            repo: "https://example.com/tool".into(),
            via: Via::Manual,
            doc: None,
        });
        archivo::guardar(&ruta, &r).unwrap();
        let registro = if resultado == "pendiente" {
            serde_json::json!({"tipo":"pendiente"})
        } else {
            serde_json::json!({"tipo":"paso","indice":1})
        };
        let respuesta = if resultado == "fallo" {
            serde_json::json!({"type":"result","subtype":"error","result":"fallo sintético"})
        } else {
            serde_json::json!({"type":"result","subtype":"success","structured_output":{
            "repo":"https://example.com/tool","doc":"https://example.com/tool/install","requiere":[],"registro_hooks":registro,
            "pasos":[{"cmd":"tool-install","modo":"auto","por_perfil":false,"cita":"tool-install"},{"cmd":"tool init","modo":"auto","por_perfil":true,"cita":"tool init"}]}})
        };
        let captura = h.ruta().join("prompt");
        let fake = h.binario(
            "bin/claude",
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\ncat <<'JSON'\n{respuesta}\nJSON\n",
                captura.display()
            )
            .as_bytes(),
        );
        let o = recetario(
            &h,
            &["investigar", "herramienta:tool"],
            Some(fake.to_str().unwrap()),
        );
        assert_eq!(o.status.success(), resultado != "fallo", "{}", texto(&o));
        let log = std::fs::read_to_string(h.ruta().join(".local/state/rebake/rebake.log")).unwrap();
        assert!(log.contains(&format!("resultado=\"{resultado}\"")), "{log}");
        assert!(log.contains("hooks_vinculados=1"), "{log}");
        let cookbook = std::fs::read_to_string(&ruta).unwrap();
        let prompt = std::fs::read_to_string(captura).unwrap();
        for salida in [&log, &cookbook, &prompt, &texto(&o)] {
            assert!(!salida.contains("secreto-sintetico"), "{salida}");
        }
        assert!(
            !log.contains("hook:") && !log.contains("API_TOKEN"),
            "{log}"
        );
        let r = archivo::leer(&ruta).unwrap();
        assert_eq!(
            r.item("herramienta:tool").unwrap().estado,
            if resultado == "ok" {
                Estado::PorRevisar
            } else {
                Estado::Pendiente
            }
        );
    }
}
