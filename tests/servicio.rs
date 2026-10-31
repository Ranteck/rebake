mod comun;
use comun::{HomeFalso, PacmanFalso, entorno, perfil};
use rebake::modelo::{Estado, Fuente, Item, Modo, Paso, Recetario, Tipo, Via};
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

#[test]
fn escanear_quita_perfiles_existentes_sin_marcador_y_sus_referencias() {
    if !comun::aislado("escanear_quita_perfiles_existentes_sin_marcador_y_sus_referencias") {
        return;
    }
    use rebake::modelo::{Ajuste, TituloClaudeMd};
    let h = HomeFalso::nuevo();
    for dir in [".claude", ".claude-personal", "perfiles/equipo"] {
        h.escribir(&format!("{dir}/settings.json"), "{}");
        h.escribir(&format!("{dir}/.claude.json"), "{}");
    }
    h.escribir("almacen/settings.json", "{}");
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        perfil("claude", "~/.claude"),
        perfil("personal", "~/.claude-personal"),
        perfil("datos", "~/almacen"),
        perfil("viaje", "~/.claude-viaje"),
        perfil("equipo", "~/perfiles/equipo"),
    ];
    r.checklist.ajustes = (0..11)
        .map(|n| Ajuste {
            perfil: "datos".into(),
            clave: format!("dato{n}"),
            valor: "valor".into(),
        })
        .chain(std::iter::once(Ajuste {
            perfil: "viaje".into(),
            clave: "theme".into(),
            valor: "dark".into(),
        }))
        .collect();
    r.checklist.titulos = ["datos", "viaje"]
        .map(|p| TituloClaudeMd {
            perfil: p.into(),
            texto: "Propio".into(),
        })
        .to_vec();
    let mut item = aprobada("skill:compartida", "true");
    item.perfiles = vec!["datos".into(), "personal".into(), "viaje".into()];
    let pasos = item.pasos.clone();
    r.items.push(item);
    let pacman = PacmanFalso(vec![]);

    let resumen = servicio::escanear(&mut r, &entorno(&h, &pacman)).unwrap();

    assert_eq!(
        r.perfiles
            .iter()
            .map(|p| p.nombre.as_str())
            .collect::<Vec<_>>(),
        ["claude", "personal", "viaje", "equipo"]
    );
    assert_eq!(r.checklist.ajustes.len(), 1);
    assert_eq!(r.checklist.ajustes[0].perfil, "viaje");
    assert_eq!(r.checklist.titulos.len(), 1);
    assert_eq!(r.checklist.titulos[0].perfil, "viaje");
    let item = r.item("skill:compartida").unwrap();
    assert_eq!(item.perfiles, ["personal", "viaje"]);
    assert_eq!(item.estado, Estado::Aprobada);
    assert_eq!(item.pasos, pasos);
    assert!(
        resumen
            .avisos
            .iter()
            .any(|a| a.contains("datos no es un perfil de Claude Code: quitado con 11 ajustes")),
        "{:?}",
        resumen.avisos
    );
    let antes = r.clone();
    let otra = servicio::escanear(&mut r, &entorno(&h, &pacman)).unwrap();
    assert_eq!(r, antes);
    assert!(!otra.avisos.iter().any(|a| a.contains("quitado")));
    assert!(h.ruta().join("almacen/settings.json").is_file());
}

#[test]
fn perder_el_ultimo_perfil_impide_instalar_la_receta_globalmente() {
    if !comun::aislado("perder_el_ultimo_perfil_impide_instalar_la_receta_globalmente") {
        return;
    }
    let h = HomeFalso::nuevo();
    h.escribir("datos/settings.json", "{}");
    let mut r = Recetario::nuevo();
    r.perfiles.push(perfil("datos", "~/datos"));
    for (id, estado) in [
        ("skill:sola", Estado::Aprobada),
        ("skill:excluida", Estado::Excluida),
    ] {
        let mut item = aprobada(id, "touch ejecucion-accidental");
        item.estado = estado;
        item.perfiles.push("datos".into());
        item.pasos[0].por_perfil = true;
        r.items.push(item);
    }
    let pacman = PacmanFalso(vec![]);
    servicio::escanear(&mut r, &entorno(&h, &pacman)).unwrap();
    assert_eq!(r.item("skill:sola").unwrap().estado, Estado::Pendiente);
    assert!(r.item("skill:sola").unwrap().perfiles.is_empty());
    assert_eq!(r.item("skill:excluida").unwrap().estado, Estado::Excluida);
    assert!(rebake::instalador::planificar(&r).unwrap().is_empty());
}

