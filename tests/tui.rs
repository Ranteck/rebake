use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use recetario::modelo::*;
use recetario::tui::app::{App, Efecto};
use recetario::tui::vista;

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

#[test]
fn investigar_todo_pide_solo_pendientes() {
    let mut app = App::nueva(recetario());
    assert_eq!(
        tecla(&mut app, KeyCode::Char('I')),
        Efecto::Investigar(vec!["statusline:statusline.sh".into()])
    );
    assert!(app.investigando.contains("statusline:statusline.sh"));
}
