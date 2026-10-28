use crate::escaner::normalizar_url;
use crate::modelo::{
    Estado, Fuente, Item, Modo, PaqueteSistema, Paso, Posible, Recetario, Tipo, Via,
};
use crate::proceso::{ejecutar, ejecutar_en_sesion};
use crate::rutas::expandir;
use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use serde_json::Value;
use std::ffi::OsStr;
use std::fmt;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::thread;
use std::time::Duration;

/// Prefijo de la pista de un requisito agregado al investigar; la pantalla lo muestra como "Lo pide".
pub const PISTA_REQUISITO: &str = "requisito de ";

pub const TOPE: Duration = Duration::from_secs(5 * 60);
pub const PARALELO: usize = 4;

#[derive(Debug)]
enum Cuenta {
    Heredada,
    Predeterminada,
    Directorio(String),
}

static CUENTA_CLAUDE: OnceLock<Cuenta> = OnceLock::new();

#[derive(Debug)]
pub struct ErrorCuenta {
    pub shell: &'static str,
    pub causa: &'static str,
    pub operativo: bool,
    mensaje: &'static str,
    origen: Option<std::io::Error>,
}

impl fmt::Display for ErrorCuenta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.mensaje)
    }
}

impl std::error::Error for ErrorCuenta {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.origen.as_ref().map(|e| e as &dyn std::error::Error)
    }
}

enum CausaCuenta {
    Nombre,
    ShellAusente,
    ShellNoSoportada,
    Inicio,
    Tiempo,
    Programa,
    Inexistente,
    SinClaude,
    Fallo,
    Respuesta,
    YaFijada,
}

fn error_cuenta(shell: &'static str, causa: CausaCuenta) -> ErrorCuenta {
    let (causa, mensaje, operativo) = match causa {
        CausaCuenta::Nombre => (
            "nombre_invalido",
            "nombre de --profile inválido: usá letras ASCII, números, guion o guion bajo, sin empezar con guion",
            false,
        ),
        CausaCuenta::ShellAusente => (
            "shell_ausente",
            "SHELL está vacía o no está definida",
            false,
        ),
        CausaCuenta::ShellNoSoportada => (
            "shell_no_soportada",
            "shell no soportada: --profile necesita zsh o bash",
            false,
        ),
        CausaCuenta::Inicio => (
            "inicio",
            "no pude iniciar la shell para resolver --profile",
            true,
        ),
        CausaCuenta::Tiempo => (
            "timeout",
            "la shell superó el tiempo para resolver --profile",
            true,
        ),
        CausaCuenta::Programa => (
            "programa",
            "--profile es un programa: solo se aceptan alias o funciones",
            false,
        ),
        CausaCuenta::Inexistente => (
            "inexistente",
            "no existe un alias o función con ese nombre",
            false,
        ),
        CausaCuenta::SinClaude => (
            "sin_claude",
            "el alias o función no llama a claude o no devolvió una respuesta válida",
            false,
        ),
        CausaCuenta::Fallo => ("shell_fallo", "la shell falló al resolver --profile", true),
        CausaCuenta::Respuesta => (
            "respuesta_invalida",
            "la shell devolvió una respuesta de perfil inválida",
            true,
        ),
        CausaCuenta::YaFijada => (
            "ya_fijada",
            "la cuenta de Claude ya fue fijada para este proceso",
            true,
        ),
    };
    ErrorCuenta {
        shell,
        causa,
        operativo,
        mensaje,
        origen: None,
    }
}

pub fn configurar_cuenta(nombre: &str) -> std::result::Result<&'static str, ErrorCuenta> {
    if CUENTA_CLAUDE.get().is_some() {
        return Err(error_cuenta("sin_consultar", CausaCuenta::YaFijada));
    }
    let shell = std::env::var_os("SHELL");
    let (cuenta, shell) = resolver_cuenta(nombre, shell.as_deref(), Duration::from_secs(5))?;
    CUENTA_CLAUDE
        .set(cuenta)
        .map_err(|_| error_cuenta(shell, CausaCuenta::YaFijada))?;
    Ok(shell)
}

