# Plan: recetario (desde spec.md 2026-10-07 e intent.md 2026-10-07)
Estado: aceptado

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** CLI pública en Rust, con TUI, que escanea el setup de Claude Code, investiga con
`claude -p` la instalación documentada de cada cosa, guarda las recetas en un `recetario.toml`
externo y las ejecuta en una PC nueva.

**Architecture:** Una librería (`src/lib.rs`) con un módulo por responsabilidad del spec
(perfiles, escaner, investigador, recetario, instalador, checklist) y dos frentes: subcomandos
(`clap`) y TUI (`ratatui`). La TUI separa el estado puro (`tui/app.rs`, testeable sin
terminal) del bucle de eventos con efectos (`tui/mod.rs`). Todo lo que toca el sistema
(pacman, `claude`, comandos de instalación) pasa por puntos inyectables para testear con
fakes.

**Tech Stack:** Rust 2024 (cargo 1.99), clap 4, ratatui 0.30 (con su crossterm reexportado en `ratatui::crossterm`), serde, toml 1,
serde_json, anyhow, tracing + tracing-subscriber + tracing-appender; tempfile (tests).

**Spec:** `intent/spec.md` (aceptado). Intent: `intent/intent.md` (aceptado).

## Global Constraints
- Ruta por defecto del recetario: `~/.config/recetario/recetario.toml`; `--archivo <ruta>` la reemplaza.
- Logs: `~/.local/state/recetario/recetario.log`; salida de la última instalación: `~/.local/state/recetario/ultima-instalacion.log`.
- Investigación: `claude -p … --restricted --strict-mcp-config --tools WebSearch,WebFetch --no-session-persistence --output-format json --json-schema <schema>`; binario `claude` reemplazable con la variable `RECETARIO_CLAUDE`; tope 5 minutos; hasta 4 en paralelo.
- Instalación: cada paso con `sh -c`, stdin cerrado, tope 10 minutos, en su propio grupo de procesos; pasos por perfil con `CLAUDE_CONFIG_DIR`.
- Estados: `pendiente | por_revisar | aprobada | excluida`. Tipos: `claude | marketplace | plugin | skill | import | hook | statusline | herramienta`. Vías: `manual | metadatos | symlink | busqueda`. Modos: `auto | manual`.
- Valor oculto en la checklist: `‹oculto›`. Claves sensibles: contienen `token`, `secret`, `key` o `password` (sin importar mayúsculas), o están bajo `env`.
- Nunca leer `.credentials.json`, `~/.codex/auth.json` ni `.claude.json`.
- El repo no contiene datos personales: `.gitignore` excluye `recetario.toml`; los fixtures de test son ficticios.
- Mensajes al usuario en español; se traducen en un solo lugar (barra de estado de la TUI o `main`). Errores internos con `anyhow` y contexto.
- Si el API de un crate difiere en la versión que resuelva cargo, ajustar a ese API sin cambiar el comportamiento ni los tests.

## Review Focus
1. **`recetario.toml` es un symlink** (así lo va a usar Denis, enlazado a un repo privado): guardar tiene que escribir en el destino del enlace y conservar el symlink. Test en Task 1.
2. **Comandos de hooks y statusline con comillas y `$HOME`** (`bash "$HOME/.claude/statusline.sh"`): el script tiene que resolverse igual. Test en Task 6.
3. **Un paso que deja un hijo en segundo plano con la salida abierta** (`sh -c 'sleep 60 & echo listo'`): el tope tiene que matar al grupo entero y devolver, sin colgar. Test en Task 2.
4. **Archivos de Claude con forma inesperada** (`settings.json` inválido, `enabledPlugins` ausente): el detector avisa y el escaneo sigue con lo demás. Test en Task 4.
5. **La investigación devuelve como requisito algo que Denis excluyó**: no tiene que revivirlo. Test en Task 10.

## Archivos que cambian
Todos nuevos; el repo hoy solo tiene `intent/`.

| Archivo | Responsabilidad |
|---|---|
| `Cargo.toml`, `.gitignore`, `README.md` | Crate, exclusiones (`target/`, `recetario.toml`), uso público |
| `src/lib.rs` | Declaración de módulos |
| `src/modelo.rs` | Tipos del recetario y validación |
| `src/rutas.rs` | `~`, rutas por defecto, fecha de hoy |
| `src/archivo.rs` | Leer con error de posición, guardar atómico a través de symlinks |
| `src/proceso.rs` | Ejecutar comandos con stdin cerrado, tope y salida en vivo |
| `src/perfiles.rs` | Detección de perfiles |
| `src/escaner/mod.rs` | `Entorno`, `Hallazgo`, `Escaneo`, orquestación |
| `src/escaner/plugins.rs` | Marketplaces y plugins |
| `src/escaner/archivos.rs` | Skills e imports (symlink → repo, lock de skills.sh) |
| `src/escaner/comandos.rs` | Hooks, statusline, herramientas, `claude` |
| `src/escaner/git.rs` | Remoto de un repo y URL de GitHub dentro de un binario |
| `src/checklist.rs` | Ajustes, títulos de `CLAUDE.md`, logins, pasos manuales |
| `src/fusion.rs` | Perfiles nuevos y fusión del escaneo en el recetario |
| `src/acciones.rs` | Aprobar, excluir, pegar link, editar, renombrar perfil |
| `src/investigador.rs` | Prompt, argumentos, ejecución en paralelo, aplicar resultado |
| `src/instalador.rs` | Orden por grafo, verificación por perfil, ejecución |
| `src/servicio.rs` | Orquestación compartida por CLI y TUI: escanear, elegir trabajos, texto de la checklist |
| `src/main.rs` | Subcomandos, logging, confirmación, mensajes |
| `src/tui/mod.rs` | Terminal, bucle de eventos, hilos, editor |
| `src/tui/app.rs` | Estado y teclas (sin terminal) |
| `src/tui/vista.rs` | Dibujo de las tres pestañas |
| `tests/comun/mod.rs` | Constructor de HOME ficticio para tests |
| `tests/*.rs` | Tests de integración por módulo |

## Tests a escribir
| Test (comportamiento observable) | REQ que cubre |
| --- | --- |
| `archivo::ida_y_vuelta_sin_perdidas` | REQ-8 |
| `archivo::campo_desconocido_da_linea_y_columna` | REQ-8 |
| `archivo::guardar_a_traves_de_symlink_conserva_el_enlace` | REQ-8 |
| `archivo::ruta_por_defecto_en_config` | REQ-8 |
| `archivo::inexistente_da_recetario_vacio` | REQ-8 |
| `archivo::ids_repetidos_son_invalidos` | REQ-8 |
| `proceso::stdin_cerrado_no_cuelga` | REQ-10 |
| `proceso::tope_mata_al_grupo` | REQ-10 |
| `proceso::hijo_en_segundo_plano_no_cuelga` | REQ-10 |
| `proceso::pasa_variables_de_entorno` | REQ-10 |
| `perfiles::detecta_claude_y_claude_guion` | REQ-1 |
| `fusion::perfiles_nuevos_se_guardan_con_tilde` | REQ-1 |
| `acciones::renombrar_perfil_actualiza_items` | REQ-1 |
| `escaner::plugin_en_dos_perfiles_es_un_item` | REQ-2 |
| `escaner::plugin_toma_repo_del_catalogo` | REQ-4 |
| `escaner::settings_invalido_avisa_y_sigue` | REQ-2 |
| `escaner::skill_symlink_a_repo_usa_remoto` | REQ-4 |
| `escaner::repo_sin_remoto_queda_con_pista` | REQ-4 |
| `escaner::skill_del_lock_usa_su_fuente` | REQ-4 |
| `escaner::synced_no_aparece` | REQ-2 |
| `escaner::import_symlink_resuelve_repo` | REQ-2, REQ-4 |
| `escaner::binario_de_pacman_no_aparece` | REQ-2 |
| `escaner::palabras_respeta_comillas_y_home` | REQ-2 |
| `escaner::statusline_con_comillas_y_home` | REQ-2 |
| `escaner::claude_siempre_esta` | REQ-2 |
| `escaner::hook_crea_herramienta_con_url_del_binario` | REQ-2, REQ-4 |
| `checklist::no_filtra_secretos` | REQ-3 |
| `checklist::ajustes_sin_claves_de_instaladores` | REQ-11 |
| `checklist::env_y_claves_sensibles_ocultas` | REQ-11 |
| `checklist::titulos_sin_imports` | REQ-11 |
| `checklist::logins_y_pasos_manuales` | REQ-11 |
| `fusion::aprobado_no_cambia_al_reescanear` | REQ-9 |
| `fusion::desinstalado_queda_ausente` | REQ-9 |
| `fusion::excluido_sigue_excluido` | REQ-7, REQ-9 |
| `fusion::nuevo_entra_pendiente` | REQ-9 |
| `acciones::aprobar_pendiente_se_rechaza` | REQ-7 |
| `acciones::excluir_exige_motivo` | REQ-7 |
| `acciones::pegar_link_deja_fuente_manual` | REQ-4, REQ-7 |
| `acciones::edicion_invalida_no_cambia` | REQ-7 |
| `acciones::edicion_valida_queda_por_revisar` | REQ-7 |
| `investigador::argumentos_sin_bash` | REQ-5 |
| `investigador::respuesta_valida_queda_por_revisar` | REQ-5 |
| `investigador::json_invalido_queda_pendiente_con_error` | REQ-5 |
| `investigador::tope_queda_pendiente` | REQ-5 |
| `investigador::requisito_nuevo_y_de_sistema` | REQ-6 |
| `investigador::requisito_excluido_no_revive` | REQ-6, REQ-7 |
| `investigador::en_paralelo_devuelve_todos` | REQ-5 |
| `instalador::solo_aprobadas_en_orden` | REQ-10 |
| `instalador::ciclo_no_ejecuta_nada` | REQ-10 |
| `instalador::verificar_por_perfil` | REQ-10 |
| `instalador::falla_bloquea_dependientes` | REQ-10 |
| `instalador::dry_run_no_ejecuta` | REQ-10 |
| `cli::escanear_crea_recetario` | REQ-8, REQ-12 |
| `cli::de_punta_a_punta_con_claude_falso` | REQ-5, REQ-10, REQ-12 |
| `tui::recetas_muestra_grupos_estados_y_detalle` | REQ-12 |
| `tui::excluir_desde_el_teclado` | REQ-7, REQ-12 |
| `tui::aprobar_pendiente_muestra_error_y_no_guarda` | REQ-7, REQ-12 |
| `tui::investigar_todo_pide_solo_pendientes` | REQ-5, REQ-12 |
| `tui_arranque::no_arranca_con_recetario_invalido` | REQ-8, REQ-12 |
| `repo::no_versiona_datos_personales` | REQ-13 |

## Orden de trabajo
1. Task 1 — Esqueleto, modelo y archivo (REQ-8, REQ-13).
2. Task 2 — Ejecución de procesos (REQ-10, base de REQ-5).
3. Task 3 — Perfiles (REQ-1).
4. Task 4 — Escáner: marketplaces y plugins (REQ-2, REQ-4).
5. Task 5 — Escáner: skills e imports (REQ-2, REQ-4).
6. Task 6 — Escáner: hooks, statusline, herramientas y `claude` (REQ-2, REQ-4).
7. Task 7 — Checklist y escaneo sin secretos (REQ-3, REQ-11).
8. Task 8 — Fusión (REQ-1, REQ-9).
9. Task 9 — Acciones de revisión (REQ-1, REQ-7).
10. Task 10 — Investigador (REQ-5, REQ-6).
11. Task 11 — Instalador (REQ-10).
12. Task 12 — CLI y orquestación compartida (REQ-8, REQ-10, REQ-11, REQ-12).
13. Task 13 — TUI: estado y vista (REQ-7, REQ-12).
14. Task 14 — TUI: bucle, hilos y editor (REQ-12).
15. Task 15 — README y evidencia sobre la máquina real (REQ-13 y prueba de punta a punta).

## Riesgos
- **`rustc` roto en esta máquina** (desincronización `rust` ↔ `llvm-libs`): nada compila hasta
  `sudo pacman -Syu`. Es el paso 0.
- **APIs de crates** (ratatui 0.30, toml 1): el código del plan puede necesitar ajustes de
  firma; los tests definen el comportamiento.
- **Salida de `claude -p`**: si cambia el formato de `--output-format json`, falla el parseo;
  queda aislado en `investigador::parsear_salida` y lo cubren sus tests.
- **Instaladores reales** que ignoren stdin cerrado o pidan confirmación: fallan por tope, no
  cuelgan; se corrigen editando la receta.

## Prueba
- `cargo test`: todos los tests de la tabla; cada REQ MUST tiene al menos uno.
- `cargo clippy --all-targets -- -D warnings` y `cargo fmt --check` limpios.
- Evidencia en la máquina real (Task 15), con salida guardada en `intent/evidencia.md`:
  1. `recetario --archivo /tmp/…/recetario.toml escanear` → recetario con los ítems reales,
     los dos perfiles y la checklist (REQ-1, 2, 4, 11).
  2. `recetario investigar` sobre 3 ítems reales (house-rules, ai-native-sdlc, codex) →
     `por_revisar` con citas (REQ-5).
  3. `recetario instalar --dry-run` → comandos en orden, nada ejecutado (REQ-10).
  4. `HOME=$(mktemp -d) recetario --archivo … instalar --si` con esos 3 ítems aprobados →
     instalación real en un HOME vacío y una segunda corrida que saltea todo (REQ-10).

---

### Task 0: Toolchain

- [x] **Step 1: Reparar `rustc`** (lo corre Denis, necesita sudo)

Run: `sudo pacman -Syu`
Expected: `llvm-libs` pasa a `23.1.1-3` (o posterior) y `rustc --version` imprime `rustc 1.99.0 …` sin `symbol lookup error`.

### Task 1: Esqueleto, modelo y archivo

**Files:**
- Create: `Cargo.toml`, `.gitignore`, `src/lib.rs`, `src/modelo.rs`, `src/rutas.rs`, `src/archivo.rs`
- Test: `tests/archivo.rs`, `tests/repo.rs`

**Interfaces:**
- Consumes: nada.
- Produces:
  - `modelo::{Recetario, Perfil, Item, Tipo, Estado, Fuente, Via, Paso, Modo, Checklist, Ajuste, PaqueteSistema, TituloClaudeMd, OCULTO}`; `Recetario::nuevo() -> Recetario`; `Recetario::item(&self, id) -> Option<&Item>`; `Recetario::item_mut(&mut self, id) -> Option<&mut Item>`; `Item::nuevo(id: &str, tipo: Tipo) -> Item`; `modelo::validar(&Recetario) -> anyhow::Result<()>`.
  - `rutas::{expandir(ruta: &str, home: &Path) -> PathBuf, contraer(ruta: &Path, home: &Path) -> String, archivo_por_defecto(home: &Path) -> PathBuf, dir_estado(home: &Path) -> PathBuf, hoy() -> String, home() -> anyhow::Result<PathBuf>}`.
  - `archivo::{leer(ruta: &Path) -> anyhow::Result<Recetario>, guardar(ruta: &Path, r: &Recetario) -> anyhow::Result<()>, parsear(texto: &str, nombre: &str) -> anyhow::Result<Recetario>}`. `leer` de una ruta inexistente devuelve `Recetario::nuevo()`.

- [ ] **Step 1: Crear el crate**

`Cargo.toml`:
```toml
[package]
name = "recetario"
version = "0.1.0"
edition = "2024"
description = "Recetario de instaladores para un setup de Claude Code"
license = "MIT"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
ratatui = "0.30"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "1"
tracing = "0.1"
tracing-appender = "0.2"
tracing-subscriber = "0.3"

[dev-dependencies]
tempfile = "3"
```

`.gitignore`:
```
/target
recetario.toml
```

`src/lib.rs`:
```rust
pub mod archivo;
pub mod modelo;
pub mod rutas;
```

- [ ] **Step 2: Escribir los tests que fallan**

`tests/archivo.rs`:
```rust
use recetario::archivo;
use recetario::modelo::*;
use std::fs;

fn ejemplo() -> Recetario {
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil { nombre: "laburo".into(), dir: "~/.claude".into() });
    let mut item = Item::nuevo("plugin:codex@openai-codex", Tipo::Plugin);
    item.perfiles = vec!["laburo".into()];
    item.estado = Estado::Aprobada;
    item.fuente = Some(Fuente {
        repo: "https://github.com/openai/codex-plugin-cc".into(),
        via: Via::Metadatos,
        doc: Some("https://github.com/openai/codex-plugin-cc#install".into()),
    });
    item.requiere = vec!["marketplace:openai-codex".into()];
    item.pasos.push(Paso {
        cmd: "claude plugin install codex@openai-codex".into(),
        modo: Modo::Auto,
        por_perfil: true,
        cita: Some("/plugin install codex@openai-codex".into()),
        nota: None,
    });
    r.items.push(item);
    r.checklist.ajustes.push(Ajuste { perfil: "laburo".into(), clave: "theme".into(), valor: "dark".into() });
    r
}

#[test]
fn ida_y_vuelta_sin_perdidas() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = dir.path().join("recetario.toml");
    let r = ejemplo();
    archivo::guardar(&ruta, &r).unwrap();
    assert_eq!(archivo::leer(&ruta).unwrap(), r);
}

#[test]
fn campo_desconocido_da_linea_y_columna() {
    let texto = "version = 1\n\n[[item]]\nid = \"claude\"\ntipo = \"claude\"\nestado = \"pendiente\"\ncolor = \"rojo\"\n";
    let err = archivo::parsear(texto, "recetario.toml").unwrap_err().to_string();
    assert!(err.starts_with("recetario.toml:"), "{err}");
    assert!(err.contains("color"), "{err}");
}

#[test]
fn guardar_a_traves_de_symlink_conserva_el_enlace() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("repo-privado").join("recetario.toml");
    fs::create_dir_all(real.parent().unwrap()).unwrap();
    fs::write(&real, "version = 1\n").unwrap();
    let enlace = dir.path().join("recetario.toml");
    std::os::unix::fs::symlink(&real, &enlace).unwrap();

    archivo::guardar(&enlace, &ejemplo()).unwrap();

    assert!(fs::symlink_metadata(&enlace).unwrap().file_type().is_symlink());
    assert!(fs::read_to_string(&real).unwrap().contains("codex@openai-codex"));
}

#[test]
fn ruta_por_defecto_en_config() {
    let home = std::path::Path::new("/home/ejemplo");
    assert_eq!(
        recetario::rutas::archivo_por_defecto(home),
        home.join(".config/recetario/recetario.toml")
    );
}

#[test]
fn inexistente_da_recetario_vacio() {
    let dir = tempfile::tempdir().unwrap();
    let r = archivo::leer(&dir.path().join("no-existe.toml")).unwrap();
    assert_eq!(r, Recetario::nuevo());
}

#[test]
fn ids_repetidos_son_invalidos() {
    let mut r = ejemplo();
    r.items.push(r.items[0].clone());
    assert!(validar(&r).is_err());
}
```

`tests/repo.rs`:
```rust
use std::path::Path;

#[test]
fn no_versiona_datos_personales() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ignorados = std::fs::read_to_string(raiz.join(".gitignore")).unwrap();
    assert!(ignorados.lines().any(|l| l.trim() == "recetario.toml"));
    for dir in ["src", "tests"] {
        for entrada in walk(&raiz.join(dir)) {
            let texto = std::fs::read_to_string(&entrada).unwrap();
            // Partidos para que este mismo archivo no los contenga literalmente.
            for prohibido in ["/home/".to_string() + "denis", "flock".to_string() + "it"] {
                assert!(!texto.contains(&prohibido), "{} contiene {prohibido}", entrada.display());
            }
        }
    }
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() { out.extend(walk(&p)) } else { out.push(p) }
    }
    out
}
```

- [ ] **Step 3: Correr los tests y verificar que fallan**

Run: `cargo test --test archivo --test repo`
Expected: no compila (`unresolved import recetario::modelo` / `archivo`).

- [ ] **Step 4: Implementar `src/modelo.rs`**
```rust
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const OCULTO: &str = "‹oculto›";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recetario {
    pub version: u32,
    #[serde(default, rename = "perfil", skip_serializing_if = "Vec::is_empty")]
    pub perfiles: Vec<Perfil>,
    #[serde(default, rename = "item", skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<Item>,
    #[serde(default, skip_serializing_if = "Checklist::vacia")]
    pub checklist: Checklist,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Perfil {
    pub nombre: String,
    pub dir: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tipo { Claude, Marketplace, Plugin, Skill, Import, Hook, Statusline, Herramienta }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Estado { Pendiente, PorRevisar, Aprobada, Excluida }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Via { Manual, Metadatos, Symlink, Busqueda }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modo { Auto, Manual }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fuente {
    pub repo: String,
    pub via: Via,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Paso {
    pub cmd: String,
    pub modo: Modo,
    #[serde(default)]
    pub por_perfil: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cita: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nota: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub tipo: Tipo,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub perfiles: Vec<String>,
    pub estado: Estado,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fuente: Option<Fuente>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pista: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requiere: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verificar: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub investigado: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motivo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "es_falso")]
    pub ausente: bool,
    #[serde(default, rename = "paso", skip_serializing_if = "Vec::is_empty")]
    pub pasos: Vec<Paso>,
}

fn es_falso(b: &bool) -> bool { !*b }

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checklist {
    #[serde(default, rename = "ajuste", skip_serializing_if = "Vec::is_empty")]
    pub ajustes: Vec<Ajuste>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sistema: Vec<PaqueteSistema>,
    #[serde(default, rename = "titulo", skip_serializing_if = "Vec::is_empty")]
    pub titulos: Vec<TituloClaudeMd>,
}

impl Checklist {
    pub fn vacia(&self) -> bool {
        self.ajustes.is_empty() && self.sistema.is_empty() && self.titulos.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ajuste { pub perfil: String, pub clave: String, pub valor: String }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaqueteSistema { pub paquete: String, pub para: String }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TituloClaudeMd { pub perfil: String, pub texto: String }

impl Recetario {
    pub fn nuevo() -> Self {
        Recetario { version: 1, perfiles: vec![], items: vec![], checklist: Checklist::default() }
    }
    pub fn item(&self, id: &str) -> Option<&Item> { self.items.iter().find(|i| i.id == id) }
    pub fn item_mut(&mut self, id: &str) -> Option<&mut Item> { self.items.iter_mut().find(|i| i.id == id) }
}

impl Item {
    pub fn nuevo(id: &str, tipo: Tipo) -> Self {
        Item {
            id: id.into(), tipo, perfiles: vec![], estado: Estado::Pendiente, fuente: None,
            pista: None, requiere: vec![], verificar: None, investigado: None, motivo: None,
            error: None, ausente: false, pasos: vec![],
        }
    }
}

/// Reglas que serde no puede expresar; corren al leer y antes de guardar.
pub fn validar(r: &Recetario) -> Result<()> {
    if r.version != 1 {
        bail!("versión de recetario no soportada: {}", r.version);
    }
    let perfiles: HashSet<&str> = r.perfiles.iter().map(|p| p.nombre.as_str()).collect();
    let mut ids = HashSet::new();
    for item in &r.items {
        if !ids.insert(item.id.as_str()) {
            bail!("el ítem {} está repetido", item.id);
        }
        if let Some(p) = item.perfiles.iter().find(|p| !perfiles.contains(p.as_str())) {
            bail!("el ítem {} usa el perfil {p}, que no existe", item.id);
        }
        if item.motivo.is_some() && item.estado != Estado::Excluida {
            bail!("el ítem {} tiene motivo pero no está excluido", item.id);
        }
    }
    Ok(())
}
```

