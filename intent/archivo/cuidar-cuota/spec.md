# Spec: cuidar la cuota al armar todas (desde intent.md 2026-10-22)
Estado: aceptado

Definiciones:
- Corrida: un `claude -p`.
- Gratis: ítem pendiente con receta directa: rebake la arma sin Claude, con el criterio
  que ya usa al investigar.
- Con Claude: ítem pendiente, no excluido y sin trabajo en curso, que gastaría una corrida: con
  origen y sin receta directa (se investiga), o sin origen ni posible origen (se busca; incluye
  las búsquedas que fallaron).

## Requisitos
- REQ-1 MUST: `t` arma todas las gratis sin preguntar. Origen: Resultado 1.
- REQ-2 MUST: si quedan con Claude, `t` no las lanza: el pie pregunta si gasta N corridas, con
  N = 5 o las que queden si son menos. La pregunta sigue a la vista hasta la respuesta, aunque
  terminen otros trabajos. Origen: Resultado 2.
- REQ-3 MUST: `s` lanza la tanda fijada al preguntar, sin las que ya no estén pendientes:
  primero las que tienen origen y después las búsquedas, en el orden de la lista en pantalla.
  Cualquier otra tecla cancela sin hacer su acción de siempre, no lanza nada y el pie lo dice;
  lo gratis ya armado queda. Origen: Resultado 2.
- REQ-4 MUST: las tandas no se encadenan: al terminar una no arranca otra. Mientras haya
  corridas de Claude en curso, de una tanda o de `i`, `t` arma lo gratis y avisa que espere en
  lugar de preguntar. Origen: Resultado 2.
- REQ-5 MUST: el pie de Recetas muestra `t` con las dos cuentas, por ejemplo
  `t armar 25 + 41 Claude`, sin la parte que da cero, y entra entero en 80 columnas con números
  de hasta tres cifras. Origen: Resultado 3.
- REQ-6 MUST: con un posible origen pendiente, el panel dice si aceptarlo gasta una corrida de
  Claude o sale gratis. Origen: Resultado 4.
- REQ-7 MUST: una tanda termina cuando terminan todas sus corridas, bien o con error. Un ítem
  cuya corrida falló vuelve a contar como "con Claude". Origen: Resultado 2.
- REQ-8 MUST: no cambian `i`, aprobar de a una, aceptar o descartar un posible origen ni lo
  excluido; `i` no cuenta para el límite de 5. Origen: Restricciones.

## Capacidades y escenarios
### Lo gratis sin preguntar (REQ-1, REQ-2)
- GIVEN 25 gratis y 41 con Claude
- WHEN `t`
- THEN se arman las 25 y el pie pregunta si gasta 5 corridas.

### Tanda confirmada o cancelada (REQ-2, REQ-3)
- GIVEN la pregunta en el pie, con 3 investigaciones y 38 búsquedas pendientes
- WHEN termina otro trabajo y después Denis aprieta `s`
- THEN la pregunta siguió a la vista y se lanzan las 3 investigaciones y 2 búsquedas.

- GIVEN la pregunta en el pie
- WHEN Denis aprieta `i`
- THEN no se investiga nada y el pie dice que no se gastó nada.

### Sin encadenar (REQ-4)
- GIVEN 2 corridas de Claude en curso
- WHEN `t`
- THEN se arma lo gratis y el pie dice que espere, sin preguntar.

### Cuenta en el pie (REQ-5)
- GIVEN 107 gratis y 141 con Claude, en 80×24
- WHEN se dibuja Recetas
- THEN el pie muestra completos las pestañas, `? ayuda` y la tecla `t` con las dos cuentas.

### Aviso al aceptar (REQ-6)
- GIVEN un posible origen pendiente que no daría receta directa
- WHEN Denis lo selecciona
- THEN el panel dice que aceptarlo gasta una corrida de Claude.

### Errores y `i` (REQ-7, REQ-8)
- GIVEN una tanda de 5 donde 2 búsquedas fallan, y durante la tanda Denis aprieta `i`
- WHEN terminan todas
- THEN `i` investigó como hoy, y las 2 fallidas vuelven a contar para la próxima tanda.

## Concerns
- **Tecla `s`.** Instalar y salir ya confirman con `s`, y Enter acepta un posible origen: un
  Enter de más dispararía 5 corridas. Codex y Antigravity coinciden; intent enmendado.
- `25 + 41 Claude` dice menos que "gratis", pero entra en 80 columnas; `?` lo explica.
- Una búsqueda que falla siempre (p. ej. sin cuota) vuelve en cada tanda confirmada.