fn resolver_cuenta(
    nombre: &str,
    shell: Option<&OsStr>,
    tope: Duration,
) -> std::result::Result<(Cuenta, &'static str), ErrorCuenta> {
    if nombre.is_empty()
        || nombre.starts_with('-')
        || !nombre
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
    {
        return Err(error_cuenta("sin_consultar", CausaCuenta::Nombre));
    }
    let ruta = shell
        .filter(|s| !s.is_empty())
        .ok_or_else(|| error_cuenta("ausente", CausaCuenta::ShellAusente))?;
    let shell = match Path::new(ruta).file_name().and_then(OsStr::to_str) {
        Some("bash") => "bash",
        Some("zsh") => "zsh",
        _ => return Err(error_cuenta("no_soportada", CausaCuenta::ShellNoSoportada)),
    };
    let tipo = if shell == "bash" {
        format!(
            "case \"$(builtin type -t -- {nombre})\" in alias|function) ;; file) exit 64 ;; *) exit 65 ;; esac"
        )
    } else {
        format!(
            "case \"$(builtin whence -w -- {nombre})\" in *': alias'|*': function') ;; *': command'|*': hashed') exit 64 ;; *) exit 65 ;; esac"
        )
    };
    let script = format!(
        "{tipo}\nfunction claude {{ builtin printf '\\nREBAKE_PROFILE:%s\\n' \"${{CLAUDE_CONFIG_DIR-}}\"; }}\n{nombre}\n"
    );
    let mut cmd = Command::new(ruta);
    cmd.args(["-ic", &script]).env_remove("CLAUDE_CONFIG_DIR");
    let salida = ejecutar_en_sesion(&mut cmd, tope, &mut |_| {}).map_err(|origen| {
        let mut error = error_cuenta(shell, CausaCuenta::Inicio);
        error.origen = Some(origen);
        error
    })?;
    let fallo = if salida.vencido {
        Some(CausaCuenta::Tiempo)
    } else {
        match salida.codigo {
            Some(0) => None,
            Some(64) => Some(CausaCuenta::Programa),
            Some(65) => Some(CausaCuenta::Inexistente),
            Some(66) => Some(CausaCuenta::SinClaude),
            _ => Some(CausaCuenta::Fallo),
        }
    };
    if let Some(causa) = fallo {
        return Err(error_cuenta(shell, causa));
    }
    let mut registros = salida
        .stdout
        .lines()
        .filter_map(|l| l.strip_prefix("REBAKE_PROFILE:"));
    let valor = registros
        .next()
        .ok_or_else(|| error_cuenta(shell, CausaCuenta::SinClaude))?;
    if registros.next().is_some() || valor.chars().any(char::is_control) {
        return Err(error_cuenta(shell, CausaCuenta::Respuesta));
    }
    let cuenta = if valor.is_empty() {
        Cuenta::Predeterminada
    } else {
        Cuenta::Directorio(valor.to_owned())
    };
    Ok((cuenta, shell))
}

fn aplicar_cuenta(cmd: &mut Command, cuenta: &Cuenta) {
    match cuenta {
        Cuenta::Heredada => {}
        Cuenta::Predeterminada => {
            cmd.env_remove("CLAUDE_CONFIG_DIR");
        }
        Cuenta::Directorio(dir) => {
            cmd.env("CLAUDE_CONFIG_DIR", dir);
        }
    }
}

#[cfg(test)]
mod cuenta_tests {
    use super::*;
    use std::ffi::OsStr;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn el_command_aplica_cada_estado_sin_leer_globales() {
        for (cuenta, esperado) in [
            (Cuenta::Heredada, Some("heredada")),
            (Cuenta::Predeterminada, None),
            (Cuenta::Directorio("elegida".into()), Some("elegida")),
        ] {
            let mut cmd = Command::new("claude");
            cmd.env("CLAUDE_CONFIG_DIR", "heredada");
            aplicar_cuenta(&mut cmd, &cuenta);
            let valor = cmd
                .get_envs()
                .find(|(k, _)| *k == "CLAUDE_CONFIG_DIR")
                .unwrap()
                .1;
            assert_eq!(valor, esperado.map(OsStr::new));
        }
    }

    #[test]
    fn la_sonda_tiene_tope() {
        let dir = tempfile::tempdir().unwrap();
        let shell = dir.path().join("bash");
        std::fs::write(&shell, "#!/bin/sh\nsleep 30\n").unwrap();
        std::fs::set_permissions(&shell, std::fs::Permissions::from_mode(0o755)).unwrap();
        let error = resolver_cuenta("clp", Some(shell.as_os_str()), Duration::ZERO).unwrap_err();
        assert_eq!(error.causa, "timeout");
        assert!(error.operativo);
    }
}