- [ ] **Step 5: Implementar `src/rutas.rs`**
```rust
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn home() -> Result<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from).context("la variable HOME no está definida")
}

pub fn expandir(ruta: &str, home: &Path) -> PathBuf {
    match ruta.strip_prefix('~') {
        Some("") => home.to_path_buf(),
        Some(resto) if resto.starts_with('/') => home.join(&resto[1..]),
        _ => PathBuf::from(ruta),
    }
}

pub fn contraer(ruta: &Path, home: &Path) -> String {
    match ruta.strip_prefix(home) {
        Ok(resto) if resto.as_os_str().is_empty() => "~".into(),
        Ok(resto) => format!("~/{}", resto.display()),
        Err(_) => ruta.display().to_string(),
    }
}

pub fn archivo_por_defecto(home: &Path) -> PathBuf {
    home.join(".config/recetario/recetario.toml")
}

pub fn dir_estado(home: &Path) -> PathBuf {
    home.join(".local/state/recetario")
}

/// Fecha local no hace falta: `investigado` es informativo, y UTC evita depender de la zona.
pub fn hoy() -> String {
    let segundos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (a, m, d) = civil_desde_dias((segundos / 86_400) as i64);
    format!("{a:04}-{m:02}-{d:02}")
}

// Algoritmo days_from_civil inverso de Howard Hinnant.
fn civil_desde_dias(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let a = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (a, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tilde_ida_y_vuelta() {
        let home = Path::new("/home/x");
        assert_eq!(expandir("~/.claude", home), home.join(".claude"));
        assert_eq!(contraer(&home.join(".claude"), home), "~/.claude");
    }
    #[test]
    fn fecha_conocida() {
        assert_eq!(civil_desde_dias(20_733), (2026, 10, 7));
    }
}
```

- [ ] **Step 6: Implementar `src/archivo.rs`**
```rust
use crate::modelo::{validar, Recetario};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn leer(ruta: &Path) -> Result<Recetario> {
    if !ruta.exists() {
        return Ok(Recetario::nuevo());
    }
    let texto = fs::read_to_string(ruta).with_context(|| format!("no pude leer {}", ruta.display()))?;
    parsear(&texto, &ruta.display().to_string())
}

pub fn parsear(texto: &str, nombre: &str) -> Result<Recetario> {
    let r: Recetario = toml::from_str(texto).map_err(|e| {
        let (linea, columna) = e.span().map(|s| posicion(texto, s.start)).unwrap_or((1, 1));
        anyhow!("{nombre}:{linea}:{columna}: {}", e.message())
    })?;
    validar(&r).with_context(|| format!("{nombre} no es válido"))?;
    Ok(r)
}

fn posicion(texto: &str, offset: usize) -> (usize, usize) {
    let antes = &texto[..offset.min(texto.len())];
    let linea = antes.matches('\n').count() + 1;
    let columna = antes.rsplit('\n').next().map(|l| l.chars().count()).unwrap_or(0) + 1;
    (linea, columna)
}

/// Escribe en el destino real del symlink para no reemplazar el enlace por un archivo.
pub fn guardar(ruta: &Path, r: &Recetario) -> Result<()> {
    validar(r)?;
    let destino = destino_real(ruta)?;
    let dir = destino.parent().context("la ruta del recetario no tiene carpeta")?;
    fs::create_dir_all(dir).with_context(|| format!("no pude crear {}", dir.display()))?;
    let texto = toml::to_string_pretty(r).context("no pude serializar el recetario")?;
    let temporal = dir.join(format!(".recetario.toml.{}.tmp", std::process::id()));
    let mut f = fs::File::create(&temporal).with_context(|| format!("no pude crear {}", temporal.display()))?;
    f.write_all(texto.as_bytes())?;
    f.sync_all()?;
    fs::rename(&temporal, &destino).with_context(|| format!("no pude reemplazar {}", destino.display()))?;
    Ok(())
}

fn destino_real(ruta: &Path) -> Result<PathBuf> {
    match fs::symlink_metadata(ruta) {
        Ok(m) if m.file_type().is_symlink() => {
            let objetivo = fs::read_link(ruta)?;
            Ok(if objetivo.is_absolute() { objetivo } else { ruta.parent().unwrap_or(Path::new(".")).join(objetivo) })
        }
        _ => Ok(ruta.to_path_buf()),
    }
}
```

- [ ] **Step 7: Correr los tests y verificar que pasan**

Run: `cargo test`
Expected: PASS de `tests/archivo.rs` (6), `tests/repo.rs` (1) y los unitarios de `rutas`.

- [ ] **Step 8: Commit**
```bash
git add Cargo.toml Cargo.lock .gitignore src tests
git commit -m "feat: recipe model and atomic TOML file through symlinks"
```

### Task 2: Ejecución de procesos

**Files:**
- Create: `src/proceso.rs`; Modify: `src/lib.rs` (agregar `pub mod proceso;`)
- Test: `tests/proceso.rs`

**Interfaces:**
- Consumes: nada.
- Produces: `proceso::Salida { codigo: Option<i32>, texto: String, vencido: bool }` con `Salida::exito(&self) -> bool`; `proceso::ejecutar(cmd: &mut Command, tope: Duration, al_leer: &mut dyn FnMut(&str)) -> std::io::Result<Salida>`. `ejecutar` fuerza stdin cerrado, stdout/stderr capturados (mezclados en `texto`, línea por línea) y grupo de procesos propio.

- [ ] **Step 1: Escribir los tests que fallan**

`tests/proceso.rs`:
```rust
use recetario::proceso::ejecutar;
use std::process::Command;
use std::time::{Duration, Instant};

fn sh(script: &str) -> Command {
    let mut c = Command::new("sh");
    c.arg("-c").arg(script);
    c
}

#[test]
fn stdin_cerrado_no_cuelga() {
    let inicio = Instant::now();
    let s = ejecutar(&mut sh("if read x; then echo leyo; else echo eof; fi"), Duration::from_secs(5), &mut |_| {}).unwrap();
    assert!(s.exito());
    assert_eq!(s.texto.trim(), "eof");
    assert!(inicio.elapsed() < Duration::from_secs(4));
}

#[test]
fn tope_mata_al_grupo() {
    let inicio = Instant::now();
    let s = ejecutar(&mut sh("sleep 30 & sleep 30"), Duration::from_millis(300), &mut |_| {}).unwrap();
    assert!(s.vencido);
    assert!(!s.exito());
    assert!(inicio.elapsed() < Duration::from_secs(5));
}

#[test]
fn hijo_en_segundo_plano_no_cuelga() {
    let inicio = Instant::now();
    let s = ejecutar(&mut sh("sleep 30 & echo listo"), Duration::from_secs(10), &mut |_| {}).unwrap();
    assert!(s.exito());
    assert!(s.texto.contains("listo"));
    assert!(inicio.elapsed() < Duration::from_secs(5));
}

#[test]
fn pasa_variables_de_entorno() {
    let mut cmd = sh("echo \"$CLAUDE_CONFIG_DIR\"; echo error >&2; exit 3");
    cmd.env("CLAUDE_CONFIG_DIR", "/tmp/perfil-x");
    let mut lineas = Vec::new();
    let s = ejecutar(&mut cmd, Duration::from_secs(5), &mut |l| lineas.push(l.to_string())).unwrap();
    assert_eq!(s.codigo, Some(3));
    assert!(lineas.contains(&"/tmp/perfil-x".to_string()));
    assert!(lineas.contains(&"error".to_string()));
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run: `cargo test --test proceso`
Expected: no compila (`unresolved import recetario::proceso`).

- [ ] **Step 3: Implementar `src/proceso.rs`**
```rust
use std::io::{BufRead, BufReader, Read};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Salida {
    pub codigo: Option<i32>,
    pub texto: String,
    pub vencido: bool,
}

impl Salida {
    pub fn exito(&self) -> bool {
        !self.vencido && self.codigo == Some(0)
    }
}

pub fn ejecutar(cmd: &mut Command, tope: Duration, al_leer: &mut dyn FnMut(&str)) -> std::io::Result<Salida> {
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).process_group(0);
    let mut hijo = cmd.spawn()?;
    let (tx, rx) = mpsc::channel::<String>();
    let mut fuentes: Vec<Box<dyn Read + Send>> = Vec::new();
    if let Some(s) = hijo.stdout.take() { fuentes.push(Box::new(s)); }
    if let Some(s) = hijo.stderr.take() { fuentes.push(Box::new(s)); }
    let lectores: Vec<_> = fuentes
        .into_iter()
        .map(|fuente| {
            let tx = tx.clone();
            thread::spawn(move || {
                for linea in BufReader::new(fuente).lines() {
                    let Ok(linea) = linea else { break };
                    if tx.send(linea).is_err() { break; }
                }
            })
        })
        .collect();
    drop(tx);

    let mut texto = String::new();
    let mut recibir = |l: String, texto: &mut String| {
        al_leer(&l);
        texto.push_str(&l);
        texto.push('\n');
    };
    let inicio = Instant::now();
    let mut vencido = false;
    let estado = loop {
        if let Some(estado) = hijo.try_wait()? { break estado; }
        if inicio.elapsed() >= tope {
            vencido = true;
            matar_grupo(hijo.id());
            break hijo.wait()?;
        }
        if let Ok(l) = rx.recv_timeout(Duration::from_millis(50)) { recibir(l, &mut texto); }
    };
    // Un nieto en segundo plano puede dejar la salida abierta y bloquear a los lectores.
    matar_grupo(hijo.id());
    for lector in lectores {
        // Un lector solo termina con error si entró en pánico, y no hay nada que recuperar.
        let _ = lector.join();
    }
    while let Ok(l) = rx.try_recv() { recibir(l, &mut texto); }
    Ok(Salida { codigo: estado.code(), texto, vencido })
}

fn matar_grupo(pid: u32) {
    // PID negativo = el grupo creado con process_group(0). Si el grupo ya no existe, `kill`
    // falla con "No such process", que es el caso normal y no hay nada que hacer.
    let _ = Command::new("kill")
        .args(["-KILL", "--", &format!("-{pid}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
```

- [ ] **Step 4: Correr los tests y verificar que pasan**

Run: `cargo test --test proceso`
Expected: PASS (4).

- [ ] **Step 5: Commit**
```bash
git add src/proceso.rs src/lib.rs tests/proceso.rs
git commit -m "feat: run commands with closed stdin, timeout and process-group kill"
```

### Task 3: Perfiles y HOME de prueba

**Files:**
- Create: `src/perfiles.rs`, `tests/comun/mod.rs`; Modify: `src/lib.rs` (`pub mod perfiles;`)
- Test: `tests/perfiles.rs`

**Interfaces:**
- Consumes: nada.
- Produces:
  - `perfiles::PerfilDetectado { nombre_sugerido: String, dir: PathBuf }`; `perfiles::detectar(home: &Path) -> anyhow::Result<Vec<PerfilDetectado>>`, ordenado por nombre de carpeta. `~/.claude` sugiere `claude`; `~/.claude-<x>` sugiere `<x>`.
  - `tests/comun/mod.rs`: `HomeFalso` con `nuevo()`, `ruta() -> &Path`, `escribir(rel, contenido) -> PathBuf`, `enlazar(rel_enlace, destino: &Path)`, `repo_git(rel, remoto: Option<&str>) -> PathBuf`, `binario(rel, contenido: &[u8]) -> PathBuf` (ejecutable).

- [ ] **Step 1: Crear el HOME de prueba**

`tests/comun/mod.rs`:
```rust
#![allow(dead_code)]
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct HomeFalso {
    pub dir: tempfile::TempDir,
}

impl HomeFalso {
    pub fn nuevo() -> Self {
        HomeFalso { dir: tempfile::tempdir().unwrap() }
    }
    pub fn ruta(&self) -> &Path {
        self.dir.path()
    }
    pub fn escribir(&self, rel: &str, contenido: &str) -> PathBuf {
        let p = self.ruta().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, contenido).unwrap();
        p
    }
    pub fn enlazar(&self, rel_enlace: &str, destino: &Path) {
        let p = self.ruta().join(rel_enlace);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(destino, p).unwrap();
    }
    pub fn repo_git(&self, rel: &str, remoto: Option<&str>) -> PathBuf {
        let p = self.ruta().join(rel);
        fs::create_dir_all(&p).unwrap();
        let git = |args: &[&str]| assert!(Command::new("git").arg("-C").arg(&p).args(args).output().unwrap().status.success());
        git(&["init", "-q"]);
        if let Some(url) = remoto {
            git(&["remote", "add", "origin", url]);
        }
        p
    }
    pub fn binario(&self, rel: &str, contenido: &[u8]) -> PathBuf {
        let p = self.ruta().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, contenido).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
        p
    }
}
```

- [ ] **Step 2: Escribir el test que falla**

`tests/perfiles.rs`:
```rust
mod comun;
use comun::HomeFalso;
use recetario::perfiles::detectar;

#[test]
fn detecta_claude_y_claude_guion() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/settings.json", "{}");
    h.escribir(".claude-personal/settings.json", "{}");
    h.escribir(".claude-vacio/otra-cosa.txt", "");
    h.escribir(".claude.json", "{}");
    let perfiles = detectar(h.ruta()).unwrap();
    let nombres: Vec<_> = perfiles.iter().map(|p| p.nombre_sugerido.as_str()).collect();
    assert_eq!(nombres, ["claude", "personal"]);
    assert_eq!(perfiles[1].dir, h.ruta().join(".claude-personal"));
}
```

- [ ] **Step 3: Correr el test y verificar que falla**

Run: `cargo test --test perfiles`
Expected: no compila (`unresolved import recetario::perfiles`).

- [ ] **Step 4: Implementar `src/perfiles.rs`**
```rust
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct PerfilDetectado {
    pub nombre_sugerido: String,
    pub dir: PathBuf,
}

pub fn detectar(home: &Path) -> Result<Vec<PerfilDetectado>> {
    let mut encontrados = Vec::new();
    for entrada in fs::read_dir(home).with_context(|| format!("no pude listar {}", home.display()))? {
        let entrada = entrada?;
        let nombre = entrada.file_name().to_string_lossy().into_owned();
        let sugerido = match nombre.as_str() {
            ".claude" => "claude".to_string(),
            n => match n.strip_prefix(".claude-") {
                Some(resto) if !resto.is_empty() => resto.to_string(),
                _ => continue,
            },
        };
        let dir = entrada.path();
        if dir.is_dir() && dir.join("settings.json").is_file() {
            encontrados.push(PerfilDetectado { nombre_sugerido: sugerido, dir });
        }
    }
    encontrados.sort_by(|a, b| a.dir.cmp(&b.dir));
    Ok(encontrados)
}
```

- [ ] **Step 5: Correr el test y verificar que pasa**

Run: `cargo test --test perfiles`
Expected: PASS (1).

- [ ] **Step 6: Commit**
```bash
git add src/perfiles.rs src/lib.rs tests/comun tests/perfiles.rs
git commit -m "feat: detect Claude Code profiles"
```

### Task 4: Escáner — marketplaces y plugins

**Files:**
- Create: `src/escaner/mod.rs`, `src/escaner/plugins.rs`; Modify: `src/lib.rs` (`pub mod escaner;`), `tests/comun/mod.rs` (agregar `PacmanFalso`, `entorno`, `perfil`)
- Test: `tests/escaner_plugins.rs`

**Interfaces:**
- Consumes: `modelo::{Perfil, Tipo, Fuente, Via, Ajuste, TituloClaudeMd}`, `rutas::{expandir, contraer}`.
- Produces:
  - `escaner::Pacman` (trait: `fn es_del_sistema(&self, binario: &Path) -> bool`), `escaner::PacmanReal`.
  - `escaner::Entorno<'a> { home: PathBuf, path: Vec<PathBuf>, pacman: &'a dyn Pacman }`; `Entorno::real(pacman: &dyn Pacman) -> anyhow::Result<Entorno>` (HOME y PATH del proceso).
  - `escaner::Hallazgo { id: String, tipo: Tipo, perfiles: Vec<String>, fuente: Option<Fuente>, pista: Option<String>, requiere: Vec<String> }`, `Hallazgo::nuevo(id: &str, tipo: Tipo, perfil: Option<&str>) -> Hallazgo`.
  - `escaner::Escaneo { hallazgos: Vec<Hallazgo>, avisos: Vec<String>, ajustes: Vec<Ajuste>, titulos: Vec<TituloClaudeMd> }` (derive `Default`).
  - `escaner::escanear(entorno: &Entorno, perfiles: &[Perfil]) -> Escaneo`: un hallazgo por id, con los perfiles unidos.
  - `escaner::normalizar_url(url: &str) -> String` (sin `/` final ni `.git`).
  - `escaner::leer_json(ruta: &Path, entorno: &Entorno, avisos: &mut Vec<String>) -> Option<serde_json::Value>`: archivo ausente → `None` sin aviso; ilegible o inválido → `None` y un aviso con la ruta contraída.
  - Cada detector: `fn detectar(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo)`.

- [ ] **Step 1: Agregar helpers de escáner a `tests/comun/mod.rs`**

Agregar al final:
```rust
pub struct PacmanFalso(pub Vec<PathBuf>);

impl recetario::escaner::Pacman for PacmanFalso {
    fn es_del_sistema(&self, binario: &Path) -> bool {
        self.0.iter().any(|p| p == binario)
    }
}

pub fn entorno<'a>(h: &HomeFalso, pacman: &'a PacmanFalso) -> recetario::escaner::Entorno<'a> {
    recetario::escaner::Entorno { home: h.ruta().to_path_buf(), path: vec![h.ruta().join("bin")], pacman }
}

pub fn perfil(nombre: &str, dir: &str) -> recetario::modelo::Perfil {
    recetario::modelo::Perfil { nombre: nombre.into(), dir: dir.into() }
}
```

- [ ] **Step 2: Escribir los tests que fallan**

`tests/escaner_plugins.rs`:
```rust
mod comun;
use comun::*;
use recetario::escaner::escanear;
use recetario::modelo::Via;

const CONOCIDOS: &str = r#"{
  "openai-codex": {"source": {"source": "github", "repo": "openai/codex-plugin-cc"}, "installLocation": "x"}
}"#;
const CATALOGO: &str = r#"{"name": "openai-codex", "plugins": [
  {"name": "codex", "source": "./plugins/codex"},
  {"name": "superpowers", "source": {"source": "url", "url": "https://github.com/obra/superpowers.git"}}
]}"#;

fn perfil_con_plugins(h: &HomeFalso, dir: &str, settings: &str) {
    h.escribir(&format!("{dir}/settings.json"), settings);
    h.escribir(&format!("{dir}/plugins/known_marketplaces.json"), CONOCIDOS);
    h.escribir(&format!("{dir}/plugins/marketplaces/openai-codex/.claude-plugin/marketplace.json"), CATALOGO);
}

#[test]
fn plugin_en_dos_perfiles_es_un_item() {
    let h = HomeFalso::nuevo();
    let settings = r#"{"enabledPlugins": {"codex@openai-codex": true, "canva@openai-codex": false}}"#;
    perfil_con_plugins(&h, ".claude", settings);
    perfil_con_plugins(&h, ".claude-personal", settings);
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude"), perfil("personal", "~/.claude-personal")]);
    let codex: Vec<_> = e.hallazgos.iter().filter(|x| x.id == "plugin:codex@openai-codex").collect();
    assert_eq!(codex.len(), 1);
    assert_eq!(codex[0].perfiles, ["laburo", "personal"]);
    assert!(e.hallazgos.iter().all(|x| !x.id.contains("canva")));
}

