# Plan: cookbook desde un repo (desde cookbook-remoto/spec.md 2026-10-08 e intent.md 2026-10-08)
Estado: aceptado

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `rebake clonar <url>` clona el repo del cookbook en su carpeta y, desde ahí, cada comando trae lo último antes de empezar y publica (commit solo del cookbook + push) al terminar.

**Architecture:** Un módulo nuevo `sincro` concentra todo lo de git y llama al `git` del sistema con `proceso::ejecutar`. `main` envuelve cada comando: si el cookbook está en un clon marcado, trae → lee "antes" → corre el comando → lee "después" → publica. Los fallos de git son avisos, no errores; solo `clonar` falla con error.

**Tech Stack:** Rust 2024, clap 4 (derive), anyhow, tracing; tests de integración con el binario, HOME falso y un repo bare local como remoto.

**Spec:** `intent/cookbook-remoto/spec.md`

## Global Constraints

- Sin dependencias nuevas: git del sistema (`Command::new("git")`), nunca `git2`.
- Toda llamada a git pasa por `proceso::ejecutar` con tope de 60 s (`sincro::TOPE`) y `GIT_TERMINAL_PROMPT=0`.
- Marca del clon: `rebake.sincronizar = true` en el `.git/config` del clon.
- Mensajes en español. Resultados a stdout; avisos a stderr con prefijo `aviso:` y `tracing::warn!`.
- Las URLs se muestran con `secretos::sin_credenciales_url`; la salida de git pasa por `sincro::limpiar` (saca usuario y token de toda URL y aplica `secretos::ocultar`).
- Los problemas de sincronización no cambian el código de salida del comando.
- Commits del repo en inglés, como el historial, con el trailer de sesión.
- Verificación de cada tarea: `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.

## Archivos que cambian

| Archivo | Estado | Responsabilidad |
|---|---|---|
| `src/sincro.rs` | nuevo | clonar, sincronizado, traer, publicar; helpers de git |
| `src/servicio.rs` | existe | `mensaje_commit` y `cantidad` |
| `src/lib.rs` | existe | `pub mod sincro;` |
| `src/main.rs` | existe | `Comando::Clonar`; envoltorio de sincronización en `correr` |
| `tests/servicio.rs` | nuevo | mensaje del commit |
| `tests/sincro.rs` | nuevo | clonar y sincronizar de punta a punta |
| `README.md` | existe | `rebake clonar` y uso en una PC nueva |
| `Cargo.toml`, `Cargo.lock` | existen | versión 0.3.0 |
| `intent/spec.md` | existe | nota de que v0.3.0 sincroniza con `rebake clonar` |

## Tests a escribir

| Test (comportamiento observable) | REQ que cubre |
| --- | --- |
| `clonar_en_carpeta_vacia_clona_y_cuenta_recetas` | REQ-1 |
| `clonar_no_pisa_una_carpeta_con_contenido` | REQ-2 |
| `clonar_un_cookbook_invalido_no_deja_nada` | REQ-1, REQ-3 |
| `clonar_un_repo_vacio_arranca_sin_recetas` | REQ-3 |
| `clonar_no_muestra_credenciales_de_la_url` | REQ-10 |
| `un_cookbook_enlazado_a_otro_repo_no_se_toca` | REQ-4 |
| `un_cambio_se_commitea_solo_y_llega_al_repo` | REQ-6 |
| `sin_cambios_no_hay_commit` | REQ-6 |
| `un_repo_vacio_recibe_el_cookbook_en_el_primer_guardado` | REQ-3, REQ-6 |
| `lo_guardado_se_publica_aunque_el_comando_falle` | REQ-9 |
| `sin_remoto_queda_commiteado_y_sale_en_la_corrida_siguiente` | REQ-8 |
| `un_commit_fallido_se_hace_en_la_corrida_siguiente` | REQ-8 |
| `trae_lo_que_otro_equipo_subio` | REQ-5 |
| `sin_red_avisa_y_sigue_con_la_copia_local` | REQ-5, REQ-8 |
| `cuenta_nuevos_y_cambios_de_estado`, `plural_y_sin_comando_para_la_tui`, `sin_nuevos_ni_estados_dice_que_actualiza` | REQ-7 |

REQ-10 "sin prompts" queda por diseño (`GIT_TERMINAL_PROMPT=0`, stdin nulo y tope); REQ-11 se revisa a mano. La TUI pasa por el mismo envoltorio que los comandos; los tests usan comandos porque no hay terminal.

---

### Task 1: mensaje del commit (REQ-7)

**Files:**
- Modify: `src/servicio.rs` (agregar al final)
- Test: `tests/servicio.rs` (nuevo)

**Interfaces:**
- Produces: `pub fn mensaje_commit(comando: Option<&str>, antes: &Recetario, despues: &Recetario) -> String` y `pub fn cantidad(n: usize, singular: &str, plural: &str) -> String`.

- [ ] **Step 1: Escribir los tests que fallan** en `tests/servicio.rs`:

```rust
use rebake::modelo::{Estado, Item, Recetario, Tipo};
use rebake::servicio::mensaje_commit;

fn con(items: &[(&str, Estado)]) -> Recetario {
    let mut r = Recetario::nuevo();
    for (id, estado) in items {
        let mut i = Item::nuevo(id, Tipo::Plugin);
        i.estado = *estado;
        r.items.push(i);
    }
    r
}

