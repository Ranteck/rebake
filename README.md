# rebake

Tu setup de Claude Code como un libro de cocina: cada plugin, skill o herramienta es una
receta con su forma de instalación, y en una PC nueva `rebake` las vuelve a hornear.

En vez de copiar carpetas (que quedan viejas o rotas), `rebake` mira qué tenés instalado,
averigua con `claude -p` cómo se instala cada cosa según su propio repo, y guarda esas recetas
en tu **cookbook**, un archivo TOML que revisás y versionás. En una PC nueva, ejecuta las
recetas aprobadas y te deja una checklist de lo que falta hacer a mano.

Cubre plugins, marketplaces, skills, imports de `CLAUDE.md`, hooks, statusline y las
herramientas que usan, en todos tus perfiles (`~/.claude` y `~/.claude-*`). No copia
memorias, historial ni credenciales, ni instala paquetes del sistema.

## Instalar

En Linux x86_64, sin necesidad de Rust:

```sh
curl -fsSL https://github.com/Ranteck/rebake/releases/latest/download/install.sh | sh
```

Baja el binario del último release, verifica su checksum y lo deja en `~/.local/bin`. Para otra
carpeta usá `REBAKE_INSTALL_DIR`; para una versión fija, `REBAKE_VERSION=v0.1.0`. Volver a
correrlo actualiza.

Con Rust instalado también se puede compilar: `cargo install --git https://github.com/Ranteck/rebake`.

Para usarlo hacen falta `git` y, para investigar recetas, Claude Code.

## Uso

```sh
rebake clonar <url>      # una vez: trae tu cookbook de su repo git y lo sincroniza
rebake escanear          # mira esta PC y agrega lo nuevo al cookbook
rebake                   # abre la TUI para revisar, investigar y aprobar recetas
rebake investigar        # investiga las recetas pendientes sin abrir la TUI
rebake instalar --dry-run
rebake instalar          # en la PC nueva: vuelve a hornear todo
```

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

### Teclas de la TUI

| Tecla | Acción |
|---|---|
| `s` | escanear |
| `i` / `I` | investigar la receta / todas las pendientes |
| `a` | aprobar (solo una receta investigada o editada) |
| `x` | excluir, con motivo |
| `l` | pegar el link del repo cuando no se encontró |
| `e` | editar la receta en `$EDITOR` |
| `r` | reinvestigar |
| `P` | instalar lo aprobado |
| `/` | filtrar |
| `1` `2` `3` | Recetas, Checklist, Instalación |
| `q` | salir |

## Cómo arma cada receta

1. Busca el repo de origen: el link que pegaste, después los metadatos locales (catálogo del
   marketplace, symlink a un repo git, lock de skills.sh, URL dentro del binario) y, si no hay
   pista, la web.
2. `claude -p --restricted` lee la documentación de instalación de ese repo, sin poder
   ejecutar comandos ni escribir archivos, y devuelve los pasos con la **cita** literal de la
   doc de la que sale cada comando.
3. Vos revisás la cita y la URL y aprobás. Nada se ejecuta sin tu aprobación.

## Seguridad

- La investigación corre sin Bash, sin escritura, sin tus hooks, plugins ni MCP, y lo que se le
  manda pasa antes por el filtro de secretos.
- El escaneo nunca lee `.credentials.json`, `~/.codex/auth.json` ni `.claude.json`.
- En el cookbook se ocultan como `‹oculto›` las variables de `env`, las claves que contienen
  `token`, `secret`, `key`, `password`, `auth`, `credential`, `cookie` o `header`, los valores
  con forma de token y las credenciales o rutas de las URLs.
- Instalar ejecuta comandos de terceros (a veces `curl … | sh`): revisá la cita antes de
  aprobar.

## Licencia

MIT