#[test]
fn plugin_toma_repo_del_catalogo() {
    let h = HomeFalso::nuevo();
    perfil_con_plugins(&h, ".claude", r#"{"enabledPlugins": {"codex@openai-codex": true, "superpowers@openai-codex": true}}"#);
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    let repo = |id: &str| e.hallazgos.iter().find(|x| x.id == id).unwrap().fuente.clone().unwrap();
    assert_eq!(repo("plugin:codex@openai-codex").repo, "https://github.com/openai/codex-plugin-cc");
    assert_eq!(repo("plugin:superpowers@openai-codex").repo, "https://github.com/obra/superpowers");
    assert_eq!(repo("marketplace:openai-codex").via, Via::Metadatos);
    let codex = e.hallazgos.iter().find(|x| x.id == "plugin:codex@openai-codex").unwrap();
    assert_eq!(codex.requiere, ["marketplace:openai-codex"]);
}

#[test]
fn settings_invalido_avisa_y_sigue() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/settings.json", "{esto no es json");
    perfil_con_plugins(&h, ".claude-personal", r#"{"enabledPlugins": {"codex@openai-codex": true}}"#);
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude"), perfil("personal", "~/.claude-personal")]);
    assert!(e.avisos.iter().any(|a| a.contains("~/.claude/settings.json")), "{:?}", e.avisos);
    assert!(e.hallazgos.iter().any(|x| x.id == "plugin:codex@openai-codex"));
}
```

- [ ] **Step 3: Correr los tests y verificar que fallan**

Run: `cargo test --test escaner_plugins`
Expected: no compila (`unresolved import recetario::escaner`).

- [ ] **Step 4: Implementar `src/escaner/mod.rs`**
```rust
pub mod plugins;

use crate::modelo::{Ajuste, Fuente, Perfil, Tipo, TituloClaudeMd};
use crate::rutas;
use anyhow::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub trait Pacman {
    fn es_del_sistema(&self, binario: &Path) -> bool;
}

pub struct PacmanReal;

impl Pacman for PacmanReal {
    fn es_del_sistema(&self, binario: &Path) -> bool {
        // Sin pacman (otra distro) nada cuenta como del sistema: el ítem aparece y Denis decide.
        Command::new("pacman")
            .arg("-Qo")
            .arg(binario)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

pub struct Entorno<'a> {
    pub home: PathBuf,
    pub path: Vec<PathBuf>,
    pub pacman: &'a dyn Pacman,
}

impl<'a> Entorno<'a> {
    pub fn real(pacman: &'a dyn Pacman) -> Result<Self> {
        let path = std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
        Ok(Entorno { home: rutas::home()?, path, pacman })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hallazgo {
    pub id: String,
    pub tipo: Tipo,
    pub perfiles: Vec<String>,
    pub fuente: Option<Fuente>,
    pub pista: Option<String>,
    pub requiere: Vec<String>,
}

impl Hallazgo {
    pub fn nuevo(id: &str, tipo: Tipo, perfil: Option<&str>) -> Self {
        Hallazgo {
            id: id.into(),
            tipo,
            perfiles: perfil.into_iter().map(String::from).collect(),
            fuente: None,
            pista: None,
            requiere: vec![],
        }
    }
}

#[derive(Debug, Default)]
pub struct Escaneo {
    pub hallazgos: Vec<Hallazgo>,
    pub avisos: Vec<String>,
    pub ajustes: Vec<Ajuste>,
    pub titulos: Vec<TituloClaudeMd>,
}

pub fn escanear(entorno: &Entorno, perfiles: &[Perfil]) -> Escaneo {
    let mut salida = Escaneo::default();
    for perfil in perfiles {
        let dir = rutas::expandir(&perfil.dir, &entorno.home);
        plugins::detectar(entorno, &perfil.nombre, &dir, &mut salida);
    }
    salida.hallazgos = agrupar(std::mem::take(&mut salida.hallazgos));
    salida
}

fn agrupar(hallazgos: Vec<Hallazgo>) -> Vec<Hallazgo> {
    let mut unidos: Vec<Hallazgo> = Vec::new();
    for h in hallazgos {
        match unidos.iter_mut().find(|u| u.id == h.id) {
            Some(u) => {
                for p in h.perfiles {
                    if !u.perfiles.contains(&p) { u.perfiles.push(p); }
                }
                for r in h.requiere {
                    if !u.requiere.contains(&r) { u.requiere.push(r); }
                }
                if u.fuente.is_none() { u.fuente = h.fuente; }
                if u.pista.is_none() { u.pista = h.pista; }
            }
            None => unidos.push(h),
        }
    }
    unidos
}

pub fn normalizar_url(url: &str) -> String {
    url.trim().trim_end_matches('/').trim_end_matches(".git").to_string()
}

pub fn leer_json(ruta: &Path, entorno: &Entorno, avisos: &mut Vec<String>) -> Option<Value> {
    let texto = match std::fs::read_to_string(ruta) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            avisos.push(format!("no pude leer {}: {e}", rutas::contraer(ruta, &entorno.home)));
            return None;
        }
    };
    match serde_json::from_str(&texto) {
        Ok(v) => Some(v),
        Err(e) => {
            avisos.push(format!("no pude leer {}: {e}", rutas::contraer(ruta, &entorno.home)));
            None
        }
    }
}
```

- [ ] **Step 5: Implementar `src/escaner/plugins.rs`**
```rust
use super::{leer_json, normalizar_url, Entorno, Escaneo, Hallazgo};
use crate::modelo::{Fuente, Tipo, Via};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

pub fn detectar(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    let mut repos: HashMap<String, Option<String>> = HashMap::new();
    if let Some(Value::Object(mapa)) = leer_json(&dir.join("plugins/known_marketplaces.json"), entorno, &mut salida.avisos) {
        for (nombre, datos) in mapa {
            let repo = repo_de_source(datos.get("source"));
            let mut h = Hallazgo::nuevo(&format!("marketplace:{nombre}"), Tipo::Marketplace, Some(perfil));
            h.requiere = vec!["claude".into()];
            asignar(&mut h, repo.clone(), "marketplace sin fuente conocida".into());
            repos.insert(nombre, repo);
            salida.hallazgos.push(h);
        }
    }
    let Some(settings) = leer_json(&dir.join("settings.json"), entorno, &mut salida.avisos) else { return };
    let Some(habilitados) = settings.get("enabledPlugins").and_then(Value::as_object) else { return };
    for (clave, valor) in habilitados {
        if valor != &Value::Bool(true) { continue; }
        let Some((nombre, mkt)) = clave.split_once('@') else {
            salida.avisos.push(format!("plugin con nombre inesperado en {perfil}: {clave}"));
            continue;
        };
        let mut h = Hallazgo::nuevo(&format!("plugin:{clave}"), Tipo::Plugin, Some(perfil));
        h.requiere = vec![format!("marketplace:{mkt}")];
        let repo_mkt = repos.get(mkt).cloned().flatten();
        let repo = repo_del_plugin(entorno, dir, nombre, mkt, repo_mkt, &mut salida.avisos);
        asignar(&mut h, repo, format!("plugin sin entrada en el catálogo de {mkt}"));
        salida.hallazgos.push(h);
    }
}

fn asignar(h: &mut Hallazgo, repo: Option<String>, pista: String) {
    match repo {
        Some(repo) => h.fuente = Some(Fuente { repo, via: Via::Metadatos, doc: None }),
        None => h.pista = Some(pista),
    }
}

fn repo_de_source(source: Option<&Value>) -> Option<String> {
    let s = source?;
    match s.get("source").and_then(Value::as_str) {
        Some("github") => s.get("repo").and_then(Value::as_str).map(|r| format!("https://github.com/{r}")),
        _ => s.get("url").and_then(Value::as_str).map(normalizar_url),
    }
}

fn repo_del_plugin(entorno: &Entorno, dir: &Path, nombre: &str, mkt: &str, repo_mkt: Option<String>, avisos: &mut Vec<String>) -> Option<String> {
    let ruta = dir.join(format!("plugins/marketplaces/{mkt}/.claude-plugin/marketplace.json"));
    let catalogo = leer_json(&ruta, entorno, avisos)?;
    let entrada = catalogo
        .get("plugins")?
        .as_array()?
        .iter()
        .find(|p| p.get("name").and_then(Value::as_str) == Some(nombre))?;
    match entrada.get("source") {
        // Una ruta relativa significa que el plugin vive dentro del repo del marketplace.
        Some(Value::String(rel)) if rel.starts_with("./") => repo_mkt,
        Some(obj @ Value::Object(_)) => repo_de_source(Some(obj)),
        _ => entrada.get("homepage").and_then(Value::as_str).map(normalizar_url),
    }
}
```

- [ ] **Step 6: Correr los tests y verificar que pasan**

Run: `cargo test --test escaner_plugins`
Expected: PASS (3).

- [ ] **Step 7: Commit**
```bash
git add src/escaner src/lib.rs tests/comun/mod.rs tests/escaner_plugins.rs
git commit -m "feat: scan marketplaces and enabled plugins with catalog sources"
```

### Task 5: Escáner — skills e imports

**Files:**
- Create: `src/escaner/git.rs`, `src/escaner/archivos.rs`; Modify: `src/escaner/mod.rs`
- Test: `tests/escaner_archivos.rs`

**Interfaces:**
- Consumes: `escaner::{Entorno, Escaneo, Hallazgo, leer_json, normalizar_url}`, `rutas::contraer`.
- Produces:
  - `escaner::git::{Origen, origen_de_archivo(ruta: &Path) -> Origen, url_en_binario(ruta: &Path) -> Option<String>}` con `enum Origen { Repo(String), RepoSinRemoto(PathBuf), Desconocido }`. Solo sigue symlinks: un archivo común nunca se atribuye a un repo.
  - `escaner::asignar_origen(h: &mut Hallazgo, ruta: &Path, entorno: &Entorno) -> bool` (devuelve `false` si el origen es desconocido y no tocó el hallazgo).
  - Pistas con texto fijo: `"repo local sin remoto: <~ruta>"`, `"archivo sin origen conocido: <~ruta>"`, `"skill sin origen conocido: <~ruta>"`.

- [ ] **Step 1: Escribir los tests que fallan**

`tests/escaner_archivos.rs`:
```rust
mod comun;
use comun::*;
use recetario::escaner::{escanear, Escaneo};
use recetario::modelo::Via;

fn escanear_laburo(h: &HomeFalso) -> Escaneo {
    h.escribir(".claude/settings.json", "{}");
    let pacman = PacmanFalso(vec![]);
    escanear(&entorno(h, &pacman), &[perfil("laburo", "~/.claude")])
}

fn hallazgo<'a>(e: &'a Escaneo, id: &str) -> &'a recetario::escaner::Hallazgo {
    e.hallazgos.iter().find(|x| x.id == id).unwrap_or_else(|| panic!("falta {id}: {:?}", e.hallazgos))
}

#[test]
fn skill_symlink_a_repo_usa_remoto() {
    let h = HomeFalso::nuevo();
    let repo = h.repo_git("Proyectos/graph-engineer", Some("https://github.com/ejemplo/graph-engineer.git"));
    h.escribir("Proyectos/graph-engineer/skills/graph-engineer/SKILL.md", "---\nname: graph-engineer\n---\n");
    h.enlazar(".claude/skills/graph-engineer", &repo.join("skills/graph-engineer"));
    let e = escanear_laburo(&h);
    let f = hallazgo(&e, "skill:graph-engineer").fuente.clone().unwrap();
    assert_eq!(f.repo, "https://github.com/ejemplo/graph-engineer");
    assert_eq!(f.via, Via::Symlink);
}

#[test]
fn repo_sin_remoto_queda_con_pista() {
    let h = HomeFalso::nuevo();
    let repo = h.repo_git("Proyectos/ai-native-sdlc", None);
    h.escribir("Proyectos/ai-native-sdlc/AI-NATIVE-SDLC.md", "# reglas\n");
    h.enlazar(".claude/AI-NATIVE-SDLC.md", &repo.join("AI-NATIVE-SDLC.md"));
    h.escribir(".claude/CLAUDE.md", "@AI-NATIVE-SDLC.md\n");
    let e = escanear_laburo(&h);
    let x = hallazgo(&e, "import:AI-NATIVE-SDLC.md");
    assert!(x.fuente.is_none());
    assert_eq!(x.pista.as_deref(), Some("repo local sin remoto: ~/Proyectos/ai-native-sdlc"));
}

#[test]
fn skill_del_lock_usa_su_fuente() {
    let h = HomeFalso::nuevo();
    let skill = h.escribir(".agents/skills/pragmatic-programmer/SKILL.md", "---\nname: pragmatic-programmer\n---\n");
    h.enlazar(".claude/skills/pragmatic-programmer", skill.parent().unwrap());
    h.escribir(
        ".local/state/skills/.skill-lock.json",
        r#"{"version": 3, "skills": {"pragmatic-programmer": {"source": "ejemplo/skills", "sourceType": "github", "sourceUrl": "https://github.com/ejemplo/skills.git"}}}"#,
    );
    let e = escanear_laburo(&h);
    let f = hallazgo(&e, "skill:pragmatic-programmer").fuente.clone().unwrap();
    assert_eq!(f.repo, "https://github.com/ejemplo/skills");
    assert_eq!(f.via, Via::Metadatos);
}

#[test]
fn synced_no_aparece() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/skills/synced/algo/SKILL.md", "x");
    h.escribir(".claude/skills/propia/SKILL.md", "x");
    let e = escanear_laburo(&h);
    assert!(e.hallazgos.iter().all(|x| !x.id.contains("synced")));
    assert_eq!(hallazgo(&e, "skill:propia").pista.as_deref(), Some("skill sin origen conocido: ~/.claude/skills/propia"));
}

