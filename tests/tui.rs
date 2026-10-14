use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rebake::instalador::Presencia;
use rebake::modelo::*;
use rebake::servicio::Novedad;
use rebake::tui::app::{App, Efecto, Pestana};
use rebake::tui::vista;

fn recetario() -> Recetario {
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil {
        nombre: "laburo".into(),
        dir: "~/.claude".into(),
    });
    let mut claude = Item::nuevo("claude", Tipo::Claude);
    claude.estado = Estado::Aprobada;
    let mut codex = Item::nuevo("plugin:codex@openai-codex", Tipo::Plugin);
    codex.estado = Estado::Aprobada;
    codex.perfiles = vec!["laburo".into()];
    codex.fuente = Some(Fuente {
        repo: "https://github.com/openai/codex-plugin-cc".into(),
        via: Via::Metadatos,
        doc: None,
    });
    codex.pasos = vec![Paso {
        cmd: "claude plugin install codex@openai-codex".into(),
        modo: Modo::Auto,
        por_perfil: true,
        cita: Some("/plugin install codex@openai-codex".into()),
        nota: None,
    }];
    let mut crawl = Item::nuevo("skill:crawl4ai", Tipo::Skill);
    crawl.estado = Estado::Excluida;
    crawl.motivo = Some("no me interesa".into());
    let mut status = Item::nuevo("statusline:statusline.sh", Tipo::Statusline);
    status.pista = Some("archivo sin origen conocido: ~/.claude/statusline.sh".into());
    r.items = vec![status, crawl, codex, claude];
    r
}

fn pantalla(app: &App) -> String {
    let mut t = Terminal::new(TestBackend::new(140, 32)).unwrap();
    t.draw(|f| vista::dibujar(f, app)).unwrap();
    let b = t.backend().buffer().clone();
    let mut s = String::new();
    for y in 0..b.area.height {
        for x in 0..b.area.width {
            s.push_str(b[(x, y)].symbol());
        }
        s.push('\n');
    }
    s
}

fn tecla(app: &mut App, c: KeyCode) -> Efecto {
    app.tecla(KeyEvent::new(c, KeyModifiers::NONE))
}

#[test]
fn recetas_muestra_grupos_estados_y_detalle() {
    let mut app = App::nueva(recetario());
    let p = pantalla(&app);
    for esperado in [
        "Claude Code",
        "Plugins",
        "Skills",
        "Statusline",
        "✓ codex@openai-codex",
        "✗ crawl4ai",
        "no me interesa",
        "○ statusline.sh",
    ] {
        assert!(p.contains(esperado), "falta {esperado:?} en:\n{p}");
    }
    app.filtro = "codex".into();
    app.seleccion = 0;
    let p = pantalla(&app);
    assert!(p.contains("github.com/openai/codex-plugin-cc"), "{p}");
    assert!(
        p.contains("cita: /plugin install codex@openai-codex"),
        "{p}"
    );
}

#[test]
fn excluir_desde_el_teclado() {
    let mut app = App::nueva(recetario());
    app.filtro = "statusline".into();
    app.seleccion = 0;
    assert_eq!(tecla(&mut app, KeyCode::Char('x')), Efecto::Nada);
    for c in "no lo uso".chars() {
        tecla(&mut app, KeyCode::Char(c));
    }
    assert_eq!(tecla(&mut app, KeyCode::Enter), Efecto::Guardar);
    let x = app.recetario.item("statusline:statusline.sh").unwrap();
    assert_eq!(x.estado, Estado::Excluida);
    assert_eq!(x.motivo.as_deref(), Some("no lo uso"));
}

#[test]
fn aprobar_pendiente_muestra_error_y_no_guarda() {
    let mut app = App::nueva(recetario());
    app.filtro = "statusline".into();
    assert_eq!(tecla(&mut app, KeyCode::Char('a')), Efecto::Nada);
    assert!(
        app.mensaje
            .as_deref()
            .unwrap()
            .contains("todavía no tiene receta")
    );
}

