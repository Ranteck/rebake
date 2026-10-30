# rebake

Vuelve a dejar tu setup de Claude Code igual en una PC recién formateada: plugins, skills,
hooks, statusline e imports de `CLAUDE.md`, en todos tus perfiles.

En vez de copiar carpetas, que quedan viejas o rotas, rebake mira qué tenés instalado y averigua
cómo se instala cada cosa según su propio repo. Cada forma de instalar es una **receta**, y las
recetas se guardan en tu **cookbook**: un archivo TOML que revisás, aprobás y versionás. En la
PC nueva, rebake ejecuta las recetas aprobadas y te deja una lista de lo que queda a mano.

```text
esta PC:   escanear  ›  armar recetas  ›  aprobar        (cookbook.toml, en tu repo privado)
PC nueva:  rebake <url-del-cookbook>  ›  instalar  ›  Pendientes (lo que queda a mano)
```

No copia memorias, historial ni credenciales, y no instala paquetes del sistema.

## Contenido

- [Requisitos](#requisitos)
- [Instalar](#instalar)
- [Armar el cookbook en tu PC](#armar-el-cookbook-en-tu-pc)
- [Guardar el cookbook en un repo](#guardar-el-cookbook-en-un-repo)
- [Instalar todo en una PC nueva](#instalar-todo-en-una-pc-nueva)
- [Comandos](#comandos)
- [La pantalla](#la-pantalla)
- [Cómo arma cada receta](#cómo-arma-cada-receta)
- [Lo que queda a mano](#lo-que-queda-a-mano)
- [Configuración](#configuración)
- [Archivos](#archivos)
- [Seguridad](#seguridad)
- [Problemas comunes](#problemas-comunes)
- [Desarrollo](#desarrollo)

## Requisitos

- Linux x86_64.
- Claude Code instalado y logueado: rebake lo usa para investigar recetas e instalar plugins.
- `git` y `curl`.
- Para las skills de skills.sh, `npx` (Node.js). Si falta, esa receta lo avisa al instalar.

rebake trabaja con todos tus perfiles de Claude Code: `~/.claude` y cada `~/.claude-<nombre>`
que tenga un `settings.json`. Por ejemplo, `~/.claude-personal` queda como el perfil
`personal`.

## Instalar

```bash
curl -fsSL https://github.com/Ranteck/rebake/releases/latest/download/install.sh | sh
```

Baja el binario del último release, verifica su checksum y lo deja en `~/.local/bin/rebake`.
Al terminar muestra la versión instalada (`rebake 0.8.4`, por ejemplo) y, si esa carpeta no
está en tu `PATH`, te avisa que la agregues. Volver a correrlo actualiza.

Para instalar en otra carpeta o una versión fija, usá las variables de
[Configuración](#configuración):

```bash
curl -fsSL https://github.com/Ranteck/rebake/releases/latest/download/install.sh | REBAKE_VERSION=v0.8.4 sh
```

Con Rust 1.88 o más nuevo también se puede compilar:

```bash
cargo install --git https://github.com/Ranteck/rebake
```

Después rebake se actualiza solo. Cuando hay una versión más nueva, la cabecera de la pantalla
dice `⬆ rebake X disponible` y `u` la instala, con el mismo `install.sh` y su checksum. La
revisión corre por debajo y no demora el arranque; sin red, no pasa nada.

## Armar el cookbook en tu PC

1. Abrí la pantalla:

   ```bash
   rebake
   ```

   Arranca en la pestaña **Recetas**. Con un cookbook vacío, el pie dice
   `` Siguiente: `s` escanea esta PC ``.

2. Apretá `s` para escanear. Aparece cada cosa instalada en tus perfiles, agrupada por tipo.
   El cookbook se crea en `~/Documentos/rebake/cookbook.toml` (`~/Documents/…` en un sistema
   en inglés).

   Las skills y los plugins sincronizados por Claude desde tu cuenta no aparecen ni se
   investigan o instalan: Claude se ocupa de ellos. Rebake reconoce su carpeta real de
   sincronización y lo recuerda en el cookbook, incluso en otra PC sin esos archivos.
   Una skill propia con el mismo nombre sigue disponible en sus perfiles propios.

3. Apretá `t` para armar todas las recetas. Las de marketplaces, plugins y skills de skills.sh
   salen al toque, sin Claude. El resto va por tandas de hasta 5 corridas de Claude: el pie
   pregunta antes de cada tanda, `s` confirma y cualquier otra tecla cancela. Las tandas no
   se encadenan. Cada búsqueda de origen también cuenta como una corrida
   (ver [Cómo arma cada receta](#cómo-arma-cada-receta)).

4. Revisá cada receta. Elegila con `↑` `↓` y leé en el panel derecho qué va a ejecutar y de
   qué parte de la documentación sale cada comando.

5. Apretá `a` (o espacio) para aprobarla. Lo que no querés instalar, excluilo con `x`.
   Se oculta sin pedir motivo. `o` muestra u oculta los excluidos; sobre uno de ellos,
   `x` lo vuelve a incluir conservando su receta. El motivo es opcional y se puede editar con `e`.

6. Salí con `q`. Todo queda guardado en el cookbook.

Lo mismo sin pantalla, para scripts:

```bash
rebake escanear
rebake investigar
```

`rebake escanear` termina con una línea como `91 ítems (91 nuevos) en
~/Documentos/rebake/cookbook.toml`, y `rebake investigar`, con `N por revisar, 0 con error`.
Aprobar se hace en la pantalla.
Los comandos también omiten lo sincronizado; pedir su id con `rebake investigar` falla
sin gastar corridas y explica cómo volver a escanear si cambiaste su origen.

## Guardar el cookbook en un repo

El cookbook tiene tus ajustes personales: versionalo en un repo **privado**.

1. Creá un repo git, vacío o con el cookbook en la raíz.
2. Clonalo con rebake:

   ```bash
   rebake clonar https://github.com/<tu-usuario>/<tu-cookbook>
   ```

   Lo clona en `~/Documentos/rebake/`. La carpeta tiene que estar vacía o no existir.

Desde ahí, cada comando hace `git pull --ff-only` antes de empezar. Si el cookbook cambió,
al terminar hace un commit solo de ese archivo y lo pushea; verás `cookbook publicado en el
repo`. Usa tu `git` y tus credenciales: para un repo privado en GitHub, `gh auth login` y
`gh auth setup-git`, o una clave ssh. Los conflictos entre dos PCs los resolvés con git en esa
carpeta.

En el repo, el cookbook tiene que ser un archivo común: si es un enlace, rebake se niega a
usarlo para no escribir fuera del clon.

Si preferís manejar git a mano, enlazá el archivo. rebake escribe a través del enlace y no toca
git:

```bash
mkdir -p ~/Documentos/rebake
ln -s ~/mi-repo-privado/cookbook.toml ~/Documentos/rebake/cookbook.toml
```

## Instalar todo en una PC nueva

1. Instalá Claude Code y logueate.
2. Instalá rebake (ver [Instalar](#instalar)).
3. Traé tu cookbook y abrí la pantalla en **Instalación**:

   ```bash
   rebake https://github.com/<tu-usuario>/<tu-cookbook>
   ```

   Aparecen las recetas aprobadas, todas marcadas. Las que llegaron o cambiaron en el repo desde
   la última vez dicen `nueva` o `cambió`.

4. Apretá `c` para comprobar cuáles ya están en esta PC.
5. Desmarcá con espacio lo que no quieras. Lo que desmarcás vale solo para esta instalación.
6. Apretá `p` para instalar. Al terminar, la pantalla pasa sola a **Pendientes**.
7. Seguí la lista de [Lo que queda a mano](#lo-que-queda-a-mano).

Lo mismo sin pantalla:

```bash
rebake clonar https://github.com/<tu-usuario>/<tu-cookbook>
rebake instalar --dry-run
rebake instalar
```

`--dry-run` muestra los comandos sin ejecutar nada. `rebake instalar` pregunta
`¿Instalar N recetas en …? [s/N]` (`--si` responde que sí), muestra el resultado de cada
receta y termina con la lista de pendientes.

## Comandos

| Comando | Qué hace |
|---|---|
| `rebake` | Abre la pantalla. |
| `rebake <url>` | Clona el cookbook la primera vez y abre la pantalla en Instalación. |
| `rebake clonar <url>` | Clona el cookbook sin abrir la pantalla. |
| `rebake escanear` | Escanea esta PC y agrega lo nuevo al cookbook. |
| `rebake investigar [ids…]` | Arma las recetas pendientes que tienen origen, o solo las de esos ids. |
| `rebake instalar [--dry-run] [--si]` | Instala las recetas aprobadas. |
| `rebake renombrar-perfil <viejo> <nuevo>` | Cambia el nombre de un perfil en todo el cookbook. |

| Opción | Para qué |
|---|---|
| `--archivo <ruta>` | Usa otro cookbook. |
| `--profile <nombre>` | Usa la cuenta del alias o función de tu shell para las corridas de `claude -p`. |

Las opciones van antes o después del subcomando. Los ids son como los muestra el título del
panel derecho, por ejemplo `plugin:superpowers@claude-plugins-official` o `skill:technical-writer`.

Para elegir la cuenta con la que investigás:

```bash
rebake --profile clp
rebake investigar --profile clp
```

`clp` tiene que ser un alias o función que llame a `claude`, definido en la shell de `SHELL`
(zsh o bash). Se resuelve una vez al arrancar; esa cuenta se usa en investigación, búsqueda de
origen y las corridas de `i`, `t` y aceptar un posible origen. Resolver no gasta una corrida.
El nombre solo admite letras ASCII, números, guion o guion bajo, sin empezar con guion.
Un programa del `PATH` no sirve como perfil.

Sin `--profile`, la cuenta se hereda como siempre. `escanear`, `clonar`, `instalar` y
`renombrar-perfil` aceptan e ignoran la opción: el escaneo lee ambos perfiles y la instalación
conserva la cuenta de cada uno.

## La pantalla

La cabecera cuenta las recetas por estado; las marcas de esos contadores (`✓ □ · ? ✗`) son las
mismas de la lista. El pie muestra las teclas de lo que tenés elegido y, debajo, la próxima tecla
útil. `?` abre la ayuda: el camino (escanear › armar recetas › aprobar › instalar) con tu paso
actual, qué significa cada marca, para qué es cada pestaña y todas las teclas. Ninguna tecla
depende de Shift, y con `NO_COLOR` la pantalla no usa colores.

En Recetas, el pie separa las cuentas: `t armar 25 + 41 Claude` son 25 recetas gratis y
41 corridas pendientes; cuenta corridas, no tokens ni porcentaje de cuota. Mientras haya
corridas de Claude en curso, `t` sigue armando gratis y pide esperar. `i` sigue de a una y
no consume lugares de la tanda.

| Tecla | En todas las pestañas |
|---|---|
| `1` `2` `3` | Recetas, Pendientes, Instalación |
| `s` | escanear |
| `p` | instalar lo marcado en Instalación |
| `u` | actualizar rebake, cuando la cabecera avisa |
| `?` | ayuda |
| `q` | salir |

**Recetas**, en tu PC:

| Tecla | Acción |
|---|---|
| `i` / `t` | armar la elegida / las gratis y preguntar por hasta 5 corridas de Claude |
| Enter / `n` | aceptar o descartar el posible origen; el panel avisa si aceptar sale gratis o gasta otra corrida |
| `l` | pegar el link del repo |
| `a` o espacio | aprobar o desaprobar una receta armada |
| `v` | aprobar todas las recetas por revisar (pregunta en el pie y confirma con `s`) |
| `x` | excluir sin pedir motivo; sobre una excluida, la vuelve a incluir conservando la receta |
| `o` | mostrar u ocultar los excluidos (ocultos al arrancar) |
| `e` | editar la receta en `$VISUAL` o `$EDITOR`; queda por revisar |
| `r` | volver a investigar |
| `/` | filtrar |

**Instalación**, en la PC nueva:

| Tecla | Acción |
|---|---|
| `↑` `↓` | moverse; el panel derecho muestra los comandos y la comprobación |
| espacio | marcar o desmarcar; arrastra a las recetas que dependen de ella |
| `t` | marcar todas o ninguna |
| `c` | comprobar qué está instalado en cada perfil |
| `Esc` | volver de los resultados al plan |

**Pendientes**, después de instalar: la lista va a la izquierda y lo elegido, entero, a la
derecha. Espacio tilda una línea (no se guarda en el cookbook) y Enter despliega o colapsa los
ajustes.

## Cómo arma cada receta

1. **Busca el origen:** el link que pegaste o los metadatos de tu PC (catálogo del marketplace,
   symlink a un repo git, lock de skills.sh o una URL dentro del binario).
2. **Receta directa:** si es un marketplace, un plugin o una skill de skills.sh, usa su
   instalador oficial (`claude plugin marketplace add`, `claude plugin install`,
   `npx skills add`) sin Claude. El paso corre una vez por perfil.
3. **Investigación:** para el resto, `claude -p --restricted` lee la documentación de
   instalación de ese repo y devuelve los pasos con la **cita** literal de la que sale cada
   comando. Corre con Sonnet, sin poder ejecutar comandos ni escribir archivos.
4. **Sin origen:** `i` o una tanda confirmada de `t` buscan en la web de qué repo viene y el
   panel te lo propone con el motivo. Enter lo acepta (la receta dice "origen buscado en la
   web"), `n` lo descarta e `i` te deja pegar el link. `t` saltea los posibles pendientes,
   descartados o no encontrados; si una corrida falla, vuelve a contar para la próxima tanda.
   Para volver a buscar un origen ya buscado, `i` y Enter vacío.
5. **Aprobación:** vos revisás la cita y la URL y aprobás. Nada se ejecuta sin tu aprobación.

## Lo que queda a mano

Los hooks reconocidos como parte de una herramienta siguen en el cookbook con «lo instala
<herramienta>»: no se investigan, aprueban ni ejecutan aparte. Se vinculan por ejecutable,
o por carpeta y nombre coincidentes. Una carpeta propia del home puede identificar una
herramienta externa; directorios de perfil, menciones y casos ambiguos conservan el flujo
anterior. Los scripts propios, como `load-intent.sh`, conservan su receta.

Al investigar una herramienta con hooks vinculados, su receta pide el registro documentado
después de instalarla. Si el instalador ya los crea, no se duplica; sin documentación,
queda pendiente. Revisá las citas y los límites de soporte por perfil antes de aprobar.
`c` comprueba los hooks vinculados por presencia en cada perfil sin ejecutarlos; un binario
presente con hooks faltantes no hace que se saltee el registro. Nunca se copian hooks a mano.

Los cookbooks nuevos guardan el responsable; si una versión anterior rechaza ese campo,
su error muestra cómo actualizar rebake.

La pestaña Pendientes, y la salida de `rebake instalar`, junta lo que rebake no hace por vos:

- **Logins:** `claude` y `/login` en cada perfil, y `gh auth login`.
- **Hooks de herramientas externas:** una línea por herramienta, con cantidad de hooks y
  perfiles; aparecen al instalarla o abrirla. Rebake no instala esas herramientas ni hooks.
- **Pasos a mano:** un bloque "revisá X" por cada receta que dejó algo para hacer vos, con una
  línea por perfil:
  - Los **servidores MCP**, como Serena, nunca se instalan solos. Su receta trae los pasos
    (por ejemplo `uv tool install …` y el `claude mcp add --scope user …`) para que los corras
    vos.
  - Los **comandos de Claude**, como `/codex:setup`, dicen en qué perfil abrir `claude` para
    correrlos.
- **Ajustes** de `settings.json` que rebake no copia, para que los pongas a mano si los querés.
- **Paquetes del sistema** que piden las recetas, como `sudo pacman -S jq`.
- **Tu texto propio en `CLAUDE.md`**, por título.

## Configuración

| Variable | Default | Para qué |
|---|---|---|
| `REBAKE_INSTALL_DIR` | `~/.local/bin` | Carpeta donde `install.sh` deja el binario. |
| `REBAKE_VERSION` | `latest` | Versión que instala `install.sh`, por ejemplo `v0.8.4`. |
| `REBAKE_BASE_URL` | `https://github.com/Ranteck/rebake/releases` | De dónde bajan `install.sh` y `u` los releases. |
| `REBAKE_MODELO` | `sonnet` | Modelo de `claude -p` para investigar y buscar orígenes. |
| `REBAKE_CLAUDE` | `claude` | Binario de Claude Code que usa rebake, también con `--profile`. |
| `SHELL` | entorno de tu sesión | Shell que resuelve `--profile`: zsh o bash, leyendo su archivo de inicio interactivo. Solo se necesita con esa opción. |
| `CLAUDE_CONFIG_DIR` | sin definir | Cuenta heredada sin `--profile`. Con la opción se reemplaza por la del alias o función; si usa la predeterminada, se quita. |
| `VISUAL`, `EDITOR` | `vi` | Editor de `e`. |
| `NO_COLOR` | sin definir | Con cualquier valor, la pantalla no usa colores. |

## Archivos

| Ruta | Qué es |
|---|---|
| `~/Documentos/rebake/cookbook.toml` | El cookbook (`~/Documents/…` en un sistema en inglés). |
| `~/.local/state/rebake/rebake.log` | Log de rebake. |
| `~/.local/state/rebake/ultima-instalacion.log` | Cada comando que corrió la última instalación, con su salida. |

## Seguridad

- Resolver `--profile` ejecuta el archivo de inicio de tu shell y el alias o función propio.
  Sus otras acciones también se ejecutan: revisalas antes de usarlo.
- La investigación corre sin Bash, sin escritura y sin tus hooks, plugins ni MCP. Lo que se le
  manda pasa antes por el filtro de secretos.
- El escaneo nunca lee `.credentials.json` ni `~/.codex/auth.json`. De `.claude.json` lee solo
  `mcpServers`, y de cada servidor, el comando, sus argumentos o el host de su URL; `env`,
  headers, la cuenta y los proyectos se descartan al leerlo.
- En el cookbook se ocultan como `‹oculto›`:
  - las variables de `env`;
  - las claves que contienen `token`, `secret`, `key`, `password`, `auth`, `credential`,
    `cookie` o `header`;
  - los valores con forma de token y las credenciales o rutas de las URLs;
  - en los comandos, la clave de `-u usuario:clave` y la de `-p` en `mysql`, `mariadb`,
    `sshpass` y `7z`.
- Ese filtro va por patrones y no reconoce cualquier secreto. Cuando agregues un hook con una
  credencial en el comando, mirá el diff del cookbook.
- Instalar ejecuta comandos de terceros, a veces `curl … | sh`: revisá la cita antes de aprobar.
- Abrir un cookbook no ejecuta nada suyo. Lo primero que corre son las comprobaciones, cuando
  apretás `c`, y cada una se puede leer antes en el panel de la receta.
- Quien puede escribir en el repo de tu cookbook decide qué instala rebake en tus PCs:
  mantenelo privado y con doble factor.

## Problemas comunes

**`` rebake: …/cookbook.toml:1757:8: unknown variant `mcp`… ``**
El cookbook lo escribió un rebake más nuevo. Actualizá rebake corriendo otra vez el
[instalador](#instalar). Desde v0.8.4 el mismo error te muestra ese comando.

**`no pude ejecutar claude: ¿está instalado?`**
Claude Code no está en el `PATH`. Instalalo y logueate, o apuntá `REBAKE_CLAUDE` a su binario.

**`aviso: … git push falló: …`**
Falló la sincronización con el repo del cookbook. El comando igual termina y el cookbook queda
en disco; lo pendiente se sube en la próxima corrida. Revisá tus credenciales de git.

**`no pude abrir la pantalla: la salida no es una terminal`**
La pantalla necesita una terminal. Para scripts usá los subcomandos (`escanear`, `investigar`,
`instalar`).

**Una receta falló al instalar.**
La línea de esa receta dice qué comando falló. El detalle completo está en
`~/.local/state/rebake/ultima-instalacion.log`.

## Desarrollo

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
```

Los tests no salen a internet ni tocan tu `~/.claude*`: arman un home falso, un `claude`
falso y, para la actualización, un servidor local. Para
publicar, subí la versión en `Cargo.toml` y pusheá el tag `vX.Y.Z`. El workflow
`.github/workflows/release.yml` verifica que el tag coincida con esa versión, corre los tests,
compila un binario estático y publica el release con su checksum.

La intención, los requisitos y los planes de cada feature están en [`intent/`](intent/).

## Licencia

MIT. Ver [`LICENSE`](LICENSE).
