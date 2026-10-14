# Plan: revisión en segundo plano (desde spec.md 2026-10-14, enmendado, e intent.md 2026-10-14)
Estado: aceptado

La versión anterior de este plan (pregunta antes de abrir) salió en v0.6.0 y queda en git.

## Archivos que cambian
Todos existen (verificado): `src/actualizar.rs`, `src/main.rs`, `src/tui/mod.rs`,
`src/tui/app.rs`, `src/tui/vista.rs`, `tests/actualizar.rs`, `tests/tui.rs`, `README.md` e
`intent/spec.md` (nota "Desde v0.6.0"). No se crean archivos.

## Diseño
- **Revisión:** `actualizar` suma una función que corre `version_nueva` en un hilo y devuelve
  el receptor de un canal. `main` la lanza después de `Cli::parse` (`--version` y `--help`
  siguen sin consultar, REQ-9) y no espera.
- **Subcomandos (REQ-7):** al terminar `correr`, `try_recv`; si ya llegó una versión nueva, la
  línea del aviso por stderr. Si no llegó, nada.
- **Pantalla (REQ-3):** `Arranque` suma el receptor, opcional para que los tests lo sigan
  armando con `Default`. El bucle de `tui/mod.rs` lo lee con `try_recv`, como los canales de
  investigar y comprobar, y guarda la versión en `App`. La cabecera la muestra fija.
- **`U` (REQ-4 a REQ-6):** `App::tecla` devuelve `Efecto::Actualizar(version)` solo con versión
  nueva y sin instalación en curso; si no, avisa "rebake está al día" o que espere la
  instalación. El bucle hace como `editar` con el editor: `ratatui::restore`, escribe
  "actualizando a X…", corre el `install.sh` y:
  - con éxito, `exec` del binario nuevo con los mismos argumentos: la pantalla vuelve en X;
  - con error, `ratatui::init` y el aviso dice "no se actualizó: <por qué>".
- **Se parte `actualizar::actualizar`:** `instalar(version, base, binario)` corre el
  `install.sh` sobre la ruta recibida (`current_exe` en uso real, una copia en los tests) y
  `reabrir(binario)` hace el `exec`. Así el reemplazo se prueba sin terminal.
- **Se borra** la pregunta `[s/N]` de `main` (`revisar_version`).
- El cookbook se guarda en disco en cada cambio; lo publica en git el proceso nuevo al cerrar,
  porque `sincro::publicar` mira el estado de git (verificado en `src/sincro.rs:168`).

## Tests a escribir
Los cuatro tests de `tests/actualizar.rs` que contestan `[s/N]` por stdin quedan viejos por la
enmienda: ya no hay pregunta. Su intención pasa a tests sobre `instalar` con la copia.

| Test (comportamiento observable) | REQ |
| --- | --- |
| Siguen: comparación, publicada igual sin aviso, puerto cerrado, `--version` sin pedidos | REQ-1, REQ-2, REQ-8, REQ-9 |
| `instalar` con publicada 99.0.0: la copia queda reemplazada y corre como "rebake 99.0.0" | REQ-4 |
| `instalar` con checksum malo: error que lo dice; la copia no cambió | REQ-5 |
| `instalar` en una carpeta sin permiso: error con la ruta; la copia no cambió | REQ-5 |
| `rebake investigar <id>` con un `claude` falso (`REBAKE_CLAUDE`) que duerme 1 s y publicada 99.0.0: sale la línea del aviso | REQ-7 |
| Servidor que acepta y no responde: `rebake escanear` termina en menos de 1 s, sin aviso | REQ-7, REQ-8 |
| `tests/tui.rs`: con versión nueva, la cabecera muestra el aviso y `U` devuelve `Actualizar` | REQ-3, REQ-4 |
| `tests/tui.rs`: `U` sin versión avisa "rebake está al día"; con instalación en curso no actualiza | REQ-6 |

## Orden de trabajo
1. **T1 (ejecutor):** código y tests, tests primero, en un worktree desde
   `feat/actualizar-en-segundo-plano`.
2. **T2 (yo):** README y nota del spec raíz.
3. **Verificación:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test --locked`, y `tests/actualizar.rs` cinco veces seguidas (usa hilos y red local).
4. Prueba real y tu OK para publicar v0.6.1.

## Riesgos
- Con `exec`, el proceso viejo no publica el cookbook al salir; lo hace el nuevo al cerrar. Si
  el nuevo falla antes, el commit queda para la próxima corrida, como hoy si git falla.
- La v0.6.0 instalada pregunta `[s/N]` al abrir: esa pregunta va a ofrecer la v0.6.1, y desde
  ahí rige la tecla.

## Prueba
- Antes de publicar, una copia marcada 0.5.0 contra la v0.6.0 real de GitHub: la pantalla abre
  enseguida, la cabecera muestra "rebake 0.6.0 disponible (tenés 0.5.0): U actualiza", `U` la
  reemplaza y vuelve la pantalla (la v0.6.0 pregunta `[s/N]`; se contesta "n").
- Arranque de un subcomando con GitHub: igual que sin red (~0,01 s).
