use recetario::modelo::OCULTO;
use recetario::secretos::{clave_sensible, ocultar};

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
