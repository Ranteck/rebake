use rebake::modelo::{Estado, Item, Modo, Paso, Recetario, Tipo};
use rebake::servicio::{self, Novedad, mensaje_commit};

#[test]
fn hooks_vinculados_no_entran_a_investigacion_y_descartan_resultados_tardios() {
    let home = tempfile::tempdir().unwrap();
    let mut r = Recetario::nuevo();
    let mut hook = Item::nuevo("hook:tool", Tipo::Hook);
    hook.instalado_por = Some(rebake::modelo::InstaladorHook::Externo("tool".into()));
    r.items.push(hook);
    assert!(servicio::sin_origen(&r).is_empty());
    assert!(servicio::trabajos(&r, &[], home.path()).unwrap().is_empty());
    assert!(servicio::trabajos(&r, &["hook:tool".into()], home.path()).is_err());
    let receta = rebake::investigador::parsear_salida(r#"{"type":"result","subtype":"success","structured_output":{"repo":"https://example.com/tool","doc":"https://example.com/tool/install","pasos":[],"requiere":[]}}"#).unwrap();
    let antes = r.clone();
    rebake::investigador::aplicar(&mut r, "hook:tool", Ok(receta), "2026-10-27");
    rebake::investigador::aplicar_posible(
        &mut r,
        "hook:tool",
        Ok(rebake::modelo::Posible {
            repo: Some("https://example.com/tool".into()),
            motivo: "tardío".into(),
            descartado: false,
        }),
    );
    assert_eq!(r, antes);
}

fn con(items: &[(&str, Estado)]) -> Recetario {
    let mut r = Recetario::nuevo();
    for (id, estado) in items {
        let mut i = Item::nuevo(id, Tipo::Plugin);
        i.estado = *estado;
        r.items.push(i);
    }
    r
}

#[test]
fn cuenta_nuevos_y_cambios_de_estado() {
    let antes = con(&[("a", Estado::Pendiente), ("b", Estado::PorRevisar)]);
    let despues = con(&[
        ("a", Estado::Aprobada),
        ("b", Estado::Excluida),
        ("c", Estado::Pendiente),
    ]);
    assert_eq!(
        mensaje_commit(Some("escanear"), &antes, &despues),
        "rebake escanear: 1 nuevo, 1 aprobada, 1 excluida"
    );
}

#[test]
fn plural_y_sin_comando_para_la_tui() {
    let antes = con(&[("a", Estado::PorRevisar), ("b", Estado::PorRevisar)]);
    let despues = con(&[("a", Estado::Aprobada), ("b", Estado::Aprobada)]);
    assert_eq!(
        mensaje_commit(None, &antes, &despues),
        "rebake: 2 aprobadas"
    );
}

#[test]
fn sin_nuevos_ni_estados_dice_que_actualiza() {
    let r = con(&[("a", Estado::Aprobada)]);
    assert_eq!(
        mensaje_commit(Some("renombrar-perfil"), &r, &r),
        "rebake renombrar-perfil: actualiza el cookbook"
    );
}

fn aprobada(id: &str, cmd: &str) -> Item {
    let mut i = Item::nuevo(id, Tipo::Plugin);
    i.estado = Estado::Aprobada;
    i.pasos = vec![Paso {
        cmd: cmd.into(),
        modo: Modo::Auto,
        por_perfil: false,
        cita: None,
        nota: None,
    }];
    i
}

#[test]
fn novedades_marca_nuevas_y_cambiadas() {
    let mut antes = Recetario::nuevo();
    antes.items = vec![
        aprobada("igual", "uno"),
        aprobada("otro-comando", "uno"),
        Item::nuevo("recien-aprobada", Tipo::Plugin),
        aprobada("otra-comprobacion", "uno"),
    ];
    let mut despues = Recetario::nuevo();
    let mut comprobacion = aprobada("otra-comprobacion", "uno");
    comprobacion.verificar = Some("true".into());
    despues.items = vec![
        aprobada("igual", "uno"),
        aprobada("otro-comando", "dos"),
        aprobada("recien-aprobada", "uno"),
        comprobacion,
        aprobada("llego", "uno"),
        Item::nuevo("llego-pendiente", Tipo::Plugin),
    ];

    let n = servicio::novedades(&antes, &despues);
    assert_eq!(n.get("otro-comando"), Some(&Novedad::Cambio));
    assert_eq!(n.get("otra-comprobacion"), Some(&Novedad::Cambio));
    assert_eq!(n.get("recien-aprobada"), Some(&Novedad::Nueva));
    assert_eq!(n.get("llego"), Some(&Novedad::Nueva));
    assert_eq!(n.len(), 4, "{n:?}");
}

#[test]
fn sin_cambios_no_hay_novedades() {
    let mut r = Recetario::nuevo();
    r.items = vec![aprobada("a", "uno")];
    assert!(servicio::novedades(&r, &r.clone()).is_empty());
}

#[test]
fn sin_origen_lista_pendientes_sin_fuente_y_trabajos_sin_ids_las_excluye() {
    let mut r = con(&[
        ("a", Estado::Pendiente),
        ("b", Estado::Pendiente),
        ("c", Estado::PorRevisar),
    ]);
    r.items[1].fuente = Some(rebake::modelo::Fuente {
        repo: "https://github.com/ejemplo/repo".into(),
        via: rebake::modelo::Via::Metadatos,
        doc: None,
    });
    assert_eq!(servicio::sin_origen(&r), vec!["a".to_string()]);
    let t = servicio::trabajos(&r, &[], std::path::Path::new("/tmp")).unwrap();
    assert_eq!(t.len(), 1);
    assert_eq!(t[0].id, "b");
}
