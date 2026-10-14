# Plan: elegir qué instalar desde la URL del cookbook (desde spec.md 2026-10-13 e intent.md 2026-10-13)
Estado: aceptado

> **Para workers:** cada tarea la implementa un `orquestar:ejecutor` distinto, con TDD: test
> primero, verlo fallar por la causa esperada, implementar, verlo pasar. Los pasos usan `- [ ]`.

**Objetivo:** `rebake <url>` abre la pantalla en Instalación, donde se ve qué está instalado, se
marca qué instalar y se ven las recetas que llegaron del repo.

**Arquitectura:** tres piezas sin pantalla (comprobar en `instalador`, novedades en `servicio`,
entrada por URL en `sincro` y `main`) y después la pestaña Instalación, que las consume. La
selección vive solo en el estado de la pantalla.

**Stack:** Rust 2024, clap 4 derive, ratatui 0.30, anyhow. Sin dependencias nuevas.

**Spec:** `intent/elegir-que-instalar/spec.md` (aceptado). Los workers leen el spec y su tarea.

## Restricciones globales

- Sin dependencias nuevas. Sin `unwrap` fuera de tests. Comentarios solo para el porqué.
- Nombres, mensajes y tests en español, como el resto del repo; commits en inglés.
- La selección nunca produce `Efecto::Guardar` ni toca el cookbook (REQ-6).
- Nada del cookbook se ejecuta sin una tecla o un comando explícito (REQ-8).
- No se cambia el comportamiento de `rebake clonar`, `escanear`, `investigar` ni
  `renombrar-perfil`, ni la sincronización (`traer`, `publicar`).
- Antes de cada commit: `cargo fmt`, `cargo clippy --all-targets -- -D warnings` y los tests del
  archivo tocado. No se hace push ni merge.

## Archivos que cambian

| Archivo | Existe | Cambio |
|---|---|---|
| `src/instalador.rs` | sí | `Presencia`, `comprobar`, `TOPE_COMPROBACION` |
| `src/servicio.rs` | sí | `Novedad`, `novedades` |
| `src/sincro.rs` | sí | `origen`, `misma_url` |
| `src/main.rs` | sí | argumento `url`, novedades, lista en `instalar` |
| `src/tui/mod.rs` | sí | `Arranque`, guarda de terminal, comprobar en segundo plano |
| `src/tui/app.rs` | sí | selección, presencia, teclas de la pestaña 3 |
| `src/tui/vista.rs` | sí | plan de instalación y pie por pestaña |
| `tests/instalador.rs`, `tests/servicio.rs`, `tests/sincro.rs`, `tests/tui.rs`, `tests/tui_arranque.rs` | sí | tests nuevos; `tui_arranque` cambia por la firma nueva |
| `README.md`, `intent/spec.md`, `Cargo.toml`, `Cargo.lock` | sí | uso, nota de versión, 0.4.0 |

No se crea ningún archivo de código.

## Tests a escribir

| Test (comportamiento observable) | Archivo | REQ |
|---|---|---|
| `la_url_sin_clon_clona_y_llega_a_la_pantalla` | `tests/sincro.rs` | REQ-1 |
| `la_url_del_mismo_clon_trae_lo_ultimo` | `tests/sincro.rs` | REQ-2 |
| `la_url_de_otro_clon_falla_sin_tocar_nada` | `tests/sincro.rs` | REQ-3 |
| `una_carpeta_que_no_es_clon_no_se_pisa` | `tests/sincro.rs` | REQ-3 |
| `url_y_subcomando_juntos_se_rechazan` | `tests/sincro.rs` | REQ-1 |
| `con_url_abre_en_instalacion_y_sin_url_en_recetas` | `tests/tui.rs` | REQ-1, REQ-2 |
| `instalacion_lista_las_aprobadas_marcadas` | `tests/tui.rs` | REQ-4 |
| `t_marca_todas_o_ninguna` | `tests/tui.rs` | REQ-4 |
| `p_confirma_cuantas_de_cuantas_e_instala_las_marcadas` | `tests/tui.rs` | REQ-5 |
| `marcar_no_cambia_el_cookbook` | `tests/tui.rs` | REQ-6 |
| `desmarcar_arrastra_a_las_que_la_requieren` | `tests/tui.rs` | REQ-7 |
| `abrir_instalacion_no_pide_comprobar` | `tests/tui.rs` | REQ-8 |
| `comprobar_dice_en_que_perfil_falta_y_no_instala` | `tests/instalador.rs` | REQ-8, REQ-9 |
| `comprobar_instalada_y_sin_comprobacion` | `tests/instalador.rs` | REQ-9 |
| `c_pide_comprobar_y_muestra_el_estado_de_cada_fila` | `tests/tui.rs` | REQ-9 |
| `instalada_pierde_la_casilla_y_no_se_instala` | `tests/tui.rs` | REQ-7, REQ-9 |
| `una_comprobacion_colgada_no_frena_a_las_demas` | `tests/instalador.rs` | REQ-10 |
| `el_detalle_muestra_los_comandos_y_la_comprobacion` | `tests/tui.rs` | REQ-11 |
| `sin_aprobadas_explica_que_hacer` | `tests/tui.rs` | REQ-4 |
| `novedades_marca_nuevas_y_cambiadas`, `sin_cambios_no_hay_novedades` | `tests/servicio.rs` | REQ-12 |
| `novedades_se_ven_en_la_lista` | `tests/tui.rs` | REQ-12 |
| `instalar_lista_lo_que_llego_del_repo`, `instalar_lista_lo_que_cambio`, `instalar_lista_sin_marcas_si_no_hubo_cambios` | `tests/sincro.rs` | REQ-12, REQ-13 |

