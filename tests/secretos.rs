use rebake::modelo::OCULTO;
use rebake::secretos::{clave_sensible, ocultar};

#[test]
fn ocultar_asignaciones_tokens_y_cabeceras() {
    assert_eq!(
        ocultar("API_TOKEN=abc123 rtk hook claude"),
        format!("API_TOKEN={OCULTO} rtk hook claude")
    );
    let curl = ocultar("curl -H \"Authorization: Bearer xyz\" https://u:p@host/x");
    assert!(!curl.contains("xyz") && !curl.contains("u:p@"), "{curl}");
    assert!(curl.contains("https://host"), "{curl}");
    assert_eq!(
        ocultar("echo sk-ant-api03-abcdefghij"),
        format!("echo {OCULTO}")
    );
    assert_eq!(ocultar("rtk hook claude"), "rtk hook claude");
}

#[test]
fn claves_sensibles_mas_alla_de_token() {
    for c in [
        "awsCredentialExport",
        "awsAuthRefresh",
        "otelHeadersHelper",
        "apiKeyHelper",
        "cookieFile",
    ] {
        assert!(clave_sensible(c), "{c}");
    }
    for c in ["theme", "voice.mode", "autoMode.allow[0]"] {
        assert!(!clave_sensible(c), "{c}");
    }
}

#[test]
fn ocultar_webhooks_flags_y_cabeceras_pegadas() {
    let webhook =
        ocultar("curl -X POST https://hooks.slack.com/services/T000/B000/XXXXSECRETO -d x");
    assert!(!webhook.contains("XXXXSECRETO"), "{webhook}");
    assert!(webhook.contains("https://hooks.slack.com"), "{webhook}");
    let flag = ocultar("notificar --token SECRETO1 --canal general");
    assert!(
        !flag.contains("SECRETO1") && flag.contains("--canal general"),
        "{flag}"
    );
    let pegada = ocultar("curl -H Authorization:Bearer SECRETO2 x");
    assert!(!pegada.contains("SECRETO2"), "{pegada}");
    let pegada2 = ocultar("curl -H X-Api-Key:SECRETO3 x");
    assert!(!pegada2.contains("SECRETO3"), "{pegada2}");
}

#[test]
fn una_url_no_se_toma_por_nombre_de_clave() {
    let git = ocultar(
        "fatal: unable to access 'https://x-access-token:ghp_abcdefghijkl@github.com/a/b/': error 403",
    );
    assert!(
        !git.contains("ghp_") && !git.contains("x-access-token"),
        "{git}"
    );
    assert!(
        git.contains("https://github.com") && git.contains("error 403"),
        "{git}"
    );
    let consulta = ocultar("curl https://yo:clave1@host/auth?x=1");
    assert!(!consulta.contains("clave1"), "{consulta}");
    assert!(consulta.contains("https://host"), "{consulta}");
}

#[test]
fn ocultar_cada_par_y_usuario_clave() {
    assert_eq!(
        ocultar("curl -d \"user=me&token=CLAVE1\" https://api.example.com/hook"),
        format!("curl -d \"user=me&token={OCULTO}\" https://api.example.com")
    );
    assert_eq!(
        ocultar("export A=1;TOKEN=CLAVE2"),
        format!("export A=1;TOKEN={OCULTO}")
    );
    assert_eq!(
        ocultar("curl -u yo:CLAVE3 https://api.example.com"),
        format!("curl -u yo:{OCULTO} https://api.example.com")
    );
    assert_eq!(
        ocultar("curl --user=yo:CLAVE4 https://api.example.com"),
        format!("curl --user=yo:{OCULTO} https://api.example.com")
    );
    assert_eq!(
        ocultar("GH=ghp_abcdefghijklmnop gh api user"),
        format!("GH={OCULTO} gh api user")
    );
    assert_eq!(
        ocultar("curl -d cb=https://x.example&api_key=CLAVE5"),
        "curl -d cb=https://x.example"
    );
    assert_eq!(
        ocultar("curl -d a=1&token=CLAVE6&cb=https://h/p"),
        format!("curl -d a=1&token={OCULTO}&cb=https://h")
    );
}

#[test]
fn ocultar_no_rompe_lo_que_no_es_secreto() {
    for igual in [
        "docker run -u 1000:1000 imagen",
        "sort -u archivo",
        "curl -u yo https://h",
        "curl -d user=me&page=2",
    ] {
        assert_eq!(ocultar(igual), igual);
    }
    assert_eq!(ocultar("curl https://h/p?a=1&sig=FIRMA"), "curl https://h");
}