#[test]
fn cuenta_nuevos_y_cambios_de_estado() {
    let antes = con(&[("a", Estado::Pendiente), ("b", Estado::PorRevisar)]);
    let despues = con(&[
        ("a", Estado::Aprobada),
        ("b", Estado::Excluida),
        ("c", Estado::Pendiente),
    ]);
    assert_eq!(
        mensaje_commit(Some("escanear"), &antes, &despues),
        "rebake escanear: 1 nuevo, 1 aprobada, 1 excluida"
    );
}

#[test]
fn plural_y_sin_comando_para_la_tui() {
    let antes = con(&[("a", Estado::PorRevisar), ("b", Estado::PorRevisar)]);
    let despues = con(&[("a", Estado::Aprobada), ("b", Estado::Aprobada)]);
    assert_eq!(mensaje_commit(None, &antes, &despues), "rebake: 2 aprobadas");
}

#[test]
fn sin_nuevos_ni_estados_dice_que_actualiza() {
    let r = con(&[("a", Estado::Aprobada)]);
    assert_eq!(
        mensaje_commit(Some("renombrar-perfil"), &r, &r),
        "rebake renombrar-perfil: actualiza el cookbook"
    );
}
```

- [ ] **Step 2: Correr y ver que falla**

Run: `cargo test --test servicio`
Expected: error de compilación, `mensaje_commit` no existe en `rebake::servicio`.

- [ ] **Step 3: Implementar** al final de `src/servicio.rs`:

```rust
/// Resumen para el commit del repo del cookbook: ítems nuevos y cuántos pasaron a cada estado.
pub fn mensaje_commit(comando: Option<&str>, antes: &Recetario, despues: &Recetario) -> String {
    let nuevos = despues
        .items
        .iter()
        .filter(|i| antes.item(&i.id).is_none())
        .count();
    let mut partes = Vec::new();
    if nuevos > 0 {
        partes.push(cantidad(nuevos, "nuevo", "nuevos"));
    }
    for (estado, singular, plural) in [
        (Estado::PorRevisar, "por revisar", "por revisar"),
        (Estado::Aprobada, "aprobada", "aprobadas"),
        (Estado::Excluida, "excluida", "excluidas"),
        (Estado::Pendiente, "pendiente", "pendientes"),
    ] {
        let n = despues
            .items
            .iter()
            .filter(|i| {
                i.estado == estado && antes.item(&i.id).is_some_and(|a| a.estado != estado)
            })
            .count();
        if n > 0 {
            partes.push(cantidad(n, singular, plural));
        }
    }
    let prefijo = match comando {
        Some(c) => format!("rebake {c}"),
        None => "rebake".to_string(),
    };
    if partes.is_empty() {
        format!("{prefijo}: actualiza el cookbook")
    } else {
        format!("{prefijo}: {}", partes.join(", "))
    }
}

pub fn cantidad(n: usize, singular: &str, plural: &str) -> String {
    format!("{n} {}", if n == 1 { singular } else { plural })
}
```

- [ ] **Step 4: Correr y ver que pasa**

Run: `cargo test --test servicio`
Expected: 3 passed.

- [ ] **Step 5: Commit**

```bash
git add src/servicio.rs tests/servicio.rs
git commit -m "feat: summarize cookbook changes for a commit message"
```

---

### Task 2: `rebake clonar` (REQ-1, REQ-2, REQ-3, REQ-10)

**Files:**
- Create: `src/sincro.rs`
- Modify: `src/lib.rs` (agregar `pub mod sincro;` después de `pub mod servicio;`), `src/main.rs`
- Test: `tests/sincro.rs` (nuevo)

**Interfaces:**
- Consumes: `servicio::cantidad` (Task 1), `archivo::parsear`, `proceso::{ejecutar, Salida}`, `secretos::{ocultar, sin_credenciales_url}`.
- Produces: `pub fn sincro::clonar(url: &str, ruta: &Path) -> Result<Recetario>`; privados que usan las Tasks 3 y 4: `TOPE`, `MARCA`, `partes(ruta: &Path) -> Result<(&Path, &str)>`, `git(dir: &Path, args: &[&str]) -> Result<String>`, `correr_git(dir: &Path, args: &[&str]) -> Result<Salida>`, `limpiar(texto: &str) -> String`. En `main`: `Comando::Clonar { url: String }` y `fn clonar(url: &str, ruta: &Path, home: &Path) -> Result<bool>`. En `tests/sincro.rs`: helpers `aislar`, `git`, `rebake`, `rebake_con`, `texto`, `remoto`, `carpeta`, `url`, `COOKBOOK`.

- [ ] **Step 1: Escribir los tests que fallan** en `tests/sincro.rs`:

```rust
mod comun;
use comun::HomeFalso;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const COOKBOOK: &str = r#"version = 1

[[perfil]]
nombre = "laburo"
dir = "~/.claude"

[[item]]
id = "plugin:a@b"
tipo = "plugin"
perfiles = ["laburo"]
estado = "pendiente"
"#;