#[test]
fn import_symlink_resuelve_repo() {
    let h = HomeFalso::nuevo();
    let repo = h.repo_git("Proyectos/house-rules", Some("https://github.com/ejemplo/house-rules"));
    h.escribir("Proyectos/house-rules/HOUSE-RULES.md", "# reglas\n");
    h.enlazar(".claude/HOUSE-RULES.md", &repo.join("HOUSE-RULES.md"));
    h.escribir(".claude/RTK.md", "# rtk\n");
    h.escribir(".claude/CLAUDE.md", "@RTK.md\n@HOUSE-RULES.md\n\n## Propio\ntexto\n");
    let e = escanear_laburo(&h);
    assert_eq!(hallazgo(&e, "import:HOUSE-RULES.md").fuente.clone().unwrap().via, Via::Symlink);
    assert_eq!(hallazgo(&e, "import:RTK.md").pista.as_deref(), Some("archivo sin origen conocido: ~/.claude/RTK.md"));
    assert_eq!(hallazgo(&e, "import:RTK.md").requiere, ["claude"]);
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run: `cargo test --test escaner_archivos`
Expected: FAIL (no hay hallazgos `skill:` ni `import:`; panic "falta skill:graph-engineer").

- [ ] **Step 3: Implementar `src/escaner/git.rs`**
```rust
use super::normalizar_url;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, PartialEq)]
pub enum Origen {
    Repo(String),
    RepoSinRemoto(PathBuf),
    Desconocido,
}

pub fn origen_de_archivo(ruta: &Path) -> Origen {
    let es_enlace = std::fs::symlink_metadata(ruta).map(|m| m.file_type().is_symlink()).unwrap_or(false);
    if !es_enlace {
        return Origen::Desconocido;
    }
    let Ok(real) = std::fs::canonicalize(ruta) else { return Origen::Desconocido };
    let dir = if real.is_dir() { real.clone() } else { real.parent().map(Path::to_path_buf).unwrap_or(real) };
    let Some(raiz) = git(&dir, &["rev-parse", "--show-toplevel"]).map(PathBuf::from) else { return Origen::Desconocido };
    match git(&raiz, &["remote", "get-url", "origin"]) {
        Some(url) => Origen::Repo(normalizar_url(&url)),
        None => Origen::RepoSinRemoto(raiz),
    }
}

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let salida = Command::new("git").arg("-C").arg(dir).args(args).output().ok()?;
    let texto = String::from_utf8(salida.stdout).ok()?.trim().to_string();
    (salida.status.success() && !texto.is_empty()).then_some(texto)
}

/// El repo de GitHub que más veces nombra el binario; suele ser el propio (p. ej. rtk-ai/rtk).
pub fn url_en_binario(ruta: &Path) -> Option<String> {
    let bytes = std::fs::read(ruta).ok()?;
    let prefijo = b"https://github.com/";
    let valido = |b: u8| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.';
    let mut conteo: HashMap<String, (usize, usize)> = HashMap::new();
    let mut i = 0;
    while let Some(pos) = bytes[i..].windows(prefijo.len()).position(|w| w == prefijo) {
        let inicio = i + pos + prefijo.len();
        let mut fin = inicio;
        let mut barras = 0;
        while fin < bytes.len() && (valido(bytes[fin]) || (bytes[fin] == b'/' && barras == 0)) {
            if bytes[fin] == b'/' { barras += 1; }
            fin += 1;
        }
        let ruta_repo = String::from_utf8_lossy(&bytes[inicio..fin]).trim_end_matches('.').to_string();
        if barras == 1 && !ruta_repo.ends_with('/') {
            let orden = conteo.len();
            conteo.entry(ruta_repo).or_insert((0, orden)).0 += 1;
        }
        i = fin.max(inicio);
    }
    conteo
        .into_iter()
        .max_by(|a, b| a.1 .0.cmp(&b.1 .0).then(b.1 .1.cmp(&a.1 .1)))
        .map(|(r, _)| normalizar_url(&format!("https://github.com/{r}")))
}
```

- [ ] **Step 4: Implementar `src/escaner/archivos.rs`**
```rust
use super::{asignar_origen, leer_json, normalizar_url, Entorno, Escaneo, Hallazgo};
use crate::modelo::{Fuente, Tipo, Via};
use crate::rutas::contraer;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn detectar(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    skills(entorno, perfil, dir, salida);
    imports(entorno, perfil, dir, salida);
}

fn skills(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    let carpeta = dir.join("skills");
    let entradas = match fs::read_dir(&carpeta) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
        Err(e) => {
            salida.avisos.push(format!("no pude listar {}: {e}", contraer(&carpeta, &entorno.home)));
            return;
        }
    };
    let lock = leer_json(&entorno.home.join(".local/state/skills/.skill-lock.json"), entorno, &mut salida.avisos);
    let mut nombres: Vec<String> = entradas.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    nombres.sort();
    for nombre in nombres {
        // `synced/` lo trae la cuenta de claude.ai; no hay nada que instalar.
        if nombre == "synced" || nombre.starts_with('.') { continue; }
        let ruta = carpeta.join(&nombre);
        let mut h = Hallazgo::nuevo(&format!("skill:{nombre}"), Tipo::Skill, Some(perfil));
        h.requiere = vec!["claude".into()];
        if !asignar_origen(&mut h, &ruta, entorno) {
            match fuente_del_lock(lock.as_ref(), &nombre) {
                Some(repo) => h.fuente = Some(Fuente { repo, via: Via::Metadatos, doc: None }),
                None => h.pista = Some(format!("skill sin origen conocido: {}", contraer(&ruta, &entorno.home))),
            }
        }
        salida.hallazgos.push(h);
    }
}

fn fuente_del_lock(lock: Option<&Value>, nombre: &str) -> Option<String> {
    let entrada = lock?.get("skills")?.get(nombre)?;
    if let Some(url) = entrada.get("sourceUrl").and_then(Value::as_str) {
        return Some(normalizar_url(url));
    }
    match entrada.get("sourceType").and_then(Value::as_str) {
        Some("github") => entrada.get("source").and_then(Value::as_str).map(|s| format!("https://github.com/{s}")),
        _ => None,
    }
}

fn imports(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    let ruta = dir.join("CLAUDE.md");
    let texto = match fs::read_to_string(&ruta) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
        Err(e) => {
            salida.avisos.push(format!("no pude leer {}: {e}", contraer(&ruta, &entorno.home)));
            return;
        }
    };
    for linea in texto.lines().map(str::trim) {
        let Some(nombre) = linea.strip_prefix('@') else { continue };
        if nombre.is_empty() || nombre.contains(char::is_whitespace) { continue; }
        let mut h = Hallazgo::nuevo(&format!("import:{nombre}"), Tipo::Import, Some(perfil));
        h.requiere = vec!["claude".into()];
        let archivo = dir.join(nombre);
        if !asignar_origen(&mut h, &archivo, entorno) {
            h.pista = Some(format!("archivo sin origen conocido: {}", contraer(&archivo, &entorno.home)));
        }
        salida.hallazgos.push(h);
    }
}
```

- [ ] **Step 5: Modificar `src/escaner/mod.rs`**

Agregar módulos y la llamada al detector:
```rust
pub mod archivos;
pub mod git;
pub mod plugins;
```
Dentro del `for perfil in perfiles` de `escanear`, después de `plugins::detectar(...)`:
```rust
        archivos::detectar(entorno, &perfil.nombre, &dir, &mut salida);
```
Agregar la función (con `use crate::modelo::Via;` en los imports):
```rust
pub fn asignar_origen(h: &mut Hallazgo, ruta: &Path, entorno: &Entorno) -> bool {
    match git::origen_de_archivo(ruta) {
        git::Origen::Repo(repo) => {
            h.fuente = Some(Fuente { repo, via: Via::Symlink, doc: None });
            true
        }
        git::Origen::RepoSinRemoto(raiz) => {
            h.pista = Some(format!("repo local sin remoto: {}", rutas::contraer(&raiz, &entorno.home)));
            true
        }
        git::Origen::Desconocido => false,
    }
}
```

- [ ] **Step 6: Correr los tests y verificar que pasan**

Run: `cargo test --test escaner_archivos --test escaner_plugins`
Expected: PASS (5 + 3).

- [ ] **Step 7: Commit**
```bash
git add src/escaner tests/escaner_archivos.rs
git commit -m "feat: scan skills and CLAUDE.md imports through symlinks and skills lock"
```

### Task 6: Escáner — hooks, statusline, herramientas y `claude`

**Files:**
- Create: `src/escaner/comandos.rs`; Modify: `src/escaner/mod.rs`
- Test: `tests/escaner_comandos.rs`

**Interfaces:**
- Consumes: `escaner::{Entorno, Escaneo, Hallazgo, leer_json, asignar_origen, git::url_en_binario}`.
- Produces:
  - `escaner::comandos::palabras(comando: &str, home: &Path) -> Vec<String>`: separa respetando comillas simples y dobles y expande `$HOME`, `${HOME}` y `~` inicial.
  - Hallazgos: `hook:<comando>` (tipo `hook`, por perfil, `requiere` incluye `claude` y, si corresponde, `herramienta:<bin>`; si hay herramienta, copia su `fuente`), `statusline:<archivo>` (tipo `statusline`, por perfil), `herramienta:<bin>` (sin perfiles; fuente por URL dentro del binario o pista `"binario sin origen conocido: <~ruta>"`), `claude` (tipo `claude`, sin perfiles, pista `"Claude Code CLI"`).
  - `escanear` deja los avisos sin repetidos, en el orden en que aparecieron.

- [ ] **Step 1: Escribir los tests que fallan**

`tests/escaner_comandos.rs`:
```rust
mod comun;
use comun::*;
use recetario::escaner::{comandos::palabras, escanear};
use recetario::modelo::Tipo;
use std::path::Path;

#[test]
fn palabras_respeta_comillas_y_home() {
    let home = Path::new("/home/x");
    assert_eq!(palabras("bash \"$HOME/.claude/statusline.sh\" 'a b'", home), ["bash", "/home/x/.claude/statusline.sh", "a b"]);
    assert_eq!(palabras("~/bin/tool ${HOME}/y", home), ["/home/x/bin/tool", "/home/x/y"]);
}

#[test]
fn binario_de_pacman_no_aparece() {
    let h = HomeFalso::nuevo();
    let jq = h.binario("bin/jq", b"#!/bin/sh\n");
    h.escribir(".claude/settings.json", r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "jq --version"}]}]}}"#);
    let pacman = PacmanFalso(vec![jq]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    assert!(e.hallazgos.iter().all(|x| x.id != "herramienta:jq"));
    assert!(e.hallazgos.iter().any(|x| x.id == "hook:jq --version"));
}

#[test]
fn statusline_con_comillas_y_home() {
    let h = HomeFalso::nuevo();
    let bash = h.binario("bin/bash", b"#!/bin/sh\n");
    h.escribir(".claude/statusline.sh", "#!/bin/bash\n");
    h.escribir(".claude/settings.json", r#"{"statusLine": {"type": "command", "command": "bash \"$HOME/.claude/statusline.sh\""}}"#);
    let pacman = PacmanFalso(vec![bash]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    let s = e.hallazgos.iter().find(|x| x.id == "statusline:statusline.sh").expect("falta la statusline");
    assert_eq!(s.tipo, Tipo::Statusline);
    assert_eq!(s.pista.as_deref(), Some("archivo sin origen conocido: ~/.claude/statusline.sh"));
    assert!(e.hallazgos.iter().all(|x| x.id != "herramienta:bash"));
}

#[test]
fn hook_crea_herramienta_con_url_del_binario() {
    let h = HomeFalso::nuevo();
    h.binario("bin/rtk", b"\x7fELF...https://github.com/ejemplo/rtk ... https://github.com/ejemplo/rtk/issues ... https://github.com/clap-rs/clap\0");
    h.escribir(".claude/settings.json", r#"{"hooks": {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "rtk hook claude"}]}]}}"#);
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    let rtk = e.hallazgos.iter().find(|x| x.id == "herramienta:rtk").expect("falta rtk");
    assert_eq!(rtk.fuente.as_ref().unwrap().repo, "https://github.com/ejemplo/rtk");
    assert!(rtk.perfiles.is_empty());
    let hook = e.hallazgos.iter().find(|x| x.id == "hook:rtk hook claude").unwrap();
    assert!(hook.requiere.contains(&"herramienta:rtk".to_string()));
    assert_eq!(hook.perfiles, ["laburo"]);
}

#[test]
fn claude_siempre_esta() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/settings.json", "{}");
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    let c = e.hallazgos.iter().find(|x| x.id == "claude").unwrap();
    assert_eq!(c.tipo, Tipo::Claude);
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run: `cargo test --test escaner_comandos`
Expected: no compila (`unresolved import recetario::escaner::comandos`).

- [ ] **Step 3: Implementar `src/escaner/comandos.rs`**
```rust
use super::{asignar_origen, git, leer_json, Entorno, Escaneo, Hallazgo};
use crate::modelo::{Fuente, Tipo, Via};
use crate::rutas::contraer;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn detectar(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    let Some(settings) = leer_json(&dir.join("settings.json"), entorno, &mut salida.avisos) else { return };
    for comando in comandos_de_hooks(&settings) {
        let mut h = Hallazgo::nuevo(&format!("hook:{comando}"), Tipo::Hook, Some(perfil));
        h.requiere = vec!["claude".into()];
        match palabras(&comando, &entorno.home).first().and_then(|b| herramienta(entorno, b, "hook", salida)) {
            Some(tool) => {
                h.requiere.push(tool.id.clone());
                h.fuente = tool.fuente.clone();
                if h.fuente.is_none() { h.pista = Some(format!("comando de hook: {comando}")); }
                salida.hallazgos.push(tool);
            }
            None => h.pista = Some(format!("comando de hook: {comando}")),
        }
        salida.hallazgos.push(h);
    }
    if let Some(comando) = settings.pointer("/statusLine/command").and_then(Value::as_str) {
        statusline(entorno, perfil, comando, salida);
    }
}

fn comandos_de_hooks(settings: &Value) -> Vec<String> {
    let mut comandos = Vec::new();
    for grupos in settings.get("hooks").and_then(Value::as_object).into_iter().flat_map(|m| m.values()) {
        for grupo in grupos.as_array().into_iter().flatten() {
            for hook in grupo.get("hooks").and_then(Value::as_array).into_iter().flatten() {
                if let Some(c) = hook.get("command").and_then(Value::as_str) {
                    if !comandos.iter().any(|x: &String| x == c) { comandos.push(c.to_string()); }
                }
            }
        }
    }
    comandos
}

fn statusline(entorno: &Entorno, perfil: &str, comando: &str, salida: &mut Escaneo) {
    let palabras = palabras(comando, &entorno.home);
    let tool = palabras.first().and_then(|b| herramienta(entorno, b, "statusline", salida));
    for palabra in &palabras {
        let ruta = PathBuf::from(palabra);
        if !palabra.contains('/') || !ruta.is_file() || Some(palabra) == palabras.first() { continue; }
        let nombre = ruta.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let mut h = Hallazgo::nuevo(&format!("statusline:{nombre}"), Tipo::Statusline, Some(perfil));
        h.requiere = vec!["claude".into()];
        if let Some(t) = &tool { h.requiere.push(t.id.clone()); }
        if !asignar_origen(&mut h, &ruta, entorno) {
            h.pista = Some(format!("archivo sin origen conocido: {}", contraer(&ruta, &entorno.home)));
        }
        salida.hallazgos.push(h);
    }
    if let Some(t) = tool { salida.hallazgos.push(t); }
}

/// `None` si el binario es de pacman (es del sistema) o no está en el PATH (con aviso).
fn herramienta(entorno: &Entorno, palabra: &str, origen: &str, salida: &mut Escaneo) -> Option<Hallazgo> {
    let Some(ruta) = buscar(entorno, palabra) else {
        salida.avisos.push(format!("no encontré en el PATH el binario {palabra} de la {origen}"));
        return None;
    };
    if entorno.pacman.es_del_sistema(&ruta) { return None; }
    let nombre = ruta.file_name()?.to_string_lossy().into_owned();
    let mut h = Hallazgo::nuevo(&format!("herramienta:{nombre}"), Tipo::Herramienta, None);
    let real = std::fs::canonicalize(&ruta).unwrap_or(ruta.clone());
    match git::url_en_binario(&real) {
        Some(repo) => h.fuente = Some(Fuente { repo, via: Via::Metadatos, doc: None }),
        None => h.pista = Some(format!("binario sin origen conocido: {}", contraer(&ruta, &entorno.home))),
    }
    Some(h)
}

fn buscar(entorno: &Entorno, palabra: &str) -> Option<PathBuf> {
    if palabra.contains('/') {
        let p = PathBuf::from(palabra);
        return p.is_file().then_some(p);
    }
    entorno.path.iter().map(|d| d.join(palabra)).find(|p| p.is_file())
}

pub fn palabras(comando: &str, home: &Path) -> Vec<String> {
    let home = home.display().to_string();
    let mut salida = Vec::new();
    let mut actual = String::new();
    let mut hay_palabra = false;
    let mut comilla: Option<char> = None;
    for c in comando.chars() {
        match (comilla, c) {
            (Some(q), c) if c == q => comilla = None,
            (Some(_), c) => actual.push(c),
            (None, '"' | '\'') => { comilla = Some(c); hay_palabra = true; }
            (None, c) if c.is_whitespace() => {
                if hay_palabra { salida.push(std::mem::take(&mut actual)); hay_palabra = false; }
            }
            (None, c) => { actual.push(c); hay_palabra = true; }
        }
    }
    if hay_palabra { salida.push(actual); }
    salida
        .into_iter()
        .map(|p| {
            let p = p.replace("${HOME}", &home).replace("$HOME", &home);
            match p.strip_prefix('~') {
                Some(resto) if resto.is_empty() || resto.starts_with('/') => format!("{home}{resto}"),
                _ => p,
            }
        })
        .collect()
}
```

- [ ] **Step 4: Modificar `src/escaner/mod.rs`**

Agregar `pub mod comandos;`. En `escanear`, antes del `for`:
```rust
    let mut claude = Hallazgo::nuevo("claude", Tipo::Claude, None);
    claude.pista = Some("Claude Code CLI".into());
    salida.hallazgos.push(claude);
```
Dentro del `for`, después de `archivos::detectar(...)`:
```rust
        comandos::detectar(entorno, &perfil.nombre, &dir, &mut salida);
```
Antes de devolver `salida` (cada detector relee `settings.json`, así que un archivo inválido avisaría una vez por detector):
```rust
    let mut vistos = std::collections::HashSet::new();
    salida.avisos.retain(|a| vistos.insert(a.clone()));
```

- [ ] **Step 5: Correr los tests y verificar que pasan**

Run: `cargo test --test escaner_comandos --test escaner_archivos --test escaner_plugins`
Expected: PASS (5 + 5 + 3).

- [ ] **Step 6: Commit**
```bash
git add src/escaner tests/escaner_comandos.rs
git commit -m "feat: scan hooks, statusline, tools and Claude Code itself"
```

### Task 7: Checklist (y sin secretos en el escaneo)

**Files:**
- Create: `src/checklist.rs`; Modify: `src/lib.rs` (`pub mod checklist;`), `src/escaner/mod.rs`
- Test: `tests/checklist.rs`

**Interfaces:**
- Consumes: `modelo::{Ajuste, TituloClaudeMd, Recetario, Paso, Modo, Estado, OCULTO}`, `escaner::{Entorno, Escaneo, leer_json}`.
- Produces:
  - `checklist::ajustes(perfil: &str, settings: &serde_json::Value) -> Vec<Ajuste>`: claves con puntos (`a.b.c`) e índices (`a[0]`); sin `enabledPlugins`, `extraKnownMarketplaces`, `hooks` ni `statusLine`; valores de `env.*` y de claves sensibles → `OCULTO`.
  - `checklist::titulos(perfil: &str, claude_md: &str) -> Vec<TituloClaudeMd>`: líneas de título (`#…`), sin `#` ni espacios.
  - `checklist::logins(r: &Recetario) -> Vec<String>`: una entrada por perfil más `gh auth login`.
  - `checklist::pasos_manuales(r: &Recetario) -> Vec<(String, Paso)>`: pasos `manual` de los ítems `aprobada`.
  - `checklist::detectar(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo)`: llena `salida.ajustes` y `salida.titulos`.

- [ ] **Step 1: Escribir los tests que fallan**

`tests/checklist.rs`:
```rust
mod comun;
use comun::*;
use recetario::checklist;
use recetario::escaner::escanear;
use recetario::modelo::*;
use serde_json::json;

fn valor(a: &[Ajuste], clave: &str) -> Option<String> {
    a.iter().find(|x| x.clave == clave).map(|x| x.valor.clone())
}

#[test]
fn ajustes_sin_claves_de_instaladores() {
    let s = json!({
        "enabledPlugins": {"codex@openai-codex": true},
        "extraKnownMarketplaces": {},
        "hooks": {"PreToolUse": []},
        "statusLine": {"type": "command", "command": "x"},
        "modelSettings": {"claude-opus-5-5": {"effortLevel": "xhigh"}},
        "voice": {"enabled": true, "mode": "hold"},
        "autoMode": {"allow": ["$defaults", "Bash(npx vitest:*)"]}
    });
    let a = checklist::ajustes("laburo", &s);
    assert_eq!(valor(&a, "modelSettings.claude-opus-5-5.effortLevel").as_deref(), Some("xhigh"));
    assert_eq!(valor(&a, "voice.enabled").as_deref(), Some("true"));
    assert_eq!(valor(&a, "autoMode.allow[1]").as_deref(), Some("Bash(npx vitest:*)"));
    for prohibida in ["enabledPlugins", "extraKnownMarketplaces", "hooks", "statusLine"] {
        assert!(a.iter().all(|x| !x.clave.starts_with(prohibida)), "{prohibida}");
    }
    assert!(a.iter().all(|x| x.perfil == "laburo"));
}

#[test]
fn env_y_claves_sensibles_ocultas() {
    let s = json!({"env": {"FOO": "bar"}, "apiKeyHelper": "/bin/x", "github": {"Token": "abc"}, "theme": "dark"});
    let a = checklist::ajustes("laburo", &s);
    assert_eq!(valor(&a, "env.FOO").as_deref(), Some(OCULTO));
    assert_eq!(valor(&a, "apiKeyHelper").as_deref(), Some(OCULTO));
    assert_eq!(valor(&a, "github.Token").as_deref(), Some(OCULTO));
    assert_eq!(valor(&a, "theme").as_deref(), Some("dark"));
}

#[test]
fn titulos_sin_imports() {
    let t = checklist::titulos("laburo", "@RTK.md\n@HOUSE-RULES.md\n\n## Skills de proceso\ntexto\n### Correcciones de inglés\n");
    let textos: Vec<_> = t.iter().map(|x| x.texto.as_str()).collect();
    assert_eq!(textos, ["Skills de proceso", "Correcciones de inglés"]);
}

#[test]
fn logins_y_pasos_manuales() {
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        Perfil { nombre: "laburo".into(), dir: "~/.claude".into() },
        Perfil { nombre: "personal".into(), dir: "~/.claude-personal".into() },
    ];
    let manual = Paso { cmd: "/codex:setup".into(), modo: Modo::Manual, por_perfil: false, cita: None, nota: None };
    let mut codex = Item::nuevo("plugin:codex@openai-codex", Tipo::Plugin);
    codex.estado = Estado::Aprobada;
    codex.pasos = vec![manual.clone()];
    let mut otro = Item::nuevo("plugin:otro@x", Tipo::Plugin);
    otro.pasos = vec![manual.clone()];
    r.items = vec![codex, otro];

    let logins = checklist::logins(&r);
    assert!(logins.iter().any(|l| l.contains("gh auth login")));
    assert!(logins.iter().any(|l| l.contains("CLAUDE_CONFIG_DIR=~/.claude-personal")));
    assert_eq!(logins.len(), 3);
    assert_eq!(checklist::pasos_manuales(&r), vec![("plugin:codex@openai-codex".to_string(), manual)]);
}

#[test]
fn no_filtra_secretos() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/.credentials.json", r#"{"token": "SECRETO-XYZ"}"#);
    h.escribir(".claude.json", r#"{"oauthAccount": "SECRETO-XYZ"}"#);
    h.escribir(".codex/auth.json", r#"{"key": "SECRETO-XYZ"}"#);
    h.escribir(".claude/settings.json", r#"{"env": {"API_TOKEN": "SECRETO-XYZ"}, "github": {"token": "SECRETO-XYZ"}}"#);
    h.escribir(".claude/CLAUDE.md", "## Propio\n");
    let pacman = PacmanFalso(vec![]);
    let e = escanear(&entorno(&h, &pacman), &[perfil("laburo", "~/.claude")]);
    assert!(!format!("{e:?}").contains("SECRETO-XYZ"));
    assert_eq!(valor(&e.ajustes, "env.API_TOKEN").as_deref(), Some(OCULTO));
    assert_eq!(e.titulos.len(), 1);
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run: `cargo test --test checklist`
Expected: no compila (`unresolved import recetario::checklist`).

- [ ] **Step 3: Implementar `src/checklist.rs`**
```rust
use crate::escaner::{leer_json, Entorno, Escaneo};
use crate::modelo::{Ajuste, Estado, Modo, Paso, Recetario, TituloClaudeMd, OCULTO};
use crate::rutas::contraer;
use serde_json::Value;
use std::path::Path;

// Estas claves las reconstruyen los instaladores; repetirlas en la checklist confunde.
const CUBIERTAS: [&str; 4] = ["enabledPlugins", "extraKnownMarketplaces", "hooks", "statusLine"];
const SENSIBLES: [&str; 4] = ["token", "secret", "key", "password"];

pub fn detectar(entorno: &Entorno, perfil: &str, dir: &Path, salida: &mut Escaneo) {
    if let Some(settings) = leer_json(&dir.join("settings.json"), entorno, &mut salida.avisos) {
        salida.ajustes.extend(ajustes(perfil, &settings));
    }
    let ruta = dir.join("CLAUDE.md");
    match std::fs::read_to_string(&ruta) {
        Ok(texto) => salida.titulos.extend(titulos(perfil, &texto)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => salida.avisos.push(format!("no pude leer {}: {e}", contraer(&ruta, &entorno.home))),
    }
}

pub fn ajustes(perfil: &str, settings: &Value) -> Vec<Ajuste> {
    let mut salida = Vec::new();
    if let Some(mapa) = settings.as_object() {
        for (clave, valor) in mapa {
            if CUBIERTAS.contains(&clave.as_str()) { continue; }
            aplanar(perfil, clave, valor, clave == "env", &mut salida);
        }
    }
    salida
}

fn aplanar(perfil: &str, clave: &str, valor: &Value, oculto: bool, salida: &mut Vec<Ajuste>) {
    let oculto = oculto || SENSIBLES.iter().any(|s| clave.to_lowercase().contains(s));
    match valor {
        Value::Object(mapa) => {
            for (k, v) in mapa { aplanar(perfil, &format!("{clave}.{k}"), v, oculto, salida); }
        }
        Value::Array(lista) => {
            for (i, v) in lista.iter().enumerate() { aplanar(perfil, &format!("{clave}[{i}]"), v, oculto, salida); }
        }
        hoja => {
            let texto = match hoja { Value::String(s) => s.clone(), otro => otro.to_string() };
            salida.push(Ajuste {
                perfil: perfil.into(),
                clave: clave.into(),
                valor: if oculto { OCULTO.into() } else { texto },
            });
        }
    }
}

pub fn titulos(perfil: &str, claude_md: &str) -> Vec<TituloClaudeMd> {
    claude_md
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('#'))
        .map(|l| l.trim_start_matches('#').trim())
        .filter(|t| !t.is_empty())
        .map(|t| TituloClaudeMd { perfil: perfil.into(), texto: t.into() })
        .collect()
}

pub fn logins(r: &Recetario) -> Vec<String> {
    let mut salida: Vec<String> = r
        .perfiles
        .iter()
        .map(|p| {
            if p.dir == "~/.claude" {
                format!("Perfil {}: abrí `claude` y corré /login", p.nombre)
            } else {
                format!("Perfil {}: abrí `CLAUDE_CONFIG_DIR={} claude` y corré /login", p.nombre, p.dir)
            }
        })
        .collect();
    salida.push("GitHub: `gh auth login`".into());
    salida
}

pub fn pasos_manuales(r: &Recetario) -> Vec<(String, Paso)> {
    r.items
        .iter()
        .filter(|i| i.estado == Estado::Aprobada)
        .flat_map(|i| i.pasos.iter().filter(|p| p.modo == Modo::Manual).map(|p| (i.id.clone(), p.clone())))
        .collect()
}
```

- [ ] **Step 4: Conectar con el escáner**

En `src/escaner/mod.rs`, dentro del `for` de `escanear`, después de `comandos::detectar(...)`:
```rust
        crate::checklist::detectar(entorno, &perfil.nombre, &dir, &mut salida);
```

- [ ] **Step 5: Correr los tests y verificar que pasan**

Run: `cargo test`
Expected: PASS de todo lo anterior más `tests/checklist.rs` (5).

- [ ] **Step 6: Commit**
```bash
git add src/checklist.rs src/lib.rs src/escaner/mod.rs tests/checklist.rs
git commit -m "feat: checklist of personal settings, CLAUDE.md headings and logins"
```

### Task 8: Fusión

**Files:**
- Create: `src/fusion.rs`; Modify: `src/lib.rs` (`pub mod fusion;`)
- Test: `tests/fusion.rs`

**Interfaces:**
- Consumes: `modelo::*`, `perfiles::PerfilDetectado`, `escaner::{Escaneo, Hallazgo}`, `rutas::contraer`.
- Produces:
  - `fusion::asegurar_perfiles(r: &mut Recetario, detectados: &[PerfilDetectado], home: &Path)`: agrega los perfiles cuyo `dir` (con `~`) no está; si el nombre sugerido ya existe, le agrega `-2`, `-3`, …
  - `fusion::fusionar(r: &mut Recetario, escaneo: Escaneo, existe_herramienta: &dyn Fn(&str) -> bool)`: aplica las reglas de REQ-9. Reemplaza `checklist.ajustes` y `checklist.titulos`; conserva `checklist.sistema`.

- [ ] **Step 1: Escribir los tests que fallan**

`tests/fusion.rs`:
```rust
use recetario::escaner::{Escaneo, Hallazgo};
use recetario::fusion::{asegurar_perfiles, fusionar};
use recetario::modelo::*;
use recetario::perfiles::PerfilDetectado;
use std::path::{Path, PathBuf};

fn base() -> Recetario {
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        Perfil { nombre: "laburo".into(), dir: "~/.claude".into() },
        Perfil { nombre: "personal".into(), dir: "~/.claude-personal".into() },
    ];
    r
}

fn hallazgo(id: &str, tipo: Tipo, perfiles: &[&str]) -> Hallazgo {
    let mut h = Hallazgo::nuevo(id, tipo, None);
    h.perfiles = perfiles.iter().map(|p| p.to_string()).collect();
    h
}

fn escaneo(hallazgos: Vec<Hallazgo>) -> Escaneo {
    Escaneo { hallazgos, ..Default::default() }
}

fn nunca(_: &str) -> bool { false }

#[test]
fn perfiles_nuevos_se_guardan_con_tilde() {
    let home = Path::new("/h");
    let detectados = vec![
        PerfilDetectado { nombre_sugerido: "claude".into(), dir: PathBuf::from("/h/.claude") },
        PerfilDetectado { nombre_sugerido: "personal".into(), dir: PathBuf::from("/h/.claude-personal") },
        PerfilDetectado { nombre_sugerido: "laburo".into(), dir: PathBuf::from("/h/.claude-laburo") },
    ];
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil { nombre: "laburo".into(), dir: "~/.claude".into() });
    asegurar_perfiles(&mut r, &detectados, home);
    asegurar_perfiles(&mut r, &detectados, home);
    let pares: Vec<_> = r.perfiles.iter().map(|p| (p.nombre.as_str(), p.dir.as_str())).collect();
    assert_eq!(pares, [("laburo", "~/.claude"), ("personal", "~/.claude-personal"), ("laburo-2", "~/.claude-laburo")]);
}

#[test]
fn aprobado_no_cambia_al_reescanear() {
    let mut r = base();
    let mut item = Item::nuevo("import:HOUSE-RULES.md", Tipo::Import);
    item.perfiles = vec!["laburo".into()];
    item.estado = Estado::Aprobada;
    item.fuente = Some(Fuente { repo: "https://github.com/ejemplo/house-rules".into(), via: Via::Symlink, doc: None });
    item.pasos = vec![Paso { cmd: "sh install.sh".into(), modo: Modo::Auto, por_perfil: true, cita: None, nota: None }];
    r.items.push(item.clone());

    let mut h = hallazgo("import:HOUSE-RULES.md", Tipo::Import, &["laburo", "personal"]);
    h.fuente = Some(Fuente { repo: "https://github.com/otro/repo".into(), via: Via::Metadatos, doc: None });
    fusionar(&mut r, escaneo(vec![h]), &nunca);

    let x = r.item("import:HOUSE-RULES.md").unwrap();
    assert_eq!(x.estado, Estado::Aprobada);
    assert_eq!(x.pasos, item.pasos);
    assert_eq!(x.fuente, item.fuente);
    assert_eq!(x.perfiles, ["laburo", "personal"]);
}

#[test]
fn desinstalado_queda_ausente() {
    let mut r = base();
    let mut viejo = Item::nuevo("plugin:viejo@x", Tipo::Plugin);
    viejo.estado = Estado::Aprobada;
    r.items.push(viejo);
    r.items.push(Item::nuevo("herramienta:codex", Tipo::Herramienta));
    fusionar(&mut r, escaneo(vec![]), &|nombre| nombre == "codex");
    assert!(r.item("plugin:viejo@x").unwrap().ausente);
    assert!(!r.item("herramienta:codex").unwrap().ausente);
}

#[test]
fn excluido_sigue_excluido() {
    let mut r = base();
    let mut x = Item::nuevo("skill:crawl4ai", Tipo::Skill);
    x.perfiles = vec!["personal".into()];
    x.estado = Estado::Excluida;
    x.motivo = Some("no me interesa".into());
    r.items.push(x);
    fusionar(&mut r, escaneo(vec![hallazgo("skill:crawl4ai", Tipo::Skill, &["personal"])]), &nunca);
    let x = r.item("skill:crawl4ai").unwrap();
    assert_eq!(x.estado, Estado::Excluida);
    assert_eq!(x.motivo.as_deref(), Some("no me interesa"));
}

