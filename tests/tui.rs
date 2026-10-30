use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rebake::instalador::Presencia;
use rebake::modelo::*;
use rebake::servicio::Novedad;
use rebake::tui::app::{App, ClaseFila, Efecto, Pestana};
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
fn hook_vinculado_no_ofrece_corridas_ni_aprobacion() {
    for estado in [Estado::Pendiente, Estado::PorRevisar, Estado::Aprobada] {
        let mut r = Recetario::nuevo();
        let mut hook = Item::nuevo("hook:tool", Tipo::Hook);
        hook.estado = estado;
        hook.instalado_por = Some(InstaladorHook::Externo("tool".into()));
        r.items.push(hook);
        let mut app = App::nueva(r);
        assert_eq!(app.por_armar(), (vec![], vec![], vec![]));
        for key in ['i', 'r', 'a', ' '] {
            assert_eq!(tecla(&mut app, KeyCode::Char(key)), Efecto::Nada);
            assert_eq!(app.recetario.items[0].estado, estado);
        }
    }
}

#[test]
fn recetas_muestra_grupos_estados_y_detalle() {
    let mut app = App::nueva(recetario());
    tecla(&mut app, KeyCode::Char('o'));
    let p = pantalla(&app);
    for esperado in [
        "Claude Code",
        "Plugins",
        "Skills",
        "Statusline",
        "✓ codex@openai-codex",
        "✗ crawl4ai",
        "no me interesa",
        "? statusline.sh",
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
    assert_eq!(tecla(&mut app, KeyCode::Char('x')), Efecto::Guardar);
    let x = app.recetario.item("statusline:statusline.sh").unwrap();
    assert_eq!(x.estado, Estado::Excluida);
    assert!(x.motivo.is_none());
    assert!(app.visibles().is_empty());
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
fn investigar_todo_investiga_las_con_fuente_y_busca_el_origen_de_las_otras() {
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
    // cuidar-cuota: la directa sigue inmediata; la búsqueda exige confirmar la tanda.
    assert!(app.buscando.is_empty());
    assert_eq!(
        tecla(&mut app, KeyCode::Char('s')),
        Efecto::Buscar(vec!["statusline:statusline.sh".into()])
    );
    assert!(app.investigando.contains("plugin:pendiente@x"));
    assert!(app.buscando.contains("statusline:statusline.sh"));
    let mensaje = app.mensaje.as_deref().unwrap();
    assert!(mensaje.contains("buscando de dónde vienen 1"), "{mensaje}");
}

#[test]
fn la_cabecera_muestra_las_sin_origen_antes_y_despues_del_lote() {
    use rebake::investigador::Receta;
    let mut r = recetario();
    r.items.push(con_fuente("plugin:pendiente@x"));
    let mut app = App::nueva(r);
    assert!(pantalla(&app).contains("? 1 sin origen"));
    tecla(&mut app, KeyCode::Char('I'));
    let receta = Receta {
        registro_hooks: None,
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
    assert!(pantalla(&app).contains("? 1 sin origen"));
}

#[test]
fn sin_pendientes_sin_origen_la_cabecera_no_las_nombra() {
    let mut r = recetario();
    r.items.retain(|i| i.id != "statusline:statusline.sh");
    let p = pantalla(&App::nueva(r));
    assert!(!p.lines().next().unwrap().contains("sin origen"), "{p}");
}

#[test]
fn i_sobre_una_receta_sin_origen_busca_y_despues_pide_el_link() {
    let mut app = App::nueva(recetario());
    app.filtro = "statusline".into();
    assert_eq!(
        tecla(&mut app, KeyCode::Char('i')),
        Efecto::Buscar(vec!["statusline:statusline.sh".into()])
    );
    assert!(app.investigando.is_empty());
    app.registrar_busqueda(
        "statusline:statusline.sh",
        Ok(Posible {
            repo: None,
            motivo: "nada claro".into(),
            descartado: false,
        }),
    );
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
fn enter_vacio_en_el_pedido_del_link_vuelve_a_buscar_el_origen() {
    let mut r = recetario();
    r.item_mut("statusline:statusline.sh").unwrap().posible = Some(Posible {
        repo: None,
        motivo: "nada claro".into(),
        descartado: false,
    });
    let mut app = App::nueva(r);
    app.filtro = "statusline".into();
    tecla(&mut app, KeyCode::Char('r'));
    assert_eq!(
        tecla(&mut app, KeyCode::Enter),
        Efecto::Buscar(vec!["statusline:statusline.sh".into()])
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
    let elegido = app.seleccionado().unwrap().id.clone();
    // Llega una investigación que agrega un requisito que se ordena antes.
    let receta = Receta {
        registro_hooks: None,
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
    assert_eq!(app.seleccionado().unwrap().id, elegido);
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
    assert_eq!(App::nueva(recetario()).pestana, Pestana::Armar);
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
    let linea = p
        .lines()
        .filter(|l| !l.starts_with('┌'))
        .find(|l| l.contains("statusline.sh"))
        .unwrap();
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
        (
            sin_origen,
            "no sé de qué repo viene: `i` lo busca en la web",
        ),
        (por_revisar, "revisá qué va a ejecutar y `a` la aprueba"),
        (aprobada, "en una PC nueva se instala con esto"),
        (excluida, "No se instala"),
    ] {
        let mut app = con_un_item(item);
        tecla(&mut app, KeyCode::Char('o'));
        let p = pantalla(&app);
        assert!(p.contains(texto), "falta «{texto}» en:\n{p}");
    }
}

#[test]
fn la_cabecera_y_el_detalle_dicen_sin_receta() {
    let p = pantalla(&con_un_item(con_fuente("plugin:a@x")));
    assert!(p.contains("· 1 sin receta"), "{p}");
    // El estado lo dice la primera línea del detalle, en palabras.
    assert!(p.contains("todavía no hay receta"), "{p}");
    assert!(!p.contains("pendiente"), "{p}");
}

#[test]
fn el_siguiente_paso_sigue_al_camino() {
    let mut r = Recetario::nuevo();
    r.items = vec![con_fuente("plugin:a@x"), con_fuente("plugin:b@x")];
    let p = pantalla(&App::nueva(r.clone()));
    assert!(p.contains("Siguiente: `t` arma 2 recetas gratis"), "{p}");
    r.items[0].estado = Estado::PorRevisar;
    r.items[1].estado = Estado::PorRevisar;
    let pie = pie(&App::nueva(r));
    assert!(
        pie[1].contains("Siguiente: 2 recetas por revisar: elegí una y `a` la aprueba"),
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
    lineas[lineas.len() - 2..].to_vec()
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
        (sin_origen, " i  buscar origen   x  excluir"),
        (
            por_revisar,
            " espacio/a  aprobar   e  editar   r  reinvestigar   x  excluir",
        ),
        (
            aprobada,
            " espacio/a  desaprobar   e  editar   r  reinvestigar   x  excluir",
        ),
        (excluida, " x  volver a incluir   e  editar"),
    ] {
        let mut app = con_un_item(item);
        tecla(&mut app, KeyCode::Char('o'));
        let pie = pie(&app);
        // Las teclas de todas las recetas viven en `?`: el pie solo suma la de la ayuda.
        assert!(
            pie[0].contains(" ?  ayuda ") && pie[0].ends_with(esperado),
            "{pie:?}"
        );
        assert!(!pie.join("").contains("investigar todas"), "{pie:?}");
    }
}

#[test]
fn el_pie_conserva_el_paso_tras_el_mensaje_de_escaneo_y_cuenta_lo_disponible() {
    let mut app = con_posible(Some("https://github.com/oraios/serena"), false);
    app.recetario.items.extend([
        con_fuente("plugin:a@x"),
        con_fuente("plugin:b@x"),
        con_fuente("plugin:armando@x"),
        Item::nuevo("mcp:sin-buscar", Tipo::Mcp),
        Item::nuevo("mcp:buscando", Tipo::Mcp),
    ]);
    app.investigando.insert("plugin:armando@x".into());
    app.buscando.insert("mcp:buscando".into());
    app.filtro = "serena".into();
    app.mensaje = Some("Escaneo terminado".into());
    let pie = pie(&app);
    assert!(pie[0].contains(" t  armar 2 + 1 Claude "), "{pie:?}");
    assert!(pie[1].contains("Escaneo terminado"), "{pie:?}");
    assert!(
        pie[0].find(" ?  ayuda ").unwrap() < pie[0].find(" t  armar 2 + 1 Claude ").unwrap(),
        "{pie:?}"
    );
    assert!(
        pie[0].find(" t  armar 2 + 1 Claude ").unwrap()
            < pie[0].find(" Enter  aceptar origen ").unwrap(),
        "{pie:?}"
    );
}

#[test]
fn el_pie_del_cookbook_vacio_ofrece_escanear() {
    let pie = pie(&App::nueva(Recetario::nuevo()));
    assert!(pie[0].ends_with(" s  escanear"), "{pie:?}");
}

#[test]
fn el_pie_con_todo_aprobado_ofrece_ir_a_instalar() {
    let mut item = con_fuente("plugin:a@x");
    item.estado = Estado::Aprobada;
    let pie = pie(&con_un_item(item));
    assert!(pie[0].contains(" 2  ir a instalar "), "{pie:?}");
}

#[test]
fn el_pie_con_solo_por_revisar_no_ofrece_otra_tecla_del_paso() {
    let mut item = con_fuente("plugin:a@x");
    item.estado = Estado::PorRevisar;
    let pie = pie(&con_un_item(item));
    for tecla in [" t  ", " s  ", " 2  "] {
        assert!(!pie[0].contains(tecla), "{pie:?}");
    }
    assert!(pie[0].contains(" espacio/a  aprobar "), "{pie:?}");
}

#[test]
fn el_pie_de_instalacion_no_ofrece_armar_todas() {
    let mut app = con_un_item(con_fuente("plugin:a@x"));
    app.pestana = Pestana::Instalacion;
    assert!(!pie(&app)[0].contains(" t  armar "));
}

#[test]
fn el_pie_conserva_ayuda_y_armar_todas_a_80_columnas_con_receta_elegida() {
    let app = con_un_item(con_fuente("plugin:a@x"));
    assert!(app.seleccionado().is_some());
    let p = pantalla_en(&app, 80, 24);
    let primera = p.lines().nth(22).unwrap();
    assert!(primera.contains("?  ayuda"), "{p}");
    assert!(primera.contains(" t  armar 1 "), "{p}");
}

#[test]
fn el_pie_ofrece_buscar_sin_origen_aunque_haya_recetas_por_revisar_o_aprobadas() {
    for estado in [Estado::PorRevisar, Estado::Aprobada] {
        let mut app = con_un_item(Item::nuevo("mcp:sin-buscar", Tipo::Mcp));
        let mut item = con_fuente("plugin:a@x");
        item.estado = estado;
        app.recetario.items.push(item);
        let pie = pie(&app);
        let esperado = match estado {
            Estado::PorRevisar => " t  1 Claude ",
            Estado::Aprobada => " t  armar 1 Claude ",
            _ => unreachable!(),
        };
        assert!(pie[0].contains(esperado), "{pie:?}");
        assert!(!pie[0].contains(" 3  ir a instalar "), "{pie:?}");
    }
}

#[test]
fn el_pie_no_ofrece_pasos_sin_trabajo_disponible() {
    let mut excluida = con_fuente("plugin:excluida@x");
    excluida.estado = Estado::Excluida;
    for app in [
        con_un_item(excluida),
        con_posible(None, false),
        con_posible(Some("https://github.com/oraios/serena"), false),
        {
            let mut app = con_un_item(con_fuente("plugin:armando@x"));
            app.investigando.insert("plugin:armando@x".into());
            app
        },
        {
            let mut app = con_un_item(Item::nuevo("mcp:buscando", Tipo::Mcp));
            app.buscando.insert("mcp:buscando".into());
            app
        },
    ] {
        let pie = pie(&app);
        for tecla in [" t  ", " s  ", " 3  "] {
            assert!(!pie[0].contains(tecla), "{pie:?}");
        }
    }
}

#[test]
fn el_mensaje_va_en_su_propia_linea() {
    let mut app = con_un_item(con_fuente("plugin:a@x"));
    tecla(&mut app, KeyCode::Char('a'));
    let pie = pie(&app);
    assert!(pie[1].contains("todavía no tiene receta"), "{pie:?}");
    assert!(!pie[0].contains("todavía"));
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
    let ayuda = con_ayuda(App::nueva(recetario()));
    assert!(ayuda.contains("t armar todas"), "{ayuda}");
    assert!(ayuda.contains("p instalar"), "{ayuda}");
}

/// La pantalla con `?` abierto.
fn con_ayuda(mut app: App) -> String {
    tecla(&mut app, KeyCode::Char('?'));
    pantalla_en(&app, 200, 40)
}

fn camino(app: App) -> String {
    con_ayuda(app)
        .lines()
        .find(|l| l.contains("Camino:"))
        .unwrap_or_default()
        .to_string()
}

#[test]
fn la_ayuda_muestra_la_leyenda_de_marcas() {
    let p = con_ayuda(App::nueva(recetario()));
    for marca in [
        "✓ aprobada",
        "□ por revisar",
        "· sin receta",
        "? sin origen",
        "✗ excluida",
        "… armando",
        "(ausente) no está en esta PC",
    ] {
        assert!(p.contains(marca), "falta {marca:?} en:\n{p}");
    }
}

#[test]
fn el_camino_resalta_el_primer_paso_con_trabajo() {
    assert!(camino(App::nueva(Recetario::nuevo())).contains("[escanear]"));
    let pendiente = con_un_item(con_fuente("plugin:a@x"));
    assert!(camino(pendiente).contains("✓ escanear › [armar recetas]"));
    let mut revisar = con_fuente("plugin:a@x");
    revisar.estado = Estado::PorRevisar;
    assert!(camino(con_un_item(revisar)).contains("› [aprobar] ›"));
    let mut aprobada = con_fuente("plugin:a@x");
    aprobada.estado = Estado::Aprobada;
    let c = camino(con_un_item(aprobada));
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
        .find(|l| l.contains("· claude-md-management"))
        .unwrap_or_else(|| panic!("{p}"));
    assert!(!oficial.contains("· claude-md-management@"), "{oficial}");
    let largo = p.lines().find(|l| l.contains("· un-nombre")).unwrap();
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
        .find(|l| l.contains("? 日"))
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
        "          instalador de plugins de Claude Code, sin Claude",
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
    for contador in ["✓2", "?1", "✗1", "u actualiza"] {
        assert!(
            cabecera.contains(contador),
            "falta {contador:?} en {cabecera:?}"
        );
    }
    // Los contadores en cero no aparecen.
    assert!(
        !cabecera.contains("□0") && !cabecera.contains("·0"),
        "{cabecera:?}"
    );
}

#[test]
fn instalacion_sin_aprobadas_solo_ofrece_lo_que_sirve() {
    let mut app = con_un_item(con_fuente("plugin:a@x"));
    tecla(&mut app, KeyCode::Char('2'));
    let pie = pie(&app);
    for sobra in [" t ", " c ", " p ", "espacio"] {
        assert!(!pie[0].contains(sobra), "sobra {sobra:?} en {pie:?}");
    }
    assert!(
        pie[0].contains("1 volver a Armar") && pie[0].contains("q salir"),
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
fn la_ayuda_dice_para_que_es_cada_pestana() {
    let ayuda = con_ayuda(App::nueva(recetario()));
    for texto in [
        "1 Armar. En esta PC: decidí cómo se instala cada cosa",
        "2 Instalar. En una PC nueva: instalá las recetas aprobadas",
        "3 Checklist. Después de instalar: lo que rebake no hace por vos",
    ] {
        assert!(ayuda.contains(texto), "falta {texto:?} en:\n{ayuda}");
    }
    let mut app = App::nueva(recetario());
    tecla(&mut app, KeyCode::Char('2'));
    assert!(pie(&app)[0].contains(" 2 Instalar "));
    tecla(&mut app, KeyCode::Char('3'));
    assert!(pie(&app)[0].contains(" 3 Checklist "));
}

#[test]
fn un_requisito_dice_quien_lo_pide_y_no_por_que_falta() {
    let mut req = con_fuente("marketplace:openai/codex-plugin-cc");
    req.estado = Estado::PorRevisar;
    req.pista = Some("requisito de skill:graph-engineer".into());
    let p = pantalla(&con_un_item(req));
    assert!(p.contains("Lo pide: skill:graph-engineer"), "{p}");
    assert!(!p.contains("Por qué falta"), "{p}");

    let mut con_link = con_fuente("skill:seo");
    con_link.estado = Estado::PorRevisar;
    con_link.pista = Some("skill sin origen conocido: ~/.claude/skills/seo".into());
    assert!(!pantalla(&con_un_item(con_link)).contains("Por qué falta"));

    let mut sin_origen = Item::nuevo("skill:seo", Tipo::Skill);
    sin_origen.pista = Some("skill sin origen conocido: ~/.claude/skills/seo".into());
    let p = pantalla(&con_un_item(sin_origen));
    assert!(
        p.contains("Por qué falta: skill sin origen conocido"),
        "{p}"
    );
}

fn por_revisar_con_pasos() -> App {
    let mut item = con_fuente("plugin:b@x");
    item.estado = Estado::PorRevisar;
    item.pasos = vec![Paso {
        cmd: "claude plugin install b@x".into(),
        modo: Modo::Auto,
        por_perfil: true,
        cita: None,
        nota: None,
    }];
    con_un_item(item)
}

fn estado(app: &App) -> Estado {
    app.recetario.item("plugin:b@x").unwrap().estado
}

#[test]
fn espacio_y_a_alternan_la_aprobacion_y_guardan() {
    for (ida, vuelta) in [(' ', 'a'), ('a', ' ')] {
        let mut app = por_revisar_con_pasos();
        assert!(pantalla(&app).contains("□"));
        assert_eq!(tecla(&mut app, KeyCode::Char(ida)), Efecto::Guardar);
        assert_eq!(estado(&app), Estado::Aprobada);
        assert!(pantalla(&app).contains("✓"));
        assert!(
            app.mensaje.as_deref().unwrap().contains("aprobada"),
            "{:?}",
            app.mensaje
        );
        assert_eq!(tecla(&mut app, KeyCode::Char(vuelta)), Efecto::Guardar);
        assert_eq!(estado(&app), Estado::PorRevisar);
        assert!(pantalla(&app).contains("□"));
        assert!(
            app.mensaje
                .as_deref()
                .unwrap()
                .contains("vuelve a por revisar"),
            "{:?}",
            app.mensaje
        );
    }
}

#[test]
fn espacio_no_aprueba_sin_receta_ni_excluida() {
    let mut app = con_un_item(con_fuente("plugin:b@x"));
    assert_eq!(tecla(&mut app, KeyCode::Char(' ')), Efecto::Nada);
    assert_eq!(estado(&app), Estado::Pendiente);
    assert!(
        app.mensaje
            .as_deref()
            .unwrap()
            .contains("todavía no tiene receta")
    );

    let mut excluida = con_fuente("plugin:b@x");
    excluida.estado = Estado::Excluida;
    let mut app = con_un_item(excluida);
    tecla(&mut app, KeyCode::Char('o'));
    assert_eq!(tecla(&mut app, KeyCode::Char('a')), Efecto::Nada);
    assert_eq!(estado(&app), Estado::Excluida);
    assert!(app.mensaje.as_deref().unwrap().contains("`x` lo devuelve"));
}

#[test]
fn x_excluye_sin_motivo_y_o_permite_recuperarla_sin_preguntar() {
    let mut app = por_revisar_con_pasos();
    assert_eq!(tecla(&mut app, KeyCode::Char('x')), Efecto::Guardar);
    assert_eq!(estado(&app), Estado::Excluida);
    assert!(app.recetario.item("plugin:b@x").unwrap().motivo.is_none());
    assert!(app.seleccionado().is_none());
    tecla(&mut app, KeyCode::Char('o'));
    assert_eq!(tecla(&mut app, KeyCode::Char('x')), Efecto::Guardar);
    assert_eq!(app.modo, rebake::tui::app::Modo::Normal);
    assert_eq!(estado(&app), Estado::PorRevisar);
    assert_eq!(app.recetario.item("plugin:b@x").unwrap().motivo, None);
    assert!(
        app.mensaje
            .as_deref()
            .unwrap()
            .contains("vuelve a por revisar")
    );
}

#[test]
fn la_ayuda_dice_que_espacio_guarda_en_recetas_y_solo_marca_en_instalacion() {
    let mut app = App::nueva(recetario());
    tecla(&mut app, KeyCode::Char('?'));
    let p = pantalla(&app);
    assert!(p.contains("queda guardado en el cookbook"), "{p}");
    assert!(p.contains("solo para esa instalación"), "{p}");
}

fn con_posible(repo: Option<&str>, descartado: bool) -> App {
    let mut item = Item::nuevo("mcp:serena", Tipo::Mcp);
    item.pista = Some("servidor MCP: serena start-mcp-server".into());
    item.posible = Some(Posible {
        repo: repo.map(String::from),
        motivo: "el paquete serena-agent es de oraios".into(),
        descartado,
    });
    con_un_item(item)
}

#[test]
fn el_panel_muestra_el_posible_origen_con_su_motivo_y_sus_teclas() {
    let app = con_posible(Some("https://github.com/oraios/serena"), false);
    let p = pantalla(&app);
    assert!(
        p.contains("Posible origen: https://github.com/oraios/serena"),
        "{p}"
    );
    assert!(
        p.contains("Por qué: el paquete serena-agent es de oraios"),
        "{p}"
    );
    assert!(p.contains("Enter lo acepta, `n` lo descarta"), "{p}");
    assert!(p.contains("posible origen│"), "{p}");
    let pie = pie(&app);
    assert!(
        pie[0].contains(" Enter  aceptar origen   n  descartar   i  pegar otro link   x  excluir"),
        "{pie:?}"
    );
}

#[test]
fn enter_acepta_el_posible_e_investiga_desde_ese_repo() {
    let mut app = con_posible(Some("https://github.com/oraios/serena"), false);
    assert_eq!(
        tecla(&mut app, KeyCode::Enter),
        Efecto::Investigar(vec!["mcp:serena".into()])
    );
    let x = app.recetario.item("mcp:serena").unwrap();
    assert_eq!(x.fuente.as_ref().unwrap().via, Via::Busqueda);
    assert!(x.posible.is_none());
}

#[test]
fn n_descarta_el_posible_sin_investigar() {
    let mut app = con_posible(Some("https://github.com/oraios/serena"), false);
    assert_eq!(tecla(&mut app, KeyCode::Char('n')), Efecto::Guardar);
    assert!(app.investigando.is_empty());
    let x = app.recetario.item("mcp:serena").unwrap();
    assert!(x.fuente.is_none());
    assert!(x.posible.as_ref().unwrap().descartado);
    let p = pantalla(&app);
    assert!(
        p.contains("Descartaste el posible origen: `i` te pide el link"),
        "{p}"
    );
    // Ya no hay nada que aceptar: Enter no hace nada.
    assert_eq!(tecla(&mut app, KeyCode::Enter), Efecto::Nada);
}

#[test]
fn un_posible_sin_aceptar_no_se_aprueba_ni_se_instala() {
    let mut app = con_posible(Some("https://github.com/oraios/serena"), false);
    tecla(&mut app, KeyCode::Char('a'));
    assert_eq!(
        app.recetario.item("mcp:serena").unwrap().estado,
        Estado::Pendiente
    );
    assert!(app.plan().is_empty());
}

#[test]
fn t_busca_una_sola_vez_por_item() {
    let mut r = Recetario::nuevo();
    let nunca = Item::nuevo("mcp:nunca", Tipo::Mcp);
    let mut esperando = Item::nuevo("mcp:esperando", Tipo::Mcp);
    esperando.posible = Some(Posible {
        repo: Some("https://github.com/a/b".into()),
        motivo: String::new(),
        descartado: false,
    });
    let mut no_encontrado = Item::nuevo("mcp:no-encontrado", Tipo::Mcp);
    no_encontrado.posible = Some(Posible {
        repo: None,
        motivo: String::new(),
        descartado: false,
    });
    let mut descartado = Item::nuevo("mcp:descartado", Tipo::Mcp);
    descartado.posible = Some(Posible {
        repo: Some("https://github.com/a/c".into()),
        motivo: String::new(),
        descartado: true,
    });
    r.items = vec![nunca, esperando, no_encontrado, descartado];
    let mut app = App::nueva(r);
    // cuidar-cuota: se conserva el único elegible, pero ahora se confirma antes de buscar.
    assert_eq!(tecla(&mut app, KeyCode::Char('t')), Efecto::Nada);
    assert!(app.buscando.is_empty());
    assert_eq!(
        tecla(&mut app, KeyCode::Char('s')),
        Efecto::Buscar(vec!["mcp:nunca".into()])
    );
    // Mientras busca, otro `t` no la vuelve a lanzar.
    assert_eq!(tecla(&mut app, KeyCode::Char('t')), Efecto::Nada);
}

#[test]
fn mientras_busca_un_spinner_se_mueve_en_la_fila_y_el_panel() {
    let mut app = con_un_item(Item::nuevo("mcp:serena", Tipo::Mcp));
    tecla(&mut app, KeyCode::Char('i'));
    assert!(app.buscando.contains("mcp:serena"));
    let antes = pantalla(&app);
    assert!(
        antes.contains("Buscando en la web de dónde viene…"),
        "{antes}"
    );
    assert!(antes.contains("buscando origen│"), "{antes}");
    assert!(antes.contains("1 buscando origen"), "{antes}");
    app.tic();
    let despues = pantalla(&app);
    let fila = |p: &str| {
        p.lines()
            .find(|l| l.contains(" serena"))
            .unwrap()
            .to_string()
    };
    assert_ne!(fila(&antes), fila(&despues));

    app.registrar_busqueda(
        "mcp:serena",
        Ok(Posible {
            repo: Some("https://github.com/oraios/serena".into()),
            motivo: "x".into(),
            descartado: false,
        }),
    );
    assert!(app.buscando.is_empty());
    assert!(pantalla(&app).contains("Posible origen: https://github.com/oraios/serena"));
}

#[test]
fn el_resumen_avisa_las_que_dejaron_pasos_a_mano() {
    use rebake::instalador::{Evento, Resultado};
    let mut r = recetario();
    r.item_mut("plugin:codex@openai-codex")
        .unwrap()
        .pasos
        .push(Paso {
            cmd: "/codex:setup".into(),
            modo: Modo::Manual,
            por_perfil: true,
            cita: None,
            nota: None,
        });
    let mut app = App::nueva(r);
    app.instalacion = vec![
        ("claude".into(), None),
        ("plugin:codex@openai-codex".into(), None),
    ];
    app.instalando = true;
    app.evento_instalacion(Evento::Fin("claude".into(), Resultado::Ok));
    app.evento_instalacion(Evento::Fin(
        "plugin:codex@openai-codex".into(),
        Resultado::Ok,
    ));
    let m = app.mensaje.clone().unwrap();
    assert!(m.contains("1 con pasos a mano: mirá Pendientes"), "{m}");
    assert!(pantalla(&app).contains("[ ] revisá codex@openai-codex"));
    // Logins tiene dos filas; la tercera elegible es el bloque, entero en el detalle.
    tecla(&mut app, KeyCode::Down);
    tecla(&mut app, KeyCode::Down);
    let p = pantalla(&app);
    assert!(
        p.contains("│revisá codex@openai-codex (plugin): instalada, pero falta esto"),
        "{p}"
    );
    assert!(
        p.contains("[laburo] abrí `claude` y corré /codex:setup"),
        "{p}"
    );
}

fn con_pendientes() -> App {
    let mut r = recetario();
    let largo = "claude mcp get serena >/dev/null 2>&1 || claude mcp add --scope user serena -- serena start-mcp-server --context=claude-code --project-from-cwd";
    let mut serena = Item::nuevo("mcp:serena", Tipo::Mcp);
    serena.estado = Estado::Aprobada;
    serena.pasos = vec![Paso {
        cmd: largo.into(),
        modo: Modo::Manual,
        por_perfil: false,
        cita: None,
        nota: None,
    }];
    r.items.push(serena);
    for (n, v) in ["### Org-wide", "**Organization**: Flock", "**Cloud**: none"]
        .iter()
        .enumerate()
    {
        r.checklist.ajustes.push(Ajuste {
            perfil: "laburo".into(),
            clave: format!("autoMode.environment[{n}]"),
            valor: v.to_string(),
        });
    }
    r.checklist.ajustes.push(Ajuste {
        perfil: "laburo".into(),
        clave: "theme".into(),
        valor: "dark".into(),
    });
    let mut app = App::nueva(r);
    app.pestana = Pestana::Checklist;
    app
}

/// Baja hasta la fila elegible cuyo texto contiene `texto`.
fn elegir_pendiente(app: &mut App, texto: &str) {
    app.seleccion_checklist = 0;
    while !app.fila_elegida().unwrap().texto.contains(texto) {
        tecla(app, KeyCode::Down);
    }
}

#[test]
fn un_comando_largo_se_ve_entero_en_el_detalle_a_80_columnas() {
    let mut app = con_pendientes();
    elegir_pendiente(&mut app, "claude mcp get serena");
    let p = pantalla_en(&app, 80, 24);
    // Partido en varias líneas del panel, pero sin perder el final.
    let detalle: String = p
        .lines()
        .filter_map(|l| l.rsplit_once('│').and(l.split('│').nth(3)))
        .collect::<Vec<_>>()
        .join("");
    let sin_espacios = |s: &str| s.split_whitespace().collect::<String>();
    assert!(
        sin_espacios(&detalle).contains(&sin_espacios("--context=claude-code --project-from-cwd")),
        "{p}"
    );
}

#[test]
fn los_ajustes_arrancan_colapsados_y_enter_los_despliega() {
    let mut app = con_pendientes();
    let p = pantalla(&app);
    assert!(p.contains("▸ Ajustes (2) · Enter despliega"), "{p}");
    assert!(!p.contains("theme = dark"), "{p}");
    elegir_pendiente(&mut app, "Ajustes");
    tecla(&mut app, KeyCode::Enter);
    let p = pantalla(&app);
    assert!(p.contains("▾ Ajustes (2) · Enter colapsa"), "{p}");
    // La lista de 3 valores es una sola fila; su texto completo va al detalle.
    assert!(
        p.contains("[ ] [laburo] autoMode.environment (3 líneas)"),
        "{p}"
    );
    assert!(!p.contains("autoMode.environment[1]"), "{p}");
    assert!(p.contains("[ ] [laburo] theme = dark"), "{p}");
    elegir_pendiente(&mut app, "autoMode.environment");
    assert!(pantalla(&app).contains("**Organization**: Flock"));
    elegir_pendiente(&mut app, "Ajustes");
    tecla(&mut app, KeyCode::Enter);
    assert!(!pantalla(&app).contains("theme = dark"));
}

#[test]
fn un_tilde_sobrevive_a_desplegar_los_ajustes() {
    let mut app = con_pendientes();
    elegir_pendiente(&mut app, "GitHub");
    tecla(&mut app, KeyCode::Char(' '));
    elegir_pendiente(&mut app, "Ajustes");
    tecla(&mut app, KeyCode::Enter);
    elegir_pendiente(&mut app, "theme");
    tecla(&mut app, KeyCode::Char(' '));
    let p = pantalla(&app);
    assert!(p.contains("[x] GitHub: `gh auth login`"), "{p}");
    assert!(p.contains("[x] [laburo] theme = dark"), "{p}");
    elegir_pendiente(&mut app, "Ajustes");
    tecla(&mut app, KeyCode::Enter);
    assert!(pantalla(&app).contains("[x] GitHub: `gh auth login`"));
}

#[test]
fn los_titulos_de_seccion_no_se_eligen() {
    let mut app = con_pendientes();
    let mut vistas = Vec::new();
    for _ in 0..20 {
        vistas.push(app.fila_elegida().unwrap());
        tecla(&mut app, KeyCode::Down);
    }
    assert!(vistas.iter().all(|f| f.clase != ClaseFila::Titulo));
    let p = pantalla(&app);
    assert!(p.contains("│Logins"), "{p}");
    assert!(p.contains("│Pasos a mano (rebake no los corre)"), "{p}");
}

#[test]
fn las_teclas_de_pendientes_entran_enteras_a_80_columnas() {
    let app = con_pendientes();
    let p = pantalla_en(&app, 80, 24);
    assert!(
        p.contains("1 Armar  2 Instalar  3 Checklist   espacio tilda  Enter ajustes  ? ayuda"),
        "{p}"
    );
}

#[test]
fn siguiente_con_solo_sin_origen_dice_si_falta_buscarlos() {
    let sin_buscar = Item::nuevo("mcp:serena", Tipo::Mcp);
    let p = pie(&con_un_item(sin_buscar.clone()));
    assert!(
        p[1].contains("Siguiente: `t` pregunta por hasta 1 de 1 corridas de Claude"),
        "{p:?}"
    );
    let mut buscado = sin_buscar;
    buscado.posible = Some(Posible {
        repo: None,
        motivo: String::new(),
        descartado: false,
    });
    let p = pie(&con_un_item(buscado));
    assert!(p[1].contains("`i` te deja pegar el link"), "{p:?}");
}

#[test]
fn clasifica_lo_armable_con_el_criterio_real_y_en_orden_de_pantalla() {
    let mut gratis = con_fuente("plugin:z@x");
    gratis.estado = Estado::Pendiente;
    let mut inseguro = con_fuente("plugin:a;rm@x");
    inseguro.fuente.as_mut().unwrap().repo = "https://github.com/a/$(id)".into();
    let mut skill = Item::nuevo("skill:b", Tipo::Skill);
    skill.fuente = Some(Fuente {
        repo: "https://github.com/a/b".into(),
        via: Via::Symlink,
        doc: None,
    });
    let mut app = con_un_item(gratis);
    app.recetario.items.extend([
        Item::nuevo("mcp:z", Tipo::Mcp),
        skill,
        inseguro,
        Item::nuevo("mcp:a", Tipo::Mcp),
    ]);
    app.filtro = "plugin:z".into();
    assert_eq!(
        app.por_armar(),
        (
            vec!["plugin:z@x".into()],
            vec!["plugin:a;rm@x".into(), "skill:b".into()],
            vec!["mcp:a".into(), "mcp:z".into()],
        )
    );
    app.investigando.insert("plugin:z@x".into());
    app.buscando.insert("mcp:a".into());
    app.recetario.item_mut("skill:b").unwrap().estado = Estado::Excluida;
    app.recetario.item_mut("mcp:z").unwrap().posible = Some(Posible {
        repo: None,
        motivo: "no encontrado".into(),
        descartado: false,
    });
    assert_eq!(
        app.por_armar(),
        (vec![], vec!["plugin:a;rm@x".into()], vec![])
    );
}

fn con_cuota(gratis: usize, investigaciones: usize, busquedas: usize) -> App {
    let mut r = Recetario::nuevo();
    for n in 0..gratis {
        r.items.push(con_fuente(&format!("plugin:gratis-{n:03}@x")));
    }
    for n in 0..investigaciones {
        let mut i = Item::nuevo(&format!("mcp:inv-{n:03}"), Tipo::Mcp);
        i.fuente = Some(Fuente {
            repo: "https://github.com/a/b".into(),
            via: Via::Manual,
            doc: None,
        });
        r.items.push(i);
    }
    for n in 0..busquedas {
        r.items
            .push(Item::nuevo(&format!("mcp:bus-{n:03}"), Tipo::Mcp));
    }
    App::nueva(r)
}

fn terminar_directas(app: &mut App, ids: &[String]) {
    for id in ids {
        let t = rebake::investigador::trabajo_para(
            &app.recetario,
            app.recetario.item(id).unwrap(),
            std::path::Path::new("/nada"),
        );
        app.registrar_investigacion(id, Ok(t.directa.unwrap()), "2026-10-22");
    }
}

#[test]
fn t_arma_25_gratis_y_pregunta_por_cinco_sin_lanzar_claude() {
    let mut app = con_cuota(25, 0, 41);
    let Efecto::Investigar(ids) = tecla(&mut app, KeyCode::Char('t')) else {
        panic!("t debe despachar solamente las directas");
    };
    assert_eq!(ids.len(), 25);
    assert!(ids.iter().all(|id| id.starts_with("plugin:gratis-")));
    assert!(app.buscando.is_empty());
    assert!(pie(&app)[1].contains("¿Gastar 5 corridas de Claude?"));
    terminar_directas(&mut app, &ids);
    assert!(
        ids.iter()
            .all(|id| app.recetario.item(id).unwrap().estado == Estado::PorRevisar)
    );
    assert!(pie(&app)[1].contains("¿Gastar 5 corridas de Claude?"));
}

#[test]
fn pregunta_por_el_minimo_entre_cinco_y_lo_pendiente() {
    for (pagas, pregunta) in [
        (0, None),
        (1, Some(1)),
        (4, Some(4)),
        (5, Some(5)),
        (6, Some(5)),
    ] {
        let mut app = con_cuota(0, 0, pagas);
        assert_eq!(tecla(&mut app, KeyCode::Char('t')), Efecto::Nada);
        assert!(app.buscando.is_empty());
        match pregunta {
            Some(n) => {
                assert!(pie(&app)[1].contains(&format!("¿Gastar {n} corridas de Claude?")));
                let Efecto::Buscar(ids) = tecla(&mut app, KeyCode::Char('s')) else {
                    panic!()
                };
                assert_eq!(ids.len(), n);
            }
            None => assert!(!pie(&app)[1].contains("¿Gastar")),
        }
    }
}

#[test]
fn confirma_primero_tres_investigaciones_y_despues_dos_busquedas() {
    let mut app = con_cuota(0, 3, 38);
    app.recetario.items.reverse();
    tecla(&mut app, KeyCode::Char('t'));
    assert_eq!(
        tecla(&mut app, KeyCode::Char('s')),
        Efecto::Lote(
            vec![
                "mcp:inv-000".into(),
                "mcp:inv-001".into(),
                "mcp:inv-002".into()
            ],
            vec!["mcp:bus-000".into(), "mcp:bus-001".into()],
        )
    );
    assert_eq!(app.investigando.len(), 3);
    assert_eq!(app.buscando.len(), 2);
}

#[test]
fn confirmar_no_rellena_huecos_ni_agrega_items_nuevos() {
    let mut app = con_cuota(0, 3, 6);
    tecla(&mut app, KeyCode::Char('t'));
    app.recetario.item_mut("mcp:inv-000").unwrap().estado = Estado::PorRevisar;
    app.recetario.item_mut("mcp:inv-001").unwrap().estado = Estado::Excluida;
    app.recetario.items.retain(|i| i.id != "mcp:inv-002");
    app.buscando.insert("mcp:bus-000".into());
    app.recetario
        .items
        .push(Item::nuevo("mcp:aaa-nuevo", Tipo::Mcp));
    assert_eq!(
        tecla(&mut app, KeyCode::Char('s')),
        Efecto::Buscar(vec!["mcp:bus-001".into()])
    );
    assert_eq!(app.investigando.len(), 0);
    assert_eq!(app.buscando.len(), 2);
}

#[test]
fn confirmar_todos_resueltos_no_despacha_un_lote_vacio() {
    let mut app = con_cuota(0, 1, 1);
    tecla(&mut app, KeyCode::Char('t'));
    for i in &mut app.recetario.items {
        i.estado = Estado::Excluida;
    }
    assert_eq!(tecla(&mut app, KeyCode::Char('s')), Efecto::Nada);
    assert!(app.investigando.is_empty());
    assert!(app.buscando.is_empty());
}

#[test]
fn cualquier_otra_tecla_cancela_sin_hacer_su_accion_y_conserva_gratis() {
    for k in [
        KeyCode::Enter,
        KeyCode::Esc,
        KeyCode::Char('i'),
        KeyCode::Char('t'),
        KeyCode::Char('q'),
        KeyCode::Char('S'),
        KeyCode::Char('3'),
        KeyCode::Char('?'),
    ] {
        let mut app = con_cuota(1, 0, 6);
        let Efecto::Investigar(ids) = tecla(&mut app, KeyCode::Char('t')) else {
            panic!()
        };
        terminar_directas(&mut app, &ids);
        assert_eq!(tecla(&mut app, k), Efecto::Nada, "{k:?}");
        assert_eq!(app.pestana, Pestana::Armar);
        assert_eq!(app.modo, rebake::tui::app::Modo::Normal);
        assert!(pie(&app)[1].contains("no se gastó nada"), "{k:?}");
        assert!(app.buscando.is_empty());
        assert_eq!(
            app.recetario.item(&ids[0]).unwrap().estado,
            Estado::PorRevisar
        );
    }
}

#[test]
fn t_espera_a_claude_individual_pero_arma_lo_gratis() {
    for con_origen in [true, false] {
        let mut app = if con_origen {
            con_cuota(1, 2, 0)
        } else {
            con_cuota(1, 0, 2)
        };
        app.filtro = if con_origen { "inv-000" } else { "bus-000" }.into();
        let efecto = tecla(&mut app, KeyCode::Char('i'));
        assert!(matches!(efecto, Efecto::Investigar(_) | Efecto::Buscar(_)));
        app.filtro.clear();
        assert_eq!(
            tecla(&mut app, KeyCode::Char('t')),
            Efecto::Investigar(vec!["plugin:gratis-000@x".into()])
        );
        assert_eq!(app.modo, rebake::tui::app::Modo::Normal);
        assert!(pie(&app)[1].contains("esperá"));
        assert!(!pie(&app)[1].contains("¿Gastar"));
    }
}

#[test]
fn una_directa_en_hilos_no_bloquea_la_pregunta_de_claude() {
    let mut app = con_cuota(2, 0, 1);
    app.filtro = "gratis-000".into();
    assert_eq!(
        tecla(&mut app, KeyCode::Char('i')),
        Efecto::Investigar(vec!["plugin:gratis-000@x".into()])
    );
    assert_eq!(
        tecla(&mut app, KeyCode::Char('t')),
        Efecto::Investigar(vec!["plugin:gratis-001@x".into()])
    );
    assert!(pie(&app)[1].contains("¿Gastar 1 corridas de Claude?"));
}

#[test]
fn la_tanda_termina_con_errores_sin_encadenar_y_no_absorbe_i() {
    use rebake::investigador::Receta;
    use rebake::tui::app::Corrida;
    use std::collections::HashSet;
    let mut app = con_cuota(0, 3, 4);
    tecla(&mut app, KeyCode::Char('t'));
    let Efecto::Lote(inv, bus) = tecla(&mut app, KeyCode::Char('s')) else {
        panic!()
    };
    app.filtro = "bus-003".into();
    assert_eq!(
        tecla(&mut app, KeyCode::Char('i')),
        Efecto::Buscar(vec!["mcp:bus-003".into()])
    );
    assert_eq!(
        Efecto::Lote(inv.clone(), bus.clone()),
        Efecto::Lote(
            vec![
                "mcp:inv-000".into(),
                "mcp:inv-001".into(),
                "mcp:inv-002".into()
            ],
            vec!["mcp:bus-000".into(), "mcp:bus-001".into()],
        )
    );
    assert_eq!(app.claude_en_curso.len(), 6);
    app.registrar_investigacion(
        &inv[0],
        Ok(Receta {
            registro_hooks: None,
            repo: "https://github.com/a/b".into(),
            doc: "https://github.com/a/b#install".into(),
            pasos: vec![],
            requiere: vec![],
            verificar: None,
        }),
        "2026-10-22",
    );
    for id in &inv[1..] {
        app.registrar_investigacion(id, Err(anyhow::anyhow!("sin cuota")), "2026-10-22");
    }
    app.registrar_busqueda(&bus[0], Err(anyhow::anyhow!("sin cuota")));
    assert_eq!(
        app.claude_en_curso,
        HashSet::from([
            Corrida::Buscar("mcp:bus-001".into()),
            Corrida::Buscar("mcp:bus-003".into()),
        ])
    );
    assert_eq!(tecla(&mut app, KeyCode::Char('t')), Efecto::Nada);
    assert!(pie(&app)[1].contains("esperá"));
    app.registrar_busqueda(&bus[1], Err(anyhow::anyhow!("sin cuota")));
    assert_eq!(
        app.claude_en_curso,
        HashSet::from([Corrida::Buscar("mcp:bus-003".into())])
    );
    assert_eq!(app.buscando.len(), 1);
    assert_eq!(app.modo, rebake::tui::app::Modo::Normal);
    assert_eq!(tecla(&mut app, KeyCode::Char('t')), Efecto::Nada);
    assert!(pie(&app)[1].contains("esperá"));
    app.registrar_busqueda(
        "mcp:bus-003",
        Ok(Posible {
            repo: None,
            motivo: "no encontrado".into(),
            descartado: false,
        }),
    );
    assert!(app.claude_en_curso.is_empty());
    assert!(app.buscando.is_empty());
    assert!(app.investigando.is_empty());
    assert_eq!(tecla(&mut app, KeyCode::Char('t')), Efecto::Nada);
    assert!(pie(&app)[1].contains("¿Gastar 5 corridas de Claude?"));
    assert_eq!(
        tecla(&mut app, KeyCode::Char('s')),
        Efecto::Lote(
            vec!["mcp:inv-001".into(), "mcp:inv-002".into()],
            vec![
                "mcp:bus-000".into(),
                "mcp:bus-001".into(),
                "mcp:bus-002".into()
            ],
        )
    );
}

#[test]
fn el_pie_mide_80_columnas_con_cuentas_de_tres_cifras_incluso_con_filtro() {
    use unicode_width::UnicodeWidthStr;
    for (g, c, texto, ancho) in [
        (107, 141, " t  armar 107 + 141 Claude ", 74),
        (999, 999, " t  armar 999 + 999 Claude ", 74),
        (999, 0, " t  armar 999 ", 61),
        (0, 999, " t  armar 999 Claude ", 68),
    ] {
        let mut app = con_cuota(g, 0, c);
        app.filtro = "bus-000".into();
        app.mensaje = Some("Escaneo terminado".into());
        let grande = pantalla_en(&app, 200, 24);
        let linea = grande.lines().nth(22).unwrap();
        let fin = linea
            .find(texto)
            .unwrap_or_else(|| panic!("{linea}"))
            .checked_add(texto.len())
            .unwrap();
        assert_eq!(UnicodeWidthStr::width(&linea[..fin]), ancho, "{linea}");
        let chica = pantalla_en(&app, 80, 24);
        let pie = chica.lines().nth(22).unwrap();
        for esperado in ["1 Armar", "2 Instalar", "3 Checklist", " ?  ayuda ", texto] {
            assert!(pie.contains(esperado), "falta {esperado:?}: {pie}");
        }
        assert!(chica.lines().nth(23).unwrap().contains("Escaneo terminado"));
    }
}

#[test]
fn el_panel_anticipa_el_costo_real_de_aceptar_sin_mutar_el_origen() {
    for (id, tipo, repo, gratis) in [
        ("plugin:a@x", Tipo::Plugin, "https://github.com/a/b", true),
        (
            "marketplace:x",
            Tipo::Marketplace,
            "https://github.com/a/b",
            true,
        ),
        ("mcp:a", Tipo::Mcp, "https://github.com/a/b", false),
        ("skill:a", Tipo::Skill, "https://github.com/a/b", false),
        (
            "plugin:a;rm@x",
            Tipo::Plugin,
            "https://github.com/a/b",
            false,
        ),
        (
            "marketplace:x",
            Tipo::Marketplace,
            "https://github.com/a/$(id)",
            false,
        ),
    ] {
        let mut i = Item::nuevo(id, tipo);
        i.posible = Some(Posible {
            repo: Some(repo.into()),
            motivo: "un candidato".into(),
            descartado: false,
        });
        let mut app = con_un_item(i);
        let antes = app.recetario.clone();
        let p = pantalla_en(&app, 80, 24);
        let costo = if gratis {
            "Aceptar: gratis, sin Claude"
        } else {
            "Aceptar: 1 corrida de Claude"
        };
        assert!(p.contains(costo), "{id}: {p}");
        assert_eq!(app.recetario, antes);
        assert_eq!(
            tecla(&mut app, KeyCode::Enter),
            Efecto::Investigar(vec![id.into()])
        );
        assert_eq!(app.claude_en_curso.len(), usize::from(!gratis));
        assert_eq!(
            app.recetario.item(id).unwrap().fuente.as_ref().unwrap().via,
            Via::Busqueda
        );
    }
}

#[test]
fn la_ayuda_explica_gratis_tandas_y_confirmacion() {
    let p = con_ayuda(con_cuota(1, 0, 1));
    for texto in ["gratis", "5 corridas", "s confirma", "otra tecla cancela"] {
        assert!(p.contains(texto), "falta {texto}: {p}");
    }
}

#[test]
fn una_tanda_con_seis_investigaciones_no_gasta_la_sexta_ni_busca() {
    let mut app = con_cuota(0, 6, 1);
    tecla(&mut app, KeyCode::Char('t'));
    assert_eq!(
        tecla(&mut app, KeyCode::Char('s')),
        Efecto::Investigar(vec![
            "mcp:inv-000".into(),
            "mcp:inv-001".into(),
            "mcp:inv-002".into(),
            "mcp:inv-003".into(),
            "mcp:inv-004".into(),
        ])
    );
    assert!(!app.investigando.contains("mcp:inv-005"));
    assert!(app.buscando.is_empty());
}

#[test]
fn terminar_una_investigacion_no_cierra_la_busqueda_del_mismo_item() {
    let mut app = con_posible(Some("https://github.com/oraios/serena"), false);
    tecla(&mut app, KeyCode::Char('i'));
    assert_eq!(
        tecla(&mut app, KeyCode::Enter),
        Efecto::Buscar(vec!["mcp:serena".into()])
    );
    assert_eq!(
        tecla(&mut app, KeyCode::Enter),
        Efecto::Investigar(vec!["mcp:serena".into()])
    );
    assert_eq!(app.claude_en_curso.len(), 2);
    app.registrar_investigacion(
        "mcp:serena",
        Err(anyhow::anyhow!("sin cuota")),
        "2026-10-22",
    );
    assert_eq!(app.claude_en_curso.len(), 1);
    assert_eq!(tecla(&mut app, KeyCode::Char('t')), Efecto::Nada);
    assert!(pie(&app)[1].contains("esperá"));
    app.registrar_busqueda("mcp:serena", Err(anyhow::anyhow!("sin cuota")));
    assert!(app.claude_en_curso.is_empty());
    tecla(&mut app, KeyCode::Char('t'));
    assert!(pie(&app)[1].contains("¿Gastar 1 corridas de Claude?"));
}

#[test]
fn la_marca_por_revisar_es_cuadrado_en_lista_cabecera_y_ayuda() {
    use unicode_width::UnicodeWidthStr;
    assert_eq!(UnicodeWidthStr::width("□"), 1);

    let mut item = con_fuente("plugin:rev@x");
    item.estado = Estado::PorRevisar;
    let app = con_un_item(item);

    let p = pantalla(&app);
    let cabecera = p.lines().next().unwrap();
    assert!(cabecera.contains("□ 1 por revisar"), "{cabecera}");
    let linea_item = p
        .lines()
        .find(|l| l.contains("rev@x") && !l.starts_with("┌"))
        .unwrap();
    assert!(linea_item.contains("□"), "{linea_item}");

    let ayuda = con_ayuda(app);
    assert!(ayuda.contains("□ por revisar"), "{ayuda}");
}

#[test]
fn v_pide_confirmacion_y_s_aprueba_todas_las_por_revisar() {
    let mut r = recetario();
    let mut rev1 = con_fuente("plugin:rev1@x");
    rev1.estado = Estado::PorRevisar;
    let mut rev2 = con_fuente("plugin:rev2@x");
    rev2.estado = Estado::PorRevisar;
    let mut pend = con_fuente("plugin:pend@x");
    pend.estado = Estado::Pendiente;
    let mut excl = con_fuente("plugin:excl@x");
    excl.estado = Estado::Excluida;
    r.items.extend([rev1, rev2, pend, excl]);
    let mut app = App::nueva(r);

    // v pide confirmación en el pie (REQ-1)
    assert_eq!(tecla(&mut app, KeyCode::Char('v')), Efecto::Nada);
    assert!(
        pie(&app)[1].contains("¿Aprobar 2 recetas? s confirma; otra tecla cancela"),
        "{:?}",
        pie(&app)[1]
    );

    // Cualquier otra tecla cancela (REQ-2)
    assert_eq!(tecla(&mut app, KeyCode::Esc), Efecto::Nada);
    assert_eq!(
        app.recetario.item("plugin:rev1@x").unwrap().estado,
        Estado::PorRevisar
    );
    assert_eq!(
        app.recetario.item("plugin:rev2@x").unwrap().estado,
        Estado::PorRevisar
    );
    assert!(
        app.mensaje.as_deref().unwrap().contains("cancelada"),
        "{:?}",
        app.mensaje
    );

    // v y luego s aprueba y guarda (REQ-2, REQ-8)
    tecla(&mut app, KeyCode::Char('v'));
    assert_eq!(tecla(&mut app, KeyCode::Char('s')), Efecto::Guardar);
    assert_eq!(
        app.recetario.item("plugin:rev1@x").unwrap().estado,
        Estado::Aprobada
    );
    assert_eq!(
        app.recetario.item("plugin:rev2@x").unwrap().estado,
        Estado::Aprobada
    );
    assert_eq!(
        app.recetario.item("plugin:pend@x").unwrap().estado,
        Estado::Pendiente
    );
    assert_eq!(
        app.recetario.item("plugin:excl@x").unwrap().estado,
        Estado::Excluida
    );
    assert!(
        app.mensaje
            .as_deref()
            .unwrap()
            .contains("2 recetas aprobadas"),
        "{:?}",
        app.mensaje
    );
}

#[test]
fn v_sin_recetas_por_revisar_avisa_y_no_pregunta() {
    let mut app = App::nueva(recetario());
    assert_eq!(tecla(&mut app, KeyCode::Char('v')), Efecto::Nada);
    assert!(
        app.mensaje
            .as_deref()
            .unwrap()
            .contains("no hay recetas por revisar"),
        "{:?}",
        app.mensaje
    );
    assert!(!matches!(app.modo, rebake::tui::app::Modo::ConfirmarSalida));
}

#[test]
fn tanda_mixta_informa_balance_y_sugerencias_con_y_sin_pendientes() {
    use rebake::investigador::Receta;
    // Caso 1: tanda mixta con pendientes restantes (sugiere v y t)
    let mut app = con_cuota(0, 3, 4);
    tecla(&mut app, KeyCode::Char('t'));
    let Efecto::Lote(inv, bus) = tecla(&mut app, KeyCode::Char('s')) else {
        panic!()
    };
    // 2 inv exitosas, 1 inv error
    app.registrar_investigacion(
        &inv[0],
        Ok(Receta {
            repo: "r".into(),
            doc: "d".into(),
            pasos: vec![],
            requiere: vec![],
            verificar: None,
            registro_hooks: None,
        }),
        "2026-10-22",
    );
    app.registrar_investigacion(
        &inv[1],
        Ok(Receta {
            repo: "r".into(),
            doc: "d".into(),
            pasos: vec![],
            requiere: vec![],
            verificar: None,
            registro_hooks: None,
        }),
        "2026-10-22",
    );
    app.registrar_investigacion(&inv[2], Err(anyhow::anyhow!("fallo")), "2026-10-22");
    // 1 bus con posible origen, 1 bus fallida
    app.registrar_busqueda(
        &bus[0],
        Ok(Posible {
            repo: Some("https://github.com/a/b".into()),
            motivo: "m".into(),
            descartado: false,
        }),
    );
    app.registrar_busqueda(&bus[1], Err(anyhow::anyhow!("fallo")));

    let msg = app.mensaje.as_deref().unwrap();
    assert!(msg.contains("2 por revisar"), "{msg}");
    assert!(msg.contains("1 posible origen"), "{msg}");
    assert!(msg.contains("2 errores"), "{msg}");
    assert!(msg.contains("`v`"), "{msg}");
    assert!(msg.contains("`t`"), "{msg}");

    // Caso 2: tanda donde no quedan pendientes restantes (sugiere v pero no t)
    let mut app2 = con_cuota(0, 2, 0);
    tecla(&mut app2, KeyCode::Char('t'));
    let Efecto::Investigar(inv2) = tecla(&mut app2, KeyCode::Char('s')) else {
        panic!()
    };
    app2.registrar_investigacion(
        &inv2[0],
        Ok(Receta {
            repo: "r".into(),
            doc: "d".into(),
            pasos: vec![],
            requiere: vec![],
            verificar: None,
            registro_hooks: None,
        }),
        "2026-10-22",
    );
    app2.registrar_investigacion(
        &inv2[1],
        Ok(Receta {
            repo: "r".into(),
            doc: "d".into(),
            pasos: vec![],
            requiere: vec![],
            verificar: None,
            registro_hooks: None,
        }),
        "2026-10-22",
    );
    let msg2 = app2.mensaje.as_deref().unwrap();
    assert!(msg2.contains("2 por revisar"), "{msg2}");
    assert!(msg2.contains("`v`"), "{msg2}");
    assert!(!msg2.contains("`t`"), "{msg2}");
}

#[test]
fn el_pie_en_80_columnas_con_pendientes_y_por_revisar_mide_78_columnas_hasta_v() {
    use unicode_width::UnicodeWidthStr;
    // 999 gratis, 999 con Claude, y 999 por revisar
    let mut app = con_cuota(999, 0, 999);
    for i in 0..999 {
        let mut item = con_fuente(&format!("plugin:rev-{i}@x"));
        item.estado = Estado::PorRevisar;
        app.recetario.items.push(item);
    }
    let grande = pantalla_en(&app, 200, 24);
    let linea = grande.lines().nth(22).unwrap();
    let texto_v = " v  aprobar 999 ";
    let fin = linea
        .find(texto_v)
        .unwrap_or_else(|| panic!("no se encontró {texto_v} en {linea}"))
        + texto_v.len();
    assert_eq!(
        UnicodeWidthStr::width(&linea[..fin]),
        77,
        "el prefijo hasta v debe medir 77 cols: {linea}"
    );

    let chica = pantalla_en(&app, 80, 24);
    let pie = chica.lines().nth(22).unwrap();
    for esperado in [
        "1 Armar",
        "2 Inst",
        "3 Check",
        " ?  ayuda ",
        " t  999 + 999 Claude ",
        " v  aprobar 999 ",
    ] {
        assert!(pie.contains(esperado), "falta {esperado:?} en {pie}");
    }
}

#[test]
fn herramienta_presente_con_hook_faltante_sigue_marcada_para_instalar() {
    let mut r = Recetario::nuevo();
    let mut tool = Item::nuevo("herramienta:tool", Tipo::Herramienta);
    tool.estado = Estado::Aprobada;
    let mut hook = Item::nuevo("hook:tool hook", Tipo::Hook);
    hook.instalado_por = Some(InstaladorHook::Item(tool.id.clone()));
    r.items = vec![tool, hook];
    let mut app = App::nueva(r);
    app.registrar_presencia("herramienta:tool", Presencia::Instalada);
    app.registrar_presencia("hook:tool hook", Presencia::Falta(vec!["personal".into()]));
    assert_eq!(app.a_instalar(), vec!["herramienta:tool"]);
}

#[test]
fn hook_muestra_responsable_a_80_columnas_sin_ofrecer_receta() {
    let mut r = Recetario::nuevo();
    let mut tool = Item::nuevo("herramienta:tool", Tipo::Herramienta);
    tool.estado = Estado::Aprobada;
    let mut hook_vinculado = Item::nuevo("hook:tool hook", Tipo::Hook);
    hook_vinculado.instalado_por = Some(InstaladorHook::Item("herramienta:tool".into()));
    let mut hook_externo = Item::nuevo("hook:ext hook", Tipo::Hook);
    hook_externo.instalado_por = Some(InstaladorHook::Externo("tool".into()));
    r.items = vec![tool, hook_vinculado, hook_externo];

    let mut app = App::nueva(r);
    let pantalla = pantalla_en(&app, 80, 24);
    assert!(
        pantalla.contains("Hooks vinculados: tool hook"),
        "{pantalla}"
    );
    assert!(
        !pantalla.lines().next().unwrap().contains("sin receta"),
        "{pantalla}"
    );
    assert!(
        !pantalla.lines().nth(22).unwrap().contains(" i "),
        "{pantalla}"
    );
    assert!(!pantalla.contains("no sé de qué repo viene"), "{pantalla}");

    app.pestana = Pestana::Checklist;
    let pantalla_chk = pantalla_en(&app, 80, 24);
    assert!(pantalla_chk.contains("tool: 1 hook"), "{pantalla_chk}");
}

#[test]
fn grupos_imports_hooks_propios_vinculados_y_externos_en_pantalla_y_listo() {
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil {
        nombre: "personal".into(),
        dir: "~/.claude".into(),
    });

    let mut tool = Item::nuevo("herramienta:jq", Tipo::Herramienta);
    tool.estado = Estado::Aprobada;
    let mut hook_vinculado = Item::nuevo("hook:jq-format", Tipo::Hook);
    hook_vinculado.instalado_por = Some(InstaladorHook::Item("herramienta:jq".into()));
    hook_vinculado.estado = Estado::Pendiente;

    let mut hook_externo = Item::nuevo("hook:git-sync", Tipo::Hook);
    hook_externo.instalado_por = Some(InstaladorHook::Externo("git".into()));
    hook_externo.estado = Estado::Pendiente;

    let mut import_item = Item::nuevo("import:house-rules", Tipo::Import);
    import_item.estado = Estado::Aprobada;

    let mut hook_propio = Item::nuevo("hook:mi-hook", Tipo::Hook);
    hook_propio.estado = Estado::Pendiente;

    let mut skill_item = Item::nuevo("skill:extra", Tipo::Skill);
    skill_item.estado = Estado::PorRevisar;

    r.items = vec![
        tool,
        hook_vinculado,
        hook_externo,
        import_item,
        hook_propio,
        skill_item,
    ];

    let mut app = App::nueva(r);

    let visibles = app.visibles();
    let ids_visibles: Vec<&str> = visibles.iter().map(|i| i.id.as_str()).collect();
    assert!(
        !ids_visibles.contains(&"hook:jq-format"),
        "hook vinculado no debe ser visible"
    );
    assert!(
        !ids_visibles.contains(&"hook:git-sync"),
        "hook externo no debe ser visible"
    );
    assert!(ids_visibles.contains(&"herramienta:jq"));
    assert!(ids_visibles.contains(&"import:house-rules"));
    assert!(ids_visibles.contains(&"hook:mi-hook"));
    assert!(ids_visibles.contains(&"skill:extra"));

    let pos_skill = ids_visibles
        .iter()
        .position(|id| *id == "skill:extra")
        .unwrap();
    let pos_hook_propio = ids_visibles
        .iter()
        .position(|id| *id == "hook:mi-hook")
        .unwrap();
    let pos_jq = ids_visibles
        .iter()
        .position(|id| *id == "herramienta:jq")
        .unwrap();
    let pos_import = ids_visibles
        .iter()
        .position(|id| *id == "import:house-rules")
        .unwrap();
    assert!(pos_skill < pos_jq);
    assert!(pos_hook_propio < pos_jq);
    assert!(pos_skill < pos_import);
    assert!(pos_hook_propio < pos_import);

    let p = pantalla(&app);
    assert!(p.contains("POR RESOLVER"), "{p}");
    assert!(p.contains("LISTAS"), "{p}");

    let idx_jq = ids_visibles
        .iter()
        .position(|id| *id == "herramienta:jq")
        .unwrap();
    app.seleccion = idx_jq;
    let p_jq = pantalla(&app);
    assert!(p_jq.contains("Hooks vinculados: jq-format"), "{p_jq}");

    app.pestana = Pestana::Checklist;
    let p_check = pantalla(&app);
    assert!(
        p_check.contains("Hooks de herramientas externas"),
        "{p_check}"
    );
    assert!(p_check.contains("git: 1 hook"), "{p_check}");

    assert!(!app.esta_listo());

    app.recetario.item_mut("hook:mi-hook").unwrap().estado = Estado::Aprobada;
    app.recetario.item_mut("skill:extra").unwrap().estado = Estado::Aprobada;
    app.pestana = Pestana::Armar;

    assert!(app.por_resolver().is_empty());
    assert!(app.esta_listo());
    let p_listo = pantalla(&app);
    assert!(p_listo.contains("COOKBOOK LISTO"), "{p_listo}");
}

#[test]
fn layout_80x24_pestanas_salida_en_todos_los_modos_y_no_color() {
    let mut app = App::nueva(recetario());
    app.color = false;

    // 1. Armar
    let p_armar = pantalla_en(&app, 80, 24);
    assert_eq!(p_armar.lines().count(), 24);
    assert!(
        p_armar.contains("1 Armar  2 Instalar  3 Checklist"),
        "{p_armar}"
    );
    assert!(p_armar.contains("?  ayuda"), "{p_armar}");

    // 2. Instalar
    app.pestana = Pestana::Instalacion;
    let p_inst = pantalla_en(&app, 80, 24);
    assert_eq!(p_inst.lines().count(), 24);
    assert!(
        p_inst.contains("1 Armar  2 Instalar  3 Checklist"),
        "{p_inst}"
    );
    assert!(p_inst.contains("↑↓ mover"), "{p_inst}");

    // 3. Checklist
    app.pestana = Pestana::Checklist;
    let p_check = pantalla_en(&app, 80, 24);
    assert_eq!(p_check.lines().count(), 24);
    assert!(
        p_check.contains("1 Armar  2 Instalar  3 Checklist"),
        "{p_check}"
    );
    assert!(p_check.contains("? ayuda"), "{p_check}");

    // 4. Ayuda
    app.modo = rebake::tui::app::Modo::Ayuda;
    let p_ayuda = pantalla_en(&app, 80, 24);
    assert_eq!(p_ayuda.lines().count(), 24);
    assert!(p_ayuda.contains("1 Armar"), "{p_ayuda}");
    assert!(p_ayuda.contains("2 Instalar"), "{p_ayuda}");
    assert!(p_ayuda.contains("3 Checklist"), "{p_ayuda}");
    assert!(p_ayuda.contains("Esc cierra"), "{p_ayuda}");
}

#[test]
fn v_no_aprueba_ni_cuenta_hooks_vinculados() {
    let mut r = Recetario::nuevo();
    let mut rev = con_fuente("plugin:rev@x");
    rev.estado = Estado::PorRevisar;
    let mut hook = Item::nuevo("hook:tool hook", Tipo::Hook);
    hook.instalado_por = Some(InstaladorHook::Externo("tool".into()));
    hook.estado = Estado::PorRevisar;
    r.items = vec![rev, hook];
    let mut app = App::nueva(r);

    // En el pie solo cuenta la receta por revisar propia (1), no el hook vinculado
    assert!(
        pie(&app)[0].contains(" v  aprobar 1 "),
        "{:?}",
        pie(&app)[0]
    );
    assert!(!pie(&app)[0].contains("aprobar 2"));

    // v pide confirmar solo 1 receta
    assert_eq!(tecla(&mut app, KeyCode::Char('v')), Efecto::Nada);
    assert!(
        pie(&app)[1].contains("¿Aprobar 1 recetas? s confirma; otra tecla cancela"),
        "{:?}",
        pie(&app)[1]
    );

    // s confirma y solo aprueba la receta independiente
    assert_eq!(tecla(&mut app, KeyCode::Char('s')), Efecto::Guardar);
    assert_eq!(
        app.recetario.item("plugin:rev@x").unwrap().estado,
        Estado::Aprobada
    );
    assert_eq!(
        app.recetario.item("hook:tool hook").unwrap().estado,
        Estado::PorRevisar
    );

    // Sin recetas por revisar (solo queda el hook vinculado), v no pregunta
    assert_eq!(tecla(&mut app, KeyCode::Char('v')), Efecto::Nada);
    assert!(
        app.mensaje
            .as_deref()
            .unwrap()
            .contains("no hay recetas por revisar"),
        "{:?}",
        app.mensaje
    );
}

#[test]
fn tanda_con_herramienta_con_hooks_vinculados_sin_registro_hooks_no_cuenta_como_por_revisar() {
    use rebake::investigador::Receta;
    let mut r = Recetario::nuevo();
    let mut tool = con_fuente("herramienta:tool");
    tool.tipo = Tipo::Herramienta;
    let mut hook = Item::nuevo("hook:tool hook", Tipo::Hook);
    hook.instalado_por = Some(InstaladorHook::Item(tool.id.clone()));
    r.items = vec![tool, hook];
    let mut app = App::nueva(r);

    tecla(&mut app, KeyCode::Char('t'));
    let Efecto::Investigar(inv) = tecla(&mut app, KeyCode::Char('s')) else {
        panic!("esperaba Efecto::Investigar");
    };
    assert_eq!(inv, vec!["herramienta:tool"]);

    app.registrar_investigacion(
        "herramienta:tool",
        Ok(Receta {
            repo: "https://github.com/a/b".into(),
            doc: "doc".into(),
            pasos: vec![],
            requiere: vec![],
            verificar: None,
            registro_hooks: None,
        }),
        "2026-10-28",
    );

    let msg = app.mensaje.as_deref().unwrap();
    assert!(!msg.contains("por revisar"), "{msg}");
    assert!(msg.contains("1 error"), "{msg}");
    assert!(!msg.contains("`v`"), "{msg}");
    assert_eq!(
        app.recetario.item("herramienta:tool").unwrap().estado,
        Estado::Pendiente
    );
}

#[test]
fn sincronizados_no_aparecen_ni_entran_en_armar_o_aprobar() {
    let mut i = con_fuente("skill:pdf");
    i.perfiles = vec!["laburo".into()];
    i.sincronizado_en = vec!["laburo".into()];
    let mut app = con_un_item(i);
    assert!(app.visibles().is_empty());
    assert_eq!(app.por_armar(), (vec![], vec![], vec![]));
    app.recetario.items[0].estado = Estado::PorRevisar;
    tecla(&mut app, KeyCode::Char('v'));
    assert_eq!(app.modo, rebake::tui::app::Modo::Normal);
    assert!(!pantalla(&app).contains("pdf"));
}

#[test]
fn excluir_ocultar_y_reincluir_conserva_pasos_y_una_seleccion_valida() {
    let mut app = App::nueva(Recetario::nuevo());
    for id in ["plugin:a@x", "plugin:b@x", "plugin:c@x"] {
        app.recetario.items.push(con_fuente(id));
    }
    app.seleccion = 1;
    assert_eq!(tecla(&mut app, KeyCode::Char('x')), Efecto::Guardar);
    assert_eq!(
        app.recetario.item("plugin:b@x").unwrap().estado,
        Estado::Excluida
    );
    assert_eq!(app.seleccionado().unwrap().id, "plugin:c@x");
    assert_eq!(app.visibles().len(), 2);
    let p = pantalla(&app);
    assert!(p.contains("1 excluidas"), "{p}");
    tecla(&mut app, KeyCode::Char('o'));
    assert_eq!(app.seleccionado().unwrap().id, "plugin:c@x");
    assert_eq!(app.visibles().len(), 3);
    let idx_b = app
        .visibles()
        .iter()
        .position(|i| i.id == "plugin:b@x")
        .unwrap();
    app.seleccion = idx_b;
    assert_eq!(tecla(&mut app, KeyCode::Char('x')), Efecto::Guardar);
    assert_eq!(
        app.recetario.item("plugin:b@x").unwrap().estado,
        Estado::Pendiente
    );
    app.filtro = "no-existe".into();
    tecla(&mut app, KeyCode::Char('o'));
    assert!(app.seleccionado().is_none());
    assert_eq!(app.seleccion, 0);
    pantalla(&app);
}

#[test]
fn ocultar_ultimo_excluido_no_deja_indice_fuera_de_lista() {
    let mut app = con_un_item(con_fuente("plugin:a@x"));
    tecla(&mut app, KeyCode::Char('x'));
    assert!(app.visibles().is_empty());
    assert!(app.seleccionado().is_none());
    assert_eq!(app.seleccion, 0);
    tecla(&mut app, KeyCode::Char('o'));
    assert_eq!(app.seleccionado().unwrap().id, "plugin:a@x");
    tecla(&mut app, KeyCode::Char('x'));
    assert_eq!(app.seleccionado().unwrap().estado, Estado::Pendiente);
}

#[test]
fn detalle_de_receta_mixta_solo_anuncia_el_perfil_propio() {
    let mut r = Recetario::nuevo();
    r.perfiles = vec![
        Perfil {
            nombre: "cuenta".into(),
            dir: "~/.claude".into(),
        },
        Perfil {
            nombre: "personal".into(),
            dir: "~/.claude-personal".into(),
        },
    ];
    let mut i = con_fuente("skill:mixta");
    i.perfiles = vec!["cuenta".into(), "personal".into()];
    i.sincronizado_en = vec!["cuenta".into()];
    r.items.push(i);
    let p = pantalla(&App::nueva(r));
    assert!(p.contains("Solo en: personal"), "{p}");
}