#[test]
fn escanear_conserva_el_perfil_si_no_puede_verificar_su_marcador() {
    if !comun::aislado("escanear_conserva_el_perfil_si_no_puede_verificar_su_marcador") {
        return;
    }
    let h = HomeFalso::nuevo();
    h.escribir("perfil/settings.json", "{}");
    let marcador = h.ruta().join("perfil/.claude.json");
    h.enlazar("perfil/.claude.json", &marcador);
    let mut r = Recetario::nuevo();
    r.perfiles.push(perfil("equipo", "~/perfil"));
    let pacman = PacmanFalso(vec![]);
    let resumen = servicio::escanear(&mut r, &entorno(&h, &pacman)).unwrap();
    assert_eq!(r.perfiles, [perfil("equipo", "~/perfil")]);
    assert!(
        resumen
            .avisos
            .iter()
            .any(|a| a.contains("no pude verificar el perfil equipo") && a.contains("se conserva")),
        "{:?}",
        resumen.avisos
    );
}

#[test]
fn sincronizados_no_entran_en_trabajos_ni_en_la_cuenta_sin_origen() {
    let h = HomeFalso::nuevo();
    let mut r = Recetario::nuevo();
    let mut i = Item::nuevo("skill:pdf", Tipo::Skill);
    i.perfiles = vec!["laburo".into()];
    i.sincronizado_en = vec!["laburo".into()];
    r.items.push(i);
    assert!(servicio::sin_origen(&r).is_empty());
    assert!(servicio::trabajos(&r, &[], h.ruta()).unwrap().is_empty());
    let err = servicio::trabajos(&r, &["skill:pdf".into()], h.ruta())
        .err()
        .unwrap()
        .to_string();
    assert!(err.contains("sincroniza"), "{err}");
    assert!(err.contains("Claude"), "{err}");
}

#[test]
fn trabajo_mixto_no_describe_el_destino_sincronizado() {
    let h = HomeFalso::nuevo();
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        perfil("cuenta", "~/.claude"),
        perfil("propio", "~/.claude-personal"),
    ];
    let mut i = Item::nuevo("statusline:mixta", Tipo::Statusline);
    i.perfiles = vec!["cuenta".into(), "propio".into()];
    i.sincronizado_en = vec!["cuenta".into()];
    r.items.push(i);
    let trabajos = servicio::trabajos(&r, &["statusline:mixta".into()], h.ruta()).unwrap();
    assert_eq!(trabajos.len(), 1);
    assert!(trabajos[0].prompt.contains("propio"));
    assert!(!trabajos[0].prompt.contains("cuenta"));
}

#[test]
fn perder_perfil_de_cuenta_conserva_marca_y_no_habilita_instalacion_global() {
    if !comun::aislado("perder_perfil_de_cuenta_conserva_marca_y_no_habilita_instalacion_global") {
        return;
    }
    let h = HomeFalso::nuevo();
    h.escribir(".claude-mem/settings.json", "{}");
    let mut r = Recetario::nuevo();
    r.perfiles.push(perfil("mem", "~/.claude-mem"));
    let mut i = Item::nuevo("skill:pdf", Tipo::Skill);
    i.perfiles = vec!["mem".into()];
    i.sincronizado_en = vec!["mem".into()];
    i.estado = Estado::Aprobada;
    r.items.push(i);
    servicio::escanear(&mut r, &entorno(&h, &PacmanFalso(vec![]))).unwrap();
    assert!(r.perfiles.is_empty());
    assert!(r.items[0].perfiles.is_empty());
    assert_eq!(r.items[0].sincronizado_en, ["mem"]);
    assert!(rebake::instalador::planificar(&r).unwrap().is_empty());
    let p = h.ruta().join("cookbook.toml");
    rebake::archivo::guardar(&p, &r).unwrap();
    assert!(rebake::archivo::leer(&p).unwrap().items[0].es_sincronizado());
}