#[test]
fn nuevo_entra_pendiente() {
    let mut r = base();
    r.checklist.sistema.push(PaqueteSistema { paquete: "jq".into(), para: "import:X.md".into() });
    let mut h = hallazgo("plugin:codex@openai-codex", Tipo::Plugin, &["laburo"]);
    h.requiere = vec!["marketplace:openai-codex".into()];
    h.pista = Some("plugin sin entrada en el catálogo".into());
    let mut e = escaneo(vec![h]);
    e.ajustes.push(Ajuste { perfil: "laburo".into(), clave: "theme".into(), valor: "dark".into() });
    fusionar(&mut r, e, &nunca);
    let x = r.item("plugin:codex@openai-codex").unwrap();
    assert_eq!(x.estado, Estado::Pendiente);
    assert_eq!(x.requiere, ["marketplace:openai-codex"]);
    assert_eq!(x.pista.as_deref(), Some("plugin sin entrada en el catálogo"));
    assert_eq!(r.checklist.ajustes.len(), 1);
    assert_eq!(r.checklist.sistema.len(), 1);
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run: `cargo test --test fusion`
Expected: no compila (`unresolved import recetario::fusion`).

- [ ] **Step 3: Implementar `src/fusion.rs`**
```rust
use crate::escaner::Escaneo;
use crate::modelo::{Estado, Item, Perfil, Recetario, Tipo};
use crate::perfiles::PerfilDetectado;
use crate::rutas::contraer;
use std::collections::HashSet;
use std::path::Path;

pub fn asegurar_perfiles(r: &mut Recetario, detectados: &[PerfilDetectado], home: &Path) {
    for d in detectados {
        let dir = contraer(&d.dir, home);
        if r.perfiles.iter().any(|p| p.dir == dir) { continue; }
        let mut nombre = d.nombre_sugerido.clone();
        let mut n = 2;
        while r.perfiles.iter().any(|p| p.nombre == nombre) {
            nombre = format!("{}-{n}", d.nombre_sugerido);
            n += 1;
        }
        r.perfiles.push(Perfil { nombre, dir });
    }
}

pub fn fusionar(r: &mut Recetario, escaneo: Escaneo, existe_herramienta: &dyn Fn(&str) -> bool) {
    let vistos: HashSet<String> = escaneo.hallazgos.iter().map(|h| h.id.clone()).collect();
    for h in escaneo.hallazgos {
        match r.item_mut(&h.id) {
            Some(item) => {
                item.perfiles = h.perfiles;
                item.ausente = false;
                item.pista = h.pista;
                for req in h.requiere {
                    if !item.requiere.contains(&req) { item.requiere.push(req); }
                }
                // Un pendiente puede ganar fuente (p. ej. el repo recibió un remoto); lo revisado no se toca.
                if item.estado == Estado::Pendiente && item.fuente.is_none() {
                    item.fuente = h.fuente;
                }
            }
            None => {
                let mut item = Item::nuevo(&h.id, h.tipo);
                item.perfiles = h.perfiles;
                item.fuente = h.fuente;
                item.pista = h.pista;
                item.requiere = h.requiere;
                r.items.push(item);
            }
        }
    }
    for item in r.items.iter_mut().filter(|i| !vistos.contains(&i.id)) {
        // Las herramientas que agrega la investigación no salen en el escaneo: se mira el PATH.
        item.ausente = match item.tipo {
            Tipo::Herramienta => !existe_herramienta(item.id.trim_start_matches("herramienta:")),
            _ => true,
        };
    }
    r.checklist.ajustes = escaneo.ajustes;
    r.checklist.titulos = escaneo.titulos;
}
```

- [ ] **Step 4: Correr los tests y verificar que pasan**

Run: `cargo test --test fusion`
Expected: PASS (5).

- [ ] **Step 5: Commit**
```bash
git add src/fusion.rs src/lib.rs tests/fusion.rs
git commit -m "feat: merge scans without touching reviewed recipes"
```

### Task 9: Acciones de revisión

**Files:**
- Create: `src/acciones.rs`; Modify: `src/lib.rs` (`pub mod acciones;`)
- Test: `tests/acciones.rs`

**Interfaces:**
- Consumes: `modelo::*` (incluida `validar`), `escaner::normalizar_url`.
- Produces (todas devuelven `anyhow::Result<()>` salvo `item_como_toml`, y si fallan no cambian `r`):
  - `acciones::aprobar(r: &mut Recetario, id: &str)`: solo desde `por_revisar`.
  - `acciones::excluir(r: &mut Recetario, id: &str, motivo: &str)`: motivo no vacío.
  - `acciones::pegar_link(r: &mut Recetario, id: &str, url: &str)`: `http(s)://`; deja `fuente.via = manual`, estado `pendiente`, sin error.
  - `acciones::item_como_toml(item: &Item) -> anyhow::Result<String>`.
  - `acciones::reemplazar_desde_toml(r: &mut Recetario, id: &str, texto: &str)`: mismo `id`; queda `por_revisar` salvo que el texto diga `excluida`.
  - `acciones::renombrar_perfil(r: &mut Recetario, viejo: &str, nuevo: &str)`.

- [ ] **Step 1: Escribir los tests que fallan**

`tests/acciones.rs`:
```rust
use recetario::acciones::*;
use recetario::modelo::*;

fn recetario_con(estado: Estado) -> Recetario {
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil { nombre: "claude".into(), dir: "~/.claude".into() });
    let mut item = Item::nuevo("statusline:statusline.sh", Tipo::Statusline);
    item.perfiles = vec!["claude".into()];
    item.estado = estado;
    r.items.push(item);
    r.checklist.ajustes.push(Ajuste { perfil: "claude".into(), clave: "theme".into(), valor: "dark".into() });
    r
}

const ID: &str = "statusline:statusline.sh";

#[test]
fn aprobar_pendiente_se_rechaza() {
    let mut r = recetario_con(Estado::Pendiente);
    assert!(aprobar(&mut r, ID).is_err());
    assert_eq!(r.item(ID).unwrap().estado, Estado::Pendiente);
    let mut r = recetario_con(Estado::PorRevisar);
    aprobar(&mut r, ID).unwrap();
    assert_eq!(r.item(ID).unwrap().estado, Estado::Aprobada);
}

#[test]
fn excluir_exige_motivo() {
    let mut r = recetario_con(Estado::Pendiente);
    assert!(excluir(&mut r, ID, "   ").is_err());
    excluir(&mut r, ID, "no me interesa").unwrap();
    let x = r.item(ID).unwrap();
    assert_eq!(x.estado, Estado::Excluida);
    assert_eq!(x.motivo.as_deref(), Some("no me interesa"));
}

#[test]
fn pegar_link_deja_fuente_manual() {
    let mut r = recetario_con(Estado::Pendiente);
    assert!(pegar_link(&mut r, ID, "no-es-una-url").is_err());
    pegar_link(&mut r, ID, "https://github.com/nilbuild/claude-statusline/").unwrap();
    let f = r.item(ID).unwrap().fuente.clone().unwrap();
    assert_eq!(f.repo, "https://github.com/nilbuild/claude-statusline");
    assert_eq!(f.via, Via::Manual);
    assert_eq!(r.item(ID).unwrap().estado, Estado::Pendiente);
}

#[test]
fn edicion_invalida_no_cambia() {
    let mut r = recetario_con(Estado::Pendiente);
    let antes = r.clone();
    let texto = item_como_toml(r.item(ID).unwrap()).unwrap() + "color = \"rojo\"\n";
    assert!(reemplazar_desde_toml(&mut r, ID, &texto).is_err());
    let otro_id = item_como_toml(r.item(ID).unwrap()).unwrap().replace(ID, "statusline:otra.sh");
    assert!(reemplazar_desde_toml(&mut r, ID, &otro_id).is_err());
    assert_eq!(r, antes);
}

#[test]
fn edicion_valida_queda_por_revisar() {
    let mut r = recetario_con(Estado::Pendiente);
    let texto = item_como_toml(r.item(ID).unwrap()).unwrap().replace("estado = \"pendiente\"", "estado = \"aprobada\"")
        + "\n[[paso]]\ncmd = \"npx @kamranahmedse/claude-statusline\"\nmodo = \"auto\"\npor_perfil = false\n";
    reemplazar_desde_toml(&mut r, ID, &texto).unwrap();
    let x = r.item(ID).unwrap();
    assert_eq!(x.estado, Estado::PorRevisar);
    assert_eq!(x.pasos.len(), 1);
}

#[test]
fn renombrar_perfil_actualiza_items() {
    let mut r = recetario_con(Estado::Pendiente);
    renombrar_perfil(&mut r, "claude", "laburo").unwrap();
    assert_eq!(r.perfiles[0].nombre, "laburo");
    assert_eq!(r.item(ID).unwrap().perfiles, ["laburo"]);
    assert_eq!(r.checklist.ajustes[0].perfil, "laburo");
    r.perfiles.push(Perfil { nombre: "personal".into(), dir: "~/.claude-personal".into() });
    assert!(renombrar_perfil(&mut r, "laburo", "personal").is_err());
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run: `cargo test --test acciones`
Expected: no compila (`unresolved import recetario::acciones`).

- [ ] **Step 3: Implementar `src/acciones.rs`**
```rust
use crate::escaner::normalizar_url;
use crate::modelo::{validar, Estado, Fuente, Item, Recetario, Via};
use anyhow::{bail, Context, Result};

fn buscar<'a>(r: &'a mut Recetario, id: &str) -> Result<&'a mut Item> {
    r.item_mut(id).with_context(|| format!("no existe el ítem {id}"))
}

pub fn aprobar(r: &mut Recetario, id: &str) -> Result<()> {
    let item = buscar(r, id)?;
    match item.estado {
        Estado::PorRevisar => item.estado = Estado::Aprobada,
        Estado::Aprobada => {}
        Estado::Pendiente => bail!("{id} todavía no tiene receta: investigalo o editalo antes de aprobar"),
        Estado::Excluida => bail!("{id} está excluido"),
    }
    Ok(())
}

pub fn excluir(r: &mut Recetario, id: &str, motivo: &str) -> Result<()> {
    let motivo = motivo.trim();
    if motivo.is_empty() {
        bail!("el motivo no puede estar vacío");
    }
    let item = buscar(r, id)?;
    item.estado = Estado::Excluida;
    item.motivo = Some(motivo.into());
    Ok(())
}

pub fn pegar_link(r: &mut Recetario, id: &str, url: &str) -> Result<()> {
    let url = url.trim();
    let resto = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"));
    if resto.map_or(true, |r| r.split('/').next().unwrap_or("").is_empty()) {
        bail!("\"{url}\" no es un link http(s)");
    }
    let item = buscar(r, id)?;
    if item.estado == Estado::Excluida {
        bail!("{id} está excluido");
    }
    item.fuente = Some(Fuente { repo: normalizar_url(url), via: Via::Manual, doc: None });
    item.estado = Estado::Pendiente;
    item.error = None;
    Ok(())
}

pub fn item_como_toml(item: &Item) -> Result<String> {
    toml::to_string_pretty(item).context("no pude convertir el ítem a TOML")
}

pub fn reemplazar_desde_toml(r: &mut Recetario, id: &str, texto: &str) -> Result<()> {
    let mut nuevo: Item = toml::from_str(texto).map_err(|e| anyhow::anyhow!("la receta editada no es válida: {}", e.message()))?;
    if nuevo.id != id {
        bail!("no se puede cambiar el id ({id} → {})", nuevo.id);
    }
    // Aprobar es una acción aparte: una edición siempre vuelve a revisión.
    if nuevo.estado != Estado::Excluida {
        nuevo.estado = Estado::PorRevisar;
        nuevo.motivo = None;
    }
    let mut copia = r.clone();
    *buscar(&mut copia, id)? = nuevo;
    validar(&copia)?;
    *r = copia;
    Ok(())
}

pub fn renombrar_perfil(r: &mut Recetario, viejo: &str, nuevo: &str) -> Result<()> {
    let nuevo = nuevo.trim();
    if nuevo.is_empty() || nuevo.contains(char::is_whitespace) {
        bail!("el nombre del perfil no puede estar vacío ni tener espacios");
    }
    if r.perfiles.iter().any(|p| p.nombre == nuevo) {
        bail!("ya existe un perfil llamado {nuevo}");
    }
    let perfil = r.perfiles.iter_mut().find(|p| p.nombre == viejo).with_context(|| format!("no existe el perfil {viejo}"))?;
    perfil.nombre = nuevo.into();
    for item in &mut r.items {
        for p in item.perfiles.iter_mut().filter(|p| *p == viejo) { *p = nuevo.into(); }
    }
    for a in r.checklist.ajustes.iter_mut().filter(|a| a.perfil == viejo) { a.perfil = nuevo.into(); }
    for t in r.checklist.titulos.iter_mut().filter(|t| t.perfil == viejo) { t.perfil = nuevo.into(); }
    Ok(())
}
```

- [ ] **Step 4: Correr los tests y verificar que pasan**

Run: `cargo test --test acciones`
Expected: PASS (6).

- [ ] **Step 5: Commit**
```bash
git add src/acciones.rs src/lib.rs tests/acciones.rs
git commit -m "feat: review actions (approve, exclude, paste link, edit, rename profile)"
```

### Task 10: Investigador

**Files:**
- Create: `src/investigador.rs`; Modify: `src/lib.rs` (`pub mod investigador;`)
- Test: `tests/investigador.rs`

**Interfaces:**
- Consumes: `proceso::ejecutar`, `modelo::*`, `escaner::normalizar_url`, `rutas::expandir`.
- Produces:
  - `investigador::ESQUEMA: &str` (JSON Schema de la respuesta).
  - `investigador::Trabajo { id: String, prompt: String }`; `trabajo_para(r: &Recetario, item: &Item, home: &Path) -> Trabajo`.
  - `investigador::Requisito { tipo: String, nombre: String }`; `investigador::Receta { repo: String, doc: String, pasos: Vec<Paso>, requiere: Vec<Requisito>, verificar: Option<String> }`.
  - `investigador::argumentos(prompt: &str) -> Vec<String>`; `binario_claude() -> String` (`RECETARIO_CLAUDE` o `claude`).
  - `investigador::investigar(t: &Trabajo, claude: &str, tope: Duration) -> anyhow::Result<Receta>`; `parsear_salida(stdout: &str) -> anyhow::Result<Receta>`.
  - `investigador::en_paralelo(trabajos: Vec<Trabajo>, claude: String, tope: Duration, paralelo: usize) -> mpsc::Receiver<(String, anyhow::Result<Receta>)>`.
  - `investigador::aplicar(r: &mut Recetario, id: &str, resultado: anyhow::Result<Receta>, hoy: &str)`.
  - `investigador::TOPE: Duration` = 5 minutos; `investigador::PARALELO: usize` = 4.

- [ ] **Step 1: Escribir los tests que fallan**

`tests/investigador.rs`:
```rust
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
    let script = format!("#!/bin/sh\ncat <<'FIN'\nruido en una linea previa\n{}\nFIN\n", salida.replace('\n', ""));
    h.binario("bin/claude", script.as_bytes()).display().to_string()
}

fn recetario() -> Recetario {
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil { nombre: "laburo".into(), dir: "~/.claude".into() });
    let mut item = Item::nuevo("import:HOUSE-RULES.md", Tipo::Import);
    item.perfiles = vec!["laburo".into()];
    item.fuente = Some(Fuente { repo: "https://github.com/ejemplo/house-rules".into(), via: Via::Symlink, doc: None });
    r.items.push(item);
    r
}

const ID: &str = "import:HOUSE-RULES.md";

fn trabajo() -> Trabajo { Trabajo { id: ID.into(), prompt: "x".into() } }