Ningún test ejecuta la pantalla real: con la salida capturada, `tui::ejecutar` falla con "no pude
abrir la pantalla". Que `rebake <url>` abra de verdad en Instalación se comprueba a mano (Prueba).

## Orden de trabajo

Tareas 1, 2 y 3 son independientes y van en paralelo, cada una en su worktree. La 4 necesita los
tipos de la 1 y la 2. La 5 necesita la 3 y la 4. La 6 cierra.

---

### Tarea 1: comprobar sin instalar (REQ-8, REQ-9, REQ-10)

**Archivos:** `src/instalador.rs`, `tests/instalador.rs`.

**Produce:**

```rust
pub const TOPE_COMPROBACION: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq)]
pub enum Presencia {
    Instalada,
    /// Perfiles donde falta; vacío si la receta no va por perfil.
    Falta(Vec<String>),
    SinComprobacion,
    NoSePudo(String),
}

/// Corre `verificar` de cada id, por perfil, sin instalar. Llama a `emitir` una vez por id.
pub fn comprobar(
    r: &Recetario,
    ids: &[String],
    home: &Path,
    tope: Duration,
    emitir: &mut dyn FnMut(String, Presencia),
);
```

Reglas: sin `verificar` → `SinComprobacion`. Con perfiles: se corre `sh -c <verificar>` con
`CLAUDE_CONFIG_DIR` de cada perfil (los perfiles se resuelven igual que en `instalar_item`,
`src/instalador.rs:138`); código 0 es "está". Sin perfiles: se corre una vez sin esa variable y
da `Instalada` o `Falta(vec![])`. Si no se puede ejecutar o vence el tope → `NoSePudo(motivo)` y
se sigue con el id siguiente. Se usa `proceso::ejecutar`, como el resto del instalador. Si sirve,
se extrae de `instalar_item` el armado del `Command` (`sh -c`, con o sin `CLAUDE_CONFIG_DIR`) para
compartirlo; `instalar_item` no cambia de comportamiento.

- [ ] **Paso 1: tests que fallan.** Agregar al final de `tests/instalador.rs`:

```rust
fn presencias(r: &Recetario, home: &Path, tope: Duration) -> Vec<(String, Presencia)> {
    let ids = planificar(r).unwrap();
    let mut salida = Vec::new();
    comprobar(r, &ids, home, tope, &mut |id, p| salida.push((id, p)));
    salida
}

#[test]
fn comprobar_dice_en_que_perfil_falta_y_no_instala() {
    let dir = tempfile::tempdir().unwrap();
    let (p1, p2) = (dir.path().join("uno"), dir.path().join("dos"));
    fs::create_dir_all(&p1).unwrap();
    fs::create_dir_all(&p2).unwrap();
    fs::write(p1.join("listo"), "").unwrap();
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        Perfil { nombre: "uno".into(), dir: p1.display().to_string() },
        Perfil { nombre: "dos".into(), dir: p2.display().to_string() },
    ];
    let mut item = aprobado("x", &[], vec![auto("touch \"$CLAUDE_CONFIG_DIR/instalado\"", true)]);
    item.perfiles = vec!["uno".into(), "dos".into()];
    item.verificar = Some("test -f \"$CLAUDE_CONFIG_DIR/listo\"".into());
    r.items.push(item);

    let p = presencias(&r, dir.path(), Duration::from_secs(10));
    assert_eq!(p, vec![("x".to_string(), Presencia::Falta(vec!["dos".into()]))]);
    assert!(!p1.join("instalado").exists() && !p2.join("instalado").exists());
}

#[test]
fn comprobar_instalada_y_sin_comprobacion() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = Recetario::nuevo();
    let mut a = aprobado("a", &[], vec![auto("true", false)]);
    a.verificar = Some("true".into());
    let b = aprobado("b", &[], vec![auto("true", false)]);
    let mut c = aprobado("c", &[], vec![auto("true", false)]);
    c.verificar = Some("false".into());
    r.items = vec![a, b, c];

    let p = presencias(&r, dir.path(), Duration::from_secs(10));
    assert_eq!(
        p,
        vec![
            ("a".to_string(), Presencia::Instalada),
            ("b".to_string(), Presencia::SinComprobacion),
            ("c".to_string(), Presencia::Falta(vec![])),
        ]
    );
}

#[test]
fn una_comprobacion_colgada_no_frena_a_las_demas() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = Recetario::nuevo();
    let mut a = aprobado("a", &[], vec![auto("true", false)]);
    a.verificar = Some("sleep 30".into());
    let mut b = aprobado("b", &[], vec![auto("true", false)]);
    b.verificar = Some("true".into());
    r.items = vec![a, b];

    let p = presencias(&r, dir.path(), Duration::from_secs(1));
    assert!(matches!(p[0].1, Presencia::NoSePudo(_)), "{:?}", p[0]);
    assert_eq!(p[1], ("b".to_string(), Presencia::Instalada));
}
```

