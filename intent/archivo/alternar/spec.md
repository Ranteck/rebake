# Spec: aprobar y excluir con la misma tecla, ida y vuelta (desde intent.md 2026-10-16)
Estado: aceptado

## Requisitos
- REQ-1 MUST: en Recetas, espacio y `a` sobre una receta por revisar la aprueban, y sobre
  una aprobada la devuelven a por revisar. Las dos teclas hacen exactamente lo mismo.
  Origen: Resultado 1, "espacio y `a` alternan la aprobación".
- REQ-2 MUST: espacio o `a` sobre un ítem sin receta o excluido no cambia nada y avisa por
  qué (sin receta: "investigalo o editalo antes de aprobar"; excluido: "`x` lo devuelve").
  Origen: Restricciones, "aprobar sigue pidiendo receta".
- REQ-3 MUST: `x` sobre un ítem no excluido pide el motivo y lo excluye, como hoy; un motivo
  vacío no excluye. Origen: Resultado 2; Restricciones, "excluir sigue pidiendo motivo".
- REQ-4 MUST: `x` sobre un ítem excluido lo devuelve sin preguntar: a por revisar si tiene
  pasos, a sin receta si no; borra el motivo. Origen: Resultado 2, "en uno excluido lo
  devuelve".
- REQ-5 MUST: cada cambio de REQ-1 y REQ-4 se guarda en el cookbook y el mensaje del pie lo
  dice ("aprobada" / "vuelve a por revisar" / "vuelve a …"). Origen: Resultado 3, "el
  cambio queda guardado en el cookbook".
- REQ-6 MUST: el pie de "Esta receta" muestra la acción que corresponde al ítem elegido:
  `espacio`/`a` "aprobar" o "desaprobar"; `x` "excluir" o "volver a incluir". Origen:
  Resultado 3, "el pie y la ayuda dicen qué hace cada tecla según el ítem elegido".
- REQ-7 MUST: la ayuda dice que en Recetas espacio guarda en el cookbook y en Instalación
  solo marca para esa instalación. Origen: Resultado 3.
- REQ-8 MUST: espacio en Instalación y en Pendientes sigue haciendo lo de hoy. Origen:
  Fuera de alcance.

## Capacidades y escenarios
### Alternar la aprobación (REQ-1, REQ-5)
- GIVEN una receta por revisar seleccionada
- WHEN se aprieta espacio
- THEN queda aprobada, se guarda y el pie dice "aprobada"
- WHEN se aprieta `a` sobre la misma
- THEN vuelve a por revisar, se guarda y el pie lo dice

### Lo que no se aprueba (REQ-2)
- GIVEN un ítem sin receta
- WHEN se aprieta espacio
- THEN sigue sin receta, no se guarda nada y el pie dice que falta la receta
- GIVEN un ítem excluido
- WHEN se aprieta `a`
- THEN sigue excluido y el pie dice que `x` lo devuelve

### Excluir y volver (REQ-3, REQ-4)
- GIVEN una receta por revisar
- WHEN se aprieta `x`, se escribe "duplicado" y Enter
- THEN queda excluida con ese motivo
- WHEN se aprieta `x` otra vez
- THEN vuelve a por revisar sin pedir nada y sin motivo
- GIVEN un ítem excluido sin pasos
- WHEN se aprieta `x`
- THEN vuelve a sin receta

### Pie y ayuda (REQ-6, REQ-7, REQ-8)
- GIVEN una receta aprobada seleccionada
- WHEN se dibuja
- THEN el pie ofrece "desaprobar", no "aprobar"
- GIVEN un ítem excluido
- WHEN se dibuja
- THEN el pie ofrece `x` "volver a incluir"
- GIVEN la pestaña Instalación
- WHEN se aprieta espacio
- THEN marca o desmarca como hoy y el cookbook no cambia

## Concerns
- Desaprobar una receta que otra aprobada necesita no rompe nada: al instalar, los
  requisitos no aprobados se saltean, igual que hoy. No se avisa; si hace falta, es otra
  feature.
- Espacio guarda en Recetas y no guarda en Instalación: la misma tecla con efectos de
  distinta duración. Por eso REQ-5 y REQ-7 lo dicen en pantalla.
- Volver a incluir no pide confirmación: si fue sin querer, `x` lo excluye de nuevo, pero hay
  que reescribir el motivo.
