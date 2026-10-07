use crate::escaner::normalizar_url;
use crate::modelo::{Estado, Fuente, Item, Modo, PaqueteSistema, Paso, Recetario, Tipo, Via};
use crate::proceso::ejecutar;
use crate::rutas::expandir;
use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

pub const TOPE: Duration = Duration::from_secs(5 * 60);
pub const PARALELO: usize = 4;

pub const ESQUEMA: &str = r#"{"type":"object","additionalProperties":false,"required":["repo","doc","pasos","requiere"],"properties":{
"repo":{"type":"string"},"doc":{"type":"string"},"verificar":{"type":"string"},
"pasos":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["cmd","modo","por_perfil","cita"],"properties":{
  "cmd":{"type":"string"},"modo":{"type":"string","enum":["auto","manual"]},"por_perfil":{"type":"boolean"},"cita":{"type":"string"},"nota":{"type":"string"}}}},
"requiere":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["tipo","nombre"],"properties":{
  "tipo":{"type":"string","enum":["herramienta","plugin","marketplace","skill","sistema"]},"nombre":{"type":"string"}}}}}}"#;

const REGLAS: &str = "Reglas:
1. Usá la instalación que documenta el repo (README o docs). No inventes comandos: cada paso lleva en `cita` la línea literal de la doc de donde sale.
2. Comandos para la terminal, no para dentro de Claude: `/plugin marketplace add X` → `claude plugin marketplace add X`; `/plugin install X` → `claude plugin install X`. Lo que solo existe dentro de Claude (p. ej. `/codex:setup`) o pide un login va con modo \"manual\".
3. Usá flags no interactivos (p. ej. -y) cuando la doc los ofrece. Cada paso tiene que poder correrse dos veces sin fallar: si un comando falla al repetirse, envolvelo con una guarda (p. ej. `[ -d ~/x ] || git clone URL ~/x`).
4. por_perfil = true si el paso configura un perfil de Claude Code: se corre una vez por perfil con CLAUDE_CONFIG_DIR apuntando a ese perfil. Si el instalador no respeta CLAUDE_CONFIG_DIR, decilo en `nota`.
5. verificar: un comando de shell que termine con código 0 solo si ya está instalado; para ítems por perfil usá \"$CLAUDE_CONFIG_DIR\".
6. requiere: lo que hay que instalar antes. Los paquetes de pacman van con tipo \"sistema\" y su nombre de paquete; no los pongas como pasos.
7. Nunca incluyas tokens, claves ni datos personales.
8. doc: la URL exacta de la sección de instalación que usaste.";

#[derive(Debug, Clone)]
pub struct Trabajo {
    pub id: String,
    pub prompt: String,
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
    std::env::var("RECETARIO_CLAUDE").unwrap_or_else(|_| "claude".into())
}