fn con_fuente(id: &str) -> Item {
    let mut i = Item::nuevo(id, Tipo::Plugin);
    i.fuente = Some(Fuente {
        repo: "https://github.com/ejemplo/repo".into(),
        via: Via::Metadatos,
        doc: None,
    });
    i
}

#[test]
fn investigar_todo_pide_solo_pendientes_con_fuente_y_avisa_las_sin_origen() {
    let mut r = recetario();
    let mut por_revisar = con_fuente("plugin:revisar@x");
    por_revisar.estado = Estado::PorRevisar;
    r.items
        .extend([por_revisar, con_fuente("plugin:pendiente@x")]);
    let mut app = App::nueva(r);
    assert_eq!(
        tecla(&mut app, KeyCode::Char('I')),
        Efecto::Investigar(vec!["plugin:pendiente@x".into()])
    );
    assert!(app.investigando.contains("plugin:pendiente@x"));
    assert!(app.mensaje.as_deref().unwrap().contains("1 sin origen"));
}

#[test]
fn la_cabecera_muestra_las_sin_origen_antes_y_despues_del_lote() {
    use rebake::investigador::Receta;
    let mut r = recetario();
    r.items.push(con_fuente("plugin:pendiente@x"));
    let mut app = App::nueva(r);
    assert!(pantalla(&app).contains("(1 sin origen)"));
    tecla(&mut app, KeyCode::Char('I'));
    let receta = Receta {
        repo: "https://github.com/ejemplo/repo".into(),
        doc: "receta directa".into(),
        pasos: vec![],
        requiere: vec![],
        verificar: None,
    };
    app.registrar_investigacion("plugin:pendiente@x", Ok(receta), "2026-10-14");
    let mensaje = app.mensaje.as_deref().unwrap();
    assert!(
        mensaje.contains("plugin:pendiente@x → por revisar"),
        "{mensaje}"
    );
    assert!(pantalla(&app).contains("(1 sin origen)"));
}

#[test]
fn sin_pendientes_sin_origen_la_cabecera_no_las_nombra() {
    let mut r = recetario();
    r.items.retain(|i| i.id != "statusline:statusline.sh");
    assert!(!pantalla(&App::nueva(r)).contains("sin origen"));
}

#[test]
fn i_sobre_una_receta_sin_origen_pide_el_link_y_esc_no_cambia_nada() {
    let mut app = App::nueva(recetario());
    app.filtro = "statusline".into();
    assert_eq!(tecla(&mut app, KeyCode::Char('i')), Efecto::Nada);
    assert!(app.investigando.is_empty());
    assert!(pantalla(&app).contains("Link del repo"));
    assert_eq!(tecla(&mut app, KeyCode::Esc), Efecto::Nada);
    let x = app.recetario.item("statusline:statusline.sh").unwrap();
    assert_eq!(x.estado, Estado::Pendiente);
    assert!(x.fuente.is_none());
    assert!(app.investigando.is_empty());
}

#[test]
fn enter_vacio_en_el_pedido_del_link_investiga_ese_item() {
    let mut app = App::nueva(recetario());
    app.filtro = "statusline".into();
    tecla(&mut app, KeyCode::Char('r'));
    assert_eq!(
        tecla(&mut app, KeyCode::Enter),
        Efecto::Investigar(vec!["statusline:statusline.sh".into()])
    );
}

