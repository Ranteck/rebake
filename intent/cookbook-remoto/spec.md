# Spec: cookbook desde un repo (desde cookbook-remoto/intent.md 2026-10-08)
Estado: aceptado

## Requisitos
- REQ-1 MUST: `rebake clonar <url>` clona el repo en la carpeta del cookbook (la de
  `~/Documentos/rebake/cookbook.toml`, o la de `--archivo`); si falla no deja un clon a medias.
  Origen: Resultado 1 — "Le paso la URL una vez y rebake clona el repo en la carpeta del
  cookbook".
- REQ-2 MUST: `clonar` se niega si la carpeta destino existe y no está vacía, sin tocarla.
  Origen: Restricciones — "Un fallo de red o de git no hace perder trabajo".
- REQ-3 MUST: `clonar` valida el cookbook del repo: si es inválido falla con
  `archivo:línea:columna` y no deja la carpeta; si el repo no lo tiene, arranca vacío.
  Origen: Usuarios — "cualquiera que use rebake con su cookbook en un repo git propio".
- REQ-4 MUST: solo se sincroniza un clon hecho por `clonar`; con cualquier otro cookbook
  (archivo suelto, symlink, otro repo) rebake no ejecuta git. Origen: Restricciones — "Solo
  sincroniza un clon hecho por rebake".
- REQ-5 MUST: en ese clon, la TUI, `escanear`, `investigar`, `instalar` y `renombrar-perfil`
  traen lo último del repo (solo avance rápido) antes de leer el cookbook. Origen: Resultado 2
  — "Al arrancar trae lo último del repo".
- REQ-6 MUST: al terminar, si el cookbook cambió, hacen un solo commit con solo ese archivo y
  pushean; si no cambió, no commitean. Origen: Resultado 2 — "si el cookbook cambió, hace un
  solo commit con un resumen y lo pushea".
- REQ-7 SHOULD: el mensaje del commit nombra el comando y cuenta los ítems nuevos y los que
  pasaron a cada estado. Origen: Resultado 2 — "con un resumen".
- REQ-8 MUST: si falla el pull, el commit o el push, el comando sigue o termina igual, avisa
  por stderr y en el log, el cookbook queda en disco y el código de salida no cambia; lo que
  quedó sin commitear o sin pushear se publica en la corrida siguiente. Origen: Restricciones
  — "Un fallo de red o de git no hace perder trabajo".
- REQ-9 MUST: si un comando falla a mitad, lo que llegó a guardar se publica igual y después
  se informa el error original. Origen: Restricciones — "no hace perder trabajo".
- REQ-10 MUST: rebake no pide ni guarda credenciales: usa el `git` del usuario, git nunca le
  pide nada por terminal (falla en vez de colgarse, con tope de tiempo) y ninguna credencial
  de una URL aparece en pantalla ni en el log. Origen: Restricciones — "rebake no maneja
  credenciales"; intent raíz — "Sin secretos".
- REQ-11 SHOULD: el README presenta `rebake clonar` como la forma de usar un cookbook en un
  repo y el uso en una PC nueva. Origen: Resultado 3 — "En una PC nueva: instalar rebake,
  pasarle la URL e instalar".

## Capacidades y escenarios
### Clonar (REQ-1, REQ-2, REQ-3)
- GIVEN un repo con un `cookbook.toml` válido de 12 recetas y `~/Documentos/rebake` inexistente
- WHEN `rebake clonar <url>`
- THEN queda el clon en `~/Documentos/rebake`, marcado, y se ve `clonado en
  ~/Documentos/rebake (12 recetas)`
- GIVEN `~/Documentos/rebake/cookbook.toml` ya existe — WHEN `rebake clonar <url>` — THEN
  falla diciendo que la carpeta no está vacía y el archivo sigue igual
- GIVEN un repo con un cookbook con un campo desconocido — WHEN `rebake clonar <url>` — THEN
  falla con línea y columna y `~/Documentos/rebake` no existe