- [ ] **Paso 2:** `cargo test --locked --test instalador comprob` → no compila: falta `comprobar`.
- [ ] **Paso 3:** implementar `Presencia`, `TOPE_COMPROBACION` y `comprobar` con las reglas de arriba.
- [ ] **Paso 4:** `cargo test --locked --test instalador` → 8 tests pasan (5 de antes más 3).
- [ ] **Paso 5:** commit `feat: check which recipes are installed without installing`.

---

### Tarea 2: novedades entre dos cookbooks (REQ-12)

**Archivos:** `src/servicio.rs`, `tests/servicio.rs`.

**Produce:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Novedad {
    Nueva,
    Cambio,
}

/// Recetas aprobadas de `despues` que no estaban aprobadas en `antes`, o que cambiaron.
pub fn novedades(antes: &Recetario, despues: &Recetario) -> HashMap<String, Novedad>;
```

Reglas, solo para ítems de `despues` en `Estado::Aprobada`: si `antes` no tiene ese id, o lo tenía
en otro estado → `Nueva`. Si estaba aprobado y cambió `pasos` o `verificar` → `Cambio`. Lo demás
no aparece.

- [ ] **Paso 1: tests que fallan.** Agregar a `tests/servicio.rs` (ajustar los `use` del archivo):

```rust
fn aprobada(id: &str, cmd: &str) -> Item {
    let mut i = Item::nuevo(id, Tipo::Plugin);
    i.estado = Estado::Aprobada;
    i.pasos = vec![Paso {
        cmd: cmd.into(),
        modo: Modo::Auto,
        por_perfil: false,
        cita: None,
        nota: None,
    }];
    i
}

#[test]
fn novedades_marca_nuevas_y_cambiadas() {
    let mut antes = Recetario::nuevo();
    antes.items = vec![
        aprobada("igual", "uno"),
        aprobada("otro-comando", "uno"),
        Item::nuevo("recien-aprobada", Tipo::Plugin),
        aprobada("otra-comprobacion", "uno"),
    ];
    let mut despues = Recetario::nuevo();
    let mut comprobacion = aprobada("otra-comprobacion", "uno");
    comprobacion.verificar = Some("true".into());
    despues.items = vec![
        aprobada("igual", "uno"),
        aprobada("otro-comando", "dos"),
        aprobada("recien-aprobada", "uno"),
        comprobacion,
        aprobada("llego", "uno"),
        Item::nuevo("llego-pendiente", Tipo::Plugin),
    ];

    let n = servicio::novedades(&antes, &despues);
    assert_eq!(n.get("otro-comando"), Some(&Novedad::Cambio));
    assert_eq!(n.get("otra-comprobacion"), Some(&Novedad::Cambio));
    assert_eq!(n.get("recien-aprobada"), Some(&Novedad::Nueva));
    assert_eq!(n.get("llego"), Some(&Novedad::Nueva));
    assert_eq!(n.len(), 4, "{n:?}");
}