pub const ESQUEMA: &str = r#"{"type":"object","additionalProperties":false,"required":["repo","doc","pasos","requiere"],"properties":{
"repo":{"type":"string"},"doc":{"type":"string"},"verificar":{"type":"string"},
"registro_hooks":{"type":"object","additionalProperties":false,"required":["tipo"],"properties":{"tipo":{"type":"string","enum":["paso","integrado","pendiente"]},"indice":{"type":"integer","minimum":0},"cita":{"type":"string"}}},
"pasos":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["cmd","modo","por_perfil","cita"],"properties":{
  "cmd":{"type":"string"},"modo":{"type":"string","enum":["auto","manual"]},"por_perfil":{"type":"boolean"},"cita":{"type":"string"},"nota":{"type":"string"}}}},
"requiere":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["tipo","nombre"],"properties":{
  "tipo":{"type":"string","enum":["herramienta","plugin","marketplace","skill","sistema"]},"nombre":{"type":"string"}}}}}}"#;

pub const ESQUEMA_ORIGEN: &str = r#"{"type":"object","additionalProperties":false,"required":["repo","motivo"],"properties":{
"repo":{"type":"string"},"motivo":{"type":"string"}}}"#;

const REGLAS: &str = "Reglas:
1. Usá la instalación que documenta el repo (README o docs). No inventes comandos: cada paso lleva en `cita` la línea literal de la doc de donde sale.
2. Comandos para la terminal, no para dentro de Claude: `/plugin marketplace add X` → `claude plugin marketplace add X`; `/plugin install X` → `claude plugin install X`. Lo que solo existe dentro de Claude (p. ej. `/codex:setup`) o pide un login va con modo \"manual\".
3. Usá flags no interactivos (p. ej. -y) cuando la doc los ofrece. Cada paso tiene que poder correrse dos veces sin fallar: si un comando falla al repetirse, envolvelo con una guarda (p. ej. `[ -d ~/x ] || git clone URL ~/x`).
4. por_perfil = true si el paso configura un perfil de Claude Code: se corre una vez por perfil con CLAUDE_CONFIG_DIR apuntando a ese perfil. Si el instalador no respeta CLAUDE_CONFIG_DIR, decilo en `nota`.
5. verificar: un comando de shell que termine con código 0 solo si ya está instalado; para ítems por perfil usá \"$CLAUDE_CONFIG_DIR\".
6. requiere: lo que hay que instalar antes. Los paquetes de pacman van con tipo \"sistema\" y su nombre de paquete; no los pongas como pasos. En `nombre` va solo el identificador (p. ej. `codex`, `openai-codex`), sin aclaraciones.
7. Nunca incluyas tokens, claves ni datos personales.
8. doc: la URL exacta de la sección de instalación que usaste.";

