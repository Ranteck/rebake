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