#[test]
fn sin_cambios_no_hay_novedades() {
    let mut r = Recetario::nuevo();
    r.items = vec![aprobada("a", "uno")];
    assert!(servicio::novedades(&r, &r.clone()).is_empty());
}
```

- [ ] **Paso 2:** `cargo test --locked --test servicio novedades` → no compila: falta `novedades`.
- [ ] **Paso 3:** implementar `Novedad` y `novedades`.
- [ ] **Paso 4:** `cargo test --locked --test servicio` → 5 tests pasan (3 de antes más 2).
- [ ] **Paso 5:** commit `feat: tell which approved recipes are new or changed`.

---

### Tarea 3: entrar por la URL (REQ-1, REQ-2, REQ-3)

**Archivos:** `src/sincro.rs`, `src/main.rs`, `src/tui/mod.rs`, `tests/sincro.rs`.

**Produce:**

```rust
// src/sincro.rs
/// URL de `origin` del clon donde está el cookbook.
pub fn origen(ruta: &Path) -> Result<String>;
/// Misma URL salvo credenciales, barra final y `.git`.
pub fn misma_url(a: &str, b: &str) -> bool;
```

En `src/main.rs`, `Cli` suma `url: Option<String>` como argumento posicional (doc: "URL del repo
de tu cookbook: lo clona la primera vez y abre la pantalla en Instalación"). Los nombres de
subcomando siguen ganando: `rebake escanear` no es una URL.

Comportamiento de `correr` cuando hay `url`:

1. Con `url` y subcomando a la vez: error "pasá la URL o un comando, no los dos".
2. Si el cookbook ya está en un clon sincronizado: si `misma_url(origen, url)` sigue el flujo
   normal (traer, pantalla, publicar). Si no: error `ya hay un clon de <origen sin credenciales>
   en <carpeta>: movelo o usá --archivo`. No se toca nada.
3. Si no hay clon: `sincro::clonar(url, ruta)`, que ya falla si la carpeta no está vacía; después
   sigue el flujo normal sobre el clon recién hecho.

En `src/tui/mod.rs`, `ejecutar` comprueba después de leer el cookbook y antes de `ratatui::init()`
que la salida sea una terminal (`std::io::IsTerminal` sobre stdout). Si no lo es:
`bail!("no pude abrir la pantalla: la salida no es una terminal")`. Hoy `ratatui::init()` entra en
pánico o, peor, toma `/dev/tty` aunque la salida esté capturada. El orden importa: el test
`no_arranca_con_recetario_invalido` espera el error del cookbook primero.

La pestaña inicial la agrega la tarea 4; acá la pantalla abre como siempre.

- [ ] **Paso 1: tests que fallan.** Agregar al final de `tests/sincro.rs`:

```rust
const SIN_PANTALLA: &str = "no pude abrir la pantalla";

fn origen_del_clon(h: &HomeFalso) -> String {
    git(h, &carpeta(h), &["config", "--get", "remote.origin.url"])
}

#[test]
fn la_url_sin_clon_clona_y_llega_a_la_pantalla() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    let o = rebake(&h, &[url(&bare)]);
    assert!(texto(&o).contains(SIN_PANTALLA), "{}", texto(&o));
    assert_eq!(
        git(&h, &carpeta(&h), &["config", "--local", "--get", "rebake.sincronizar"]),
        "true"
    );
}

#[test]
fn la_url_del_mismo_clon_trae_lo_ultimo() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    clonado(&h, &bare);
    let otro = h.ruta().join("otro");
    git(&h, h.ruta(), &["clone", "-q", url(&bare), url(&otro)]);
    let aprobado = COOKBOOK.replace("estado = \"pendiente\"", "estado = \"aprobada\"");
    fs::write(otro.join("cookbook.toml"), aprobado).unwrap();
    git(&h, &otro, &["commit", "-q", "-am", "otro equipo"]);
    git(&h, &otro, &["push", "-q", "origin", "HEAD"]);

    let o = rebake(&h, &[url(&bare)]);
    assert!(texto(&o).contains(SIN_PANTALLA), "{}", texto(&o));
    let local = fs::read_to_string(carpeta(&h).join("cookbook.toml")).unwrap();
    assert!(local.contains("aprobada"), "{local}");
}

#[test]
fn la_url_de_otro_clon_falla_sin_tocar_nada() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    clonado(&h, &bare);
    let ajeno = h.ruta().join("ajeno.git");
    git(&h, h.ruta(), &["init", "-q", "--bare", url(&ajeno)]);

    let o = rebake(&h, &[url(&ajeno)]);
    assert!(!o.status.success(), "{}", texto(&o));
    assert!(texto(&o).contains("remoto.git"), "{}", texto(&o));
    assert!(!texto(&o).contains(SIN_PANTALLA), "{}", texto(&o));
    assert_eq!(origen_del_clon(&h), url(&bare));
}

#[test]
fn una_carpeta_que_no_es_clon_no_se_pisa() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    let suelto = h.escribir("Documents/rebake/cookbook.toml", "version = 1\n");
    let o = rebake(&h, &[url(&bare)]);
    assert!(!o.status.success(), "{}", texto(&o));
    assert!(texto(&o).contains("no está vacía"), "{}", texto(&o));
    assert_eq!(fs::read_to_string(&suelto).unwrap(), "version = 1\n");
}

