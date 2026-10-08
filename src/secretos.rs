//! Todo lo que se guarda en el recetario pasa por acá: el archivo se versiona.
use crate::modelo::OCULTO;

const SENSIBLES: [&str; 8] = [
    "token",
    "secret",
    "key",
    "password",
    "auth",
    "credential",
    "cookie",
    "header",
];
const PREFIJOS: [&str; 10] = [
    "sk-",
    "ghp_",
    "gho_",
    "ghu_",
    "ghs_",
    "github_pat_",
    "glpat-",
    "xoxb-",
    "xoxp-",
    "AKIA",
];

pub fn clave_sensible(clave: &str) -> bool {
    let clave = clave.to_lowercase();
    SENSIBLES.iter().any(|s| clave.contains(s))
}

/// Valores con forma de credencial aunque su clave no lo diga (tokens con prefijo conocido, JWT).
pub fn parece_secreto(valor: &str) -> bool {
    let v = valor.trim_matches(|c| c == '"' || c == '\'');
    PREFIJOS
        .iter()
        .any(|p| v.starts_with(p) && v.len() > p.len() + 8)
        || (v.starts_with("eyJ") && v.matches('.').count() == 2)
}

pub fn sin_credenciales_url(url: &str) -> String {
    let Some((esquema, resto)) = url.split_once("://") else {
        return url.to_string();
    };
    let (autoridad, ruta) = match resto.split_once('/') {
        Some((a, r)) => (a, Some(r)),
        None => (resto, None),
    };
    match autoridad.rsplit_once('@') {
        Some((_, host)) => format!(
            "{esquema}://{host}{}",
            ruta.map(|r| format!("/{r}")).unwrap_or_default()
        ),
        None => url.to_string(),
    }
}

fn es_esquema_de_auth(palabra: &str) -> bool {
    palabra.eq_ignore_ascii_case("bearer") || palabra.eq_ignore_ascii_case("basic")
}

/// Oculta secretos dentro de un texto libre (un comando de hook, un valor de ajuste).
pub fn ocultar(texto: &str) -> String {
    let mut ocultar_siguiente = false;
    texto
        .split(' ')
        .map(|parte| {
            let limpia = parte.trim_matches(|c| c == '"' || c == '\'');
            if limpia.is_empty() {
                return parte.to_string();
            }
            if ocultar_siguiente {
                if es_esquema_de_auth(limpia) {
                    return parte.to_string();
                }
                ocultar_siguiente = false;
                return parte.replacen(limpia, OCULTO, 1);
            }
            // `Bearer X`, `Authorization: X` y `--token X`: el secreto es la palabra siguiente.
            let flag_sensible = limpia.starts_with('-')
                && !limpia.contains('=')
                && clave_sensible(limpia.trim_start_matches('-'));
            if es_esquema_de_auth(limpia)
                || limpia.strip_suffix(':').is_some_and(clave_sensible)
                || flag_sensible
            {
                ocultar_siguiente = true;
                return parte.to_string();
            }
            // Cabecera pegada en una sola palabra: `X-Api-Key:valor`, `Authorization:Bearer`.
            if !limpia.contains("://")
                && let Some((nombre, valor)) = limpia.split_once(':')
                && clave_sensible(nombre)
                && !valor.is_empty()
            {
                if es_esquema_de_auth(valor) {
                    ocultar_siguiente = true;
                    return parte.to_string();
                }
                return parte.replacen(limpia, &format!("{nombre}:{OCULTO}"), 1);
            }
            if let Some((nombre, valor)) = limpia.split_once('=')
                && clave_sensible(nombre)
                && !valor.is_empty()
            {
                return parte.replacen(limpia, &format!("{nombre}={OCULTO}"), 1);
            }
            if parece_secreto(limpia) {
                return parte.replacen(limpia, OCULTO, 1);
            }
            // En texto libre una URL puede llevar el secreto en la ruta (webhooks): basta el host.
            if limpia.contains("://") {
                return parte.replacen(limpia, &solo_host(limpia), 1);
            }
            parte.to_string()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn solo_host(url: &str) -> String {
    let sin_credenciales = sin_credenciales_url(url);
    let Some((esquema, resto)) = sin_credenciales.split_once("://") else {
        return sin_credenciales;
    };
    let host = resto.split(['/', '?', '#']).next().unwrap_or_default();
    format!("{esquema}://{host}")
}