#[test]
fn excluir_usa_el_item_elegido_aunque_cambie_la_lista() {
    use rebake::investigador::{Receta, Requisito};
    let mut app = App::nueva(recetario());
    let pos = app
        .visibles()
        .iter()
        .position(|i| i.id == "statusline:statusline.sh")
        .unwrap();
    app.seleccion = pos;
    tecla(&mut app, KeyCode::Char('x'));
    for c in "no lo uso".chars() {
        tecla(&mut app, KeyCode::Char(c));
    }
    // Llega una investigación que agrega un requisito que se ordena antes.
    let receta = Receta {
        repo: "https://github.com/openai/codex-plugin-cc".into(),
        doc: "https://github.com/openai/codex-plugin-cc#install".into(),
        pasos: vec![],
        requiere: vec![Requisito {
            tipo: "marketplace".into(),
            nombre: "openai-codex".into(),
        }],
        verificar: None,
    };
    app.registrar_investigacion("plugin:codex@openai-codex", Ok(receta), "2026-10-07");
    assert_eq!(app.seleccionado().unwrap().id, "statusline:statusline.sh");
    tecla(&mut app, KeyCode::Enter);
    assert_eq!(
        app.recetario
            .item("statusline:statusline.sh")
            .unwrap()
            .estado,
        Estado::Excluida
    );
    assert_ne!(
        app.recetario
            .item("marketplace:openai-codex")
            .unwrap()
            .estado,
        Estado::Excluida
    );
}

#[test]
fn durante_la_instalacion_no_se_reinstala_ni_se_sale_sin_confirmar() {
    let mut app = App::nueva(recetario());
    app.instalando = true;
    assert_eq!(tecla(&mut app, KeyCode::Char('P')), Efecto::Nada);
    assert!(app.mensaje.as_deref().unwrap().contains("en curso"));
    assert_eq!(tecla(&mut app, KeyCode::Char('q')), Efecto::Nada);
    assert_eq!(tecla(&mut app, KeyCode::Char('s')), Efecto::Salir);
}

#[test]
fn al_terminar_la_instalacion_resume_y_va_a_la_checklist() {
    use rebake::instalador::{Evento, Resultado};
    let mut app = App::nueva(recetario());
    app.instalacion = vec![
        ("claude".into(), None),
        ("plugin:codex@openai-codex".into(), None),
    ];
    app.instalando = true;
    app.evento_instalacion(Evento::Inicio("claude".into()));
    app.evento_instalacion(Evento::Fin("claude".into(), Resultado::Ok));
    app.evento_instalacion(Evento::Inicio("plugin:codex@openai-codex".into()));
    app.evento_instalacion(Evento::Fin(
        "plugin:codex@openai-codex".into(),
        Resultado::Fallo("`exit 7` terminó con código Some(7)".into()),
    ));
    assert!(!app.instalando);
    assert_eq!(app.pestana, rebake::tui::app::Pestana::Checklist);
    let m = app.mensaje.clone().unwrap();
    assert!(
        m.contains("1 instaladas")
            && m.contains("1 con error")
            && m.contains("ultima-instalacion.log"),
        "{m}"
    );
    app.pestana = rebake::tui::app::Pestana::Instalacion;
    let p = pantalla(&app);
    assert!(
        p.contains("✗ plugin:codex@openai-codex") && p.contains("código Some(7)"),
        "{p}"
    );
}

const MKT: &str = "marketplace:openai-codex";
const CODEX: &str = "plugin:codex@openai-codex";

/// `recetario()` más un marketplace aprobado del que depende codex. Plan: MKT, CODEX, claude
/// (codex está antes que claude en el recetario y arrastra primero a su requisito).
fn con_plan() -> App {
    let mut r = recetario();
    let mut mkt = Item::nuevo(MKT, Tipo::Marketplace);
    mkt.estado = Estado::Aprobada;
    r.items.push(mkt);
    let codex = r.items.iter_mut().find(|i| i.id == CODEX).unwrap();
    codex.requiere = vec![MKT.into()];
    codex.verificar = Some("claude plugin list | grep codex".into());
    let mut app = App::nueva(r);
    app.pestana = Pestana::Instalacion;
    app
}

fn resaltar(app: &mut App, id: &str) {
    app.seleccion_plan = app.plan().iter().position(|x| x == id).unwrap();
}