#[test]
fn url_y_subcomando_juntos_se_rechazan() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    let o = rebake(&h, &[url(&bare), "escanear"]);
    assert!(!o.status.success(), "{}", texto(&o));
    assert!(!carpeta(&h).exists());
}
```

- [ ] **Paso 2:** `cargo test --locked --test sincro la_url` → fallan: clap rechaza el argumento.
- [ ] **Paso 3:** implementar `origen`, `misma_url`, el argumento `url`, la rama en `correr` y la
  guarda de terminal en `tui::ejecutar`.
- [ ] **Paso 4:** `cargo test --locked --test sincro` → 21 tests pasan (16 de antes más 5), y
  `cargo test --locked --test tui_arranque --test cli` siguen pasando.
- [ ] **Paso 5:** commit `feat: open the cookbook from its URL with rebake <url>`.

---

### Tarea 4: la pestaña Instalación (REQ-4 a REQ-9, REQ-11, REQ-12)

**Archivos:** `src/tui/app.rs`, `src/tui/vista.rs`, `src/tui/mod.rs`, `tests/tui.rs`,
`tests/tui_arranque.rs`.

**Consume:** `instalador::{Presencia, comprobar, TOPE_COMPROBACION, planificar}` (tarea 1),
`servicio::Novedad` (tarea 2).

**Produce:**

```rust
// src/tui/mod.rs
#[derive(Default)]
pub struct Arranque {
    pub instalacion: bool,                       // abrir en la pestaña 3
    pub novedades: HashMap<String, Novedad>,
}
pub fn ejecutar(ruta: &Path, home: &Path, arranque: Arranque) -> Result<()>;

// src/tui/app.rs
pub enum Efecto { /* los de hoy */ Comprobar }
impl App {
    pub fn nueva(recetario: Recetario) -> Self;          // no cambia
    pub fn con_arranque(self, instalacion: bool, novedades: HashMap<String, Novedad>) -> Self;
    /// Aprobadas en el orden en que se instalan; vacío si `planificar` falla.
    pub fn plan(&self) -> Vec<String>;
    /// Lo que `P` instala: del plan, lo marcado que no está comprobado como instalado.
    pub fn a_instalar(&self) -> Vec<String>;
    pub fn registrar_presencia(&mut self, id: &str, p: Presencia);
    pub fn terminar_comprobacion(&mut self);
}
// campos públicos nuevos de App
pub desmarcadas: HashSet<String>,
pub presencia: HashMap<String, Presencia>,
pub novedades: HashMap<String, Novedad>,
pub seleccion_plan: usize,
pub comprobando: bool,
```

Decisiones:

- Se guardan las **desmarcadas**, no las marcadas: una receta que se aprueba después aparece
  marcada sin tener que refrescar nada. `plan()` se calcula al vuelo con `planificar`.
- Teclas en la pestaña 3, cuando no hay instalación en curso ni resultados a la vista: flechas o
  `j`/`k` mueven `seleccion_plan`; espacio marca o desmarca; `t` desmarca todas si hay alguna
  marcada y si no las marca todas; `c` devuelve `Efecto::Comprobar` y pone `comprobando`.
- Desmarcar una receta desmarca todas las del plan que la requieren, directa o indirectamente.
  Marcar una receta marca todo lo que requiere. Una fila `Instalada` no tiene casilla y espacio
  no hace nada, así que lo que depende de ella no se arrastra.
- `P` (desde cualquier pestaña) pregunta `¿Instalar N de M recetas? s/n`, con N = `a_instalar()`
  y M = `plan()`. Con N = 0 avisa "no hay nada marcado para instalar" y no pregunta.
- `mod.rs::instalar` usa `app.a_instalar()` como orden, en lugar de `planificar`.
- Después de instalar la pestaña muestra los resultados, como hoy. `Esc` vuelve al plan.
- `Efecto::Comprobar`: `mod.rs` lanza un hilo que corre `instalador::comprobar` sobre
  `app.plan()` con `TOPE_COMPROBACION` y manda cada `(id, Presencia)` por un canal, igual que la
  instalación (`mod.rs:144`). Al cerrarse el canal llama `terminar_comprobacion`.
- Texto de cada fila: sin dato `sin comprobar` (o `comprobando…` si `comprobando`); `Instalada`
  → `instalada`; `Falta` con todos los perfiles de la receta, o sin perfiles → `falta`; `Falta`
  con algunos → `falta en a, b`; `SinComprobacion` → `no se puede saber`; `NoSePudo` → `no se
  pudo comprobar`. Casilla: `[x]`, `[ ]` o ` ✓ `. Novedad al final: `nueva` o `cambió`.
- Panel derecho: `lineas_detalle` de la receta resaltada (`vista.rs:160`), que ya muestra
  Requiere, Verificar y Pasos. No se escribe un detalle nuevo.
- Plan vacío: `No hay recetas aprobadas: investigá (i) y aprobá (a) en la pestaña 1.` Si
  `planificar` falla (ciclo), se muestra su error en ese lugar.
- Pie: la lista de teclas depende de la pestaña. En la 3:
  `↑↓ mover  espacio marcar  t todas/ninguna  c comprobar  P instalar  q salir`.
- `tests/tui_arranque.rs` cambia solo la llamada: `ejecutar(&ruta, dir.path(), Default::default())`.
  Es por la firma nueva; la aserción no cambia.

- [ ] **Paso 1: tests que fallan.** Agregar a `tests/tui.rs` (sumar los `use` de
  `rebake::instalador::Presencia`, `rebake::servicio::Novedad` y `rebake::tui::app::Pestana`):

```rust
const MKT: &str = "marketplace:openai-codex";
const CODEX: &str = "plugin:codex@openai-codex";

