# Spec: hooks que instala su herramienta (desde intent.md 2026-10-27)
Estado: borrador

## Requisitos
- REQ-1 MUST: reconocer al responsable por el ejecutable o la ruta ejecutada inequívocos:
  `rtk` en su función de hook, scripts de `~/.orca/agent-hooks/` y el ejecutable
  `skills/impeccable/scripts/impeccable` del perfil. Admitir envoltorios solo si permiten
  identificar esa ejecución; una mención, un intérprete, compartir repo o un requisito no
  prueban autoría. Ante dudas, conservar el tratamiento actual.
  Origen: Preguntas abiertas, «por el binario o la ruta del comando»; Resultado esperado 4.
- REQ-2 MUST: el hook reconocido queda asociado a su herramienta y perfiles; la TUI dice
  «lo instala <herramienta>». No se investiga, aprueba ni ejecuta como receta independiente,
  incluso si el cookbook ya contenía una. Origen: Resultado esperado 1, «no tiene receta
  propia» y «no vale la pena que se reinstalen dos veces».
- REQ-3 MUST: cuando rebake instala la herramienta, su receta incluye el registro de hooks
  documentado por su repo, después de instalarla; si su instalador ya los crea, no repetirlo.
  Conserva revisión y aprobación de la receta y cubre los perfiles afectados según soporte
  documentado. Sin comando confirmado, informa lo pendiente sin inventarlo.
  Origen: Resultado esperado 2, «incluye el comando que crea el hook»;
  Restricciones, «el comando sale del repo».
- REQ-4 MUST: comprobar distingue herramienta instalada de hooks presentes por perfil.
  Tener el binario no permite omitir un registro faltante.
  Origen: Resultado esperado 2, «al reinstalar en otra PC el hook aparece solo».
- REQ-5 MUST: para herramientas externas, no instalar herramienta ni hook; la checklist
  de CLI y TUI indica responsable, perfiles y que aparece al instalar o abrir esa herramienta.
  Origen: Resultado esperado 3, «como Orca»; Fuera de alcance, «herramientas de escritorio
  o de pacman».
- REQ-6 MUST: hooks propios o no atribuibles conservan su flujo actual de receta,
  requisitos, revisión y verificación. Origen: Resultado esperado 4, «siguen como hoy»;
  Preguntas abiertas.
- REQ-7 MUST: no exponer secretos del comando en cookbook, pantalla, logs ni investigación;
  no copiar hooks a mano a la configuración. Origen: Restricciones, «Sin secretos» y
  «Nunca se copia el hook a mano».

## Capacidades y escenarios
### Reconocer y vincular (REQ-1, REQ-2)
- GIVEN hooks de RTK en ambos perfiles y tres del ejecutable de impeccable en el laboral
- WHEN se escanean
- THEN muestran su responsable y perfiles, sin recetas independientes ni investigaciones duplicadas.

- GIVEN un hook reconocido con receta propia previa
- WHEN se vuelve a escanear
- THEN queda vinculado y esa receta no se ejecuta aparte.

### Reinstalar y comprobar (REQ-3, REQ-4)
- GIVEN RTK aprobado y su registro documentado mediante `rtk init -g`
- WHEN se instala por CLI o TUI en otra PC
- THEN su receta crea los hooks en los perfiles soportados, sin pasos independientes.

- GIVEN RTK instalado pero falta el hook en un perfil
- WHEN se comprueba y luego se instala
- THEN señala ese perfil y no saltea el registro por existir el binario.

- GIVEN falta documentación del registro
- WHEN se prepara la receta
- THEN informa lo pendiente sin inventar comandos.

### Herramienta externa (REQ-1, REQ-2, REQ-5)
- GIVEN quince hooks de Orca con envoltorios condicionales y rutas inequívocas
- WHEN se escanea e instala
- THEN dice «lo instala Orca»; la checklist avisa instalar o abrir Orca, sin ejecutarla.

### Compatibilidad y privacidad (REQ-1, REQ-6, REQ-7)
- GIVEN `load-intent.sh`, `sync-plugins.sh` y un comando ambiguo que solo menciona RTK
- WHEN se escanean
- THEN conservan su tratamiento actual, aunque compartan repo o dependencia con una herramienta.

- GIVEN un hook con un secreto
- WHEN se escanea, investiga o instala
- THEN no expone el secreto ni reconstruye el hook copiándolo.

## Concerns
- Una ruta o un nombre parecidos pueden atribuir un script propio incorrectamente: ante
  ambigüedad prevalece el flujo actual.
- El registro oficial puede cubrir solo un perfil; señalar la limitación sin prometer ambos.