#[test]
fn directa_nueva_queda_por_revisar_sin_claude_y_existente_queda_intacta() {
    if !comun::aislado("directa_nueva_queda_por_revisar_sin_claude_y_existente_queda_intacta") {
        return;
    }
    let h = HomeFalso::nuevo();
    h.escribir(".claude/settings.json", "{}");
    h.escribir(".claude/.claude.json", "{}");
    h.escribir(
        ".claude/plugins/known_marketplaces.json",
        r#"{"oficial": {"source": {"source": "github", "repo": "anthropics/claude-plugins"}}}"#,
    );
    let mut r = Recetario::nuevo();
    r.perfiles.push(perfil("laburo", "~/.claude"));
    let mut existente = Item::nuevo("skill:propia", Tipo::Skill);
    existente.estado = Estado::Pendiente;
    existente.perfiles.push("laburo".into());
    r.items.push(existente);

    let pacman = PacmanFalso(vec![]);
    let resumen = servicio::escanear(&mut r, &entorno(&h, &pacman)).unwrap();

    let mkt = r.item("marketplace:oficial").unwrap();
    assert_eq!(mkt.estado, Estado::PorRevisar);
    assert!(!mkt.pasos.is_empty());
    assert!(mkt.investigado.is_some());
    assert_eq!(resumen.directas, 1);

    let prop = r.item("skill:propia").unwrap();
    assert_eq!(prop.estado, Estado::Pendiente);
    assert!(prop.pasos.is_empty());
}

#[test]
fn directa_pendiente_existente_se_arma_gratis_y_sin_origen_queda_intacta() {
    if !comun::aislado("directa_pendiente_existente_se_arma_gratis_y_sin_origen_queda_intacta") {
        return;
    }
    let h = HomeFalso::nuevo();
    let mut r = Recetario::nuevo();
    let mut directa = Item::nuevo("plugin:conocido@oficial", Tipo::Plugin);
    directa.fuente = Some(Fuente {
        repo: "https://github.com/anthropics/claude-plugins".into(),
        via: Via::Metadatos,
        doc: None,
    });
    directa.ausente = true;
    let mut sin_origen = Item::nuevo("skill:propia", Tipo::Skill);
    sin_origen.ausente = true;
    let mut aprobada = directa.clone();
    aprobada.id = "plugin:aprobado@oficial".into();
    aprobada.estado = Estado::Aprobada;
    let mut excluida = directa.clone();
    excluida.id = "plugin:excluido@oficial".into();
    excluida.estado = Estado::Excluida;
    r.items = vec![
        directa,
        sin_origen.clone(),
        aprobada.clone(),
        excluida.clone(),
    ];

    let resumen = servicio::escanear(&mut r, &entorno(&h, &PacmanFalso(vec![]))).unwrap();
    assert_eq!(resumen.directas, 1);
    let armada = r.item("plugin:conocido@oficial").unwrap();
    assert_eq!(armada.estado, Estado::PorRevisar);
    assert!(
        armada.pasos[0]
            .cmd
            .contains("claude plugin install conocido@oficial")
    );
    assert_eq!(r.item(&sin_origen.id), Some(&sin_origen));
    assert_eq!(r.item(&aprobada.id), Some(&aprobada));
    assert_eq!(r.item(&excluida.id), Some(&excluida));
    let otra = servicio::escanear(&mut r, &entorno(&h, &PacmanFalso(vec![]))).unwrap();
    assert_eq!(otra.directas, 0);
}
