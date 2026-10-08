mod comun;
use comun::HomeFalso;
use recetario::investigador::*;
use recetario::modelo::*;
use std::time::Duration;

const VALIDA: &str = r#"{"type":"result","subtype":"success","is_error":false,"result":"ok","structured_output":{
 "repo":"https://github.com/ejemplo/house-rules.git","doc":"https://github.com/ejemplo/house-rules#install",
 "verificar":"test -f \"$CLAUDE_CONFIG_DIR/HOUSE-RULES.md\"",
 "pasos":[{"cmd":"curl -fsSL https://ejemplo/install.sh | sh","modo":"auto","por_perfil":true,"cita":"curl -fsSL https://ejemplo/install.sh | sh"}],
 "requiere":[{"tipo":"herramienta","nombre":"codex"},{"tipo":"sistema","nombre":"jq"}]}}"#;

fn claude_falso(h: &HomeFalso, salida: &str) -> String {
    let script = format!(
        "#!/bin/sh\ncat <<'FIN'\nruido en una linea previa\n{}\nFIN\n",
        salida.replace('\n', "")
    );
    h.binario("bin/claude", script.as_bytes())
        .display()
        .to_string()
}

fn recetario() -> Recetario {
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil {
        nombre: "laburo".into(),
        dir: "~/.claude".into(),
    });
    let mut item = Item::nuevo("import:HOUSE-RULES.md", Tipo::Import);
    item.perfiles = vec!["laburo".into()];
    item.fuente = Some(Fuente {
        repo: "https://github.com/ejemplo/house-rules".into(),
        via: Via::Symlink,
        doc: None,
    });
    r.items.push(item);
    r
}

const ID: &str = "import:HOUSE-RULES.md";

fn trabajo() -> Trabajo {
    Trabajo {
        id: ID.into(),
        prompt: "x".into(),
    }
}

#[test]
fn argumentos_sin_bash() {
    let a = argumentos("investigá");
    for flag in [
        "-p",
        "--restricted",
        "--strict-mcp-config",
        "--no-session-persistence",
        "--json-schema",
    ] {
        assert!(a.iter().any(|x| x == flag), "falta {flag}");
    }
    let tools = a.iter().position(|x| x == "--tools").unwrap();
    assert_eq!(a[tools + 1], "WebSearch,WebFetch");
    assert!(a.iter().all(|x| !x.contains("Bash")));
}

#[test]
fn respuesta_valida_queda_por_revisar() {
    let h = HomeFalso::nuevo();
    let claude = claude_falso(&h, VALIDA);
    let mut r = recetario();
    let receta = investigar(&trabajo(), &claude, Duration::from_secs(10));
    aplicar(&mut r, ID, receta, "2026-10-07");
    let x = r.item(ID).unwrap();
    assert_eq!(x.estado, Estado::PorRevisar);
    assert_eq!(x.pasos.len(), 1);
    assert_eq!(
        x.pasos[0].cita.as_deref(),
        Some("curl -fsSL https://ejemplo/install.sh | sh")
    );
    let f = x.fuente.clone().unwrap();
    assert_eq!(f.via, Via::Symlink);
    assert_eq!(
        f.doc.as_deref(),
        Some("https://github.com/ejemplo/house-rules#install")
    );
    assert_eq!(x.investigado.as_deref(), Some("2026-10-07"));
    assert!(x.error.is_none());
}

#[test]
fn json_invalido_queda_pendiente_con_error() {
    let h = HomeFalso::nuevo();
    let claude = claude_falso(
        &h,
        r#"{"type":"result","subtype":"success","is_error":false,"structured_output":{"repo":"x","doc":"y"}}"#,
    );
    let mut r = recetario();
    aplicar(
        &mut r,
        ID,
        investigar(&trabajo(), &claude, Duration::from_secs(10)),
        "2026-10-07",
    );
    let x = r.item(ID).unwrap();
    assert_eq!(x.estado, Estado::Pendiente);
    assert!(
        x.error.as_deref().unwrap().contains("formato"),
        "{:?}",
        x.error
    );
}