/// Git aislado de la config del usuario, con identidad y rama por defecto fijas.
fn aislar<'a>(c: &'a mut Command, h: &HomeFalso) -> &'a mut Command {
    c.env("HOME", h.ruta())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "init.defaultBranch")
        .env("GIT_CONFIG_VALUE_0", "main")
        .env("GIT_AUTHOR_NAME", "Prueba")
        .env("GIT_AUTHOR_EMAIL", "prueba@example.com")
        .env("GIT_COMMITTER_NAME", "Prueba")
        .env("GIT_COMMITTER_EMAIL", "prueba@example.com")
}

fn git(h: &HomeFalso, dir: &Path, args: &[&str]) -> String {
    let mut c = Command::new("git");
    c.arg("-C").arg(dir).args(args);
    let o = aislar(&mut c, h).output().unwrap();
    assert!(
        o.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

fn rebake_con(h: &HomeFalso, args: &[&str], entorno: &[(&str, &str)]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_rebake"));
    aislar(&mut c, h)
        .args(args)
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", h.ruta().join("bin").display()),
        )
        .env_remove("CLAUDE_CONFIG_DIR")
        .envs(entorno.iter().copied());
    c.output().unwrap()
}

fn rebake(h: &HomeFalso, args: &[&str]) -> Output {
    rebake_con(h, args, &[])
}

fn texto(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// Repo bare que hace de GitHub; con `cookbook` ya tiene un commit con ese archivo.
fn remoto(h: &HomeFalso, cookbook: Option<&str>) -> PathBuf {
    let bare = h.ruta().join("remoto.git");
    git(h, h.ruta(), &["init", "-q", "--bare", url(&bare)]);
    if let Some(contenido) = cookbook {
        let semilla = h.ruta().join("semilla");
        git(h, h.ruta(), &["clone", "-q", url(&bare), url(&semilla)]);
        fs::write(semilla.join("cookbook.toml"), contenido).unwrap();
        git(h, &semilla, &["add", "cookbook.toml"]);
        git(h, &semilla, &["commit", "-q", "-m", "semilla"]);
        git(h, &semilla, &["push", "-q", "origin", "HEAD"]);
    }
    bare
}

fn carpeta(h: &HomeFalso) -> PathBuf {
    h.ruta().join("Documents/rebake")
}

fn url(ruta: &Path) -> &str {
    ruta.to_str().unwrap()
}

#[test]
fn clonar_en_carpeta_vacia_clona_y_cuenta_recetas() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    fs::create_dir_all(carpeta(&h)).unwrap();
    let o = rebake(&h, &["clonar", url(&bare)]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("clonado en ~/Documents/rebake (1 receta)"),
        "{}",
        texto(&o)
    );
    assert_eq!(
        fs::read_to_string(carpeta(&h).join("cookbook.toml")).unwrap(),
        COOKBOOK
    );
}

#[test]
fn clonar_no_pisa_una_carpeta_con_contenido() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    let propio = h.escribir("Documents/rebake/cookbook.toml", "version = 1\n");
    let o = rebake(&h, &["clonar", url(&bare)]);
    assert!(!o.status.success());
    assert!(texto(&o).contains("no está vacía"), "{}", texto(&o));
    assert_eq!(fs::read_to_string(propio).unwrap(), "version = 1\n");
}

#[test]
fn clonar_un_cookbook_invalido_no_deja_nada() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some("version = 1\ncampo_raro = 1\n"));
    let o = rebake(&h, &["clonar", url(&bare)]);
    assert!(!o.status.success());
    assert!(texto(&o).contains("cookbook.toml:2:1"), "{}", texto(&o));
    let documentos: Vec<_> = fs::read_dir(h.ruta().join("Documents"))
        .unwrap()
        .collect();
    assert!(documentos.is_empty(), "{documentos:?}");
}

#[test]
fn clonar_un_repo_vacio_arranca_sin_recetas() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, None);
    let o = rebake(&h, &["clonar", url(&bare)]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("(0 recetas)"),
        "{}",
        texto(&o)
    );
}

#[test]
fn clonar_no_muestra_credenciales_de_la_url() {
    let h = HomeFalso::nuevo();
    let o = rebake(
        &h,
        &["clonar", "https://usuario:secreto@127.0.0.1:1/repo.git"],
    );
    assert!(!o.status.success());
    let salida = texto(&o);
    assert!(salida.contains("gh auth login"), "{salida}");
    assert!(!salida.contains("secreto"), "{salida}");
    let log = fs::read_to_string(h.ruta().join(".local/state/rebake/rebake.log")).unwrap();
    assert!(!log.contains("secreto"), "{log}");
}
```

- [ ] **Step 2: Correr y ver que falla**

Run: `cargo test --test sincro`
Expected: los 5 fallan; clap rechaza el subcomando `clonar` ("unrecognized subcommand").

- [ ] **Step 3: Crear `src/sincro.rs`**

```rust
//! El cookbook en un repo git: `clonar` lo trae y marca el clon, y solo ese clon se sincroniza.
use crate::archivo;
use crate::modelo::Recetario;
use crate::proceso::{Salida, ejecutar};
use crate::secretos::{ocultar, sin_credenciales_url};
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

pub const TOPE: Duration = Duration::from_secs(60);
const MARCA: &str = "rebake.sincronizar";