#[test]
fn con_url_abre_en_instalacion_y_sin_url_en_recetas() {
    assert_eq!(App::nueva(recetario()).pestana, Pestana::Recetas);
    let app = App::nueva(recetario()).con_arranque(true, Default::default());
    assert_eq!(app.pestana, Pestana::Instalacion);
}

#[test]
fn instalacion_lista_las_aprobadas_marcadas() {
    let app = con_plan();
    assert_eq!(app.plan(), [MKT, CODEX, "claude"]);
    assert_eq!(app.a_instalar(), app.plan());
    let p = pantalla(&app);
    assert!(p.contains(&format!("[x] {MKT}")), "{p}");
    assert!(p.contains(&format!("[x] {CODEX}")), "{p}");
    assert!(p.contains("sin comprobar"), "{p}");
    assert!(!p.contains("crawl4ai"), "{p}");
}

#[test]
fn t_marca_todas_o_ninguna() {
    let mut app = con_plan();
    assert_eq!(tecla(&mut app, KeyCode::Char('t')), Efecto::Nada);
    assert!(app.a_instalar().is_empty());
    tecla(&mut app, KeyCode::Char('t'));
    assert_eq!(app.a_instalar().len(), 3);
}

#[test]
fn desmarcar_arrastra_a_las_que_la_requieren() {
    let mut app = con_plan();
    resaltar(&mut app, MKT);
    tecla(&mut app, KeyCode::Char(' '));
    assert_eq!(app.a_instalar(), ["claude"]);
    assert!(pantalla(&app).contains(&format!("[ ] {CODEX}")));

    resaltar(&mut app, CODEX);
    tecla(&mut app, KeyCode::Char(' '));
    assert_eq!(app.a_instalar(), [MKT, CODEX, "claude"]);
}

#[test]
fn marcar_no_cambia_el_cookbook() {
    let mut app = con_plan();
    let antes = app.recetario.clone();
    resaltar(&mut app, MKT);
    assert_eq!(tecla(&mut app, KeyCode::Char(' ')), Efecto::Nada);
    assert_eq!(tecla(&mut app, KeyCode::Char('t')), Efecto::Nada);
    assert_eq!(app.recetario, antes);
}

#[test]
fn p_confirma_cuantas_de_cuantas_e_instala_las_marcadas() {
    let mut app = con_plan();
    resaltar(&mut app, "claude");
    tecla(&mut app, KeyCode::Char(' '));
    assert_eq!(tecla(&mut app, KeyCode::Char('P')), Efecto::Nada);
    assert!(
        app.mensaje.as_deref().unwrap().contains("2 de 3"),
        "{:?}",
        app.mensaje
    );
    assert_eq!(tecla(&mut app, KeyCode::Char('s')), Efecto::Instalar);
    assert_eq!(app.a_instalar(), [MKT, CODEX]);
}

#[test]
fn abrir_instalacion_no_pide_comprobar() {
    let mut app = App::nueva(recetario());
    assert_eq!(tecla(&mut app, KeyCode::Char('3')), Efecto::Nada);
    assert!(app.presencia.is_empty() && !app.comprobando);
}

#[test]
fn c_pide_comprobar_y_muestra_el_estado_de_cada_fila() {
    let mut app = con_plan();
    app.recetario.perfiles.push(Perfil {
        nombre: "personal".into(),
        dir: "~/.claude-personal".into(),
    });
    app.recetario.item_mut(CODEX).unwrap().perfiles = vec!["laburo".into(), "personal".into()];
    assert_eq!(tecla(&mut app, KeyCode::Char('c')), Efecto::Comprobar);
    assert!(app.comprobando);
    assert!(pantalla(&app).contains("comprobando"));

    app.registrar_presencia(CODEX, Presencia::Falta(vec!["personal".into()]));
    app.registrar_presencia(MKT, Presencia::SinComprobacion);
    app.registrar_presencia("claude", Presencia::NoSePudo("tope".into()));
    app.terminar_comprobacion();
    let p = pantalla(&app);
    assert!(p.contains("falta en personal"), "{p}");
    assert!(p.contains("no se puede saber"), "{p}");
    assert!(p.contains("no se pudo comprobar"), "{p}");
    assert!(!app.comprobando);
    assert_eq!(app.a_instalar().len(), 3);
}