/// `recetario()` más un marketplace aprobado del que depende codex. Plan: MKT, CODEX, claude
/// (codex está antes que claude en el recetario y arrastra primero a su requisito).
fn con_plan() -> App {
    let mut r = recetario();
    let mut mkt = Item::nuevo(MKT, Tipo::Marketplace);
    mkt.estado = Estado::Aprobada;
    r.items.push(mkt);
    let codex = r.items.iter_mut().find(|i| i.id == CODEX).unwrap();
    codex.requiere = vec![MKT.into()];
    codex.verificar = Some("claude plugin list | grep codex".into());
    let mut app = App::nueva(r);
    app.pestana = Pestana::Instalacion;
    app
}

fn resaltar(app: &mut App, id: &str) {
    app.seleccion_plan = app.plan().iter().position(|x| x == id).unwrap();
}

#[test]
fn con_url_abre_en_instalacion_y_sin_url_en_recetas() {
    assert_eq!(App::nueva(recetario()).pestana, Pestana::Recetas);
    let app = App::nueva(recetario()).con_arranque(true, Default::default());
    assert_eq!(app.pestana, Pestana::Instalacion);
}

#[test]
fn instalacion_lista_las_aprobadas_marcadas() {
    let app = con_plan();
    assert_eq!(app.plan(), [MKT, CODEX, "claude"]);
    assert_eq!(app.a_instalar(), app.plan());
    let p = pantalla(&app);
    assert!(p.contains(&format!("[x] {MKT}")), "{p}");
    assert!(p.contains(&format!("[x] {CODEX}")), "{p}");
    assert!(p.contains("sin comprobar"), "{p}");
    assert!(!p.contains("crawl4ai"), "{p}");
}

#[test]
fn t_marca_todas_o_ninguna() {
    let mut app = con_plan();
    assert_eq!(tecla(&mut app, KeyCode::Char('t')), Efecto::Nada);
    assert!(app.a_instalar().is_empty());
    tecla(&mut app, KeyCode::Char('t'));
    assert_eq!(app.a_instalar().len(), 3);
}

#[test]
fn desmarcar_arrastra_a_las_que_la_requieren() {
    let mut app = con_plan();
    resaltar(&mut app, MKT);
    tecla(&mut app, KeyCode::Char(' '));
    assert_eq!(app.a_instalar(), ["claude"]);
    assert!(pantalla(&app).contains(&format!("[ ] {CODEX}")));

    resaltar(&mut app, CODEX);
    tecla(&mut app, KeyCode::Char(' '));
    assert_eq!(app.a_instalar(), [MKT, CODEX, "claude"]);
}

#[test]
fn marcar_no_cambia_el_cookbook() {
    let mut app = con_plan();
    let antes = app.recetario.clone();
    resaltar(&mut app, MKT);
    assert_eq!(tecla(&mut app, KeyCode::Char(' ')), Efecto::Nada);
    assert_eq!(tecla(&mut app, KeyCode::Char('t')), Efecto::Nada);
    assert_eq!(app.recetario, antes);
}

#[test]
fn p_confirma_cuantas_de_cuantas_e_instala_las_marcadas() {
    let mut app = con_plan();
    resaltar(&mut app, "claude");
    tecla(&mut app, KeyCode::Char(' '));
    assert_eq!(tecla(&mut app, KeyCode::Char('P')), Efecto::Nada);
    assert!(app.mensaje.as_deref().unwrap().contains("2 de 3"), "{:?}", app.mensaje);
    assert_eq!(tecla(&mut app, KeyCode::Char('s')), Efecto::Instalar);
    assert_eq!(app.a_instalar(), [MKT, CODEX]);
}

#[test]
fn abrir_instalacion_no_pide_comprobar() {
    let mut app = App::nueva(recetario());
    assert_eq!(tecla(&mut app, KeyCode::Char('3')), Efecto::Nada);
    assert!(app.presencia.is_empty() && !app.comprobando);
}