pub fn argumentos(prompt: &str) -> Vec<String> {
    [
        "-p", prompt, "--restricted", "--strict-mcp-config", "--tools", "WebSearch,WebFetch",
        "--no-session-persistence", "--output-format", "json", "--json-schema", ESQUEMA,
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
    let repo = item.fuente.as_ref().map(|f| f.repo.clone()).unwrap_or_else(|| "ninguno: buscalo en la web".into());
    let mut prompt = format!(
        "Sos un investigador de instaladores. Encontrá cómo se instala UNA cosa del setup de Claude Code de un usuario en Linux (Arch), leyendo la documentación oficial de su repositorio, y devolvé la receta en el formato JSON pedido.\n\nÍtem: {}\nTipo: {:?}\nRepo conocido: {repo}\nPista: {}\nPerfiles: {}\n",
        item.id,
        item.tipo,
        item.pista.as_deref().unwrap_or("ninguna"),
        if perfiles.is_empty() { "ninguno (va una sola vez)".into() } else { perfiles.join(", ") },
    );
    if let Some(cabeza) = cabeza_de_archivo(item, home) {
        prompt.push_str(&format!("\nPrimeras líneas del archivo:\n```\n{cabeza}\n```\n"));
    }
    prompt.push('\n');
    prompt.push_str(REGLAS);
    Trabajo { id: item.id.clone(), prompt }
}

/// La pista de un archivo sin origen termina en su ruta (`…: ~/x`); su contenido orienta la búsqueda.
fn cabeza_de_archivo(item: &Item, home: &Path) -> Option<String> {
    let ruta = expandir(item.pista.as_ref()?.rsplit_once(": ")?.1, home);
    let archivo = if ruta.is_dir() { ruta.join("SKILL.md") } else { ruta };
    let texto = std::fs::read_to_string(archivo).ok()?;
    Some(texto.lines().take(20).collect::<Vec<_>>().join("\n"))
}

pub fn investigar(t: &Trabajo, claude: &str, tope: Duration) -> Result<Receta> {
    let mut cmd = Command::new(claude);
    cmd.args(argumentos(&t.prompt)).current_dir(std::env::temp_dir());
    let salida = ejecutar(&mut cmd, tope, &mut |_| {}).with_context(|| format!("no pude ejecutar {claude}: ¿está instalado?"))?;
    if salida.vencido {
        bail!("la investigación superó {} segundos", tope.as_secs_f32());
    }
    parsear_salida(&salida.texto).with_context(|| match salida.codigo {
        Some(0) => "claude respondió algo inesperado".to_string(),
        c => format!("claude terminó con código {c:?}"),
    })
}

pub fn parsear_salida(stdout: &str) -> Result<Receta> {
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
        let detalle = resultado.get("result").and_then(Value::as_str).unwrap_or("sin detalle");
        bail!("claude no pudo investigar: {detalle}");
    }
    let estructurada = resultado.get("structured_output").cloned().ok_or_else(|| anyhow!("la respuesta no trae structured_output"))?;
    let r: Respuesta = serde_json::from_value(estructurada).map_err(|e| anyhow!("la respuesta no cumple el formato: {e}"))?;
    if r.pasos.iter().any(|p| p.cmd.trim().is_empty()) {
        bail!("la respuesta no cumple el formato: hay un paso sin comando");
    }
    Ok(Receta {
        repo: r.repo,
        doc: r.doc,
        verificar: r.verificar.filter(|v| !v.trim().is_empty()),
        requiere: r.requiere,
        pasos: r
            .pasos
            .into_iter()
            .map(|p| Paso { cmd: p.cmd, modo: p.modo, por_perfil: p.por_perfil, cita: Some(p.cita), nota: p.nota })
            .collect(),
    })
}

pub fn en_paralelo(trabajos: Vec<Trabajo>, claude: String, tope: Duration, paralelo: usize) -> mpsc::Receiver<(String, Result<Receta>)> {
    let (tx, rx) = mpsc::channel();
    let cola = Arc::new(Mutex::new(trabajos));
    for _ in 0..paralelo.max(1) {
        let (tx, cola, claude) = (tx.clone(), Arc::clone(&cola), claude.clone());
        thread::spawn(move || loop {
            let siguiente = cola.lock().unwrap_or_else(|e| e.into_inner()).pop();
            let Some(t) = siguiente else { break };
            let resultado = investigar(&t, &claude, tope);
            if tx.send((t.id, resultado)).is_err() { break; }
        });
    }
    rx
}

pub fn aplicar(r: &mut Recetario, id: &str, resultado: Result<Receta>, hoy: &str) {
    let Some(item) = r.item_mut(id) else { return };
    let receta = match resultado {
        Ok(receta) => receta,
        Err(e) => {
            item.error = Some(format!("{e:#}"));
            return;
        }
    };
    // Si Denis lo excluyó mientras se investigaba, gana su decisión.
    if item.estado == Estado::Excluida { return; }
    let repo = normalizar_url(&receta.repo);
    let via = match &item.fuente {
        Some(f) if normalizar_url(&f.repo).eq_ignore_ascii_case(&repo) => f.via,
        _ => Via::Busqueda,
    };
    item.fuente = Some(Fuente { repo, via, doc: Some(receta.doc) });
    item.pasos = receta.pasos;
    item.verificar = receta.verificar;
    item.estado = Estado::PorRevisar;
    item.investigado = Some(hoy.into());
    item.error = None;
    let mut nuevos = Vec::new();
    let mut sistema = Vec::new();
    for req in receta.requiere {
        if req.tipo == "sistema" {
            sistema.push(PaqueteSistema { paquete: req.nombre, para: id.into() });
            continue;
        }
        let rid = format!("{}:{}", req.tipo, req.nombre);
        if rid == id { continue; }
        if !item.requiere.contains(&rid) { item.requiere.push(rid.clone()); }
        nuevos.push((rid, tipo_de(&req.tipo)));
    }
    for (rid, tipo) in nuevos {
        // Si ya existe (aunque esté excluido) no se toca: lo decidido por Denis se respeta.
        if r.item(&rid).is_none() {
            let mut nuevo = Item::nuevo(&rid, tipo);
            nuevo.pista = Some(format!("requisito de {id}"));
            r.items.push(nuevo);
        }
    }
    for s in sistema {
        if !r.checklist.sistema.contains(&s) { r.checklist.sistema.push(s); }
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