/// Clona `url` en la carpeta del cookbook y la marca para sincronizar; devuelve el cookbook.
pub fn clonar(url: &str, ruta: &Path) -> Result<Recetario> {
    let (destino, nombre) = partes(ruta)?;
    if destino.exists()
        && fs::read_dir(destino)
            .with_context(|| format!("no pude leer {}", destino.display()))?
            .next()
            .is_some()
    {
        bail!(
            "{} ya existe y no está vacía: movela o usá --archivo",
            destino.display()
        );
    }
    let padre = destino
        .parent()
        .context("la carpeta del cookbook no tiene carpeta padre")?;
    fs::create_dir_all(padre).with_context(|| format!("no pude crear {}", padre.display()))?;
    // Se clona al lado y se mueve al final: un fallo no deja un clon a medias en el destino.
    let carpeta = format!(".rebake-clon.{}", std::process::id());
    let temporal = padre.join(&carpeta);
    let resultado = clonar_en(url, padre, &carpeta, nombre).and_then(|r| {
        fs::rename(&temporal, destino)
            .with_context(|| format!("no pude mover el clon a {}", destino.display()))?;
        Ok(r)
    });
    if resultado.is_err()
        && temporal.exists()
        && let Err(e) = fs::remove_dir_all(&temporal)
    {
        // El error que importa es el del clonado; este solo deja una carpeta oculta de más.
        tracing::warn!(ruta = %temporal.display(), error = %e, "no pude borrar el clon temporal");
    }
    resultado
}

fn clonar_en(url: &str, padre: &Path, carpeta: &str, nombre: &str) -> Result<Recetario> {
    git(padre, &["clone", "--quiet", "--", url, carpeta]).with_context(|| {
        format!(
            "no pude clonar {}; si el repo es privado, autenticá git con `gh auth login` y \
             `gh auth setup-git`, o con una clave ssh",
            sin_credenciales_url(url)
        )
    })?;
    let clon = padre.join(carpeta);
    let ruta = clon.join(nombre);
    // Un repo recién creado no tiene cookbook: se arranca vacío y el primer guardado lo sube.
    let r = if ruta.exists() {
        let texto = fs::read_to_string(&ruta)
            .with_context(|| format!("no pude leer {}", ruta.display()))?;
        archivo::parsear(&texto, nombre).context("el cookbook del repo no es válido")?
    } else {
        Recetario::nuevo()
    };
    git(&clon, &["config", MARCA, "true"])?;
    Ok(r)
}

/// La carpeta del cookbook (la del clon) y el nombre del archivo dentro de ella.
fn partes(ruta: &Path) -> Result<(&Path, &str)> {
    let dir = ruta
        .parent()
        .context("la ruta del cookbook no tiene carpeta")?;
    let nombre = ruta
        .file_name()
        .and_then(|n| n.to_str())
        .context("el nombre del cookbook no es texto válido")?;
    Ok((dir, nombre))
}

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let salida = correr_git(dir, args)?;
    if !salida.exito() {
        bail!("git {} falló: {}", args[0], limpiar(&salida.texto));
    }
    Ok(salida.texto)
}

/// Sin terminal: git falla en vez de pedir usuario o contraseña, y el tope corta lo que se cuelgue.
fn correr_git(dir: &Path, args: &[&str]) -> Result<Salida> {
    let mut c = Command::new("git");
    c.arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0");
    let salida = ejecutar(&mut c, TOPE, &mut |_| {})
        .with_context(|| format!("no pude ejecutar git {}", args[0]))?;
    if salida.vencido {
        bail!("git {} superó {} segundos", args[0], TOPE.as_secs());
    }
    Ok(salida)
}

/// La salida de git puede repetir la URL del remoto con usuario y token: se quitan antes de
/// `ocultar`, que no reconoce una URL seguida de `:`.
fn limpiar(texto: &str) -> String {
    texto
        .lines()
        .map(|linea| {
            let sin_usuario: Vec<String> = linea
                .split(' ')
                .map(|p| {
                    if p.contains("://") {
                        sin_credenciales_url(p)
                    } else {
                        p.to_string()
                    }
                })
                .collect();
            ocultar(&sin_usuario.join(" "))
        })
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" / ")
}
```

- [ ] **Step 4: Registrar el módulo** en `src/lib.rs`, después de `pub mod servicio;`:

```rust
pub mod sincro;
```

- [ ] **Step 5: Subcomando en `src/main.rs`**

Import: `use rebake::{acciones, archivo, rutas, servicio, sincro};`

Variante nueva al principio de `enum Comando`:

```rust
    /// Clona el repo git de tu cookbook en su carpeta; desde ahí cada comando lo sincroniza
    Clonar { url: String },
```

En `correr`, borrar `let mut r = archivo::leer(&ruta)?;` y dejar el `match` así (cada brazo lee el cookbook, para que `clonar` no lea antes de clonar):

```rust
    match comando {
        Comando::Clonar { url } => clonar(&url, &ruta, home),
        Comando::Escanear => {
            let mut r = archivo::leer(&ruta)?;
            let pacman = PacmanReal;
            // … resto del brazo sin cambios …
        }
        Comando::Investigar { ids } => {
            investigar(&mut archivo::leer(&ruta)?, &ruta, &ids, home)
        }
        Comando::Instalar { dry_run, si } => instalar(&archivo::leer(&ruta)?, home, dry_run, si),
        Comando::RenombrarPerfil { viejo, nuevo } => {
            let mut r = archivo::leer(&ruta)?;
            acciones::renombrar_perfil(&mut r, &viejo, &nuevo)?;
            archivo::guardar(&ruta, &r)?;
            println!("perfil {viejo} → {nuevo}");
            Ok(true)
        }
    }
