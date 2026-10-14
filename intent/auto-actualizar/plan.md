# Plan: rebake se actualiza solo (desde spec.md 2026-10-14 e intent.md 2026-10-14)
Estado: aceptado

## Archivos que cambian
- Nuevos: `src/actualizar.rs` y `tests/actualizar.rs`.
- Existentes (verificados): `src/lib.rs` (suma el módulo), `src/main.rs`, `README.md` (sección
  de instalación, línea 25: "correrlo actualiza") e `intent/spec.md` (nota "Desde v0.6.0").

## Diseño
- **No se reescribe la descarga.** El `install.sh` ya baja, verifica el checksum y reemplaza el
  binario sin dejarlo a medias (`mv` sobre un temporal), y ya tiene tests de checksum
  (`tests/instalar_sh.rs`). Actualizar es correr el `install.sh` de la versión nueva, que se
  publica junto con cada release, con `REBAKE_VERSION=vX.Y.Z` (exactamente la que se ofreció) y
  `REBAKE_INSTALL_DIR` en la carpeta del binario que está corriendo. Sin dependencias nuevas.
- **Versión publicada:** `curl` con tope de 2 segundos sobre `<base>/latest`, que redirige a
  `…/tag/vX.Y.Z`; se toma el número de la URL final. La base es `REBAKE_BASE_URL` (la misma
  variable del `install.sh`), por defecto el repo en GitHub. Si `curl` falta, falla, tarda o la
  URL no termina en una versión, el resultado es "no sé" y no se muestra nada (REQ-8).
- **Comparación:** las versiones se parten en tres números y se comparan como tuplas; lo que no
  se pueda leer cuenta como "no sé" (REQ-1, REQ-2).
- **Dónde corre:** en `main`, después de `Cli::parse`. `--version` y `--help` terminan dentro del
  parse, así que nunca consultan (REQ-9).
  - Sin subcomando (la pantalla): pregunta por stderr y lee una línea de stdin; solo "s" o "si"
    actualiza. Con éxito, reemplaza el proceso por el binario nuevo con los mismos argumentos
    (`exec`), así la pantalla abre con la versión nueva (REQ-3, REQ-4, REQ-6). Si el
    `install.sh` falla, muestra su salida, dice
    "no se actualizó" y sigue con la versión actual (REQ-5).
  - Con subcomando: una línea por stderr y sigue (REQ-7).

## Tests a escribir
Todos en `tests/actualizar.rs`. Copian el binario compilado a una carpeta temporal como
`rebake` (el test reemplaza esa copia, nunca el de `target/`) y usan un servidor HTTP local de
pocas líneas sobre `TcpListener`: `/latest` redirige a `/tag/vX`, y `/download/vX/` sirve el
`install.sh` del repo y un release falso armado como en `tests/instalar_sh.rs`, con un `rebake`
que imprime su versión.

| Test (comportamiento observable) | REQ |
| --- | --- |
| Comparación: 0.5.10 > 0.5.9, igual no es nueva, "v0.6.0" se lee, "latest" no | REQ-1, REQ-2 |
| Publicada igual a la propia: `rebake escanear` no imprime aviso | REQ-2 |
| Publicada 99.0.0: `rebake escanear` imprime la línea del aviso, no lee stdin y termina bien | REQ-7 |
| Sin subcomando, publicada 99.0.0, stdin "s": la copia queda reemplazada y la salida dice "rebake 99.0.0" (el `exec` corrió la nueva) | REQ-3, REQ-4 |
| Sin subcomando, stdin vacío: la pregunta aparece y la copia no cambió (byte a byte) | REQ-3, REQ-6 |
| Sin subcomando, checksum malo, stdin "s": dice "no se actualizó", la copia no cambió | REQ-5 |
| Carpeta de la copia sin permiso de escritura, stdin "s": dice "no se actualizó" y la ruta, la copia no cambió | REQ-5 |
| Puerto cerrado: sin aviso. Servidor que acepta y no responde: termina en menos de 3 s, sin aviso | REQ-8 |
| `rebake --version` con un servidor que anota pedidos: ningún pedido | REQ-9 |

Sin terminal la pantalla no abre; los tests miran solo la pregunta y el binario.

## Orden de trabajo
Rama `feat/auto-actualizar` desde `main`.
1. **T1 (ejecutor):** `src/actualizar.rs`, `lib.rs`, `main.rs` y `tests/actualizar.rs`, tests
   primero. REQ-1 a REQ-9.
2. **T2 (mecánico, después de T1):** README y nota en `intent/spec.md`.
3. **Verificador:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test --locked`.
4. Prueba real (abajo) y tu OK para publicar v0.6.0.

## Riesgos
- `install.sh` escribe siempre `<carpeta>/rebake`: si el binario se llama distinto (renombrado a
  mano), se instala al lado y el `exec` corre el viejo. Se detecta comparando el nombre y en
  ese caso solo avisa, sin actualizar.
- Si GitHub cambia la redirección de `/latest`, rebake deja de avisar, sin romperse.
- La primera release con esto (v0.6.0) no puede probar su propio salto: lo prueba la siguiente.

## Prueba
- Tests: 158 más unos 10.
- Real, en una carpeta temporal: instalar v0.5.3 con `REBAKE_VERSION=v0.5.3` y el binario
  nuevo compilado con versión 0.5.2 apuntando a GitHub: pregunta "Hay rebake 0.5.3 (tenés
  0.5.2)", con "s" se reemplaza y abre la pantalla en 0.5.3 (REQ-3, REQ-4). Sin red (`unshare`
  o base inválida): arranca sin aviso (REQ-8).