#[test]
fn instalada_pierde_la_casilla_y_no_se_instala() {
    let mut app = con_plan();
    app.registrar_presencia(MKT, Presencia::Instalada);
    let p = pantalla(&app);
    assert!(
        p.contains(&format!("✓  {MKT}")) || p.contains(&format!("✓ {MKT}")),
        "{p}"
    );
    assert!(p.contains("instalada"), "{p}");
    assert_eq!(app.a_instalar(), [CODEX, "claude"]);

    resaltar(&mut app, MKT);
    tecla(&mut app, KeyCode::Char(' '));
    assert_eq!(app.a_instalar(), [CODEX, "claude"]);
}

#[test]
fn el_detalle_muestra_los_comandos_y_la_comprobacion() {
    let mut app = con_plan();
    resaltar(&mut app, CODEX);
    let p = pantalla(&app);
    assert!(
        p.contains("claude plugin install codex@openai-codex"),
        "{p}"
    );
    assert!(p.contains("claude plugin list | grep codex"), "{p}");
    assert!(p.contains(MKT), "{p}");
}

#[test]
fn novedades_se_ven_en_la_lista() {
    let mut app = con_plan();
    app.novedades.insert(CODEX.into(), Novedad::Cambio);
    app.novedades.insert(MKT.into(), Novedad::Nueva);
    let p = pantalla(&app);
    assert!(p.contains("cambió") && p.contains("nueva"), "{p}");
}

#[test]
fn sin_aprobadas_explica_que_hacer() {
    let mut r = Recetario::nuevo();
    r.items.push(Item::nuevo("plugin:a@b", Tipo::Plugin));
    let mut app = App::nueva(r);
    app.pestana = Pestana::Instalacion;
    assert!(pantalla(&app).contains("No hay recetas aprobadas"));
    assert_eq!(tecla(&mut app, KeyCode::Char('P')), Efecto::Nada);
}

#[test]
fn c_no_lanza_otra_comprobacion_si_hay_una_corriendo() {
    let mut app = con_plan();
    assert_eq!(tecla(&mut app, KeyCode::Char('c')), Efecto::Comprobar);
    assert_eq!(tecla(&mut app, KeyCode::Char('c')), Efecto::Nada);
    app.terminar_comprobacion();
    assert_eq!(tecla(&mut app, KeyCode::Char('c')), Efecto::Comprobar);
}

#[test]
fn un_ciclo_en_requiere_se_informa_al_instalar() {
    let mut app = con_plan();
    app.recetario.item_mut(MKT).unwrap().requiere = vec![CODEX.into()];
    assert_eq!(tecla(&mut app, KeyCode::Char('P')), Efecto::Nada);
    assert!(
        app.mensaje.as_deref().unwrap().contains("ciclo"),
        "{:?}",
        app.mensaje
    );
    assert!(pantalla(&app).contains("ciclo"));
}

#[test]
fn instalar_actualiza_lo_comprobado() {
    use rebake::instalador::{Evento, Resultado};
    let mut app = con_plan();
    app.registrar_presencia(MKT, Presencia::Falta(vec![]));
    app.registrar_presencia(CODEX, Presencia::Falta(vec!["laburo".into()]));
    app.evento_instalacion(Evento::Fin(MKT.into(), Resultado::Salteado));
    app.evento_instalacion(Evento::Fin(CODEX.into(), Resultado::Ok));
    assert_eq!(app.presencia.get(MKT), Some(&Presencia::Instalada));
    assert_eq!(app.presencia.get(CODEX), None);
}