```

Función nueva, después de `correr`:

```rust
fn clonar(url: &str, ruta: &Path, home: &Path) -> Result<bool> {
    let r = sincro::clonar(url, ruta)?;
    let carpeta = ruta.parent().unwrap_or(ruta);
    tracing::info!(comando = "clonar", items = r.items.len());
    println!(
        "clonado en {} ({})",
        rutas::contraer(carpeta, home),
        servicio::cantidad(r.items.len(), "receta", "recetas")
    );
    Ok(true)
}
```

- [ ] **Step 6: Correr y ver que pasa**

Run: `cargo test --test sincro && cargo test`
Expected: 5 passed en `sincro`; la suite completa en verde.

- [ ] **Step 7: Lint y commit**

```bash
cargo clippy --all-targets -- -D warnings && cargo fmt --check
git add src/sincro.rs src/lib.rs src/main.rs tests/sincro.rs
git commit -m "feat: clone the cookbook from its git repo with rebake clonar"
```

---

### Task 3: publicar al terminar (REQ-4, REQ-6, REQ-8, REQ-9)

**Files:**
- Modify: `src/sincro.rs` (agregar `sincronizado`, `publicar`, `Rama`, `rama`), `src/main.rs`
- Test: `tests/sincro.rs` (agregar)

**Interfaces:**
- Consumes: `partes`, `git`, `correr_git`, `MARCA` (Task 2); `servicio::mensaje_commit` (Task 1).
- Produces: `pub fn sincro::sincronizado(ruta: &Path) -> Result<bool>`, `pub fn sincro::publicar(ruta: &Path, mensaje: &str) -> Result<bool>`, privados `struct Rama { con_commits: bool, adelante: Option<u32>, cookbook_cambiado: bool }` y `fn rama(dir: &Path, nombre: &str) -> Result<Rama>`. En `main`: `fn avisar(e: &anyhow::Error)` (lo usa la Task 4) y `fn ejecutar_comando`.

- [ ] **Step 1: Escribir los tests** al final de `tests/sincro.rs`:

```rust
/// Clona `bare` con rebake y devuelve la carpeta del clon.
fn clonado(h: &HomeFalso, bare: &Path) -> PathBuf {
    let o = rebake(h, &["clonar", url(bare)]);
    assert!(o.status.success(), "{}", texto(&o));
    carpeta(h)
}

fn commits(h: &HomeFalso, repo: &Path) -> String {
    git(h, repo, &["rev-list", "--count", "--all"])
}

fn ultimo_mensaje(h: &HomeFalso, bare: &Path) -> String {
    git(h, bare, &["log", "-1", "--format=%s", "main"])
}