#[derive(Debug, Clone)]
pub struct Trabajo {
    pub id: String,
    pub prompt: String,
    pub directa: Option<Receta>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requisito {
    pub tipo: String,
    pub nombre: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Receta {
    pub repo: String,
    pub doc: String,
    pub pasos: Vec<Paso>,
    pub requiere: Vec<Requisito>,
    pub verificar: Option<String>,
    pub registro_hooks: Option<RegistroHooks>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case", deny_unknown_fields)]
pub enum RegistroHooks {
    Paso { indice: usize },
    Integrado { cita: String },
    Pendiente,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Respuesta {
    repo: String,
    doc: String,
    #[serde(default)]
    verificar: Option<String>,
    pasos: Vec<PasoRespuesta>,
    requiere: Vec<Requisito>,
    #[serde(default)]
    registro_hooks: Option<RegistroHooks>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasoRespuesta {
    cmd: String,
    modo: Modo,
    por_perfil: bool,
    cita: String,
    #[serde(default)]
    nota: Option<String>,
}

pub fn binario_claude() -> String {
    std::env::var("REBAKE_CLAUDE").unwrap_or_else(|_| "claude".into())
}

pub fn modelo() -> String {
    std::env::var("REBAKE_MODELO").unwrap_or_else(|_| "sonnet".into())
}

pub fn argumentos(prompt: &str, modelo: &str) -> Vec<String> {
    argumentos_con(prompt, modelo, ESQUEMA)
}

fn argumentos_con(prompt: &str, modelo: &str, esquema: &str) -> Vec<String> {
    [
        "-p",
        prompt,
        "--model",
        modelo,
        "--restricted",
        "--strict-mcp-config",
        "--tools",
        "WebSearch,WebFetch",
        "--allowedTools",
        "WebSearch,WebFetch",
        "--no-session-persistence",
        "--output-format",
        "json",
        "--json-schema",
        esquema,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

pub fn trabajo_para(r: &Recetario, item: &Item, home: &Path) -> Trabajo {
    let perfiles: Vec<String> = item
        .perfiles
        .iter()
        .map(|n| match r.perfiles.iter().find(|p| &p.nombre == n) {
            Some(p) => format!("{n} ({})", p.dir),
            None => n.clone(),
        })
        .collect();
    let repo = item
        .fuente
        .as_ref()
        .map(|f| f.repo.clone())
        .unwrap_or_else(|| "ninguno: buscalo en la web".into());
    let mut prompt = format!(
        "Sos un investigador de instaladores. Encontrá cómo se instala UNA cosa del setup de Claude Code de un usuario en Linux (Arch), leyendo la documentación oficial de su repositorio, y devolvé la receta en el formato JSON pedido.\n\nÍtem: {}\nTipo: {:?}\nRepo conocido: {repo}\nPista: {}\nPerfiles: {}\n",
        item.id,
        item.tipo,
        item.pista.as_deref().unwrap_or("ninguna"),
        if perfiles.is_empty() {
            "ninguno (va una sola vez)".into()
        } else {
            perfiles.join(", ")
        },
    );
    if let Some(cabeza) = cabeza_de_archivo(item, home) {
        prompt.push_str(&format!(
            "\nPrimeras líneas del archivo:\n```\n{cabeza}\n```\n"
        ));
    }
    prompt.push('\n');
    prompt.push_str(REGLAS);
    let hooks = r.hooks_de(&item.id).count();
    if hooks > 0 {
        prompt.push_str(&format!("\nTiene hooks vinculados: {hooks}. Pedí a la documentación del repo cómo registrarlos después de instalar. Devolvé registro_hooks: {{\"tipo\":\"paso\",\"indice\":N}} si un paso citado registra los hooks (índice desde cero, después del instalador); {{\"tipo\":\"integrado\",\"cita\":\"texto literal de la doc\"}} si el instalador ya los crea; o {{\"tipo\":\"pendiente\"}} si no hay documentación. Nunca inventes el registro ni copies hooks a settings.json. Indicá el alcance por perfil y sus límites."));
    }
    if item.tipo == Tipo::Mcp {
        let nombre = item.id.trim_start_matches("mcp:");
        prompt.push_str(&format!(
            "\n9. Es un servidor MCP: los pasos instalan el programa que lo sirve y el último lo registra en Claude Code con `claude mcp add --scope user {nombre} -- <el comando de la pista>` (o `--transport http {nombre} <url>` si la pista es una URL). Todos los pasos van con modo \"manual\": los corre el usuario. No pongas CLAUDE_CONFIG_DIR en los comandos ni en las notas: rebake arma la línea de cada perfil."
        ));
    }
    Trabajo {
        id: item.id.clone(),
        prompt,
        directa: receta_directa(r, item),
    }
}

fn es_identificador(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

// Estos valores terminan en `sh -c`: lista blanca estricta en vez de buscar metacaracteres.
fn es_repo_seguro(s: &str) -> bool {
    s.starts_with("https://")
        && s.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || matches!(c, '.' | '_' | '-' | '/' | ':' | '~' | '%' | '+' | '=' | '@')
        })
}

/// Lo que tiene instalador conocido no necesita a Claude; si algo no es seguro, se investiga.
pub(crate) fn receta_directa(r: &Recetario, item: &Item) -> Option<Receta> {
    if item.instalado_por.is_some() || r.hooks_de(&item.id).next().is_some() {
        return None;
    }
    let fuente = item.fuente.as_ref()?;
    let repo = fuente.repo.as_str();
    if !es_repo_seguro(repo) {
        return None;
    }
    let (doc, cmd, verificar) = match item.tipo {
        Tipo::Marketplace => {
            let nombre = item.id.strip_prefix("marketplace:")?;
            if !es_identificador(nombre) {
                return None;
            }
            (
                "receta directa: instalador de plugins de Claude Code",
                format!("claude plugin marketplace add {repo}"),
                format!("claude plugin marketplace list | grep -qF -- '{nombre}'"),
            )
        }
        Tipo::Plugin => {
            let completo = item.id.strip_prefix("plugin:")?;
            let (nombre, mercado) = completo.split_once('@')?;
            if !es_identificador(nombre) || !es_identificador(mercado) {
                return None;
            }
            (
                "receta directa: instalador de plugins de Claude Code",
                format!(
                    "claude plugin marketplace list | grep -qF -- '{mercado}' || {{ echo 'falta el marketplace {mercado}: aprobá marketplace:{mercado}' >&2; exit 1; }}; claude plugin install {completo}"
                ),
                format!("claude plugin list | grep -qF -- '{completo}'"),
            )
        }
        Tipo::Skill if fuente.via == Via::Metadatos => {
            let nombre = item.id.strip_prefix("skill:")?;
            if !es_identificador(nombre) {
                return None;
            }
            (
                "receta directa: instalador de skills.sh",
                format!(
                    "command -v npx >/dev/null || {{ echo 'falta npx: instalá Node.js' >&2; exit 1; }}; npx -y skills add {repo} --skill {nombre} -g -a claude-code -y"
                ),
                format!("test -f \"$CLAUDE_CONFIG_DIR/skills/{nombre}/SKILL.md\""),
            )
        }
        _ => return None,
    };
    Some(Receta {
        repo: repo.into(),
        doc: doc.into(),
        pasos: vec![Paso {
            cmd,
            modo: Modo::Auto,
            por_perfil: true,
            cita: None,
            nota: Some(doc.into()),
        }],
        requiere: vec![],
        verificar: Some(verificar),
        registro_hooks: None,
    })
}

/// La pista de un archivo sin origen termina en su ruta (`…: ~/x`); su contenido orienta la búsqueda.
fn cabeza_de_archivo(item: &Item, home: &Path) -> Option<String> {
    let ruta = expandir(item.pista.as_ref()?.rsplit_once(": ")?.1, home);
    let archivo = if ruta.is_dir() {
        ruta.join("SKILL.md")
    } else {
        ruta
    };
    let texto = std::fs::read_to_string(archivo).ok()?;
    // Lo que va al prompt puede terminar en una URL si una página inyecta instrucciones:
    // sin secretos, no hay nada que filtrar.
    Some(
        texto
            .lines()
            .take(20)
            .map(crate::secretos::ocultar)
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

pub fn investigar(t: &Trabajo, claude: &str, tope: Duration) -> Result<Receta> {
    if let Some(directa) = &t.directa {
        return Ok(directa.clone());
    }
    let mut cmd = Command::new(claude);
    aplicar_cuenta(&mut cmd, CUENTA_CLAUDE.get().unwrap_or(&Cuenta::Heredada));
    cmd.args(argumentos(&t.prompt, &modelo()))
        .current_dir(std::env::temp_dir());
    let salida = ejecutar(&mut cmd, tope, &mut |_| {})
        .with_context(|| format!("no pude ejecutar {claude}: ¿está instalado?"))?;
    if salida.vencido {
        bail!("la investigación superó {} segundos", tope.as_secs_f32());
    }
    parsear_salida(&salida.texto).with_context(|| match salida.codigo {
        Some(0) => "claude respondió algo inesperado".to_string(),
        c => format!("claude terminó con código {c:?}"),
    })
}

pub fn parsear_salida(stdout: &str) -> Result<Receta> {
    let r: Respuesta = serde_json::from_value(estructurada(stdout)?)
        .map_err(|e| anyhow!("la respuesta no cumple el formato: {e}"))?;
    if r.pasos.iter().any(|p| p.cmd.trim().is_empty()) {
        bail!("la respuesta no cumple el formato: hay un paso sin comando");
    }
    let doc = primera_url(&r.doc)
        .ok_or_else(|| anyhow!("la respuesta no trae la URL de la doc: {}", r.doc))?;
    let receta = Receta {
        repo: r.repo,
        doc,
        verificar: r.verificar.filter(|v| !v.trim().is_empty()),
        registro_hooks: r.registro_hooks,
        requiere: r.requiere,
        pasos: r
            .pasos
            .into_iter()
            .map(|p| Paso {
                cmd: p.cmd,
                modo: p.modo,
                por_perfil: p.por_perfil,
                cita: Some(p.cita),
                nota: p.nota,
            })
            .collect(),
    };
    match &receta.registro_hooks {
        Some(RegistroHooks::Paso { indice })
            if *indice == 0
                || receta
                    .pasos
                    .get(*indice)
                    .is_none_or(|p| p.cita.as_ref().is_none_or(|c| c.trim().is_empty())) =>
        {
            bail!("registro de hooks sin paso posterior citado")
        }
        Some(RegistroHooks::Integrado { cita })
            if cita.trim().is_empty() || receta.pasos.is_empty() =>
        {
            bail!("registro integrado sin documentación")
        }
        _ => {}
    }
    Ok(receta)
}

/// El `structured_output` de la respuesta de `claude -p`, si terminó bien y pudo usar la web.
fn estructurada(stdout: &str) -> Result<Value> {
    // stdout y stderr llegan mezclados: el resultado es la última línea JSON de tipo "result".
    let resultado: Value = stdout
        .lines()
        .rev()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .find(|v| v.get("type").and_then(Value::as_str) == Some("result"))
        .ok_or_else(|| anyhow!("no encontré la respuesta de claude en la salida"))?;
    if resultado.get("is_error").and_then(Value::as_bool) == Some(true)
        || resultado.get("subtype").and_then(Value::as_str) != Some("success")
    {
        let detalle = resultado
            .get("result")
            .and_then(Value::as_str)
            .unwrap_or("sin detalle");
        bail!("claude no pudo investigar: {detalle}");
    }
    // Sin permiso para leer la web, la "receta" sería inventada: se trata como error.
    let denegados: Vec<&str> = resultado
        .get("permission_denials")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|d| d.get("tool_name").and_then(Value::as_str))
        .collect();
    if !denegados.is_empty() {
        bail!("claude no tuvo permiso para usar {}", denegados.join(", "));
    }
    resultado
        .get("structured_output")
        .cloned()
        .ok_or_else(|| anyhow!("la respuesta no trae structured_output"))
}

pub fn en_paralelo(
    trabajos: Vec<Trabajo>,
    claude: String,
    tope: Duration,
    paralelo: usize,
) -> mpsc::Receiver<(String, Result<Receta>)> {
    let mut trabajos = trabajos;
    // La fila se toma desde el final: las directas van ahí para no esperar a las de Claude.
    trabajos.sort_by_key(|t| t.directa.is_some());
    en_hilos(trabajos, paralelo, move |t| {
        let r = investigar(&t, &claude, tope);
        (t.id, r)
    })
}

/// Busca el posible origen de cada `(id, prompt)`, de a `paralelo` por vez.
pub fn buscar_origenes(
    trabajos: Vec<(String, String)>,
    claude: String,
    tope: Duration,
    paralelo: usize,
) -> mpsc::Receiver<(String, Result<Posible>)> {
    en_hilos(trabajos, paralelo, move |(id, prompt)| {
        let r = buscar_origen(&prompt, &claude, tope);
        (id, r)
    })
}

fn en_hilos<T, R, F>(trabajos: Vec<T>, paralelo: usize, f: F) -> mpsc::Receiver<(String, R)>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> (String, R) + Send + Sync + 'static,
{
    let (tx, rx) = mpsc::channel();
    let cola = Arc::new(Mutex::new(trabajos));
    let f = Arc::new(f);
    for _ in 0..paralelo.max(1) {
        let (tx, cola, f) = (tx.clone(), Arc::clone(&cola), Arc::clone(&f));
        thread::spawn(move || {
            loop {
                let siguiente = cola.lock().unwrap_or_else(|e| e.into_inner()).pop();
                let Some(t) = siguiente else { break };
                if tx.send(f(t)).is_err() {
                    break;
                }
            }
        });
    }
    rx
}

/// Pregunta corta: solo de qué repo viene, sin receta. Lo que vuelve espera el OK de Denis.
pub fn prompt_origen(item: &Item, home: &Path) -> String {
    let mut prompt = format!(
        "Buscá en la web de qué repositorio público viene UNA cosa del setup de Claude Code de un usuario en Linux. No armes la receta de instalación.\n\nÍtem: {}\nTipo: {:?}\nPista: {}\n",
        item.id,
        item.tipo,
        item.pista.as_deref().unwrap_or("ninguna"),
    );
    if let Some(cabeza) = cabeza_de_archivo(item, home) {
        prompt.push_str(&format!(
            "\nPrimeras líneas del archivo:\n```\n{cabeza}\n```\n"
        ));
    }
    prompt.push_str("\nDevolvé en `repo` la URL https del repositorio oficial (p. ej. https://github.com/dueño/repo), o vacío si no estás seguro: es mejor no proponer nada que proponer uno equivocado. En `motivo`, una oración con la evidencia que lo une al ítem. Nunca incluyas tokens, claves ni datos personales.");
    prompt
}

pub fn buscar_origen(prompt: &str, claude: &str, tope: Duration) -> Result<Posible> {
    let mut cmd = Command::new(claude);
    aplicar_cuenta(&mut cmd, CUENTA_CLAUDE.get().unwrap_or(&Cuenta::Heredada));
    cmd.args(argumentos_con(prompt, &modelo(), ESQUEMA_ORIGEN))
        .current_dir(std::env::temp_dir());
    let salida = ejecutar(&mut cmd, tope, &mut |_| {})
        .with_context(|| format!("no pude ejecutar {claude}: ¿está instalado?"))?;
    if salida.vencido {
        bail!("la búsqueda superó {} segundos", tope.as_secs_f32());
    }
    parsear_origen(&salida.texto).with_context(|| match salida.codigo {
        Some(0) => "claude respondió algo inesperado".to_string(),
        c => format!("claude terminó con código {c:?}"),
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RespuestaOrigen {
    repo: String,
    motivo: String,
}

pub fn parsear_origen(stdout: &str) -> Result<Posible> {
    let r: RespuestaOrigen = serde_json::from_value(estructurada(stdout)?)
        .map_err(|e| anyhow!("la respuesta no cumple el formato: {e}"))?;
    let motivo = r.motivo.trim().to_string();
    let repo = normalizar_url(&r.repo);
    if repo.is_empty() {
        return Ok(Posible {
            repo: None,
            motivo,
            descartado: false,
        });
    }
    // El repo termina en un `sh -c` si se acepta: lo raro ni se propone.
    if !es_repo_seguro(&repo) {
        return Ok(Posible {
            repo: None,
            motivo: "lo que encontró no es un link https válido".into(),
            descartado: false,
        });
    }
    Ok(Posible {
        repo: Some(repo),
        motivo,
        descartado: false,
    })
}

/// Guarda lo que encontró la búsqueda, salvo que Denis ya haya resuelto el origen o lo excluyó.
pub fn aplicar_posible(r: &mut Recetario, id: &str, resultado: Result<Posible>) {
    let Some(item) = r.item_mut(id) else { return };
    if item.instalado_por.is_some() || item.fuente.is_some() || item.estado == Estado::Excluida {
        return;
    }
    match resultado {
        Ok(p) => {
            item.posible = Some(p);
            item.error = None;
        }
        Err(e) => item.error = Some(format!("{e:#}")),
    }
}

pub fn aplicar(r: &mut Recetario, id: &str, resultado: Result<Receta>, hoy: &str) {
    let necesita_registro = r.hooks_de(id).next().is_some();
    // El repo de cada ítem ya está verificado: un requisito que llega como `dueño/repo` se
    // resuelve contra él en vez de crear un ítem con el nombre mal armado.
    let por_repo: Vec<(Tipo, String, String)> = r
        .items
        .iter()
        .filter_map(|i| {
            let f = i.fuente.as_ref()?;
            Some((i.tipo, normalizar_url(&f.repo).to_lowercase(), i.id.clone()))
        })
        .collect();
    let Some(item) = r.item_mut(id) else { return };
    if item.instalado_por.is_some() {
        return;
    }
    let receta = match resultado {
        Ok(receta) => receta,
        Err(e) => {
            item.error = Some(format!("{e:#}"));
            return;
        }
    };
    // Si Denis lo excluyó mientras se investigaba, gana su decisión.
    if item.estado == Estado::Excluida {
        return;
    }
    if necesita_registro && matches!(receta.registro_hooks, None | Some(RegistroHooks::Pendiente)) {
        item.estado = Estado::Pendiente;
        item.pasos.clear();
        item.error =
            Some("registro de hooks pendiente: falta documentación del repositorio".into());
        return;
    }
    let repo = normalizar_url(&receta.repo);
    let via = match &item.fuente {
        Some(f) if normalizar_url(&f.repo).eq_ignore_ascii_case(&repo) => f.via,
        _ => Via::Busqueda,
    };
    item.fuente = Some(Fuente {
        repo,
        via,
        doc: Some(receta.doc),
    });
    item.pasos = receta.pasos;
    if let Some(RegistroHooks::Integrado { cita }) = receta.registro_hooks
        && let Some(paso) = item.pasos.first_mut()
    {
        let nota = format!("Hooks incluidos por el instalador: {cita}");
        paso.nota = Some(match paso.nota.take() {
            Some(anterior) => format!("{anterior}\n{nota}"),
            None => nota,
        });
    }
    item.verificar = receta.verificar;
    if item.tipo == Tipo::Mcp {
        // Los corre Denis después de leerlos; el instalador igual los saltea.
        for p in &mut item.pasos {
            p.modo = Modo::Manual;
        }
        let nombre = item.id.trim_start_matches("mcp:");
        item.verificar =
            es_identificador(nombre).then(|| format!("claude mcp get '{nombre}' >/dev/null 2>&1"));
    }
    item.estado = Estado::PorRevisar;
    item.investigado = Some(hoy.into());
    item.error = None;
    let mut nuevos = Vec::new();
    let mut sistema = Vec::new();
    for req in receta.requiere {
        // El modelo a veces agrega aclaraciones ("codex (@openai/codex)"); el id es solo el nombre.
        let nombre = req
            .nombre
            .split(|c: char| c.is_whitespace() || c == '(')
            .next()
            .unwrap_or_default()
            .to_string();
        if nombre.is_empty() {
            continue;
        }
        if req.tipo == "sistema" {
            sistema.push(PaqueteSistema {
                paquete: nombre,
                para: id.into(),
            });
            continue;
        }
        // Claude Code es requisito de rebake, no una receta (desde v0.6.2).
        if nombre == "claude" {
            continue;
        }
        let tipo = tipo_de(&req.tipo);
        let slug = normalizar_url(&nombre).to_lowercase();
        // Solo si un único ítem de ese tipo tiene ese repo: un marketplace trae muchos plugins.
        let mismos: Vec<&String> = por_repo
            .iter()
            .filter(|(t, repo, _)| {
                slug.contains('/')
                    && *t == tipo
                    && (repo == &slug || repo.ends_with(&format!("/{slug}")))
            })
            .map(|(_, _, existente)| existente)
            .collect();
        let rid = match mismos.as_slice() {
            [unico] => (*unico).clone(),
            _ => format!("{}:{}", req.tipo, nombre),
        };
        if rid == id {
            continue;
        }
        if !item.requiere.contains(&rid) {
            item.requiere.push(rid.clone());
        }
        nuevos.push((rid, tipo));
    }
    for (rid, tipo) in nuevos {
        // Si ya existe (aunque esté excluido) no se toca: lo decidido por Denis se respeta.
        if r.item(&rid).is_none() {
            let mut nuevo = Item::nuevo(&rid, tipo);
            nuevo.pista = Some(format!("{PISTA_REQUISITO}{id}"));
            r.items.push(nuevo);
        }
    }
    for s in sistema {
        if !r.checklist.sistema.contains(&s) {
            r.checklist.sistema.push(s);
        }
    }
}

fn tipo_de(tipo: &str) -> Tipo {
    match tipo {
        "plugin" => Tipo::Plugin,
        "marketplace" => Tipo::Marketplace,
        "skill" => Tipo::Skill,
        _ => Tipo::Herramienta,
    }
}

fn primera_url(texto: &str) -> Option<String> {
    let inicio = texto.find("https://").or_else(|| texto.find("http://"))?;
    let url: String = texto[inicio..]
        .chars()
        .take_while(|c| !c.is_whitespace() && !matches!(c, ')' | ']' | '>' | '"' | '\''))
        .collect();
    Some(url.trim_end_matches(['.', ',']).to_string())
}