#[test]
fn c_pide_comprobar_y_muestra_el_estado_de_cada_fila() {
    let mut app = con_plan();
    app.recetario.perfiles.push(Perfil { nombre: "personal".into(), dir: "~/.claude-personal".into() });
    app.recetario.item_mut(CODEX).unwrap().perfiles = vec!["laburo".into(), "personal".into()];
    assert_eq!(tecla(&mut app, KeyCode::Char('c')), Efecto::Comprobar);
    assert!(app.comprobando);
    assert!(pantalla(&app).contains("comprobando"));

    app.registrar_presencia(CODEX, Presencia::Falta(vec!["personal".into()]));
    app.registrar_presencia(MKT, Presencia::SinComprobacion);
    app.registrar_presencia("claude", Presencia::NoSePudo("tope".into()));
    app.terminar_comprobacion();
    let p = pantalla(&app);
    assert!(p.contains("falta en personal"), "{p}");
    assert!(p.contains("no se puede saber"), "{p}");
    assert!(p.contains("no se pudo comprobar"), "{p}");
    assert!(!app.comprobando);
    assert_eq!(app.a_instalar().len(), 3);
}

#[test]
fn instalada_pierde_la_casilla_y_no_se_instala() {
    let mut app = con_plan();
    app.registrar_presencia(MKT, Presencia::Instalada);
    let p = pantalla(&app);
    assert!(p.contains(&format!("✓  {MKT}")) || p.contains(&format!("✓ {MKT}")), "{p}");
    assert!(p.contains("instalada"), "{p}");
    assert_eq!(app.a_instalar(), [CODEX, "claude"]);

    resaltar(&mut app, MKT);
    tecla(&mut app, KeyCode::Char(' '));
    assert_eq!(app.a_instalar(), [CODEX, "claude"]);
}

#[test]
fn el_detalle_muestra_los_comandos_y_la_comprobacion() {
    let mut app = con_plan();
    resaltar(&mut app, CODEX);
    let p = pantalla(&app);
    assert!(p.contains("claude plugin install codex@openai-codex"), "{p}");
    assert!(p.contains("claude plugin list | grep codex"), "{p}");
    assert!(p.contains(MKT), "{p}");
}

#[test]
fn novedades_se_ven_en_la_lista() {
    let mut app = con_plan();
    app.novedades.insert(CODEX.into(), Novedad::Cambio);
    app.novedades.insert(MKT.into(), Novedad::Nueva);
    let p = pantalla(&app);
    assert!(p.contains("cambió") && p.contains("nueva"), "{p}");
}

#[test]
fn sin_aprobadas_explica_que_hacer() {
    let mut r = Recetario::nuevo();
    r.items.push(Item::nuevo("plugin:a@b", Tipo::Plugin));
    let mut app = App::nueva(r);
    app.pestana = Pestana::Instalacion;
    assert!(pantalla(&app).contains("No hay recetas aprobadas"));
    assert_eq!(tecla(&mut app, KeyCode::Char('P')), Efecto::Nada);
}
```

  Si `Recetario` no tiene `item_mut` público o `Pestana` no deriva `Debug`/`PartialEq`, el worker
  lo reporta antes de cambiar el test (`modelo.rs:169` y `app.rs:11` dicen que sí).

- [ ] **Paso 2:** `cargo test --locked --test tui` → no compila: faltan `plan`, `a_instalar`,
  `Efecto::Comprobar`.
- [ ] **Paso 3:** implementar en `app.rs` el estado, las teclas y los métodos; en `vista.rs` el
  plan, el detalle y el pie por pestaña; en `mod.rs` `Arranque`, la firma de `ejecutar`, el hilo
  de comprobación y `instalar` con `a_instalar()`. Actualizar la llamada en `src/main.rs` con
  `Default::default()` para que compile (la tarea 5 le pasa los datos reales) y la de
  `tests/tui_arranque.rs`.
- [ ] **Paso 4:** `cargo test --locked --test tui --test tui_arranque` → 20 tests pasan (7 + 1 de
  antes más 12), y la suite entera sigue pasando.
- [ ] **Paso 5:** commit `feat: choose what to install and see what is installed in the TUI`.

---

### Tarea 5: novedades del repo en la pantalla y en `instalar` (REQ-1, REQ-2, REQ-12, REQ-13)

**Archivos:** `src/main.rs`, `tests/sincro.rs`.

**Consume:** `servicio::novedades` (tarea 2), `tui::Arranque` (tarea 4), la rama de URL (tarea 3).

Cambios en `correr` (`src/main.rs`):

- Con el clon sincronizado y **sin** haberlo clonado en esta corrida: leer el cookbook antes de
  `traer` (`previo`) y, después de traer, calcular `servicio::novedades(&previo, &antes)`. En un
  clonado de esta corrida, novedades vacías (REQ-12).
- La pantalla recibe `Arranque { instalacion: <hubo url>, novedades }`.
- `instalar` recibe las novedades y, antes de la confirmación y también con `--si` y con
  `--dry-run`, escribe en stderr `Va a instalar:` y una línea por receta del orden:
  `  <id>`, `  <id>  (nueva)` o `  <id>  (cambió)`.

- [ ] **Paso 1: tests que fallan.** Agregar al final de `tests/sincro.rs`:

```rust
const PASO: &str = "\n[[item.paso]]\ncmd = \"true\"\nmodo = \"auto\"\n";