#[test]
fn un_cambio_se_commitea_solo_y_llega_al_repo() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    let clon = clonado(&h, &bare);
    fs::write(clon.join("notas.md"), "algo\n").unwrap();
    git(&h, &clon, &["add", "notas.md"]);
    let o = rebake(&h, &["renombrar-perfil", "laburo", "casa"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert_eq!(
        ultimo_mensaje(&h, &bare),
        "rebake renombrar-perfil: actualiza el cookbook"
    );
    assert_eq!(
        git(&h, &bare, &["show", "--name-only", "--format=", "main"]),
        "cookbook.toml"
    );
}

#[test]
fn sin_cambios_no_hay_commit() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    clonado(&h, &bare);
    let o = rebake(&h, &["instalar", "--dry-run"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert_eq!(commits(&h, &bare), "1");
}

#[test]
fn un_cookbook_enlazado_a_otro_repo_no_se_toca() {
    let h = HomeFalso::nuevo();
    let dotfiles = h.ruta().join("dotfiles");
    fs::create_dir_all(&dotfiles).unwrap();
    git(&h, &dotfiles, &["init", "-q"]);
    let real = h.escribir("dotfiles/cookbook.toml", COOKBOOK);
    h.enlazar("Documents/rebake/cookbook.toml", &real);
    let o = rebake(&h, &["renombrar-perfil", "laburo", "casa"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(fs::read_to_string(&real).unwrap().contains("casa"));
    assert_eq!(commits(&h, &dotfiles), "0");
}

#[test]
fn un_repo_vacio_recibe_el_cookbook_en_el_primer_guardado() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, None);
    clonado(&h, &bare);
    h.escribir(".claude/settings.json", "{}");
    let o = rebake(&h, &["escanear"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        ultimo_mensaje(&h, &bare).starts_with("rebake escanear"),
        "{}",
        texto(&o)
    );
    git(&h, &bare, &["cat-file", "-e", "main:cookbook.toml"]);
}

#[test]
fn lo_guardado_se_publica_aunque_el_comando_falle() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    clonado(&h, &bare);
    let claude = h.binario("bin/claude-roto", b"#!/bin/sh\nexit 1\n");
    let o = rebake_con(
        &h,
        &["investigar"],
        &[("REBAKE_CLAUDE", claude.to_str().unwrap())],
    );
    assert!(!o.status.success(), "{}", texto(&o));
    assert_eq!(commits(&h, &bare), "2");
}

#[test]
fn sin_remoto_queda_commiteado_y_sale_en_la_corrida_siguiente() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    let clon = clonado(&h, &bare);
    let apagado = h.ruta().join("apagado.git");
    fs::rename(&bare, &apagado).unwrap();
    let o = rebake(&h, &["renombrar-perfil", "laburo", "casa"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        texto(&o).contains("se pushea en la próxima corrida"),
        "{}",
        texto(&o)
    );
    assert!(git(&h, &clon, &["log", "-1", "--format=%s"]).starts_with("rebake renombrar-perfil"));
    fs::rename(&apagado, &bare).unwrap();
    let o = rebake(&h, &["instalar", "--dry-run"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(ultimo_mensaje(&h, &bare).starts_with("rebake renombrar-perfil"));
}

#[test]
fn un_commit_fallido_se_hace_en_la_corrida_siguiente() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    let clon = clonado(&h, &bare);
    let o = rebake_con(
        &h,
        &["renombrar-perfil", "laburo", "casa"],
        &[("GIT_AUTHOR_NAME", ""), ("GIT_COMMITTER_NAME", "")],
    );
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        texto(&o).contains("se commitea en la próxima corrida"),
        "{}",
        texto(&o)
    );
    assert!(fs::read_to_string(clon.join("cookbook.toml")).unwrap().contains("casa"));
    assert_eq!(commits(&h, &bare), "1");
    let o = rebake(&h, &["instalar", "--dry-run"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert_eq!(commits(&h, &bare), "2");
}
```

- [ ] **Step 2: Correr y ver que fallan**

Run: `cargo test --test sincro`
Expected: fallan `un_cambio_se_commitea_solo_y_llega_al_repo`, `un_repo_vacio_recibe_el_cookbook_en_el_primer_guardado`, `lo_guardado_se_publica_aunque_el_comando_falle`, `sin_remoto_queda_commiteado_y_sale_en_la_corrida_siguiente` y `un_commit_fallido_se_hace_en_la_corrida_siguiente`, porque nada se publica. `sin_cambios_no_hay_commit` y `un_cookbook_enlazado_a_otro_repo_no_se_toca` ya pasan: cuidan REQ-4 y REQ-6 de acá en adelante.

- [ ] **Step 3: Agregar a `src/sincro.rs`**

```rust
/// Si el cookbook está en la raíz de un clon hecho por `clonar`. Sin `.git` en esa carpeta no
/// se ejecuta git: un cookbook suelto o enlazado desde otro repo no se toca.
pub fn sincronizado(ruta: &Path) -> Result<bool> {
    let (dir, _) = partes(ruta)?;
    if !dir.join(".git").is_dir() {
        return Ok(false);
    }
    let salida = correr_git(dir, &["config", "--local", "--get", MARCA])?;
    Ok(salida.exito() && salida.texto.trim() == "true")
}

/// Commitea solo el cookbook si cambió y pushea lo que la rama tenga sin subir; `Ok(true)` si
/// subió algo.
pub fn publicar(ruta: &Path, mensaje: &str) -> Result<bool> {
    let (dir, nombre) = partes(ruta)?;
    if rama(dir, nombre)?.cookbook_cambiado {
        git(dir, &["add", "--", nombre])?;
        // Con la ruta, el commit lleva solo el cookbook aunque haya otras cosas en el índice.
        git(dir, &["commit", "--quiet", "-m", mensaje, "--", nombre]).context(
            "no pude commitear el cookbook; quedó guardado en disco y se commitea en la próxima \
             corrida",
        )?;
    }
    let estado = rama(dir, nombre)?;
    if !estado.con_commits || estado.adelante == Some(0) {
        return Ok(false);
    }
    git(dir, &["push", "--quiet", "--set-upstream", "origin", "HEAD"])
        .context("el cookbook quedó commiteado en local y se pushea en la próxima corrida")?;
    Ok(true)
}

/// Lo que dice `git status` de la rama y del cookbook, sin tocar la red.
struct Rama {
    con_commits: bool,
    /// Commits sin subir; `None` si la rama todavía no tiene rama remota (repo recién creado o
    /// un primer push que no salió), y entonces todo commit está pendiente.
    adelante: Option<u32>,
    cookbook_cambiado: bool,
}

fn rama(dir: &Path, nombre: &str) -> Result<Rama> {
    let texto = git(dir, &["status", "--porcelain=v2", "--branch", "--", nombre])?;
    let mut estado = Rama {
        con_commits: true,
        adelante: None,
        cookbook_cambiado: false,
    };
    for linea in texto.lines() {
        if linea == "# branch.oid (initial)" {
            estado.con_commits = false;
        } else if let Some(ab) = linea.strip_prefix("# branch.ab +") {
            let n = ab.split_once(' ').map_or(ab, |(n, _)| n);
            estado.adelante = Some(
                n.parse()
                    .with_context(|| format!("git status devolvió algo inesperado: {linea}"))?,
            );
        } else if ["1 ", "2 ", "u ", "? "]
            .iter()
            .any(|p| linea.starts_with(*p))
        {
            estado.cookbook_cambiado = true;
        }
    }
    Ok(estado)
}
```

- [ ] **Step 4: Envoltorio en `src/main.rs`**

Renombrar la `correr` actual a `ejecutar_comando`, con firma `fn ejecutar_comando(comando: Option<Comando>, ruta: &Path, home: &Path) -> Result<bool>`. Su cuerpo pasa a empezar con `let Some(comando) = comando else { rebake::tui::ejecutar(ruta, home)?; return Ok(true); };`, ya no calcula `ruta`, y cada `&ruta` del cuerpo pasa a ser `ruta`. Arriba de ella va la `correr` nueva y, después, los helpers:

```rust
/// `Ok(false)` = terminó, pero algo no salió (p. ej. una receta falló).
fn correr(cli: Cli, home: &Path) -> Result<bool> {
    let ruta = cli
        .archivo
        .unwrap_or_else(|| rutas::archivo_por_defecto(home));
    // `clonar` crea el clon: no hay nada que traer antes ni que publicar después.
    let sincroniza =
        !matches!(cli.comando, Some(Comando::Clonar { .. })) && sincronizado(&ruta);
    let antes = if sincroniza {
        Some(archivo::leer(&ruta)?)
    } else {
        None
    };
    let nombre = cli.comando.as_ref().map(Comando::nombre);
    let resultado = ejecutar_comando(cli.comando, &ruta, home);
    // También si el comando falló: lo que llegó a guardar no se pierde.
    if let Some(antes) = &antes {
        publicar(&ruta, nombre, antes);
    }
    resultado
}

/// Si no se puede saber si el clon se sincroniza, el comando sigue sin tocar git.
fn sincronizado(ruta: &Path) -> bool {
    sincro::sincronizado(ruta).unwrap_or_else(|e| {
        avisar(&e.context("no pude ver si el cookbook está en un clon de rebake"));
        false
    })
}

fn publicar(ruta: &Path, comando: Option<&str>, antes: &rebake::modelo::Recetario) {
    let despues = match archivo::leer(ruta) {
        Ok(r) => r,
        Err(e) => {
            avisar(&e.context("no pude leer el cookbook para publicarlo"));
            return;
        }
    };
    match sincro::publicar(ruta, &servicio::mensaje_commit(comando, antes, &despues)) {
        Ok(true) => {
            tracing::info!(accion = "publicar", "cookbook publicado en el repo");
            eprintln!("cookbook publicado en el repo");
        }
        Ok(false) => {}
        Err(e) => avisar(&e),
    }
}

/// Un fallo de git no frena el comando: el cookbook ya quedó en disco.
fn avisar(e: &anyhow::Error) {
    tracing::warn!(error = %format!("{e:#}"), "sincronización con el repo");
    eprintln!("aviso: {e:#}");
}
```

Y el nombre de cada comando, para el mensaje del commit:

```rust
impl Comando {
    fn nombre(&self) -> &'static str {
        match self {
            Comando::Clonar { .. } => "clonar",
            Comando::Escanear => "escanear",
            Comando::Investigar { .. } => "investigar",
            Comando::Instalar { .. } => "instalar",
            Comando::RenombrarPerfil { .. } => "renombrar-perfil",
        }
    }
}
```

- [ ] **Step 5: Correr y ver que pasa**

Run: `cargo test --test sincro && cargo test`
Expected: 12 passed en `sincro`; la suite completa en verde.

- [ ] **Step 6: Lint y commit**

```bash
cargo clippy --all-targets -- -D warnings && cargo fmt --check
git add src/sincro.rs src/main.rs tests/sincro.rs
git commit -m "feat: commit and push the cookbook after each command in a rebake clone"
```

---

### Task 4: traer al arrancar (REQ-5, REQ-8)

**Files:**
- Modify: `src/sincro.rs` (agregar `traer`), `src/main.rs` (`correr`)
- Test: `tests/sincro.rs` (agregar)

**Interfaces:**
- Consumes: `partes`, `rama`, `git` (Tasks 2 y 3); `avisar` en `main` (Task 3).
- Produces: `pub fn sincro::traer(ruta: &Path) -> Result<()>`.

- [ ] **Step 1: Escribir los tests que fallan** al final de `tests/sincro.rs`:

```rust
#[test]
fn trae_lo_que_otro_equipo_subio() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    clonado(&h, &bare);
    let otro = h.ruta().join("otro");
    git(&h, h.ruta(), &["clone", "-q", url(&bare), url(&otro)]);
    let aprobado = COOKBOOK.replace("estado = \"pendiente\"", "estado = \"aprobada\"");
    fs::write(otro.join("cookbook.toml"), aprobado).unwrap();
    git(&h, &otro, &["commit", "-q", "-am", "otro equipo"]);
    git(&h, &otro, &["push", "-q", "origin", "HEAD"]);
    let o = rebake(&h, &["instalar", "--dry-run"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("plugin:a@b: simulado"),
        "{}",
        texto(&o)
    );
}

#[test]
fn sin_red_avisa_y_sigue_con_la_copia_local() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    clonado(&h, &bare);
    fs::rename(&bare, h.ruta().join("apagado.git")).unwrap();
    let o = rebake(&h, &["instalar", "--dry-run"]);
    assert!(o.status.success(), "{}", texto(&o));
    assert!(texto(&o).contains("sigo con la copia local"), "{}", texto(&o));
}
```

- [ ] **Step 2: Correr y ver que fallan**

Run: `cargo test --test sincro trae_lo_que_otro_equipo_subio sin_red_avisa_y_sigue_con_la_copia_local`
Expected: los 2 fallan. El primero dice "no hay recetas aprobadas" porque no trajo el cambio; el segundo no muestra aviso.

- [ ] **Step 3: Agregar a `src/sincro.rs`**

```rust
/// Trae lo último del repo con avance rápido. Sin rama remota todavía no hay nada que traer.
pub fn traer(ruta: &Path) -> Result<()> {
    let (dir, nombre) = partes(ruta)?;
    let estado = rama(dir, nombre)?;
    if !estado.con_commits || estado.adelante.is_none() {
        return Ok(());
    }
    git(dir, &["pull", "--ff-only", "--quiet"])
        .context("no pude traer lo último del repo; sigo con la copia local")?;
    Ok(())
}
```

- [ ] **Step 4: Llamarlo en `correr`** (`src/main.rs`), entre `let sincroniza = …;` y `let antes = …;`:

```rust
    if sincroniza && let Err(e) = sincro::traer(&ruta) {
        avisar(&e);
    }
```

- [ ] **Step 5: Correr y ver que pasa**

Run: `cargo test --test sincro && cargo test`
Expected: 14 passed en `sincro`; la suite completa en verde.

- [ ] **Step 6: Lint y commit**

```bash
cargo clippy --all-targets -- -D warnings && cargo fmt --check
git add src/sincro.rs src/main.rs tests/sincro.rs
git commit -m "feat: pull the cookbook repo before each command"
```

---

### Task 5: README, versión y nota en el spec raíz (REQ-11)

**Files:**
- Modify: `README.md`, `Cargo.toml` (+ `Cargo.lock`), `intent/spec.md`

- [ ] **Step 1: README.** En el bloque de `## Uso`, primera línea:

```sh
rebake clonar <url>      # una vez: trae tu cookbook de su repo git y lo sincroniza
```

Reemplazar desde "El cookbook vive en tu carpeta de Documentos" hasta "`rebake` escribe a través del enlace y lo conserva." por:

````markdown
El cookbook vive en tu carpeta de Documentos: `~/Documentos/rebake/cookbook.toml`
(`~/Documents/…` en un sistema en inglés), o donde indique `--archivo`. Tiene tus ajustes
personales, así que si lo versionás, que sea en un repo **privado**.

### Cookbook en un repo

Con el cookbook en la raíz de un repo git (puede estar vacío), pasale la URL una vez:

```sh
rebake clonar https://github.com/vos/mi-cookbook
```

Lo clona en `~/Documentos/rebake/` y desde ahí cada comando trae lo último antes de empezar
y, si el cookbook cambió, hace un commit solo de ese archivo y lo pushea. Usa tu `git` y tus
credenciales: para un repo privado en GitHub, `gh auth login` y `gh auth setup-git`, o una
clave ssh. Si algo de git falla, rebake avisa y sigue: el cookbook queda en disco y lo
pendiente se sube en la corrida siguiente. Los conflictos entre dos PCs los resolvés vos con
git en esa carpeta.

En una PC nueva:

```sh
curl -fsSL https://github.com/Ranteck/rebake/releases/latest/download/install.sh | sh
rebake clonar <url-de-tu-cookbook>
rebake instalar
```

Si preferís manejar git a mano, enlazá el archivo; rebake escribe a través del enlace, lo
conserva y no toca git:

```sh
mkdir -p ~/Documentos/rebake
ln -s ~/mi-repo-privado/cookbook.toml ~/Documentos/rebake/cookbook.toml
```
````

- [ ] **Step 2: Versión.** En `Cargo.toml`, `version = "0.3.0"`; después `cargo build` para actualizar `Cargo.lock`.

- [ ] **Step 3: Nota en `intent/spec.md`.** Agregar al final del bloque de notas del encabezado:

```markdown
> Desde v0.3.0 `rebake clonar <url>` clona el cookbook de su repo y lo sincroniza en cada
> comando (ver `cookbook-remoto/`).
```

- [ ] **Step 4: Verificación completa**

Run: `cargo test --locked && cargo clippy --all-targets -- -D warnings && cargo fmt --check`
Expected: todo en verde (la suite anterior más 3 de `servicio` y 14 de `sincro`).

- [ ] **Step 5: Commit**

```bash
git add README.md Cargo.toml Cargo.lock intent/spec.md
git commit -m "docs: rebake clonar in the README; bump to 0.3.0"
```

## Riesgos

- `main.rs` reordena `correr`: un `&ruta` mal convertido o un brazo que no lea el cookbook rompe comandos existentes. Lo cubren los tests de `tests/cli.rs`.
- Los tests dependen de que git respete `GIT_CONFIG_COUNT` (git ≥ 2.31); el runner de CI lo cumple.
- La TUI no tiene test de sincronización: usa el mismo `correr` que los comandos y git corre antes de abrir y después de cerrar la terminal.
- Publicar el release (tag `v0.3.0` y push) queda fuera de este plan y necesita tu OK aparte.

## Prueba

- `cargo test --locked`: los 14 tests de `tests/sincro.rs` y los 3 de `tests/servicio.rs` muestran REQ-1 a REQ-10 según la tabla; la suite existente muestra que nada se rompió.
- `cargo clippy --all-targets -- -D warnings` y `cargo fmt --check` limpios.
- Manual (REQ-11 y uso real), solo con tu OK porque pushea al repo real: `rebake clonar https://github.com/Ranteck/rebake-cookbok` en una carpeta de prueba con `--archivo`, un `rebake renombrar-perfil` de ida y vuelta, y ver los commits en GitHub.