- GIVEN un repo vacío — WHEN `rebake clonar <url>` y después `rebake escanear` — THEN el repo
  remoto recibe un commit que crea `cookbook.toml`

### Sincronizar (REQ-4 a REQ-7)
- GIVEN un clon marcado y otro equipo que pusheó un cambio — WHEN `rebake` — THEN la TUI
  muestra ese cambio
- GIVEN un clon marcado — WHEN `rebake renombrar-perfil a b` — THEN el remoto recibe un commit
  que solo toca `cookbook.toml`, con un mensaje que empieza con `rebake renombrar-perfil`
- GIVEN un clon marcado — WHEN `rebake instalar --dry-run` — THEN no hay commits nuevos
- GIVEN un cookbook enlazado a un repo de dotfiles — WHEN `rebake escanear` — THEN ese repo no
  tiene commits nuevos ni recibe push

### Fallos (REQ-8, REQ-9)
- GIVEN el remoto inaccesible — WHEN `rebake renombrar-perfil a b` — THEN sale con 0, avisa
  que el cambio quedó commiteado en local, y con el remoto de vuelta la próxima corrida lo
  pushea
- GIVEN git sin `user.name` — WHEN un comando cambia el cookbook — THEN avisa la causa y el
  archivo queda cambiado en disco

### Credenciales (REQ-10)
- GIVEN una URL `https://usuario:secreto@…` inaccesible — WHEN `rebake clonar` — THEN falla
  con una pista para autenticar git y `secreto` no aparece en la salida ni en el log

## Diseño

### Módulos
| Módulo | Cambio |
|---|---|
| `sincro` (nuevo) | `clonar`, `sincronizado` (el clon y su marca), `traer` (pull) y `publicar` (commit del archivo + push si la rama va adelante). Llama a `git -C <dir>` con `proceso::ejecutar`, tope de 60 s y `GIT_TERMINAL_PROMPT=0`. |
| `servicio` | Arma el mensaje del commit comparando el cookbook antes y después (REQ-7). |
| `main` | `Comando::Clonar`; para los demás comandos: traer → leer "antes" → comando → leer "después" → publicar, también si el comando falló (REQ-9). La TUI no cambia: `tui::ejecutar` vuelve después de restaurar la terminal. |
| `README` | `rebake clonar` reemplaza al symlink como forma recomendada; el symlink queda sin sincronización. |

### Marca y clonado
`clonar` clona en una carpeta temporal al lado del destino, valida con `archivo::leer`, pone
`rebake.sincronizar = true` en el `.git/config` del clon y la renombra al destino. Un clon
sincronizado es una carpeta que es la raíz de un repo git con esa marca. El cookbook es el
archivo de la raíz del clon con el nombre de la ruta en uso (`cookbook.toml` por defecto, o el
de `--archivo`).

### Errores
Los fallos de git se muestran como `aviso:` y se loguean como `warn` (degradado y recuperado);
solo `clonar` falla con error. Las URLs pasan por `secretos::sin_credenciales_url` y la salida
de git por `secretos::ocultar`. Un fallo de autenticación sugiere `gh auth login` y
`gh auth setup-git`, o ssh.

### Tests
Sin red: el remoto es un repo bare local; la identidad de git va por variables de entorno.

## Concerns
- Dos PCs que editan a la vez: el pull de avance rápido falla, rebake avisa y sigue; el push
  también falla y el usuario resuelve el merge (fuera de alcance).
- Una clave ssh con passphrase sin agente no puede pedirla: git falla por tope a los 60 s.
- En una PC nueva sin `user.name`, los commits fallan hasta configurarlo; el aviso lo dice.
- El repo de Denis se llama `rebake-cookbok`: si es un typo, conviene renombrarlo antes de
  clonarlo, porque la URL queda como `origin` del clon.