#[test]
fn argumentos_sin_bash() {
    let a = argumentos("investigá");
    for flag in ["-p", "--restricted", "--strict-mcp-config", "--no-session-persistence", "--json-schema"] {
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
    assert_eq!(x.pasos[0].cita.as_deref(), Some("curl -fsSL https://ejemplo/install.sh | sh"));
    let f = x.fuente.clone().unwrap();
    assert_eq!(f.via, Via::Symlink);
    assert_eq!(f.doc.as_deref(), Some("https://github.com/ejemplo/house-rules#install"));
    assert_eq!(x.investigado.as_deref(), Some("2026-10-07"));
    assert!(x.error.is_none());
}

#[test]
fn json_invalido_queda_pendiente_con_error() {
    let h = HomeFalso::nuevo();
    let claude = claude_falso(&h, r#"{"type":"result","subtype":"success","is_error":false,"structured_output":{"repo":"x","doc":"y"}}"#);
    let mut r = recetario();
    aplicar(&mut r, ID, investigar(&trabajo(), &claude, Duration::from_secs(10)), "2026-10-07");
    let x = r.item(ID).unwrap();
    assert_eq!(x.estado, Estado::Pendiente);
    assert!(x.error.as_deref().unwrap().contains("formato"), "{:?}", x.error);
}

#[test]
fn tope_queda_pendiente() {
    let h = HomeFalso::nuevo();
    let claude = h.binario("bin/claude", b"#!/bin/sh\nsleep 30\n").display().to_string();
    let mut r = recetario();
    aplicar(&mut r, ID, investigar(&trabajo(), &claude, Duration::from_millis(300)), "2026-10-07");
    let x = r.item(ID).unwrap();
    assert_eq!(x.estado, Estado::Pendiente);
    assert!(x.error.as_deref().unwrap().contains("superó"));
}

#[test]
fn requisito_nuevo_y_de_sistema() {
    let h = HomeFalso::nuevo();
    let claude = claude_falso(&h, VALIDA);
    let mut r = recetario();
    aplicar(&mut r, ID, investigar(&trabajo(), &claude, Duration::from_secs(10)), "2026-10-07");
    let codex = r.item("herramienta:codex").unwrap();
    assert_eq!(codex.estado, Estado::Pendiente);
    assert_eq!(codex.pista.as_deref(), Some("requisito de import:HOUSE-RULES.md"));
    assert!(r.item(ID).unwrap().requiere.contains(&"herramienta:codex".to_string()));
    assert_eq!(r.checklist.sistema, vec![PaqueteSistema { paquete: "jq".into(), para: ID.into() }]);
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
    aplicar(&mut r, ID, investigar(&trabajo(), &claude, Duration::from_secs(10)), "2026-10-07");
    assert_eq!(r.item("herramienta:codex").unwrap().estado, Estado::Excluida);
}

#[test]
fn en_paralelo_devuelve_todos() {
    let h = HomeFalso::nuevo();
    let claude = claude_falso(&h, VALIDA);
    let trabajos = (0..3).map(|i| Trabajo { id: format!("skill:s{i}"), prompt: "x".into() }).collect();
    let rx = en_paralelo(trabajos, claude, Duration::from_secs(10), 2);
    let mut ids: Vec<String> = rx.iter().map(|(id, r)| { assert!(r.is_ok()); id }).collect();
    ids.sort();
    assert_eq!(ids, ["skill:s0", "skill:s1", "skill:s2"]);
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run: `cargo test --test investigador`
Expected: no compila (`unresolved import recetario::investigador`).

- [ ] **Step 3: Implementar `src/investigador.rs`**
```rust
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
```

- [ ] **Step 4: Correr los tests y verificar que pasan**

Run: `cargo test --test investigador`
Expected: PASS (7).

- [ ] **Step 5: Commit**
```bash
git add src/investigador.rs src/lib.rs tests/investigador.rs
git commit -m "feat: research documented installers with restricted claude -p"
```

### Task 11: Instalador

**Files:**
- Create: `src/instalador.rs`; Modify: `src/lib.rs` (`pub mod instalador;`)
- Test: `tests/instalador.rs`

**Interfaces:**
- Consumes: `modelo::*`, `proceso::ejecutar`, `rutas::expandir`.
- Produces:
  - `instalador::Resultado { Ok, Salteado, Simulado, Fallo(String), Bloqueado(String) }` (`Debug, Clone, PartialEq`).
  - `instalador::Evento { Inicio(String), Linea(String, String), Fin(String, Resultado) }` (id, texto).
  - `instalador::Opciones { dry_run: bool, tope: Duration, home: PathBuf, log: Option<PathBuf> }`; `instalador::TOPE` = 10 minutos.
  - `instalador::planificar(r: &Recetario) -> anyhow::Result<Vec<String>>`: ids `aprobada` en orden de `requiere` (dependencias no aprobadas se ignoran para el orden); ciclo → error con el recorrido.
  - `instalador::instalar(r: &Recetario, orden: &[String], op: &Opciones, emitir: &mut dyn FnMut(Evento)) -> Vec<(String, Resultado)>`.

- [ ] **Step 1: Escribir los tests que fallan**

`tests/instalador.rs`:
```rust
use recetario::instalador::*;
use recetario::modelo::*;
use std::fs;
use std::path::Path;
use std::time::Duration;

fn auto(cmd: &str, por_perfil: bool) -> Paso {
    Paso { cmd: cmd.into(), modo: Modo::Auto, por_perfil, cita: None, nota: None }
}

fn aprobado(id: &str, requiere: &[&str], pasos: Vec<Paso>) -> Item {
    let mut i = Item::nuevo(id, Tipo::Herramienta);
    i.estado = Estado::Aprobada;
    i.requiere = requiere.iter().map(|s| s.to_string()).collect();
    i.pasos = pasos;
    i
}

fn opciones(home: &Path, dry_run: bool) -> Opciones {
    Opciones { dry_run, tope: Duration::from_secs(10), home: home.to_path_buf(), log: Some(home.join("ultima.log")) }
}

fn correr(r: &Recetario, home: &Path, dry_run: bool) -> (Vec<(String, Resultado)>, Vec<Evento>) {
    let orden = planificar(r).unwrap();
    let mut eventos = Vec::new();
    let res = instalar(r, &orden, &opciones(home, dry_run), &mut |e| eventos.push(e));
    (res, eventos)
}

#[test]
fn solo_aprobadas_en_orden() {
    let dir = tempfile::tempdir().unwrap();
    let traza = dir.path().join("traza");
    let eco = |s: &str| auto(&format!("echo {s} >> '{}'", traza.display()), false);
    let mut r = Recetario::nuevo();
    r.items.push(aprobado("plugin:p@m", &["marketplace:m"], vec![eco("plugin")]));
    r.items.push(aprobado("marketplace:m", &["claude"], vec![eco("marketplace")]));
    r.items.push(aprobado("claude", &[], vec![eco("claude")]));
    let mut pendiente = aprobado("skill:x", &[], vec![eco("skill")]);
    pendiente.estado = Estado::Pendiente;
    r.items.push(pendiente);
    assert_eq!(planificar(&r).unwrap(), ["claude", "marketplace:m", "plugin:p@m"]);
    let (res, _) = correr(&r, dir.path(), false);
    assert!(res.iter().all(|(_, x)| *x == Resultado::Ok), "{res:?}");
    assert_eq!(fs::read_to_string(&traza).unwrap(), "claude\nmarketplace\nplugin\n");
}

#[test]
fn ciclo_no_ejecuta_nada() {
    let mut r = Recetario::nuevo();
    r.items.push(aprobado("a", &["b"], vec![]));
    r.items.push(aprobado("b", &["a"], vec![]));
    let err = planificar(&r).unwrap_err().to_string();
    assert!(err.contains("ciclo"), "{err}");
}

#[test]
fn verificar_por_perfil() {
    let dir = tempfile::tempdir().unwrap();
    let (p1, p2) = (dir.path().join("p1"), dir.path().join("p2"));
    fs::create_dir_all(&p1).unwrap();
    fs::create_dir_all(&p2).unwrap();
    fs::write(p1.join("listo"), "").unwrap();
    let traza = dir.path().join("traza");
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        Perfil { nombre: "uno".into(), dir: p1.display().to_string() },
        Perfil { nombre: "dos".into(), dir: p2.display().to_string() },
    ];
    let mut item = aprobado("import:X.md", &[], vec![
        auto(&format!("echo global >> '{}'", traza.display()), false),
        auto("touch \"$CLAUDE_CONFIG_DIR/instalado\"", true),
    ]);
    item.perfiles = vec!["uno".into(), "dos".into()];
    item.verificar = Some("test -f \"$CLAUDE_CONFIG_DIR/listo\" || test -f \"$CLAUDE_CONFIG_DIR/instalado\"".into());
    r.items.push(item);

    let (res, _) = correr(&r, dir.path(), false);
    assert_eq!(res[0].1, Resultado::Ok);
    assert!(!p1.join("instalado").exists());
    assert!(p2.join("instalado").exists());
    assert_eq!(fs::read_to_string(&traza).unwrap(), "global\n");

    let (res, _) = correr(&r, dir.path(), false);
    assert_eq!(res[0].1, Resultado::Salteado);
}

#[test]
fn falla_bloquea_dependientes() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = Recetario::nuevo();
    r.items.push(aprobado("a", &[], vec![auto("exit 7", false)]));
    r.items.push(aprobado("b", &["a"], vec![auto("true", false)]));
    r.items.push(aprobado("c", &[], vec![auto("true", false)]));
    let (res, _) = correr(&r, dir.path(), false);
    let de = |id: &str| res.iter().find(|(x, _)| x == id).unwrap().1.clone();
    assert!(matches!(de("a"), Resultado::Fallo(m) if m.contains('7')));
    assert!(matches!(de("b"), Resultado::Bloqueado(_)));
    assert_eq!(de("c"), Resultado::Ok);
    assert!(fs::read_to_string(dir.path().join("ultima.log")).unwrap().contains("exit 7"));
}

#[test]
fn dry_run_no_ejecuta() {
    let dir = tempfile::tempdir().unwrap();
    let marca = dir.path().join("marca");
    let mut r = Recetario::nuevo();
    let mut item = aprobado("a", &[], vec![auto(&format!("touch '{}'", marca.display()), false)]);
    item.verificar = Some(format!("touch '{}.verificado'", marca.display()));
    r.items.push(item);
    let (res, eventos) = correr(&r, dir.path(), true);
    assert_eq!(res[0].1, Resultado::Simulado);
    assert!(!marca.exists());
    assert!(!dir.path().join("marca.verificado").exists());
    assert!(eventos.iter().any(|e| matches!(e, Evento::Linea(_, t) if t.contains("touch"))));
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run: `cargo test --test instalador`
Expected: no compila (`unresolved import recetario::instalador`).

- [ ] **Step 3: Implementar `src/instalador.rs`**
```rust
use crate::modelo::{Estado, Item, Modo, Recetario};
use crate::proceso::ejecutar;
use crate::rutas::expandir;
use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

pub const TOPE: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Clone, PartialEq)]
pub enum Resultado { Ok, Salteado, Simulado, Fallo(String), Bloqueado(String) }

#[derive(Debug, Clone)]
pub enum Evento { Inicio(String), Linea(String, String), Fin(String, Resultado) }

pub struct Opciones {
    pub dry_run: bool,
    pub tope: Duration,
    pub home: PathBuf,
    pub log: Option<PathBuf>,
}

pub fn planificar(r: &Recetario) -> Result<Vec<String>> {
    let aprobadas: HashSet<&str> = r.items.iter().filter(|i| i.estado == Estado::Aprobada).map(|i| i.id.as_str()).collect();
    let mut hecho = HashSet::new();
    let mut pila: Vec<String> = Vec::new();
    let mut orden = Vec::new();
    for item in r.items.iter().filter(|i| aprobadas.contains(i.id.as_str())) {
        visitar(r, &item.id, &aprobadas, &mut hecho, &mut pila, &mut orden)?;
    }
    Ok(orden)
}

fn visitar(r: &Recetario, id: &str, aprobadas: &HashSet<&str>, hecho: &mut HashSet<String>, pila: &mut Vec<String>, orden: &mut Vec<String>) -> Result<()> {
    if hecho.contains(id) { return Ok(()); }
    if let Some(i) = pila.iter().position(|x| x == id) {
        bail!("ciclo en requiere: {} → {id}", pila[i..].join(" → "));
    }
    pila.push(id.to_string());
    if let Some(item) = r.item(id) {
        for dep in item.requiere.iter().filter(|d| aprobadas.contains(d.as_str())) {
            visitar(r, dep, aprobadas, hecho, pila, orden)?;
        }
    }
    pila.pop();
    hecho.insert(id.to_string());
    orden.push(id.to_string());
    Ok(())
}

pub fn instalar(r: &Recetario, orden: &[String], op: &Opciones, emitir: &mut dyn FnMut(Evento)) -> Vec<(String, Resultado)> {
    let mut log = abrir_log(op);
    let mut resultados: Vec<(String, Resultado)> = Vec::new();
    for id in orden {
        let Some(item) = r.item(id) else { continue };
        emitir(Evento::Inicio(id.clone()));
        let fallida = item.requiere.iter().find(|d| {
            resultados.iter().any(|(x, res)| x == *d && matches!(res, Resultado::Fallo(_) | Resultado::Bloqueado(_)))
        });
        let resultado = match fallida {
            Some(dep) => Resultado::Bloqueado(format!("depende de {dep}, que no se instaló")),
            None => instalar_item(r, item, op, &mut log, emitir),
        };
        emitir(Evento::Fin(id.clone(), resultado.clone()));
        resultados.push((id.clone(), resultado));
    }
    resultados
}

fn abrir_log(op: &Opciones) -> Option<File> {
    let ruta = op.log.as_ref()?;
    let abrir = || -> std::io::Result<File> {
        if let Some(dir) = ruta.parent() { fs::create_dir_all(dir)?; }
        OpenOptions::new().create(true).write(true).truncate(true).open(ruta)
    };
    match abrir() {
        Ok(f) => Some(f),
        Err(e) => {
            // Sin log la instalación sigue: la salida igual se ve en vivo.
            tracing::warn!(ruta = %ruta.display(), error = %e, "no pude abrir el log de instalación");
            None
        }
    }
}

fn instalar_item(r: &Recetario, item: &Item, op: &Opciones, log: &mut Option<File>, emitir: &mut dyn FnMut(Evento)) -> Resultado {
    let perfiles: Vec<(String, PathBuf)> = item
        .perfiles
        .iter()
        .filter_map(|n| r.perfiles.iter().find(|p| &p.nombre == n))
        .map(|p| (p.nombre.clone(), expandir(&p.dir, &op.home)))
        .collect();
    let autos: Vec<_> = item.pasos.iter().filter(|p| p.modo == Modo::Auto).collect();
    if op.dry_run {
        for paso in &autos {
            if paso.por_perfil && !perfiles.is_empty() {
                for (nombre, _) in &perfiles { emitir(Evento::Linea(item.id.clone(), format!("[{nombre}] {}", paso.cmd))); }
            } else {
                emitir(Evento::Linea(item.id.clone(), paso.cmd.clone()));
            }
        }
        return Resultado::Simulado;
    }
    let mut correr = |cmd: &str, dir: Option<&PathBuf>| -> Result<(), String> {
        let mut c = Command::new("sh");
        c.arg("-c").arg(cmd);
        match dir {
            Some(d) => { c.env("CLAUDE_CONFIG_DIR", d); }
            None => { c.env_remove("CLAUDE_CONFIG_DIR"); }
        }
        if let Some(f) = log.as_mut() {
            // Un log que no se puede escribir no frena la instalación.
            let _ = writeln!(f, "== {} $ {cmd}", item.id);
        }
        let salida = ejecutar(&mut c, op.tope, &mut |l| {
            if let Some(f) = log.as_mut() { let _ = writeln!(f, "{l}"); }
            emitir(Evento::Linea(item.id.clone(), l.to_string()));
        })
        .map_err(|e| format!("no pude ejecutar `{cmd}`: {e}"))?;
        if salida.vencido { return Err(format!("`{cmd}` superó {} segundos", op.tope.as_secs())); }
        if !salida.exito() { return Err(format!("`{cmd}` terminó con código {:?}", salida.codigo)); }
        Ok(())
    };
    let pendientes: Vec<&(String, PathBuf)> = match &item.verificar {
        None => perfiles.iter().collect(),
        Some(v) => perfiles.iter().filter(|(_, d)| correr(v, Some(d)).is_err()).collect(),
    };
    let listo = match &item.verificar {
        Some(v) if perfiles.is_empty() => correr(v, None).is_ok(),
        Some(_) => pendientes.is_empty(),
        None => false,
    };
    if listo { return Resultado::Salteado; }
    for paso in autos {
        let res = if paso.por_perfil && !perfiles.is_empty() {
            pendientes.iter().try_for_each(|(_, d)| correr(&paso.cmd, Some(d)))
        } else {
            correr(&paso.cmd, None)
        };
        if let Err(m) = res { return Resultado::Fallo(m); }
    }
    Resultado::Ok
}
```

- [ ] **Step 4: Correr los tests y verificar que pasan**

Run: `cargo test --test instalador`
Expected: PASS (5).

- [ ] **Step 5: Commit**
```bash
git add src/instalador.rs src/lib.rs tests/instalador.rs
git commit -m "feat: install approved recipes in dependency order with per-profile checks"
```

### Task 12: CLI (subcomandos)

**Files:**
- Create: `src/servicio.rs`, `src/main.rs`; Modify: `src/lib.rs` (`pub mod servicio;`)
- Test: `tests/cli.rs`

**Interfaces:**
- Consumes: todo lo anterior.
- Produces:
  - `servicio::ResumenEscaneo { nuevos: usize, avisos: Vec<String> }`; `servicio::escanear(r: &mut Recetario, entorno: &Entorno) -> anyhow::Result<ResumenEscaneo>` (detecta perfiles, escanea y fusiona).
  - `servicio::trabajos(r: &Recetario, ids: &[String], home: &Path) -> anyhow::Result<Vec<Trabajo>>`: sin ids, todos los `pendiente`; con ids, esos (error si alguno no existe o está excluido).
  - `servicio::texto_checklist(r: &Recetario) -> String`: logins, pasos manuales, ajustes, paquetes del sistema y títulos, en ese orden y con encabezados.
  - Binario `recetario [--archivo RUTA] [escanear | investigar [IDS…] | instalar [--dry-run] [--si] | renombrar-perfil VIEJO NUEVO]`. Sin subcomando, por ahora muestra la ayuda (la Task 14 lo cambia por la TUI). Resultados a stdout; progreso, avisos y errores a stderr; código de salida 1 si algo falló.

- [ ] **Step 1: Escribir los tests que fallan**

`tests/cli.rs`:
```rust
mod comun;
use comun::HomeFalso;
use recetario::{archivo, modelo::Estado};
use std::process::{Command, Output};

fn recetario(h: &HomeFalso, args: &[&str], claude: Option<&str>) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_recetario"));
    c.args(args)
        .env("HOME", h.ruta())
        .env("PATH", format!("{}:/usr/bin:/bin", h.ruta().join("bin").display()))
        .env_remove("CLAUDE_CONFIG_DIR");
    if let Some(cl) = claude { c.env("RECETARIO_CLAUDE", cl); }
    c.output().unwrap()
}

fn texto(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

#[test]
fn escanear_crea_recetario() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/settings.json", r#"{"enabledPlugins": {"codex@openai-codex": true}, "theme": "dark"}"#);
    let o = recetario(&h, &["escanear"], None);
    assert!(o.status.success(), "{}", texto(&o));
    let r = archivo::leer(&h.ruta().join(".config/recetario/recetario.toml")).unwrap();
    assert_eq!(r.perfiles[0].dir, "~/.claude");
    assert!(r.item("plugin:codex@openai-codex").is_some());
    assert!(r.checklist.ajustes.iter().any(|a| a.clave == "theme"));
}

const RESPUESTA: &str = r#"{"type":"result","subtype":"success","is_error":false,"structured_output":{"repo":"https://github.com/ejemplo/house-rules","doc":"https://github.com/ejemplo/house-rules#install","verificar":"test -f \"$CLAUDE_CONFIG_DIR/instalado\"","pasos":[{"cmd":"touch \"$CLAUDE_CONFIG_DIR/instalado\"","modo":"auto","por_perfil":true,"cita":"install.sh"},{"cmd":"/algo:setup","modo":"manual","por_perfil":false,"cita":"/algo:setup"}],"requiere":[]}}"#;