fn aprobado_con(paso: &str) -> String {
    COOKBOOK.replace("estado = \"pendiente\"", "estado = \"aprobada\"") + paso
}

/// Otro equipo reemplaza el cookbook del repo.
fn subir(h: &HomeFalso, bare: &Path, contenido: &str) {
    let otro = h.ruta().join("otro");
    git(h, h.ruta(), &["clone", "-q", url(bare), url(&otro)]);
    fs::write(otro.join("cookbook.toml"), contenido).unwrap();
    git(h, &otro, &["commit", "-q", "-am", "otro equipo"]);
    git(h, &otro, &["push", "-q", "origin", "HEAD"]);
}

#[test]
fn instalar_lista_lo_que_llego_del_repo() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(COOKBOOK));
    clonado(&h, &bare);
    subir(&h, &bare, &aprobado_con(PASO));
    let o = rebake(&h, &["instalar", "--si"]);
    assert!(o.status.success(), "{}", texto(&o));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("plugin:a@b  (nueva)"), "{err}");
}

#[test]
fn instalar_lista_lo_que_cambio() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(&aprobado_con(PASO)));
    clonado(&h, &bare);
    subir(&h, &bare, &aprobado_con(&PASO.replace("\"true\"", "\"true && true\"")));
    let o = rebake(&h, &["instalar", "--si"]);
    assert!(o.status.success(), "{}", texto(&o));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("plugin:a@b  (cambió)"), "{err}");
}

#[test]
fn instalar_lista_sin_marcas_si_no_hubo_cambios() {
    let h = HomeFalso::nuevo();
    let bare = remoto(&h, Some(&aprobado_con(PASO)));
    clonado(&h, &bare);
    let o = rebake(&h, &["instalar", "--si"]);
    assert!(o.status.success(), "{}", texto(&o));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("  plugin:a@b"), "{err}");
    assert!(!err.contains("(nueva)") && !err.contains("(cambió)"), "{err}");
}
```

- [ ] **Paso 2:** `cargo test --locked --test sincro instalar_lista` → fallan: no se lista nada.
- [ ] **Paso 3:** implementar los tres cambios de `correr` e `instalar`.
- [ ] **Paso 4:** `cargo test --locked` → pasa toda la suite.
- [ ] **Paso 5:** commit `feat: show what arrived from the repo before installing`.

---

### Tarea 6: documentos y versión (la hace la sesión principal)

- [ ] `README.md`: en Uso, `rebake <url>`; en "Cookbook en un repo", el flujo de PC nueva pasa a
  `rebake <url>`; en "Teclas de la TUI", las de la pestaña Instalación; en Seguridad, que `c`
  ejecuta las comprobaciones del cookbook y que abrir no ejecuta nada.
- [ ] `intent/spec.md`: una línea en la nota de versiones ("Desde v0.4.0 …, ver
  `elegir-que-instalar/`").
- [ ] `Cargo.toml` y `Cargo.lock`: 0.4.0.
- [ ] Commit `docs: rebake <url> and the install tab in the README; bump to 0.4.0`.

## Riesgos

- **clap y el argumento posicional.** Un `url` posicional junto a subcomandos puede cambiar cómo
  se interpreta `rebake --archivo x escanear`. Lo cubren los tests existentes de `cli.rs` y
  `sincro.rs`; si alguno falla, la tarea 3 frena y reporta en lugar de tocar esos tests.
- **Tres tareas en paralelo sobre el mismo repo.** Van en worktrees separados y la sesión
  principal las integra de a una; la suite se corre sobre el resultado integrado.
- **La pantalla real no tiene test.** Se cubre la lógica (teclas, estado, texto dibujado con
  `TestBackend`), no el terminal de verdad.
- **`P` cambia de alcance.** Hoy instala todas las aprobadas; pasa a instalar las marcadas, que
  al abrir son todas. Sin tocar nada, el resultado es el de hoy.

## Prueba

1. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked`: se
   esperan 141 tests (116 de hoy más 25), corridos por un `verificador` sobre la rama integrada.
2. A mano, con un HOME falso y un repo bare local, dentro de un pseudo-terminal:
   `rebake <url>` abre en Instalación (REQ-1); `c` cambia las filas (REQ-9); desmarcar una y `P`
   instala el resto (REQ-5); el cookbook del clon queda sin cambios en `git status` (REQ-6).
3. Con el clon real de Denis: `rebake` abre en Recetas y la pestaña 3 dice que no hay recetas
   aprobadas (las 73 están pendientes).
4. Revisión de seguridad del diff completo antes de proponer el merge.
