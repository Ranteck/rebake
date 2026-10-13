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

/// Qué hacer con la palabra que viene después de la que se acaba de leer.
enum Siguiente {
    Nada,
    Secreto,
    UsuarioClave,
}

/// `usuario:clave` sin la clave. Un par de números es usuario y grupo (`docker run -u 1000:1000`).
fn ocultar_usuario_clave(texto: &str) -> Option<String> {
    let (usuario, clave) = texto.split_once(':')?;
    let numero = |t: &str| t.bytes().all(|b| b.is_ascii_digit());
    if usuario.is_empty() || clave.is_empty() || texto.contains("://") {
        return None;
    }
    if numero(usuario) && numero(clave) {
        return None;
    }
    Some(format!("{usuario}:{OCULTO}"))
}

/// Oculta secretos dentro de un texto libre (un comando de hook, un valor de ajuste).
pub fn ocultar(texto: &str) -> String {
    let mut siguiente = Siguiente::Nada;
    texto
        .split(' ')
        .map(|parte| {
            let limpia = parte.trim_matches(|c| c == '"' || c == '\'');
            if limpia.is_empty() {
                return parte.to_string();
            }
            match std::mem::replace(&mut siguiente, Siguiente::Nada) {
                Siguiente::Secreto if es_esquema_de_auth(limpia) => {
                    siguiente = Siguiente::Secreto;
                    return parte.to_string();
                }
                Siguiente::Secreto => return parte.replacen(limpia, OCULTO, 1),
                Siguiente::UsuarioClave => {
                    if let Some(oculta) = ocultar_usuario_clave(limpia) {
                        return parte.replacen(limpia, &oculta, 1);
                    }
                }
                Siguiente::Nada => {}
            }
            // `-u usuario:clave` y `--user usuario:clave` (curl).
            if limpia == "-u" || limpia == "--user" {
                siguiente = Siguiente::UsuarioClave;
                return parte.to_string();
            }
            if let Some(oculta) = limpia
                .strip_prefix("--user=")
                .and_then(ocultar_usuario_clave)
            {
                return parte.replacen(limpia, &format!("--user={oculta}"), 1);
            }
            // `Bearer X`, `Authorization: X` y `--token X`: el secreto es la palabra siguiente.
            let flag_sensible = limpia.starts_with('-')
                && !limpia.contains('=')
                && clave_sensible(limpia.trim_start_matches('-'));
            if es_esquema_de_auth(limpia)
                || (!limpia.contains("://") && limpia.strip_suffix(':').is_some_and(clave_sensible))
                || flag_sensible
            {
                siguiente = Siguiente::Secreto;
                return parte.to_string();
            }
            // Cabecera pegada en una sola palabra: `X-Api-Key:valor`, `Authorization:Bearer`.
            if !limpia.contains("://")
                && let Some((nombre, valor)) = limpia.split_once(':')
                && clave_sensible(nombre)
                && !valor.is_empty()
            {
                if es_esquema_de_auth(valor) {
                    siguiente = Siguiente::Secreto;
                    return parte.to_string();
                }
                return parte.replacen(limpia, &format!("{nombre}:{OCULTO}"), 1);
            }
            parte.replacen(limpia, &ocultar_palabra(limpia), 1)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Una palabra puede encadenar pares (`a=1&token=x`, `A=1;TOKEN=x`) y terminar en una URL.
fn ocultar_palabra(palabra: &str) -> String {
    // Desde la URL no se separa: ahí `&` es parte de la consulta, que se descarta entera.
    let corte = palabra.find("://").map_or(palabra.len(), |url| {
        palabra[..url].rfind(['&', ';']).map_or(0, |sep| sep + 1)
    });
    let (pares, url) = palabra.split_at(corte);
    let mut salida: String = pares.split_inclusive(['&', ';']).map(ocultar_par).collect();
    if !url.is_empty() {
        salida.push_str(&ocultar_url(url));
    }
    salida
}

/// `trozo` es un par `nombre=valor` (o texto suelto), con su separador final si lo trae.
fn ocultar_par(trozo: &str) -> String {
    let par = trozo.trim_end_matches(['&', ';']);
    let separador = &trozo[par.len()..];
    if let Some((nombre, valor)) = par.split_once('=')
        && !valor.is_empty()
        && (clave_sensible(nombre) || parece_secreto(valor))
    {
        return format!("{nombre}={OCULTO}{separador}");
    }
    if parece_secreto(par) {
        return format!("{OCULTO}{separador}");
    }
    trozo.to_string()
}

fn ocultar_url(url: &str) -> String {
    if let Some((nombre, valor)) = url.split_once('=')
        && !nombre.contains("://")
        && clave_sensible(nombre)
        && !valor.is_empty()
    {
        return format!("{nombre}={OCULTO}");
    }
    // En texto libre una URL puede llevar el secreto en la ruta (webhooks): basta el host.
    solo_host(url)
}

fn solo_host(url: &str) -> String {
    let sin_credenciales = sin_credenciales_url(url);
    let Some((esquema, resto)) = sin_credenciales.split_once("://") else {
        return sin_credenciales;
    };
    // `&` y `;` no van en un host: lo que sigue son otros pares pegados a la URL.
    let host = resto
        .split(['/', '?', '#', '&', ';'])
        .next()
        .unwrap_or_default();
    format!("{esquema}://{host}")
}