#[test]
fn de_punta_a_punta_con_claude_falso() {
    let h = HomeFalso::nuevo();
    let repo = h.repo_git("Proyectos/house-rules", Some("https://github.com/ejemplo/house-rules"));
    h.escribir("Proyectos/house-rules/HOUSE-RULES.md", "# reglas\n");
    h.enlazar(".claude/HOUSE-RULES.md", &repo.join("HOUSE-RULES.md"));
    h.escribir(".claude/CLAUDE.md", "@HOUSE-RULES.md\n");
    h.escribir(".claude/settings.json", "{}");
    let claude = h.binario("bin/claude-falso", format!("#!/bin/sh\ncat <<'FIN'\n{RESPUESTA}\nFIN\n").as_bytes());
    let claude = claude.to_str().unwrap();
    let ruta = h.ruta().join(".config/recetario/recetario.toml");
    let id = "import:HOUSE-RULES.md";

    assert!(recetario(&h, &["escanear"], None).status.success());
    let o = recetario(&h, &["investigar", id], Some(claude));
    assert!(o.status.success(), "{}", texto(&o));
    let mut r = archivo::leer(&ruta).unwrap();
    assert_eq!(r.item(id).unwrap().estado, Estado::PorRevisar);

    recetario::acciones::aprobar(&mut r, id).unwrap();
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
    assert!(String::from_utf8_lossy(&o.stdout).contains("salteado"), "{}", texto(&o));
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run: `cargo test --test cli`
Expected: no compila (`CARGO_BIN_EXE_recetario` no existe todavía porque no hay `src/main.rs`).

- [ ] **Step 3: Implementar `src/servicio.rs`**
```rust
use crate::checklist;
use crate::escaner::{self, Entorno};
use crate::fusion;
use crate::investigador::{trabajo_para, Trabajo};
use crate::modelo::{Estado, Recetario};
use crate::perfiles;
use anyhow::{bail, Result};
use std::collections::HashSet;
use std::fmt::Write;
use std::path::Path;

pub struct ResumenEscaneo {
    pub nuevos: usize,
    pub avisos: Vec<String>,
}

pub fn escanear(r: &mut Recetario, entorno: &Entorno) -> Result<ResumenEscaneo> {
    let detectados = perfiles::detectar(&entorno.home)?;
    fusion::asegurar_perfiles(r, &detectados, &entorno.home);
    let antes: HashSet<String> = r.items.iter().map(|i| i.id.clone()).collect();
    let escaneo = escaner::escanear(entorno, &r.perfiles);
    let avisos = escaneo.avisos.clone();
    let existe = |nombre: &str| entorno.path.iter().any(|d| d.join(nombre).is_file());
    fusion::fusionar(r, escaneo, &existe);
    let nuevos = r.items.iter().filter(|i| !antes.contains(&i.id)).count();
    Ok(ResumenEscaneo { nuevos, avisos })
}

pub fn trabajos(r: &Recetario, ids: &[String], home: &Path) -> Result<Vec<Trabajo>> {
    if ids.is_empty() {
        return Ok(r.items.iter().filter(|i| i.estado == Estado::Pendiente).map(|i| trabajo_para(r, i, home)).collect());
    }
    ids.iter()
        .map(|id| match r.item(id) {
            None => bail!("no existe el ítem {id}"),
            Some(i) if i.estado == Estado::Excluida => bail!("{id} está excluido"),
            Some(i) => Ok(trabajo_para(r, i, home)),
        })
        .collect()
}

pub fn texto_checklist(r: &Recetario) -> String {
    let mut s = String::from("\n== Checklist ==\n\nLogins\n");
    for l in checklist::logins(r) { let _ = writeln!(s, "  [ ] {l}"); }
    let manuales = checklist::pasos_manuales(r);
    if !manuales.is_empty() {
        s.push_str("\nPasos manuales\n");
        for (id, p) in manuales {
            let nota = p.nota.map(|n| format!(" ({n})")).unwrap_or_default();
            let _ = writeln!(s, "  [ ] {id}: {}{nota}", p.cmd);
        }
    }
    if !r.checklist.ajustes.is_empty() {
        s.push_str("\nAjustes\n");
        for a in &r.checklist.ajustes { let _ = writeln!(s, "  [ ] [{}] {} = {}", a.perfil, a.clave, a.valor); }
    }
    if !r.checklist.sistema.is_empty() {
        s.push_str("\nPaquetes del sistema\n");
        for p in &r.checklist.sistema { let _ = writeln!(s, "  [ ] sudo pacman -S {} (para {})", p.paquete, p.para); }
    }
    if !r.checklist.titulos.is_empty() {
        s.push_str("\nTu texto propio en CLAUDE.md\n");
        for t in &r.checklist.titulos { let _ = writeln!(s, "  [ ] [{}] {}", t.perfil, t.texto); }
    }
    s
}
```
(`writeln!` sobre un `String` no puede fallar; por eso se descarta el `Result`.)

- [ ] **Step 4: Implementar `src/main.rs`**
```rust
use anyhow::{bail, Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use recetario::escaner::{Entorno, PacmanReal};
use recetario::instalador::{self, Evento, Opciones, Resultado};
use recetario::investigador::{self, aplicar, en_paralelo};
use recetario::{acciones, archivo, rutas, servicio};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "recetario", version, about = "Recetario de instaladores para un setup de Claude Code")]
struct Cli {
    /// Recetario a usar (por defecto ~/.config/recetario/recetario.toml)
    #[arg(long, global = true)]
    archivo: Option<PathBuf>,
    #[command(subcommand)]
    comando: Option<Comando>,
}

#[derive(Subcommand)]
enum Comando {
    /// Escanea esta PC y agrega lo nuevo al recetario
    Escanear,
    /// Investiga con claude -p los ítems pendientes (o los ids indicados)
    Investigar { ids: Vec<String> },
    /// Instala las recetas aprobadas
    Instalar {
        /// Muestra los comandos sin ejecutar nada
        #[arg(long)]
        dry_run: bool,
        /// No pide confirmación
        #[arg(long)]
        si: bool,
    },
    /// Cambia el nombre de un perfil en todo el recetario
    RenombrarPerfil { viejo: String, nuevo: String },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let home = match rutas::home() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("recetario: {e:#}");
            return ExitCode::FAILURE;
        }
    };
    let _guardia = iniciar_logs(&home);
    match correr(cli, &home) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            tracing::error!(error = %format!("{e:#}"), "el comando falló");
            eprintln!("recetario: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn iniciar_logs(home: &Path) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let dir = rutas::dir_estado(home);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("recetario: sin logs, no pude crear {}: {e}", dir.display());
        return None;
    }
    let (escritor, guardia) = tracing_appender::non_blocking(tracing_appender::rolling::never(&dir, "recetario.log"));
    tracing_subscriber::fmt().with_writer(escritor).with_ansi(false).with_max_level(tracing::Level::INFO).init();
    Some(guardia)
}

/// `Ok(false)` = terminó, pero algo no salió (p. ej. una receta falló).
fn correr(cli: Cli, home: &Path) -> Result<bool> {
    let ruta = cli.archivo.unwrap_or_else(|| rutas::archivo_por_defecto(home));
    let Some(comando) = cli.comando else {
        Cli::command().print_help()?;
        return Ok(true);
    };
    let mut r = archivo::leer(&ruta)?;
    match comando {
        Comando::Escanear => {
            let pacman = PacmanReal;
            let entorno = Entorno::real(&pacman)?;
            let resumen = servicio::escanear(&mut r, &entorno)?;
            archivo::guardar(&ruta, &r)?;
            for a in &resumen.avisos { eprintln!("aviso: {a}"); }
            tracing::info!(comando = "escanear", items = r.items.len(), nuevos = resumen.nuevos, avisos = resumen.avisos.len());
            println!("{} ítems ({} nuevos) en {}", r.items.len(), resumen.nuevos, ruta.display());
            Ok(true)
        }
        Comando::Investigar { ids } => investigar(&mut r, &ruta, &ids, home),
        Comando::Instalar { dry_run, si } => instalar(&r, home, dry_run, si),
        Comando::RenombrarPerfil { viejo, nuevo } => {
            acciones::renombrar_perfil(&mut r, &viejo, &nuevo)?;
            archivo::guardar(&ruta, &r)?;
            println!("perfil {viejo} → {nuevo}");
            Ok(true)
        }
    }
}

fn investigar(r: &mut recetario::modelo::Recetario, ruta: &Path, ids: &[String], home: &Path) -> Result<bool> {
    let trabajos = servicio::trabajos(r, ids, home)?;
    let total = trabajos.len();
    let rx = en_paralelo(trabajos, investigador::binario_claude(), investigador::TOPE, investigador::PARALELO);
    let (mut ok, mut fallidos) = (0, 0);
    for (n, (id, resultado)) in rx.iter().enumerate() {
        match &resultado {
            Ok(_) => { ok += 1; eprintln!("[{}/{total}] {id} → por revisar", n + 1); }
            Err(e) => { fallidos += 1; eprintln!("[{}/{total}] {id} → error: {e:#}", n + 1); }
        }
        tracing::info!(comando = "investigar", id = %id, ok = resultado.is_ok());
        aplicar(r, &id, resultado, &rutas::hoy());
        // Se guarda tras cada ítem para no perder lo investigado si se corta.
        archivo::guardar(ruta, r)?;
    }
    println!("{ok} por revisar, {fallidos} con error");
    Ok(fallidos == 0)
}

fn instalar(r: &recetario::modelo::Recetario, home: &Path, dry_run: bool, si: bool) -> Result<bool> {
    let orden = instalador::planificar(r)?;
    if orden.is_empty() {
        println!("no hay recetas aprobadas");
        return Ok(true);
    }
    let perfiles: Vec<&str> = r.perfiles.iter().map(|p| p.nombre.as_str()).collect();
    if !dry_run && !si && !confirmar(&format!("¿Instalar {} recetas en {}? [s/N] ", orden.len(), perfiles.join(", ")))? {
        bail!("instalación cancelada");
    }
    let opciones = Opciones {
        dry_run,
        tope: instalador::TOPE,
        home: home.to_path_buf(),
        log: (!dry_run).then(|| rutas::dir_estado(home).join("ultima-instalacion.log")),
    };
    let resultados = instalador::instalar(r, &orden, &opciones, &mut |e| match e {
        Evento::Inicio(id) => eprintln!("→ {id}"),
        Evento::Linea(_, l) => eprintln!("    {l}"),
        Evento::Fin(..) => {}
    });
    let mut todo_bien = true;
    for (id, res) in &resultados {
        let texto = match res {
            Resultado::Ok => "instalado".to_string(),
            Resultado::Salteado => "salteado (ya estaba)".to_string(),
            Resultado::Simulado => "simulado".to_string(),
            Resultado::Fallo(m) => { todo_bien = false; format!("falló: {m}") }
            Resultado::Bloqueado(m) => { todo_bien = false; format!("bloqueado: {m}") }
        };
        tracing::info!(comando = "instalar", id = %id, resultado = %texto);
        println!("{id}: {texto}");
    }
    if !dry_run { print!("{}", servicio::texto_checklist(r)); }
    Ok(todo_bien)
}

fn confirmar(pregunta: &str) -> Result<bool> {
    eprint!("{pregunta}");
    std::io::stderr().flush()?;
    let mut linea = String::new();
    std::io::stdin().lock().read_line(&mut linea).context("no pude leer la respuesta")?;
    Ok(matches!(linea.trim(), "s" | "S" | "si" | "sí"))
}
```

- [ ] **Step 5: Correr los tests y verificar que pasan**

Run: `cargo test`
Expected: PASS de todo, incluidos `tests/cli.rs` (2).

- [ ] **Step 6: Commit**
```bash
git add src/servicio.rs src/main.rs src/lib.rs tests/cli.rs
git commit -m "feat: scan, research, install and rename-profile subcommands"
```

### Task 13: TUI — estado y vista

**Files:**
- Create: `src/tui/mod.rs` (solo `pub mod app; pub mod vista;` en esta tarea), `src/tui/app.rs`, `src/tui/vista.rs`; Modify: `src/lib.rs` (`pub mod tui;`)
- Test: `tests/tui.rs`

**Interfaces:**
- Consumes: `modelo::*`, `acciones::*`, `investigador::aplicar`, `instalador::{Evento, Resultado}`, `checklist::{logins, pasos_manuales}`.
- Produces:
  - `tui::app::{Pestana, Entrada, Modo, Efecto, App}`:
    - `Pestana { Recetas, Checklist, Instalacion }`; `Entrada { Link, Motivo, Filtro }`; `Modo { Normal, Escribiendo { para: Entrada, texto: String }, ConfirmarInstalacion }`.
    - `Efecto { Nada, Guardar, Escanear, Investigar(Vec<String>), Editar(String), Instalar, Salir }`. `Investigar` implica guardar antes de lanzar.
    - `App::nueva(r: Recetario) -> App`; campos públicos `recetario, pestana, seleccion, filtro, modo, mensaje: Option<String>, investigando: HashSet<String>, instalacion: Vec<(String, Option<Resultado>)>, en_curso: Option<String>, salida: Vec<String>, hechos: HashSet<usize>, seleccion_checklist: usize`.
    - `App::visibles(&self) -> Vec<&Item>` (orden por tipo y luego id; filtro por subcadena del id, sin distinguir mayúsculas); `App::seleccionado(&self) -> Option<&Item>`; `App::tecla(&mut self, k: KeyEvent) -> Efecto`; `App::lineas_checklist(&self) -> Vec<String>`; `App::registrar_investigacion(&mut self, id: &str, resultado: anyhow::Result<Receta>, hoy: &str)`; `App::evento_instalacion(&mut self, e: Evento)`.
  - `tui::vista::dibujar(f: &mut Frame, app: &App)`.

- [ ] **Step 1: Escribir los tests que fallan**

`tests/tui.rs`:
```rust
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use recetario::modelo::*;
use recetario::tui::app::{App, Efecto};
use recetario::tui::vista;

fn recetario() -> Recetario {
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil { nombre: "laburo".into(), dir: "~/.claude".into() });
    let mut claude = Item::nuevo("claude", Tipo::Claude);
    claude.estado = Estado::Aprobada;
    let mut codex = Item::nuevo("plugin:codex@openai-codex", Tipo::Plugin);
    codex.estado = Estado::Aprobada;
    codex.perfiles = vec!["laburo".into()];
    codex.fuente = Some(Fuente { repo: "https://github.com/openai/codex-plugin-cc".into(), via: Via::Metadatos, doc: None });
    codex.pasos = vec![Paso {
        cmd: "claude plugin install codex@openai-codex".into(),
        modo: Modo::Auto,
        por_perfil: true,
        cita: Some("/plugin install codex@openai-codex".into()),
        nota: None,
    }];
    let mut crawl = Item::nuevo("skill:crawl4ai", Tipo::Skill);
    crawl.estado = Estado::Excluida;
    crawl.motivo = Some("no me interesa".into());
    let mut status = Item::nuevo("statusline:statusline.sh", Tipo::Statusline);
    status.pista = Some("archivo sin origen conocido: ~/.claude/statusline.sh".into());
    r.items = vec![status, crawl, codex, claude];
    r
}

fn pantalla(app: &App) -> String {
    let mut t = Terminal::new(TestBackend::new(140, 32)).unwrap();
    t.draw(|f| vista::dibujar(f, app)).unwrap();
    let b = t.backend().buffer().clone();
    let mut s = String::new();
    for y in 0..b.area.height {
        for x in 0..b.area.width { s.push_str(b[(x, y)].symbol()); }
        s.push('\n');
    }
    s
}

fn tecla(app: &mut App, c: KeyCode) -> Efecto {
    app.tecla(KeyEvent::new(c, KeyModifiers::NONE))
}

#[test]
fn recetas_muestra_grupos_estados_y_detalle() {
    let mut app = App::nueva(recetario());
    let p = pantalla(&app);
    for esperado in ["Claude Code", "Plugins", "Skills", "Statusline", "✓ codex@openai-codex", "✗ crawl4ai", "no me interesa", "○ statusline.sh"] {
        assert!(p.contains(esperado), "falta {esperado:?} en:\n{p}");
    }
    app.filtro = "codex".into();
    app.seleccion = 0;
    let p = pantalla(&app);
    assert!(p.contains("github.com/openai/codex-plugin-cc"), "{p}");
    assert!(p.contains("cita: /plugin install codex@openai-codex"), "{p}");
}

#[test]
fn excluir_desde_el_teclado() {
    let mut app = App::nueva(recetario());
    app.filtro = "statusline".into();
    app.seleccion = 0;
    assert_eq!(tecla(&mut app, KeyCode::Char('x')), Efecto::Nada);
    for c in "no lo uso".chars() { tecla(&mut app, KeyCode::Char(c)); }
    assert_eq!(tecla(&mut app, KeyCode::Enter), Efecto::Guardar);
    let x = app.recetario.item("statusline:statusline.sh").unwrap();
    assert_eq!(x.estado, Estado::Excluida);
    assert_eq!(x.motivo.as_deref(), Some("no lo uso"));
}

#[test]
fn aprobar_pendiente_muestra_error_y_no_guarda() {
    let mut app = App::nueva(recetario());
    app.filtro = "statusline".into();
    assert_eq!(tecla(&mut app, KeyCode::Char('a')), Efecto::Nada);
    assert!(app.mensaje.as_deref().unwrap().contains("todavía no tiene receta"));
}

#[test]
fn investigar_todo_pide_solo_pendientes() {
    let mut app = App::nueva(recetario());
    assert_eq!(tecla(&mut app, KeyCode::Char('I')), Efecto::Investigar(vec!["statusline:statusline.sh".into()]));
    assert!(app.investigando.contains("statusline:statusline.sh"));
}
```

- [ ] **Step 2: Correr los tests y verificar que fallan**

Run: `cargo test --test tui`
Expected: no compila (`unresolved import recetario::tui`).

- [ ] **Step 3: Implementar `src/tui/app.rs`**
```rust
use crate::acciones;
use crate::checklist;
use crate::instalador::{Evento, Resultado};
use crate::investigador::{aplicar, Receta};
use crate::modelo::{Estado, Item, Recetario};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashSet;

const MAX_SALIDA: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pestana { Recetas, Checklist, Instalacion }

#[derive(Debug, Clone, PartialEq)]
pub enum Entrada { Link, Motivo, Filtro }

#[derive(Debug, Clone, PartialEq)]
pub enum Modo { Normal, Escribiendo { para: Entrada, texto: String }, ConfirmarInstalacion }

#[derive(Debug, Clone, PartialEq)]
pub enum Efecto { Nada, Guardar, Escanear, Investigar(Vec<String>), Editar(String), Instalar, Salir }

pub struct App {
    pub recetario: Recetario,
    pub pestana: Pestana,
    pub seleccion: usize,
    pub filtro: String,
    pub modo: Modo,
    pub mensaje: Option<String>,
    pub investigando: HashSet<String>,
    pub instalacion: Vec<(String, Option<Resultado>)>,
    pub en_curso: Option<String>,
    pub salida: Vec<String>,
    pub hechos: HashSet<usize>,
    pub seleccion_checklist: usize,
}

impl App {
    pub fn nueva(recetario: Recetario) -> Self {
        App {
            recetario, pestana: Pestana::Recetas, seleccion: 0, filtro: String::new(), modo: Modo::Normal,
            mensaje: None, investigando: HashSet::new(), instalacion: vec![], en_curso: None,
            salida: vec![], hechos: HashSet::new(), seleccion_checklist: 0,
        }
    }

    pub fn visibles(&self) -> Vec<&Item> {
        let filtro = self.filtro.to_lowercase();
        let mut items: Vec<&Item> = self.recetario.items.iter().filter(|i| i.id.to_lowercase().contains(&filtro)).collect();
        items.sort_by(|a, b| (a.tipo, &a.id).cmp(&(b.tipo, &b.id)));
        items
    }

    pub fn seleccionado(&self) -> Option<&Item> {
        self.visibles().get(self.seleccion).copied()
    }

    fn id_seleccionado(&self) -> Option<String> {
        self.seleccionado().map(|i| i.id.clone())
    }

    pub fn tecla(&mut self, k: KeyEvent) -> Efecto {
        match self.modo.clone() {
            Modo::Escribiendo { para, texto } => self.escribiendo(k.code, para, texto),
            Modo::ConfirmarInstalacion => {
                self.modo = Modo::Normal;
                if k.code == KeyCode::Char('s') { Efecto::Instalar } else { self.avisar("instalación cancelada"); Efecto::Nada }
            }
            Modo::Normal => self.normal(k.code),
        }
    }

    fn normal(&mut self, codigo: KeyCode) -> Efecto {
        match codigo {
            KeyCode::Char('q') => return Efecto::Salir,
            KeyCode::Char('1') => self.pestana = Pestana::Recetas,
            KeyCode::Char('2') => self.pestana = Pestana::Checklist,
            KeyCode::Char('3') => self.pestana = Pestana::Instalacion,
            KeyCode::Char('s') => return Efecto::Escanear,
            KeyCode::Char('P') => {
                let n = self.recetario.items.iter().filter(|i| i.estado == Estado::Aprobada).count();
                self.mensaje = Some(format!("¿Instalar {n} recetas aprobadas? s/n"));
                self.modo = Modo::ConfirmarInstalacion;
            }
            _ if self.pestana == Pestana::Checklist => self.checklist(codigo),
            _ if self.pestana == Pestana::Recetas => return self.recetas(codigo),
            _ => {}
        }
        Efecto::Nada
    }

    fn checklist(&mut self, codigo: KeyCode) {
        let total = self.lineas_checklist().len();
        match codigo {
            KeyCode::Down | KeyCode::Char('j') if self.seleccion_checklist + 1 < total => self.seleccion_checklist += 1,
            KeyCode::Up | KeyCode::Char('k') => self.seleccion_checklist = self.seleccion_checklist.saturating_sub(1),
            KeyCode::Char(' ') => {
                if !self.hechos.remove(&self.seleccion_checklist) { self.hechos.insert(self.seleccion_checklist); }
            }
            _ => {}
        }
    }

    fn recetas(&mut self, codigo: KeyCode) -> Efecto {
        let total = self.visibles().len();
        let id = self.id_seleccionado();
        match (codigo, id) {
            (KeyCode::Down | KeyCode::Char('j'), _) if self.seleccion + 1 < total => self.seleccion += 1,
            (KeyCode::Up | KeyCode::Char('k'), _) => self.seleccion = self.seleccion.saturating_sub(1),
            (KeyCode::Char('/'), _) => self.modo = Modo::Escribiendo { para: Entrada::Filtro, texto: self.filtro.clone() },
            (KeyCode::Char('x'), Some(_)) => self.modo = Modo::Escribiendo { para: Entrada::Motivo, texto: String::new() },
            (KeyCode::Char('l'), Some(_)) => self.modo = Modo::Escribiendo { para: Entrada::Link, texto: String::new() },
            (KeyCode::Char('e'), Some(id)) => return Efecto::Editar(id),
            (KeyCode::Char('a'), Some(id)) => {
                return match acciones::aprobar(&mut self.recetario, &id) {
                    Ok(()) => { self.avisar(&format!("{id} aprobado")); Efecto::Guardar }
                    Err(e) => { self.avisar(&format!("{e:#}")); Efecto::Nada }
                };
            }
            (KeyCode::Char('i' | 'r'), Some(id)) => return self.investigar(vec![id]),
            (KeyCode::Char('I'), _) => {
                let ids = self.recetario.items.iter().filter(|i| i.estado == Estado::Pendiente).map(|i| i.id.clone()).collect();
                return self.investigar(ids);
            }
            _ => {}
        }
        Efecto::Nada
    }

    fn investigar(&mut self, ids: Vec<String>) -> Efecto {
        let ids: Vec<String> = ids
            .into_iter()
            .filter(|id| !self.investigando.contains(id))
            .filter(|id| self.recetario.item(id).is_some_and(|i| i.estado != Estado::Excluida))
            .collect();
        if ids.is_empty() {
            self.avisar("no hay nada para investigar");
            return Efecto::Nada;
        }
        self.investigando.extend(ids.iter().cloned());
        self.avisar(&format!("investigando {} ítems…", ids.len()));
        Efecto::Investigar(ids)
    }

    fn escribiendo(&mut self, codigo: KeyCode, para: Entrada, mut texto: String) -> Efecto {
        match codigo {
            KeyCode::Esc => {
                self.modo = Modo::Normal;
                if para == Entrada::Filtro { self.filtro.clear(); self.seleccion = 0; }
            }
            KeyCode::Backspace => { texto.pop(); self.modo = Modo::Escribiendo { para, texto }; }
            KeyCode::Char(c) => { texto.push(c); self.modo = Modo::Escribiendo { para, texto }; }
            KeyCode::Enter => {
                self.modo = Modo::Normal;
                // El filtro se aplica aunque hoy no haya ningún ítem seleccionado.
                if para == Entrada::Filtro {
                    self.filtro = texto;
                    self.seleccion = 0;
                    return Efecto::Nada;
                }
                let Some(id) = self.id_seleccionado() else { return Efecto::Nada };
                let resultado = if para == Entrada::Motivo {
                    acciones::excluir(&mut self.recetario, &id, &texto).map(|_| Efecto::Guardar)
                } else {
                    acciones::pegar_link(&mut self.recetario, &id, &texto).map(|_| Efecto::Investigar(vec![id.clone()]))
                };
                return match resultado {
                    Ok(Efecto::Investigar(ids)) => self.investigar(ids),
                    Ok(efecto) => { self.avisar(&format!("{id} actualizado")); efecto }
                    Err(e) => { self.avisar(&format!("{e:#}")); Efecto::Nada }
                };
            }
            _ => self.modo = Modo::Escribiendo { para, texto },
        }
        Efecto::Nada
    }

    fn avisar(&mut self, texto: &str) {
        self.mensaje = Some(texto.to_string());
    }

    pub fn lineas_checklist(&self) -> Vec<String> {
        let r = &self.recetario;
        let mut l: Vec<String> = checklist::logins(r);
        l.extend(checklist::pasos_manuales(r).into_iter().map(|(id, p)| format!("{id}: {}", p.cmd)));
        l.extend(r.checklist.ajustes.iter().map(|a| format!("[{}] {} = {}", a.perfil, a.clave, a.valor)));
        l.extend(r.checklist.sistema.iter().map(|p| format!("sudo pacman -S {} (para {})", p.paquete, p.para)));
        l.extend(r.checklist.titulos.iter().map(|t| format!("[{}] CLAUDE.md: {}", t.perfil, t.texto)));
        l
    }

    pub fn registrar_investigacion(&mut self, id: &str, resultado: anyhow::Result<Receta>, hoy: &str) {
        self.investigando.remove(id);
        let texto = match &resultado {
            Ok(_) => format!("{id} → por revisar"),
            Err(e) => format!("{id} → error: {e:#}"),
        };
        aplicar(&mut self.recetario, id, resultado, hoy);
        self.avisar(&texto);
    }

    pub fn evento_instalacion(&mut self, e: Evento) {
        match e {
            Evento::Inicio(id) => { self.en_curso = Some(id); self.salida.clear(); }
            Evento::Linea(_, linea) => {
                self.salida.push(linea);
                if self.salida.len() > MAX_SALIDA { self.salida.remove(0); }
            }
            Evento::Fin(id, res) => {
                if let Some(fila) = self.instalacion.iter_mut().find(|(x, _)| *x == id) { fila.1 = Some(res); }
                self.en_curso = None;
            }
        }
    }
}
```

- [ ] **Step 4: Implementar `src/tui/vista.rs`**
```rust
use super::app::{App, Entrada, Modo, Pestana};
use crate::instalador::Resultado;
use crate::modelo::{Estado, Item, Modo as ModoPaso, Tipo, Via};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

const TECLAS: &str = "s escanear  i investigar  I investigar todo  a aprobar  x excluir  l pegar link  e editar  r reinvestigar  P instalar  / filtrar  q salir";

pub fn dibujar(f: &mut Frame, app: &App) {
    let [cabecera, cuerpo, pie] = Layout::vertical([Constraint::Length(1), Constraint::Min(3), Constraint::Length(2)]).areas(f.area());
    f.render_widget(Paragraph::new(texto_cabecera(app)), cabecera);
    match app.pestana {
        Pestana::Recetas => recetas(f, app, cuerpo),
        Pestana::Checklist => checklist(f, app, cuerpo),
        Pestana::Instalacion => instalacion(f, app, cuerpo),
    }
    f.render_widget(Paragraph::new(lineas_pie(app)), pie);
}

fn texto_cabecera(app: &App) -> String {
    let r = &app.recetario;
    let contar = |e: Estado| r.items.iter().filter(|i| i.estado == e).count();
    let perfiles: Vec<&str> = r.perfiles.iter().map(|p| p.nombre.as_str()).collect();
    format!(
        " recetario · {}   ✓ {} aprobadas  ? {} por revisar  ○ {} pendientes  ✗ {} excluidas",
        perfiles.join(" + "),
        contar(Estado::Aprobada), contar(Estado::PorRevisar), contar(Estado::Pendiente), contar(Estado::Excluida)
    )
}

fn grupo(t: Tipo) -> &'static str {
    match t {
        Tipo::Claude => "Claude Code",
        Tipo::Marketplace => "Marketplaces",
        Tipo::Plugin => "Plugins",
        Tipo::Skill => "Skills",
        Tipo::Import => "Imports CLAUDE.md",
        Tipo::Hook => "Hooks",
        Tipo::Statusline => "Statusline",
        Tipo::Herramienta => "Herramientas",
    }
}

fn simbolo(e: Estado) -> &'static str {
    match e { Estado::Aprobada => "✓", Estado::PorRevisar => "?", Estado::Pendiente => "○", Estado::Excluida => "✗" }
}

fn nombre_estado(e: Estado) -> &'static str {
    match e { Estado::Aprobada => "aprobada", Estado::PorRevisar => "por revisar", Estado::Pendiente => "pendiente", Estado::Excluida => "excluida" }
}

fn nombre_via(v: Via) -> &'static str {
    match v { Via::Manual => "manual", Via::Metadatos => "metadatos", Via::Symlink => "symlink", Via::Busqueda => "búsqueda" }
}

fn recetas(f: &mut Frame, app: &App, area: Rect) {
    let [izq, der] = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(area);
    let mut lineas: Vec<Line> = Vec::new();
    let mut linea_sel = 0;
    let mut tipo_actual = None;
    for (n, item) in app.visibles().into_iter().enumerate() {
        if tipo_actual != Some(item.tipo) {
            tipo_actual = Some(item.tipo);
            lineas.push(Line::from(Span::styled(grupo(item.tipo), Style::default().add_modifier(Modifier::BOLD))));
        }
        if n == app.seleccion { linea_sel = lineas.len(); }
        lineas.push(linea_item(app, item, n == app.seleccion));
    }
    let alto = izq.height.saturating_sub(2) as usize;
    let desde = linea_sel.saturating_sub(alto.saturating_sub(1));
    f.render_widget(
        Paragraph::new(lineas).scroll((desde as u16, 0)).block(Block::default().borders(Borders::ALL).title("Recetas")),
        izq,
    );
    let (titulo, detalle) = match app.seleccionado() {
        Some(item) => (item.id.clone(), lineas_detalle(app, item)),
        None => ("Detalle".into(), vec![Line::from("sin ítems (¿probaste escanear con s?)")]),
    };
    f.render_widget(
        Paragraph::new(detalle).wrap(Wrap { trim: false }).block(Block::default().borders(Borders::ALL).title(titulo)),
        der,
    );
}

fn linea_item(app: &App, item: &Item, seleccionado: bool) -> Line<'static> {
    let nombre = item.id.split_once(':').map(|(_, n)| n).unwrap_or(&item.id).to_string();
    let marca = if app.investigando.contains(&item.id) { "⟳" } else { simbolo(item.estado) };
    let extra = match (item.estado, &item.motivo, &item.pista, &item.error) {
        (Estado::Excluida, Some(m), _, _) => format!("  {m}"),
        (_, _, _, Some(_)) => "  error".into(),
        (Estado::Pendiente, _, Some(p), _) if p.starts_with("repo local sin remoto") => "  sin remoto".into(),
        (Estado::Pendiente, _, Some(p), _) if p.contains("sin origen") => "  sin origen".into(),
        _ => String::new(),
    };
    let ausente = if item.ausente { "  (ausente)" } else { "" };
    let mut estilo = Style::default();
    if item.ausente { estilo = estilo.add_modifier(Modifier::DIM); }
    if seleccionado { estilo = estilo.add_modifier(Modifier::REVERSED); }
    let flecha = if seleccionado { "▶" } else { " " };
    Line::from(Span::styled(format!("{flecha}{marca} {nombre}{extra}{ausente}"), estilo))
}

fn lineas_detalle(app: &App, item: &Item) -> Vec<Line<'static>> {
    let mut l = vec![Line::from(format!(
        "Estado: {}   Perfiles: {}",
        nombre_estado(item.estado),
        if item.perfiles.is_empty() { "—".to_string() } else { item.perfiles.join(", ") }
    ))];
    match &item.fuente {
        Some(f) => {
            l.push(Line::from(format!("Fuente: {} ({})", f.repo, nombre_via(f.via))));
            if let Some(doc) = &f.doc { l.push(Line::from(format!("Doc: {doc}"))); }
        }
        None => l.push(Line::from("Fuente: —")),
    }
    if !item.requiere.is_empty() { l.push(Line::from(format!("Requiere: {}", item.requiere.join(", ")))); }
    if let Some(v) = &item.verificar { l.push(Line::from(format!("Verificar: {v}"))); }
    for (etiqueta, valor) in [("Pista", &item.pista), ("Motivo", &item.motivo), ("Error", &item.error), ("Investigado", &item.investigado)] {
        if let Some(v) = valor { l.push(Line::from(format!("{etiqueta}: {v}"))); }
    }
    if app.investigando.contains(&item.id) { l.push(Line::from("Investigando…")); }
    if !item.pasos.is_empty() {
        l.push(Line::from(""));
        l.push(Line::from(Span::styled("Pasos", Style::default().add_modifier(Modifier::BOLD))));
        for (n, p) in item.pasos.iter().enumerate() {
            let modo = if p.modo == ModoPaso::Auto { "auto  " } else { "manual" };
            let perfil = if p.por_perfil { "por perfil " } else { "una vez    " };
            l.push(Line::from(format!("{} {modo} {perfil} {}", n + 1, p.cmd)));
            if let Some(c) = &p.cita { l.push(Line::from(format!("   cita: {c}"))); }
            if let Some(nota) = &p.nota { l.push(Line::from(format!("   nota: {nota}"))); }
        }
    }
    l
}

fn checklist(f: &mut Frame, app: &App, area: Rect) {
    let lineas: Vec<Line> = app
        .lineas_checklist()
        .into_iter()
        .enumerate()
        .map(|(n, texto)| {
            let marca = if app.hechos.contains(&n) { "[x]" } else { "[ ]" };
            let estilo = if n == app.seleccion_checklist { Style::default().add_modifier(Modifier::REVERSED) } else { Style::default() };
            Line::from(Span::styled(format!("{marca} {texto}"), estilo))
        })
        .collect();
    let alto = area.height.saturating_sub(2) as usize;
    let desde = app.seleccion_checklist.saturating_sub(alto.saturating_sub(1));
    f.render_widget(
        Paragraph::new(lineas).scroll((desde as u16, 0)).block(Block::default().borders(Borders::ALL).title("Checklist (espacio marca)")),
        area,
    );
}

fn instalacion(f: &mut Frame, app: &App, area: Rect) {
    let [izq, der] = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(area);
    let lineas: Vec<Line> = app
        .instalacion
        .iter()
        .map(|(id, res)| {
            let s = match res {
                None if app.en_curso.as_deref() == Some(id) => "⏳",
                None => " ",
                Some(Resultado::Ok) => "✓",
                Some(Resultado::Salteado) => "⤼",
                Some(Resultado::Simulado) => "~",
                Some(Resultado::Fallo(_)) => "✗",
                Some(Resultado::Bloqueado(_)) => "⊘",
            };
            Line::from(format!("{s} {id}"))
        })
        .collect();
    f.render_widget(Paragraph::new(lineas).block(Block::default().borders(Borders::ALL).title("Instalación")), izq);
    let alto = der.height.saturating_sub(2) as usize;
    let salida: Vec<Line> = app.salida.iter().skip(app.salida.len().saturating_sub(alto)).map(|l| Line::from(l.clone())).collect();
    let titulo = app.en_curso.clone().unwrap_or_else(|| "Salida".into());
    f.render_widget(Paragraph::new(salida).block(Block::default().borders(Borders::ALL).title(titulo)), der);
}

fn lineas_pie(app: &App) -> Vec<Line<'static>> {
    let pestanas = [(Pestana::Recetas, "[1]Recetas"), (Pestana::Checklist, "[2]Checklist"), (Pestana::Instalacion, "[3]Instalación")];
    let mut spans: Vec<Span> = Vec::new();
    for (p, texto) in pestanas {
        let estilo = if p == app.pestana { Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED) } else { Style::default() };
        spans.push(Span::styled(format!(" {texto} "), estilo));
    }
    spans.push(Span::raw(format!("  {TECLAS}")));
    let segunda = match &app.modo {
        Modo::Escribiendo { para, texto } => {
            let etiqueta = match para { Entrada::Link => "Link del repo", Entrada::Motivo => "Motivo para excluir", Entrada::Filtro => "Filtro" };
            format!(" {etiqueta}: {texto}_   (Enter confirma, Esc cancela)")
        }
        _ => format!(" {}", app.mensaje.clone().unwrap_or_default()),
    };
    vec![Line::from(spans), Line::from(segunda)]
}
```

- [ ] **Step 5: Crear `src/tui/mod.rs` y declararlo**

`src/tui/mod.rs`:
```rust
pub mod app;
pub mod vista;
```
En `src/lib.rs` agregar `pub mod tui;`.

- [ ] **Step 6: Correr los tests y verificar que pasan**

Run: `cargo test --test tui`
Expected: PASS (4).

- [ ] **Step 7: Commit**
```bash
git add src/tui src/lib.rs tests/tui.rs
git commit -m "feat: TUI state and views for recipes, checklist and install"
```

### Task 14: TUI — bucle, hilos y editor

**Files:**
- Modify: `src/tui/mod.rs`, `src/main.rs`
- Test: `tests/tui_arranque.rs`

**Interfaces:**
- Consumes: `tui::app::*`, `tui::vista::dibujar`, `servicio::{escanear, trabajos}`, `investigador::{en_paralelo, binario_claude, TOPE, PARALELO}`, `instalador::{planificar, instalar, Opciones, TOPE}`, `acciones::{item_como_toml, reemplazar_desde_toml}`, `archivo::{leer, guardar}`.
- Produces: `tui::ejecutar(ruta: &Path, home: &Path) -> anyhow::Result<()>`. Lee el recetario **antes** de tocar la terminal: un archivo inválido devuelve el error con línea y columna y la terminal queda intacta.

- [ ] **Step 1: Escribir el test que falla**

`tests/tui_arranque.rs`:
```rust
#[test]
fn no_arranca_con_recetario_invalido() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = dir.path().join("recetario.toml");
    std::fs::write(&ruta, "version = 1\n[[item]]\nid = \"claude\"\ntipo = \"claude\"\nestado = \"pendiente\"\ncolor = 1\n").unwrap();
    let err = recetario::tui::ejecutar(&ruta, dir.path()).unwrap_err().to_string();
    assert!(err.contains("recetario.toml:"), "{err}");
    assert!(err.contains("color"), "{err}");
}
```

- [ ] **Step 2: Correr el test y verificar que falla**

Run: `cargo test --test tui_arranque`
Expected: no compila (`cannot find function ejecutar in module recetario::tui`).

- [ ] **Step 3: Implementar el bucle en `src/tui/mod.rs`**
```rust
pub mod app;
pub mod vista;

use crate::escaner::{Entorno, PacmanReal};
use crate::instalador::{self, Evento, Opciones};
use crate::investigador::{self, Receta};
use crate::{acciones, archivo, rutas, servicio};
use anyhow::{Context, Result};
use app::{App, Efecto, Pestana};
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

type Investigado = (String, Result<Receta>);

pub fn ejecutar(ruta: &Path, home: &Path) -> Result<()> {
    let recetario = archivo::leer(ruta)?;
    let mut app = App::nueva(recetario);
    let mut terminal = ratatui::init();
    let resultado = bucle(&mut terminal, &mut app, ruta, home);
    ratatui::restore();
    resultado
}

fn bucle(terminal: &mut DefaultTerminal, app: &mut App, ruta: &Path, home: &Path) -> Result<()> {
    let (tx_inv, rx_inv) = mpsc::channel::<Investigado>();
    let mut rx_inst: Option<mpsc::Receiver<Evento>> = None;
    loop {
        while let Ok((id, res)) = rx_inv.try_recv() {
            tracing::info!(accion = "investigar", id = %id, ok = res.is_ok());
            app.registrar_investigacion(&id, res, &rutas::hoy());
            guardar(app, ruta);
        }
        if let Some(rx) = &rx_inst {
            while let Ok(e) = rx.try_recv() { app.evento_instalacion(e); }
        }
        terminal.draw(|f| vista::dibujar(f, app))?;
        if !event::poll(Duration::from_millis(100))? { continue; }
        let Event::Key(tecla) = event::read()? else { continue };
        if tecla.kind != KeyEventKind::Press { continue; }
        match app.tecla(tecla) {
            Efecto::Nada => {}
            Efecto::Salir => return Ok(()),
            Efecto::Guardar => guardar(app, ruta),
            Efecto::Escanear => escanear(app, ruta),
            Efecto::Investigar(ids) => investigar(app, ruta, home, ids, &tx_inv),
            Efecto::Editar(id) => editar(terminal, app, ruta, &id)?,
            Efecto::Instalar => rx_inst = instalar(app, home).or(rx_inst.take()),
        }
    }
}

fn guardar(app: &mut App, ruta: &Path) {
    if let Err(e) = archivo::guardar(ruta, &app.recetario) {
        tracing::error!(error = %format!("{e:#}"), "no pude guardar el recetario");
        app.mensaje = Some(format!("no pude guardar: {e:#}"));
    }
}

fn escanear(app: &mut App, ruta: &Path) {
    let pacman = PacmanReal;
    match Entorno::real(&pacman).and_then(|e| servicio::escanear(&mut app.recetario, &e)) {
        Ok(res) => {
            let avisos = if res.avisos.is_empty() { String::new() } else { format!(" · aviso: {}", res.avisos.join(" · ")) };
            app.mensaje = Some(format!("escaneo: {} nuevos{avisos}", res.nuevos));
            guardar(app, ruta);
        }
        Err(e) => app.mensaje = Some(format!("no pude escanear: {e:#}")),
    }
}

fn investigar(app: &mut App, ruta: &Path, home: &Path, ids: Vec<String>, tx: &mpsc::Sender<Investigado>) {
    guardar(app, ruta);
    match servicio::trabajos(&app.recetario, &ids, home) {
        Ok(trabajos) => {
            let rx = investigador::en_paralelo(trabajos, investigador::binario_claude(), investigador::TOPE, investigador::PARALELO);
            let tx = tx.clone();
            thread::spawn(move || {
                for m in rx {
                    if tx.send(m).is_err() { break; }
                }
            });
        }
        Err(e) => {
            for id in &ids { app.investigando.remove(id); }
            app.mensaje = Some(format!("{e:#}"));
        }
    }
}

fn instalar(app: &mut App, home: &Path) -> Option<mpsc::Receiver<Evento>> {
    let orden = match instalador::planificar(&app.recetario) {
        Ok(o) => o,
        Err(e) => {
            app.mensaje = Some(format!("{e:#}"));
            return None;
        }
    };
    app.instalacion = orden.iter().map(|id| (id.clone(), None)).collect();
    app.pestana = Pestana::Instalacion;
    let recetario = app.recetario.clone();
    let opciones = Opciones {
        dry_run: false,
        tope: instalador::TOPE,
        home: home.to_path_buf(),
        log: Some(rutas::dir_estado(home).join("ultima-instalacion.log")),
    };
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        instalador::instalar(&recetario, &orden, &opciones, &mut |e| {
            // Si la TUI se cerró, nadie escucha: la instalación sigue y queda en el log.
            let _ = tx.send(e);
        });
    });
    Some(rx)
}

fn editar(terminal: &mut DefaultTerminal, app: &mut App, ruta: &Path, id: &str) -> Result<()> {
    let item = app.recetario.item(id).context("el ítem a editar ya no existe")?;
    let temporal = std::env::temp_dir().join(format!("recetario-{}-editar.toml", std::process::id()));
    std::fs::write(&temporal, acciones::item_como_toml(item)?)?;
    ratatui::restore();
    let editor = std::env::var("VISUAL").or_else(|_| std::env::var("EDITOR")).unwrap_or_else(|_| "vi".into());
    // Vía `sh` para aceptar editores con argumentos (p. ej. "code --wait").
    let estado = Command::new("sh").arg("-c").arg(format!("{editor} \"$1\"")).arg("sh").arg(&temporal).status();
    *terminal = ratatui::init();
    app.mensaje = Some(match estado {
        Ok(s) if s.success() => {
            let texto = std::fs::read_to_string(&temporal)?;
            match acciones::reemplazar_desde_toml(&mut app.recetario, id, &texto) {
                Ok(()) => {
                    guardar(app, ruta);
                    format!("{id} editado; queda por revisar")
                }
                Err(e) => format!("no se cambió nada: {e:#}"),
            }
        }
        Ok(s) => format!("el editor terminó con {s}; no se cambió nada"),
        Err(e) => format!("no pude abrir {editor}: {e}"),
    });
    if let Err(e) = std::fs::remove_file(&temporal) {
        tracing::warn!(ruta = %temporal.display(), error = %e, "no pude borrar el temporal de edición");
    }
    Ok(())
}
```

- [ ] **Step 4: Abrir la TUI desde `main` cuando no hay subcomando**

En `src/main.rs`, dentro de `correr`, reemplazar:
```rust
    let Some(comando) = cli.comando else {
        Cli::command().print_help()?;
        return Ok(true);
    };
```
por:
```rust
    let Some(comando) = cli.comando else {
        recetario::tui::ejecutar(&ruta, home)?;
        return Ok(true);
    };
```
y quitar `CommandFactory` del `use clap::{…}`.

- [ ] **Step 5: Correr los tests y verificar que pasan**

Run: `cargo test`
Expected: PASS de toda la suite, incluido `tests/tui_arranque.rs` (1).

- [ ] **Step 6: Prueba manual de la TUI en un HOME temporal**

Run:
```bash
cargo build --release
export H=$(mktemp -d) && mkdir -p "$H/.claude" && echo '{"theme":"dark"}' > "$H/.claude/settings.json"
printf '@NOTAS.md\n## Propio\n' > "$H/.claude/CLAUDE.md" && echo '# notas' > "$H/.claude/NOTAS.md"
HOME=$H EDITOR=micro ./target/release/recetario escanear
HOME=$H EDITOR=micro ./target/release/recetario
```
Expected, en la TUI:
- `j`/`k` mueven la selección; `/` + `NOTAS` + Enter filtra; Esc en el filtro lo limpia.
- `x` + motivo + Enter excluye `import:NOTAS.md` y el motivo se ve en la lista.
- `e` abre micro con el ítem; agregar un `[[paso]]` con `cmd = "true"`, `modo = "auto"`, guardar y salir → el ítem queda `?` (por revisar); `a` lo aprueba (`✓`).
- `2` muestra la checklist con `theme = dark` y "Propio"; `espacio` marca `[x]`.
- `P` + `s` instala y la pestaña 3 muestra `✓` en los aprobados.
- `q` sale y la terminal queda normal.

- [ ] **Step 7: Commit**
```bash
git add src/tui/mod.rs src/main.rs tests/tui_arranque.rs
git commit -m "feat: interactive TUI loop with background research, install and editor"
```

### Task 15: README y evidencia en la máquina real

**Files:**
- Create: `README.md`, `LICENSE`, `intent/evidencia.md`
- Test: la suite completa y las corridas reales de abajo

**Interfaces:**
- Consumes: el binario terminado.
- Produces: documentación pública y la evidencia del SDLC (pregunta, procedimiento, resultado y conclusión por cada prueba).

- [ ] **Step 1: Escribir `README.md`**
````markdown
# recetario

Recetario de instaladores para un setup de Claude Code. En vez de copiar carpetas (que quedan
viejas o rotas), `recetario` mira qué tenés instalado, averigua con `claude -p` cómo se instala
cada cosa según su propio repo, y guarda esas recetas en un archivo TOML que revisás y
versionás. En una PC nueva, ejecuta las recetas aprobadas y te deja una checklist de lo que
falta hacer a mano.

Cubre plugins, marketplaces, skills, imports de `CLAUDE.md`, hooks, statusline y las
herramientas que usan, en todos tus perfiles (`~/.claude` y `~/.claude-*`). No copia
memorias, historial ni credenciales, ni instala paquetes del sistema.

## Instalar

Requiere Rust (en Arch: `sudo pacman -S rust`), `git` y Claude Code para investigar.

```sh
cargo install --git https://github.com/Ranteck/recetario
```

## Uso

```sh
recetario escanear          # mira esta PC y agrega lo nuevo al recetario
recetario                   # abre la TUI para revisar, investigar y aprobar
recetario investigar        # investiga los pendientes sin abrir la TUI
recetario instalar --dry-run
recetario instalar          # en la PC nueva
```

El recetario vive en `~/.config/recetario/recetario.toml` (o donde indique `--archivo`).
Tiene tus ajustes personales, así que guardalo en un repo **privado** y enlazalo:

```sh
mkdir -p ~/.config/recetario
ln -s ~/mi-repo-privado/recetario.toml ~/.config/recetario/recetario.toml
```

`recetario` escribe a través del enlace y lo conserva.

### Teclas de la TUI

| Tecla | Acción |
|---|---|
| `s` | escanear |
| `i` / `I` | investigar el ítem / todos los pendientes |
| `a` | aprobar (solo un ítem investigado o editado) |
| `x` | excluir, con motivo |
| `l` | pegar el link del repo cuando no se encontró |
| `e` | editar la receta en `$EDITOR` |
| `r` | reinvestigar |
| `P` | instalar lo aprobado |
| `/` | filtrar |
| `1` `2` `3` | Recetas, Checklist, Instalación |
| `q` | salir |

## Cómo decide cada receta

1. Busca el repo de origen: el link que pegaste, después los metadatos locales (catálogo del
   marketplace, symlink a un repo git, lock de skills.sh, URL dentro del binario) y, si no hay
   pista, la web.
2. `claude -p --restricted` lee la documentación de instalación de ese repo, sin poder
   ejecutar comandos ni escribir archivos, y devuelve los pasos con la **cita** literal de la
   doc de la que sale cada comando.
3. Vos revisás la cita y la URL y aprobás. Nada se ejecuta sin tu aprobación.

## Seguridad

- La investigación corre sin Bash, sin escritura, sin tus hooks, plugins ni MCP.
- El escaneo nunca lee `.credentials.json`, `~/.codex/auth.json` ni `.claude.json`.
- En la checklist, las variables de `env` y las claves con `token`, `secret`, `key` o
  `password` se guardan como `‹oculto›`.
- Instalar ejecuta comandos de terceros (a veces `curl … | sh`): revisá la cita antes de
  aprobar.

## Licencia

MIT
````

- [ ] **Step 2: Escribir `LICENSE`**

Texto MIT estándar, con la misma línea de copyright que `~/Proyectos/ai-native-sdlc/LICENSE` y el año 2026.

- [ ] **Step 3: Chequeos de calidad**

Run: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: sin diferencias de formato, sin warnings, todos los tests en verde.

- [ ] **Step 4: Evidencia 1 — escaneo real**

Run:
```bash
cargo build --release
mkdir -p /tmp/recetario-evidencia
./target/release/recetario --archivo /tmp/recetario-evidencia/recetario.toml escanear
grep -cE '^\[\[item\]\]' /tmp/recetario-evidencia/recetario.toml
grep -nE 'sk-ant|oauth|refresh_token' /tmp/recetario-evidencia/recetario.toml || echo "sin secretos"
```
Expected:
- Los dos perfiles (`claude` → `~/.claude`, `personal` → `~/.claude-personal`); renombrar con `recetario --archivo … renombrar-perfil claude laburo`.
- `plugin:codex@openai-codex` con fuente `https://github.com/openai/codex-plugin-cc`.
- `import:HOUSE-RULES.md` e `import:AI-NATIVE-SDLC.md` con `via = "symlink"` hacia `Ranteck/house-rules` y `Ranteck/ai-native-sdlc`.
- `herramienta:rtk` con fuente `https://github.com/rtk-ai/rtk`.
- `statusline:statusline.sh` pendiente "sin origen".
- Ajustes con `modelSettings.claude-opus-5-5.effortLevel = "xhigh"`.
- "sin secretos".

- [ ] **Step 5: Evidencia 2 — investigación real de tres ítems**

Run:
```bash
./target/release/recetario --archivo /tmp/recetario-evidencia/recetario.toml investigar \
  import:HOUSE-RULES.md import:AI-NATIVE-SDLC.md plugin:codex@openai-codex
```
Expected: `3 por revisar, 0 con error`. Abrir el TOML y comprobar a mano que cada `cita` aparece en el README del repo correspondiente; anotar en la evidencia si alguna no aparece o si un paso no respeta `CLAUDE_CONFIG_DIR`.

- [ ] **Step 6: Evidencia 3 — dry-run**

En la TUI (`./target/release/recetario --archivo /tmp/recetario-evidencia/recetario.toml`), aprobar esos tres ítems y `marketplace:openai-codex` (investigarlo antes con `i`). Después:
```bash
./target/release/recetario --archivo /tmp/recetario-evidencia/recetario.toml instalar --dry-run
```
Expected: los comandos en orden (`marketplace:openai-codex` antes que el plugin), con `[claude]`/`[personal]` en los pasos por perfil, y nada ejecutado.

- [ ] **Step 7: Evidencia 4 — instalación real en un HOME vacío**

Run:
```bash
H=$(mktemp -d) && mkdir -p "$H/.claude" "$H/.claude-personal"
HOME=$H ./target/release/recetario --archivo /tmp/recetario-evidencia/recetario.toml instalar --si
HOME=$H ./target/release/recetario --archivo /tmp/recetario-evidencia/recetario.toml instalar --si
```
Expected: primera corrida, los cuatro ítems `instalado` (en `$H/.claude*` aparecen `HOUSE-RULES.md`, `AI-NATIVE-SDLC.md` y el plugin de Codex) y la checklist impresa; segunda corrida, todos `salteado (ya estaba)`. Tu `~/.claude` real no cambia (comprobar con `git -C ~/Proyectos/hyprland-config status` y la fecha de `~/.claude/settings.json`).

- [ ] **Step 8: Escribir `intent/evidencia.md`**

Una sección por evidencia (1 a 4) con **Pregunta**, **Procedimiento** (los comandos), **Resultado** (salida relevante, recortada) y **Conclusión** (qué REQ queda demostrado y cualquier desvío). Si un desvío cambia el comportamiento pedido, frenar y consultarlo con Denis antes de corregir.

- [ ] **Step 9: Commit**
```bash
git add README.md LICENSE intent/evidencia.md
git commit -m "docs: README, license and real-machine evidence"
```
Publicar en GitHub (`gh repo create Ranteck/recetario --public --source . --push`) **solo** cuando Denis lo pida.