#[test]
fn con_version_nueva_la_cabecera_avisa_y_u_actualiza() {
    let mut app = App::nueva(recetario());
    app.version_nueva = Some("99.0.0".into());
    let p = pantalla(&app);
    assert!(p.contains("rebake 99.0.0 disponible"), "{p}");
    assert!(p.contains("u actualiza"), "{p}");
    assert_eq!(
        tecla(&mut app, KeyCode::Char('U')),
        Efecto::Actualizar("99.0.0".into())
    );
}

#[test]
fn u_sin_version_nueva_avisa_que_esta_al_dia() {
    let mut app = App::nueva(recetario());
    assert_eq!(tecla(&mut app, KeyCode::Char('U')), Efecto::Nada);
    assert_eq!(app.mensaje.as_deref(), Some("rebake está al día"));
}

#[test]
fn u_con_instalacion_en_curso_no_actualiza() {
    let mut app = App::nueva(recetario());
    app.version_nueva = Some("99.0.0".into());
    app.instalando = true;
    assert_eq!(tecla(&mut app, KeyCode::Char('U')), Efecto::Nada);
    assert!(app.mensaje.unwrap().contains("esperá"));
}

fn con_un_item(item: Item) -> App {
    let mut r = Recetario::nuevo();
    r.items = vec![item];
    App::nueva(r)
}

#[test]
fn sin_origen_va_contra_el_borde_del_panel() {
    let p = pantalla(&App::nueva(recetario()));
    let linea = p.lines().find(|l| l.contains("statusline.sh")).unwrap();
    assert!(linea.contains("sin origen│"), "{linea}");
}

#[test]
fn el_detalle_dice_que_falta_en_cada_estado() {
    let mut sin_origen = Item::nuevo("statusline:s.sh", Tipo::Statusline);
    sin_origen.pista = Some("archivo sin origen conocido: ~/.claude/s.sh".into());
    let mut por_revisar = con_fuente("plugin:b@x");
    por_revisar.estado = Estado::PorRevisar;
    let mut aprobada = con_fuente("plugin:c@x");
    aprobada.estado = Estado::Aprobada;
    let mut excluida = con_fuente("plugin:d@x");
    excluida.estado = Estado::Excluida;
    for (item, texto) in [
        (
            con_fuente("plugin:a@x"),
            "todavía no hay receta: `i` la arma",
        ),
        (sin_origen, "no sé de qué repo viene: `i` te pide el link"),
        (por_revisar, "revisá los pasos y la cita, y `a` la aprueba"),
        (aprobada, "en una PC nueva se instala con esto"),
        (excluida, "No se instala"),
    ] {
        let p = pantalla(&con_un_item(item));
        assert!(p.contains(texto), "falta «{texto}» en:\n{p}");
    }
}

#[test]
fn la_cabecera_y_el_detalle_dicen_sin_receta() {
    let p = pantalla(&con_un_item(con_fuente("plugin:a@x")));
    assert!(p.contains("○ 1 sin receta"), "{p}");
    assert!(p.contains("Estado: sin receta"), "{p}");
    assert!(!p.contains("pendiente"), "{p}");
}

#[test]
fn el_siguiente_paso_aparece_hasta_que_hay_una_receta() {
    let mut r = Recetario::nuevo();
    r.items = vec![con_fuente("plugin:a@x"), con_fuente("plugin:b@x")];
    let p = pantalla(&App::nueva(r.clone()));
    assert!(
        p.contains("Siguiente: `I` arma 2 recetas (las que tienen origen)"),
        "{p}"
    );
    r.items[1].estado = Estado::PorRevisar;
    assert!(!pantalla(&App::nueva(r)).contains("Siguiente:"));
}

#[test]
fn u_minuscula_tambien_actualiza() {
    let mut app = App::nueva(recetario());
    app.version_nueva = Some("99.0.0".into());
    assert_eq!(
        tecla(&mut app, KeyCode::Char('u')),
        Efecto::Actualizar("99.0.0".into())
    );
}
