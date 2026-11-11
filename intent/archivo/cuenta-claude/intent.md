# Intención: elegir con qué cuenta corre Claude
Autor: Denis · Estado: aceptado · Fecha: 2026-10-23

## Problema
Cuando `t` busca recetas, rebake llama a `claude -p`, que gasta la cuenta de `~/.claude`. Denis
tiene dos perfiles con alias, `cl` y `clp`, y en el principal "no tengo cupo ya porque no tengo
tokens". Quiere que rebake use el otro "para hacer la búsqueda de nuevas recetas", con algo como
`rebake --profile clp`.

## Resultado esperado
1. `rebake --profile clp`, también con un subcomando (`rebake investigar --profile clp`), hace
   que toda corrida de `claude -p` gaste la cuenta que usa `clp`, como si fuera `clp -p`.
2. Vale cualquier alias o función de la shell de Denis que termine llamando a `claude`:
   `clp`, `claude-personal` o `cl`. rebake le pregunta a la shell una vez, al arrancar, qué
   perfil usa.
3. Si el nombre no existe o no llama a `claude`, rebake frena antes de gastar nada y lo dice.
4. Sin `--profile`, todo sigue como hoy.

## Usuarios y sistemas afectados
Denis y su `.zshrc`; la CLI (`investigar`) y la TUI (`i`, `t` y aceptar un posible origen);
`claude -p`.

## Restricciones
- Solo cambia qué cuenta se gasta: el escaneo sigue leyendo los dos perfiles y la instalación
  sigue armando cada perfil con el suyo.
- Sin configurar nada nuevo en la shell ni en el cookbook.
- La shell se consulta una sola vez, no en cada corrida.
- Solo acepta alias o funciones de nombre simple (letras, números, guion o guion bajo, sin empezar con guion): nunca ejecuta un programa ni un texto con espacios o ;

## Fuera de alcance
- Pasar solo a otra cuenta cuando una se queda sin cuota.
- Medir la cuota de cada cuenta.

## Preguntas abiertas
Ninguna.

## Cambios
- 2026-10-23: solo alias o funciones de nombre simple, para no ejecutar programas al resolver --profile; lo marcó Antigravity. Aprobado por Denis con el OK de Codex y Antigravity.
