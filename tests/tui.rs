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
    pantalla_en(app, 140, 32)
}

fn pantalla_en(app: &App, ancho: u16, alto: u16) -> String {
    let mut t = Terminal::new(TestBackend::new(ancho, alto)).unwrap();
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
        p.contains("según el repo: /plugin install codex@openai-codex"),
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
        (por_revisar, "revisá qué va a ejecutar y `a` la aprueba"),
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
fn el_siguiente_paso_sigue_al_camino() {
    let mut r = Recetario::nuevo();
    r.items = vec![con_fuente("plugin:a@x"), con_fuente("plugin:b@x")];
    let p = pantalla(&App::nueva(r.clone()));
    assert!(
        p.contains("Siguiente: `t` arma 2 recetas (las que tienen origen)"),
        "{p}"
    );
    r.items[0].estado = Estado::PorRevisar;
    r.items[1].estado = Estado::PorRevisar;
    let pie = pie(&App::nueva(r));
    assert!(
        pie[2].contains("Siguiente: 2 recetas por revisar: elegí una y `a` la aprueba"),
        "{pie:?}"
    );
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

#[test]
fn mientras_investiga_la_pantalla_lo_dice_con_texto() {
    let mut app = con_un_item(con_fuente("plugin:a@x"));
    tecla(&mut app, KeyCode::Char('i'));
    let p = pantalla(&app);
    assert!(p.contains("Armando la receta…"), "{p}");
    assert!(!p.contains("todavía no hay receta"), "{p}");
    assert!(p.contains("… a@x"), "{p}");
    assert!(p.contains("… 1 investigando"), "{p}");
}

fn pie(app: &App) -> Vec<String> {
    let p = pantalla(app);
    let lineas: Vec<String> = p.lines().map(|l| l.trim_end().to_string()).collect();
    lineas[lineas.len() - 3..].to_vec()
}

#[test]
fn el_pie_muestra_solo_las_teclas_de_la_receta_elegida() {
    let mut sin_origen = Item::nuevo("statusline:s.sh", Tipo::Statusline);
    sin_origen.pista = Some("archivo sin origen conocido: ~/.claude/s.sh".into());
    let mut por_revisar = con_fuente("plugin:b@x");
    por_revisar.estado = Estado::PorRevisar;
    let mut aprobada = con_fuente("plugin:c@x");
    aprobada.estado = Estado::Aprobada;
    let mut excluida = con_fuente("plugin:d@x");
    excluida.estado = Estado::Excluida;
    for (item, esperado) in [
        (con_fuente("plugin:a@x"), " i  armar receta   x  excluir"),
        (sin_origen, " i  pasar el link   x  excluir"),
        (
            por_revisar,
            " a  aprobar   e  editar   r  reinvestigar   x  excluir",
        ),
        (aprobada, " e  editar   r  reinvestigar   x  excluir"),
        (excluida, " e  editar"),
    ] {
        let pie = pie(&con_un_item(item));
        assert!(
            pie[0].contains("Esta receta:") && pie[0].ends_with(esperado),
            "{pie:?}"
        );
        assert!(
            pie[1].contains("Todas:") && pie[1].contains(" t  investigar todas"),
            "{pie:?}"
        );
    }
}

#[test]
fn el_mensaje_va_en_su_propia_linea() {
    let mut app = con_un_item(con_fuente("plugin:a@x"));
    tecla(&mut app, KeyCode::Char('a'));
    let pie = pie(&app);
    assert!(pie[2].contains("todavía no tiene receta"), "{pie:?}");
    assert!(!pie[0].contains("todavía") && !pie[1].contains("todavía"));
}

#[test]
fn ninguna_tecla_depende_de_shift() {
    for (minuscula, mayuscula) in [('t', 'I'), ('p', 'P')] {
        let mut r = recetario();
        r.items.push(con_fuente("plugin:pendiente@x"));
        let (mut a, mut b) = (App::nueva(r.clone()), App::nueva(r));
        assert_eq!(
            tecla(&mut a, KeyCode::Char(minuscula)),
            tecla(&mut b, KeyCode::Char(mayuscula)),
            "{minuscula} / {mayuscula}"
        );
        assert_eq!(a.mensaje, b.mensaje, "{minuscula} / {mayuscula}");
        assert_eq!(a.modo, b.modo, "{minuscula} / {mayuscula}");
    }
    let pie = pie(&App::nueva(recetario()));
    assert!(pie[1].contains(" t  investigar todas"), "{pie:?}");
    assert!(pie[1].contains(" p  instalar"), "{pie:?}");
}

fn camino(app: &App) -> String {
    pantalla(app).lines().nth(1).unwrap().to_string()
}

#[test]
fn recetas_muestra_la_leyenda_de_marcas() {
    let p = pantalla(&App::nueva(recetario()));
    for marca in [
        "✓ aprobada",
        "? por revisar",
        "○ sin receta",
        "✗ excluida",
        "… armando",
        "(ausente) no está en esta PC",
    ] {
        assert!(p.contains(marca), "falta {marca:?} en:\n{p}");
    }
}

#[test]
fn el_camino_resalta_el_primer_paso_con_trabajo() {
    assert!(camino(&App::nueva(Recetario::nuevo())).contains("[escanear]"));
    let pendiente = con_un_item(con_fuente("plugin:a@x"));
    assert!(camino(&pendiente).contains("✓ escanear › [armar recetas]"));
    let mut revisar = con_fuente("plugin:a@x");
    revisar.estado = Estado::PorRevisar;
    assert!(camino(&con_un_item(revisar)).contains("› [aprobar] ›"));
    let mut aprobada = con_fuente("plugin:a@x");
    aprobada.estado = Estado::Aprobada;
    let c = camino(&con_un_item(aprobada));
    assert!(c.contains("✓ aprobar › [instalar]"), "{c}");
}

#[test]
fn el_plugin_oficial_se_lista_sin_su_marketplace_y_lo_largo_termina_en_elipsis() {
    let mut r = Recetario::nuevo();
    r.items = vec![
        con_fuente("plugin:claude-md-management@claude-plugins-official"),
        con_fuente("plugin:un-nombre-larguisimo-que-no-entra-en-la-lista@x"),
    ];
    let app = App::nueva(r);
    let p = pantalla_en(&app, 70, 24);
    let oficial = p
        .lines()
        .find(|l| l.contains("○ claude-md-management"))
        .unwrap_or_else(|| panic!("{p}"));
    assert!(!oficial.contains("○ claude-md-management@"), "{oficial}");
    let largo = p.lines().find(|l| l.contains("○ un-nombre")).unwrap();
    assert!(largo.contains("…│"), "{largo}");
    // El título del detalle no entra en 70 columnas: se corta con `…`, y entero en 140.
    assert!(
        p.lines()
            .any(|l| l.contains("┐┌plugin:claude-md-management@") && l.ends_with("…┐")),
        "{p}"
    );
    assert!(
        pantalla(&app).contains("plugin:claude-md-management@claude-plugins-official"),
        "{p}"
    );
}

#[test]
fn un_nombre_ancho_no_corre_la_columna_de_estado() {
    let mut item = Item::nuevo("skill:日本語の名前", Tipo::Skill);
    item.pista = Some("skill sin origen conocido: ~/.claude/skills/x".into());
    let p = pantalla(&con_un_item(item));
    let linea = p
        .lines()
        .find(|l| l.contains("○ 日"))
        .unwrap_or_else(|| panic!("{p}"));
    assert!(linea.contains("sin origen│"), "{linea}");
}

#[test]
fn el_detalle_habla_en_palabras_simples() {
    let mut mkt = Item::nuevo("marketplace:ponytail", Tipo::Marketplace);
    mkt.estado = Estado::PorRevisar;
    mkt.fuente = Some(Fuente {
        repo: "https://github.com/DietrichGebert/ponytail".into(),
        via: Via::Metadatos,
        doc: Some("receta directa: instalador de plugins de Claude Code".into()),
    });
    mkt.verificar = Some("claude plugin marketplace list".into());
    mkt.pasos = vec![Paso {
        cmd: "claude plugin marketplace add https://github.com/DietrichGebert/ponytail".into(),
        modo: Modo::Auto,
        por_perfil: true,
        cita: None,
        nota: Some("receta directa: instalador de plugins de Claude Code".into()),
    }];
    let p = pantalla(&con_un_item(mkt));
    for esperado in [
        "Viene de: https://github.com/DietrichGebert/ponytail (lo registró su instalador)",
        "Cómo se arma: instalador de plugins de Claude Code, sin Claude",
        "Para saber si ya está: claude plugin marketplace list",
        "Qué va a ejecutar",
        "en cada perfil, automático",
    ] {
        assert!(p.contains(esperado), "falta {esperado:?} en:\n{p}");
    }
    for viejo in ["metadatos", "Doc:", "Fuente:", "por perfil", "nota:"] {
        assert!(!p.contains(viejo), "sobra {viejo:?} en:\n{p}");
    }
}

#[test]
fn en_80_columnas_la_cabecera_conserva_los_contadores() {
    let mut app = App::nueva(recetario());
    app.recetario.perfiles.push(Perfil {
        nombre: "personal".into(),
        dir: "~/.claude-personal".into(),
    });
    app.version_nueva = Some("99.0.0".into());
    let p = pantalla_en(&app, 80, 24);
    let cabecera = p.lines().next().unwrap();
    for contador in ["✓2", "?0", "○1", "✗1", "u actualiza"] {
        assert!(
            cabecera.contains(contador),
            "falta {contador:?} en {cabecera:?}"
        );
    }
}

#[test]
fn instalacion_sin_aprobadas_solo_ofrece_lo_que_sirve() {
    let mut app = con_un_item(con_fuente("plugin:a@x"));
    tecla(&mut app, KeyCode::Char('3'));
    let pie = pie(&app);
    for sobra in [" t ", " c ", " p ", "espacio"] {
        assert!(!pie[0].contains(sobra), "sobra {sobra:?} en {pie:?}");
    }
    assert!(
        pie[0].contains("1 volver a Recetas") && pie[0].contains("q salir"),
        "{pie:?}"
    );
}

fn colores(app: &App) -> Vec<ratatui::style::Color> {
    let mut t = Terminal::new(TestBackend::new(140, 32)).unwrap();
    t.draw(|f| vista::dibujar(f, app)).unwrap();
    t.backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.fg)
        .collect()
}

#[test]
fn los_estados_tienen_color_salvo_con_no_color() {
    use ratatui::style::Color;
    let mut app = App::nueva(recetario());
    assert!(colores(&app).contains(&Color::Green));
    app.color = false;
    assert!(colores(&app).iter().all(|c| *c == Color::Reset));
    assert!(pantalla(&app).contains("✓ codex@openai-codex"));
}

#[test]
fn la_ayuda_se_abre_con_signo_y_vuelve_a_la_misma_seleccion() {
    let mut app = App::nueva(recetario());
    app.seleccion = 2;
    assert_eq!(tecla(&mut app, KeyCode::Char('?')), Efecto::Nada);
    let p = pantalla(&app);
    assert!(
        p.contains("Ayuda") && p.contains("armar recetas (i, t)"),
        "{p}"
    );
    tecla(&mut app, KeyCode::Char('j'));
    tecla(&mut app, KeyCode::Esc);
    assert_eq!(app.modo, rebake::tui::app::Modo::Normal);
    assert_eq!(app.seleccion, 2);
    assert!(!pantalla(&app).contains("armar recetas (i, t)"));
}

#[test]
fn el_signo_en_el_filtro_se_escribe() {
    let mut app = App::nueva(recetario());
    tecla(&mut app, KeyCode::Char('/'));
    tecla(&mut app, KeyCode::Char('?'));
    assert!(matches!(&app.modo, rebake::tui::app::Modo::Escribiendo { texto, .. } if texto == "?"));
}

#[test]
fn cada_pestana_dice_para_que_es() {
    let mut app = App::nueva(recetario());
    assert!(pantalla(&app).contains("En esta PC: decidí cómo se instala cada cosa"));
    tecla(&mut app, KeyCode::Char('2'));
    let p = pantalla(&app);
    assert!(
        p.contains("Después de instalar: lo que rebake no hace por vos"),
        "{p}"
    );
    assert!(p.contains("[2]Pendientes"), "{p}");
    tecla(&mut app, KeyCode::Char('3'));
    assert!(pantalla(&app).contains("En una PC nueva: instalá las recetas aprobadas"));
}
