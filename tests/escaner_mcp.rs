mod comun;
use comun::*;
use rebake::escaner::{escanear, mcp::PISTA_MCP};
use rebake::fusion::fusionar;
use rebake::modelo::{Recetario, Tipo};

const CLAUDE_JSON: &str = r#"{
  "oauthAccount": {"emailAddress": "denis@ejemplo.com", "accountUuid": "cuenta-secreta"},
  "userID": "id-de-usuario-123",
  "projects": {"/home/x/repo": {"mcpServers": {"de-proyecto": {"command": "otro"}}, "history": [{"display": "historial privado"}]}},
  "mcpServers": {
    "serena": {"type": "stdio", "command": "serena", "args": ["start-mcp-server", "--context=claude-code"], "env": {"API_KEY": "sk-ant-api03-claveclaveclave"}},
    "remoto": {"type": "http", "url": "https://usuario:clave@mcp.ejemplo.com/v1", "headers": {"Authorization": "Bearer tokentokentoken"}},
    "con-token": {"command": "npx", "args": ["-y", "algo-mcp", "--token", "ghp_abcdefghijklmnopqrstuvwxyz0123456789"]}
  }
}"#;

fn cookbook(
    h: &HomeFalso,
    perfiles: &[rebake::modelo::Perfil],
) -> (rebake::escaner::Escaneo, String) {
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(h, &pacman), perfiles);
    let mut r = Recetario::nuevo();
    r.perfiles = perfiles.to_vec();
    let copia = rebake::escaner::Escaneo {
        hallazgos: e.hallazgos.clone(),
        avisos: e.avisos.clone(),
        ..Default::default()
    };
    fusionar(&mut r, copia, &|_| true);
    (e, toml::to_string(&r).unwrap())
}

#[test]
fn de_claude_json_solo_llegan_los_servidores() {
    if !comun::aislado("de_claude_json_solo_llegan_los_servidores") {
        return;
    }
    let h = HomeFalso::nuevo();
    h.escribir(".claude.json", CLAUDE_JSON);
    h.binario(
        "bin/serena",
        b"\x7fELF https://github.com/oraios/serena \x00",
    );
    let (e, texto) = cookbook(&h, &[perfil("laburo", "~/.claude")]);
    let serena = e
        .hallazgos
        .iter()
        .find(|x| x.id == "mcp:serena")
        .expect("falta serena");
    assert_eq!(serena.tipo, Tipo::Mcp);
    assert_eq!(serena.perfiles, ["laburo"]);
    assert_eq!(
        serena.pista.as_deref(),
        Some(&*format!(
            "{PISTA_MCP}serena start-mcp-server --context=claude-code"
        ))
    );
    assert_eq!(
        serena.fuente.as_ref().map(|f| f.repo.as_str()),
        Some("https://github.com/oraios/serena")
    );
    // El binario se usa solo para el origen: no aparece como herramienta aparte.
    assert!(e.hallazgos.iter().all(|x| x.id != "herramienta:serena"));
    for privado in [
        "denis@ejemplo.com",
        "cuenta-secreta",
        "id-de-usuario-123",
        "historial privado",
        "de-proyecto",
        "sk-ant-api03",
        "API_KEY",
        "tokentokentoken",
        "Authorization",
        "usuario:clave",
        "ghp_abcdefghijklmnopqrstuvwxyz0123456789",
    ] {
        assert!(
            !texto.contains(privado),
            "{privado} llegó al cookbook:\n{texto}"
        );
    }
}

#[test]
fn un_servidor_remoto_queda_con_su_host_sin_credenciales_ni_ruta() {
    if !comun::aislado("un_servidor_remoto_queda_con_su_host_sin_credenciales_ni_ruta") {
        return;
    }
    let h = HomeFalso::nuevo();
    h.escribir(".claude.json", CLAUDE_JSON);
    let (e, _) = cookbook(&h, &[perfil("laburo", "~/.claude")]);
    let remoto = e
        .hallazgos
        .iter()
        .find(|x| x.id == "mcp:remoto")
        .expect("falta remoto");
    assert_eq!(
        remoto.pista.as_deref(),
        Some(&*format!("{PISTA_MCP}https://mcp.ejemplo.com"))
    );
    assert!(remoto.fuente.is_none());
}

#[test]
fn cada_perfil_lee_su_propio_claude_json() {
    if !comun::aislado("cada_perfil_lee_su_propio_claude_json") {
        return;
    }
    let h = HomeFalso::nuevo();
    h.escribir(
        ".claude.json",
        r#"{"mcpServers": {"uno": {"command": "a"}}}"#,
    );
    h.escribir(
        ".claude-personal/.claude.json",
        r#"{"mcpServers": {"dos": {"command": "b"}}}"#,
    );
    // Con la variable, Claude Code buscaría `~/.claude/.claude.json`: ese no cuenta.
    h.escribir(
        ".claude/.claude.json",
        r#"{"mcpServers": {"tres": {"command": "c"}}}"#,
    );
    let (e, _) = cookbook(
        &h,
        &[
            perfil("laburo", "~/.claude"),
            perfil("personal", "~/.claude-personal"),
        ],
    );
    let perfiles = |id: &str| {
        e.hallazgos
            .iter()
            .find(|x| x.id == id)
            .map(|x| x.perfiles.clone())
    };
    assert_eq!(perfiles("mcp:uno"), Some(vec!["laburo".to_string()]));
    assert_eq!(perfiles("mcp:dos"), Some(vec!["personal".to_string()]));
    assert_eq!(perfiles("mcp:tres"), None);
}

#[test]
fn sin_claude_json_no_hay_aviso_y_uno_roto_avisa_sin_contenido() {
    if !comun::aislado("sin_claude_json_no_hay_aviso_y_uno_roto_avisa_sin_contenido") {
        return;
    }
    let h = HomeFalso::nuevo();
    let (e, _) = cookbook(&h, &[perfil("laburo", "~/.claude")]);
    assert!(e.hallazgos.iter().all(|x| x.tipo != Tipo::Mcp));
    assert!(e.avisos.is_empty(), "{:?}", e.avisos);

    h.escribir(".claude.json", r#"{"oauthAccount": "secreto-roto", "#);
    let (e, _) = cookbook(&h, &[perfil("laburo", "~/.claude")]);
    assert_eq!(e.avisos.len(), 1, "{:?}", e.avisos);
    assert!(e.avisos[0].contains("~/.claude.json"));
    assert!(!e.avisos[0].contains("secreto-roto"));
}
