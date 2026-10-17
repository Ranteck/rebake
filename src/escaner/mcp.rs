use super::{Entorno, Escaneo, Hallazgo, comandos};
use crate::modelo::Tipo;
use crate::rutas::contraer;
use crate::secretos;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Prefijo de la pista de un servidor MCP: lo que sigue es el comando que se registra.
pub const PISTA_MCP: &str = "servidor MCP: ";

// `.claude.json` guarda también la cuenta y el historial: serde descarta todo lo que no
// está acá al leerlo, así nada más llega a memoria. `env` y `headers` tampoco se leen.
#[derive(Deserialize)]
struct ClaudeJson {
    #[serde(default, rename = "mcpServers")]
    servidores: BTreeMap<String, Servidor>,
}

#[derive(Deserialize)]
struct Servidor {
    command: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    url: Option<String>,
}

/// Claude Code lee `~/.claude.json` para el perfil por defecto y `<dir>/.claude.json` para el resto.
pub fn ruta_config(dir: &Path, home: &Path) -> PathBuf {
    if dir == home.join(".claude") {
        home.join(".claude.json")
    } else {
        dir.join(".claude.json")
    }
}

pub fn detectar(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    let ruta = ruta_config(dir, &entorno.home);
    let archivo = match std::fs::File::open(&ruta) {
        Ok(a) => a,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
        Err(e) => {
            salida.avisos.push(format!(
                "no pude leer {}: {e}",
                contraer(&ruta, &entorno.home)
            ));
            return;
        }
    };
    let config: ClaudeJson = match serde_json::from_reader(std::io::BufReader::new(archivo)) {
        Ok(c) => c,
        Err(e) => {
            salida.avisos.push(format!(
                "no pude leer {}: {e}",
                contraer(&ruta, &entorno.home)
            ));
            return;
        }
    };
    let home = entorno.home.to_string_lossy();
    for (nombre, s) in config.servidores {
        let mut h = Hallazgo::nuevo(&format!("mcp:{nombre}"), Tipo::Mcp, Some(perfil));
        let linea = match (&s.command, &s.url) {
            (Some(c), _) => std::iter::once(c.as_str())
                .chain(s.args.iter().map(String::as_str))
                .collect::<Vec<_>>()
                .join(" "),
            (None, Some(u)) => secretos::sin_credenciales_url(u),
            (None, None) => String::new(),
        };
        let visible = secretos::ocultar(&linea).replace(&*home, "~");
        h.pista = Some(format!("{PISTA_MCP}{visible}"));
        // El repo del binario, si lo dice; el ítem `herramienta:` no se agrega: lo instala la receta.
        if let Some(tool) = s
            .command
            .and_then(|c| comandos::herramienta(entorno, &c, "servidor MCP", salida))
        {
            h.fuente = tool.fuente;
        }
        salida.hallazgos.push(h);
    }
}