#[test]
fn tope_queda_pendiente() {
    let h = HomeFalso::nuevo();
    let claude = h
        .binario("bin/claude", b"#!/bin/sh\nsleep 30\n")
        .display()
        .to_string();
    let mut r = recetario();
    aplicar(
        &mut r,
        ID,
        investigar(&trabajo(), &claude, Duration::from_millis(300)),
        "2026-10-07",
    );
    let x = r.item(ID).unwrap();
    assert_eq!(x.estado, Estado::Pendiente);
    assert!(x.error.as_deref().unwrap().contains("superó"));
}

#[test]
fn requisito_nuevo_y_de_sistema() {
    let h = HomeFalso::nuevo();
    let claude = claude_falso(&h, VALIDA);
    let mut r = recetario();
    aplicar(
        &mut r,
        ID,
        investigar(&trabajo(), &claude, Duration::from_secs(10)),
        "2026-10-07",
    );
    let codex = r.item("herramienta:codex").unwrap();
    assert_eq!(codex.estado, Estado::Pendiente);
    assert_eq!(
        codex.pista.as_deref(),
        Some("requisito de import:HOUSE-RULES.md")
    );
    assert!(
        r.item(ID)
            .unwrap()
            .requiere
            .contains(&"herramienta:codex".to_string())
    );
    assert_eq!(
        r.checklist.sistema,
        vec![PaqueteSistema {
            paquete: "jq".into(),
            para: ID.into()
        }]
    );
    assert!(r.item("sistema:jq").is_none());
}

#[test]
fn requisito_excluido_no_revive() {
    let h = HomeFalso::nuevo();
    let claude = claude_falso(&h, VALIDA);
    let mut r = recetario();
    let mut codex = Item::nuevo("herramienta:codex", Tipo::Herramienta);
    codex.estado = Estado::Excluida;
    codex.motivo = Some("no lo uso".into());
    r.items.push(codex);
    aplicar(
        &mut r,
        ID,
        investigar(&trabajo(), &claude, Duration::from_secs(10)),
        "2026-10-07",
    );
    assert_eq!(
        r.item("herramienta:codex").unwrap().estado,
        Estado::Excluida
    );
}

#[test]
fn en_paralelo_devuelve_todos() {
    let h = HomeFalso::nuevo();
    let claude = claude_falso(&h, VALIDA);
    let trabajos = (0..3)
        .map(|i| Trabajo {
            id: format!("skill:s{i}"),
            prompt: "x".into(),
        })
        .collect();
    let rx = en_paralelo(trabajos, claude, Duration::from_secs(10), 2);
    let mut ids: Vec<String> = rx
        .iter()
        .map(|(id, r)| {
            assert!(r.is_ok());
            id
        })
        .collect();
    ids.sort();
    assert_eq!(ids, ["skill:s0", "skill:s1", "skill:s2"]);
}

#[test]
fn argumentos_preaprueban_la_web() {
    let a = argumentos("investigá");
    let i = a
        .iter()
        .position(|x| x == "--allowedTools")
        .expect("falta --allowedTools");
    assert_eq!(a[i + 1], "WebSearch,WebFetch");
}

fn salida(estructurada: &str, denegados: &str) -> String {
    format!(
        r#"{{"type":"result","subtype":"success","is_error":false,"permission_denials":{denegados},"structured_output":{estructurada}}}"#
    )
}

#[test]
fn permiso_denegado_es_error() {
    let s = salida(
        r#"{"repo":"https://github.com/a/b","doc":"https://github.com/a/b#install","pasos":[],"requiere":[]}"#,
        r#"[{"tool_name":"WebFetch","tool_use_id":"x","tool_input":{}}]"#,
    );
    let err = parsear_salida(&s).unwrap_err().to_string();
    assert!(err.contains("permiso") && err.contains("WebFetch"), "{err}");
}

#[test]
fn doc_sin_url_es_error_y_con_texto_se_extrae() {
    let sin_url = salida(
        r#"{"repo":"https://github.com/a/b","doc":"NO VERIFICADO: no se pudo leer el README","pasos":[],"requiere":[]}"#,
        "[]",
    );
    assert!(
        parsear_salida(&sin_url)
            .unwrap_err()
            .to_string()
            .contains("URL")
    );
    let con_texto = salida(
        r#"{"repo":"https://github.com/a/b","doc":"README.md (https://github.com/a/b#readme)","pasos":[],"requiere":[]}"#,
        "[]",
    );
    assert_eq!(
        parsear_salida(&con_texto).unwrap().doc,
        "https://github.com/a/b#readme"
    );
}
