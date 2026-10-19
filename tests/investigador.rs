mod comun;
use comun::HomeFalso;
use rebake::investigador::*;
use rebake::modelo::*;
use std::time::Duration;

const VALIDA: &str = r#"{"type":"result","subtype":"success","is_error":false,"result":"ok","structured_output":{
 "repo":"https://github.com/ejemplo/house-rules.git","doc":"https://github.com/ejemplo/house-rules#install",
 "verificar":"test -f \"$CLAUDE_CONFIG_DIR/HOUSE-RULES.md\"",
 "pasos":[{"cmd":"curl -fsSL https://ejemplo/install.sh | sh","modo":"auto","por_perfil":true,"cita":"curl -fsSL https://ejemplo/install.sh | sh"}],
 "requiere":[{"tipo":"herramienta","nombre":"codex"},{"tipo":"sistema","nombre":"jq"}]}}"#;

fn claude_falso(h: &HomeFalso, salida: &str) -> String {
    let script = format!(
        "#!/bin/sh\ncat <<'FIN'\nruido en una linea previa\n{}\nFIN\n",
        salida.replace('\n', "")
    );
    h.binario("bin/claude", script.as_bytes())
        .display()
        .to_string()
}

fn recetario() -> Recetario {
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil {
        nombre: "laburo".into(),
        dir: "~/.claude".into(),
    });
    let mut item = Item::nuevo("import:HOUSE-RULES.md", Tipo::Import);
    item.perfiles = vec!["laburo".into()];
    item.fuente = Some(Fuente {
        repo: "https://github.com/ejemplo/house-rules".into(),
        via: Via::Symlink,
        doc: None,
    });
    r.items.push(item);
    r
}

const ID: &str = "import:HOUSE-RULES.md";

fn trabajo() -> Trabajo {
    Trabajo {
        id: ID.into(),
        prompt: "x".into(),
        directa: None,
    }
}

#[test]
fn argumentos_sin_bash() {
    let a = argumentos("investigá", "sonnet");
    for flag in [
        "-p",
        "--restricted",
        "--strict-mcp-config",
        "--no-session-persistence",
        "--json-schema",
    ] {
        assert!(a.iter().any(|x| x == flag), "falta {flag}");
    }
    let tools = a.iter().position(|x| x == "--tools").unwrap();
    assert_eq!(a[tools + 1], "WebSearch,WebFetch");
    assert!(a.iter().all(|x| !x.contains("Bash")));
    let m = a.iter().position(|x| x == "--model").unwrap();
    assert_eq!(a[m + 1], "sonnet");
}

#[test]
fn argumentos_respetan_el_modelo() {
    assert!(
        argumentos("x", "opus")
            .windows(2)
            .any(|w| w == ["--model", "opus"])
    );
}

#[test]
fn respuesta_valida_queda_por_revisar() {
    let h = HomeFalso::nuevo();
    let claude = claude_falso(&h, VALIDA);
    let mut r = recetario();
    let receta = investigar(&trabajo(), &claude, Duration::from_secs(10));
    aplicar(&mut r, ID, receta, "2026-10-07");
    let x = r.item(ID).unwrap();
    assert_eq!(x.estado, Estado::PorRevisar);
    assert_eq!(x.pasos.len(), 1);
    assert_eq!(
        x.pasos[0].cita.as_deref(),
        Some("curl -fsSL https://ejemplo/install.sh | sh")
    );
    let f = x.fuente.clone().unwrap();
    assert_eq!(f.via, Via::Symlink);
    assert_eq!(
        f.doc.as_deref(),
        Some("https://github.com/ejemplo/house-rules#install")
    );
    assert_eq!(x.investigado.as_deref(), Some("2026-10-07"));
    assert!(x.error.is_none());
}

#[test]
fn json_invalido_queda_pendiente_con_error() {
    let h = HomeFalso::nuevo();
    let claude = claude_falso(
        &h,
        r#"{"type":"result","subtype":"success","is_error":false,"structured_output":{"repo":"x","doc":"y"}}"#,
    );
    let mut r = recetario();
    aplicar(
        &mut r,
        ID,
        investigar(&trabajo(), &claude, Duration::from_secs(10)),
        "2026-10-07",
    );
    let x = r.item(ID).unwrap();
    assert_eq!(x.estado, Estado::Pendiente);
    assert!(
        x.error.as_deref().unwrap().contains("formato"),
        "{:?}",
        x.error
    );
}

#[test]
fn tope_queda_pendiente() {
    let h = HomeFalso::nuevo();
    let claude = h
        .binario("bin/claude", b"#!/bin/sh\nsleep 30\n")
        .display()
        .to_string();
    let mut r = recetario();
    aplicar(
        &mut r,
        ID,
        investigar(&trabajo(), &claude, Duration::from_millis(300)),
        "2026-10-07",
    );
    let x = r.item(ID).unwrap();
    assert_eq!(x.estado, Estado::Pendiente);
    assert!(x.error.as_deref().unwrap().contains("superó"));
}

#[test]
fn requisito_nuevo_y_de_sistema() {
    let h = HomeFalso::nuevo();
    let claude = claude_falso(&h, VALIDA);
    let mut r = recetario();
    aplicar(
        &mut r,
        ID,
        investigar(&trabajo(), &claude, Duration::from_secs(10)),
        "2026-10-07",
    );
    let codex = r.item("herramienta:codex").unwrap();
    assert_eq!(codex.estado, Estado::Pendiente);
    assert_eq!(
        codex.pista.as_deref(),
        Some("requisito de import:HOUSE-RULES.md")
    );
    assert!(
        r.item(ID)
            .unwrap()
            .requiere
            .contains(&"herramienta:codex".to_string())
    );
    assert_eq!(
        r.checklist.sistema,
        vec![PaqueteSistema {
            paquete: "jq".into(),
            para: ID.into()
        }]
    );
    assert!(r.item("sistema:jq").is_none());
}

#[test]
fn requisito_excluido_no_revive() {
    let h = HomeFalso::nuevo();
    let claude = claude_falso(&h, VALIDA);
    let mut r = recetario();
    let mut codex = Item::nuevo("herramienta:codex", Tipo::Herramienta);
    codex.estado = Estado::Excluida;
    codex.motivo = Some("no lo uso".into());
    r.items.push(codex);
    aplicar(
        &mut r,
        ID,
        investigar(&trabajo(), &claude, Duration::from_secs(10)),
        "2026-10-07",
    );
    assert_eq!(
        r.item("herramienta:codex").unwrap().estado,
        Estado::Excluida
    );
}

#[test]
fn en_paralelo_devuelve_todos() {
    let h = HomeFalso::nuevo();
    let claude = claude_falso(&h, VALIDA);
    let trabajos = (0..3)
        .map(|i| Trabajo {
            id: format!("skill:s{i}"),
            prompt: "x".into(),
            directa: None,
        })
        .collect();
    let rx = en_paralelo(trabajos, claude, Duration::from_secs(10), 2);
    let mut ids: Vec<String> = rx
        .iter()
        .map(|(id, r)| {
            assert!(r.is_ok());
            id
        })
        .collect();
    ids.sort();
    assert_eq!(ids, ["skill:s0", "skill:s1", "skill:s2"]);
}

#[test]
fn argumentos_preaprueban_la_web() {
    let a = argumentos("investigá", "sonnet");
    let i = a
        .iter()
        .position(|x| x == "--allowedTools")
        .expect("falta --allowedTools");
    assert_eq!(a[i + 1], "WebSearch,WebFetch");
}

fn salida(estructurada: &str, denegados: &str) -> String {
    format!(
        r#"{{"type":"result","subtype":"success","is_error":false,"permission_denials":{denegados},"structured_output":{estructurada}}}"#
    )
}

#[test]
fn permiso_denegado_es_error() {
    let s = salida(
        r#"{"repo":"https://github.com/a/b","doc":"https://github.com/a/b#install","pasos":[],"requiere":[]}"#,
        r#"[{"tool_name":"WebFetch","tool_use_id":"x","tool_input":{}}]"#,
    );
    let err = parsear_salida(&s).unwrap_err().to_string();
    assert!(err.contains("permiso") && err.contains("WebFetch"), "{err}");
}

#[test]
fn doc_sin_url_es_error_y_con_texto_se_extrae() {
    let sin_url = salida(
        r#"{"repo":"https://github.com/a/b","doc":"NO VERIFICADO: no se pudo leer el README","pasos":[],"requiere":[]}"#,
        "[]",
    );
    assert!(
        parsear_salida(&sin_url)
            .unwrap_err()
            .to_string()
            .contains("URL")
    );
    let con_texto = salida(
        r#"{"repo":"https://github.com/a/b","doc":"README.md (https://github.com/a/b#readme)","pasos":[],"requiere":[]}"#,
        "[]",
    );
    assert_eq!(
        parsear_salida(&con_texto).unwrap().doc,
        "https://github.com/a/b#readme"
    );
}

#[test]
fn requisito_con_aclaracion_usa_solo_el_identificador() {
    let mut r = recetario();
    r.items
        .push(Item::nuevo("marketplace:openai-codex", Tipo::Marketplace));
    let receta = Receta {
        repo: "https://github.com/ejemplo/house-rules".into(),
        doc: "https://github.com/ejemplo/house-rules#install".into(),
        pasos: vec![],
        requiere: vec![
            Requisito {
                tipo: "marketplace".into(),
                nombre: "openai-codex (openai/codex-plugin-cc)".into(),
            },
            Requisito {
                tipo: "herramienta".into(),
                nombre: "codex (@openai/codex, CLI global)".into(),
            },
        ],
        verificar: None,
    };
    aplicar(&mut r, ID, Ok(receta), "2026-10-07");
    assert_eq!(
        r.item(ID).unwrap().requiere,
        ["marketplace:openai-codex", "herramienta:codex"]
    );
    assert!(
        r.items.iter().all(|i| !i.id.contains('(')),
        "{:?}",
        r.items.iter().map(|i| &i.id).collect::<Vec<_>>()
    );
}

#[test]
fn el_prompt_no_lleva_secretos_de_los_archivos() {
    let h = HomeFalso::nuevo();
    h.escribir(".claude/statusline.sh", "#!/bin/bash\nAPI_TOKEN=supersecreto\ncurl -H \"Authorization: Bearer otrosecreto\" https://x\n");
    let mut r = recetario();
    let mut item = Item::nuevo("statusline:statusline.sh", Tipo::Statusline);
    item.perfiles = vec!["laburo".into()];
    item.pista = Some("archivo sin origen conocido: ~/.claude/statusline.sh".into());
    r.items.push(item.clone());
    let t = trabajo_para(&r, &item, h.ruta());
    assert!(
        t.prompt.contains("#!/bin/bash"),
        "el prompt tiene que incluir el archivo"
    );
    assert!(
        !t.prompt.contains("supersecreto") && !t.prompt.contains("otrosecreto"),
        "{}",
        t.prompt
    );
}

fn item_con(id: &str, tipo: Tipo, repo: Option<&str>, via: Via) -> Item {
    let mut item = Item::nuevo(id, tipo);
    item.fuente = repo.map(|r| Fuente {
        repo: r.into(),
        via,
        doc: None,
    });
    item
}

fn directa(item: &Item) -> Option<Receta> {
    trabajo_para(&Recetario::nuevo(), item, std::path::Path::new("/nada")).directa
}

#[test]
fn marketplace_con_repo_es_directa_y_no_ejecuta_claude() {
    let item = item_con(
        "marketplace:claude-plugins-official",
        Tipo::Marketplace,
        Some("https://github.com/anthropics/claude-plugins-official"),
        Via::Metadatos,
    );
    let t = trabajo_para(&Recetario::nuevo(), &item, std::path::Path::new("/nada"));
    let receta = t.directa.clone().expect("tiene que ser directa");
    assert_eq!(
        receta.repo,
        "https://github.com/anthropics/claude-plugins-official"
    );
    assert_eq!(receta.pasos.len(), 1);
    let p = &receta.pasos[0];
    assert_eq!(
        p.cmd,
        "claude plugin marketplace add https://github.com/anthropics/claude-plugins-official"
    );
    assert!(p.por_perfil && p.modo == Modo::Auto && p.cita.is_none());
    assert_eq!(
        receta.verificar.as_deref(),
        Some("claude plugin marketplace list | grep -qF -- 'claude-plugins-official'")
    );
    let h = HomeFalso::nuevo();
    let falso = h
        .binario("bin/claude", b"#!/bin/sh\nexit 1\n")
        .display()
        .to_string();
    assert_eq!(
        investigar(&t, &falso, Duration::from_secs(10)).unwrap(),
        receta
    );
}

#[test]
fn plugin_directo_guarda_el_marketplace() {
    let item = item_con(
        "plugin:amd-skills@claude-plugins-official",
        Tipo::Plugin,
        Some("https://github.com/anthropics/claude-plugins-official"),
        Via::Metadatos,
    );
    let receta = directa(&item).unwrap();
    let m = "claude-plugins-official";
    assert_eq!(
        receta.pasos[0].cmd,
        format!(
            "claude plugin marketplace list | grep -qF -- '{m}' || {{ echo 'falta el marketplace {m}: aprobá marketplace:{m}' >&2; exit 1; }}; claude plugin install amd-skills@{m}"
        )
    );
    assert_eq!(
        receta.verificar.as_deref(),
        Some("claude plugin list | grep -qF -- 'amd-skills@claude-plugins-official'")
    );
}

#[test]
fn skill_de_metadatos_es_directa_con_npx() {
    let item = item_con(
        "skill:pragmatic-programmer",
        Tipo::Skill,
        Some("https://github.com/wondelai/skills"),
        Via::Metadatos,
    );
    let receta = directa(&item).unwrap();
    let p = &receta.pasos[0];
    assert_eq!(
        p.cmd,
        "command -v npx >/dev/null || { echo 'falta npx: instalá Node.js' >&2; exit 1; }; npx -y skills add https://github.com/wondelai/skills --skill pragmatic-programmer -g -a claude-code -y"
    );
    assert!(p.nota.as_deref().unwrap().starts_with("receta directa"));
    assert_eq!(
        receta.verificar.as_deref(),
        Some("test -f \"$CLAUDE_CONFIG_DIR/skills/pragmatic-programmer/SKILL.md\"")
    );
}

#[test]
fn los_demas_casos_no_son_directos() {
    let repo = Some("https://github.com/a/b");
    for item in [
        item_con("skill:x", Tipo::Skill, repo, Via::Symlink),
        item_con("herramienta:x", Tipo::Herramienta, repo, Via::Metadatos),
        item_con("skill:x", Tipo::Skill, None, Via::Metadatos),
        item_con("plugin:x;rm@m", Tipo::Plugin, repo, Via::Metadatos),
        item_con("plugin:x@m y", Tipo::Plugin, repo, Via::Metadatos),
        item_con("marketplace:", Tipo::Marketplace, repo, Via::Metadatos),
        item_con(
            "marketplace:m",
            Tipo::Marketplace,
            Some("https://github.com/a/$(id)"),
            Via::Metadatos,
        ),
        item_con(
            "marketplace:m",
            Tipo::Marketplace,
            Some("git@github.com:a/b"),
            Via::Metadatos,
        ),
    ] {
        assert!(directa(&item).is_none(), "{}", item.id);
    }
}

#[test]
fn aplicar_directa_conserva_la_via() {
    let mut r = Recetario::nuevo();
    let item = item_con(
        "skill:pragmatic-programmer",
        Tipo::Skill,
        Some("https://github.com/wondelai/skills"),
        Via::Metadatos,
    );
    r.items.push(item.clone());
    let t = trabajo_para(&r, &item, std::path::Path::new("/nada"));
    aplicar(&mut r, &item.id, Ok(t.directa.unwrap()), "2026-10-14");
    let x = r.item(&item.id).unwrap();
    assert_eq!(x.estado, Estado::PorRevisar);
    assert_eq!(x.fuente.as_ref().unwrap().via, Via::Metadatos);
}

#[test]
fn las_directas_salen_antes_que_las_que_esperan_a_claude() {
    let h = HomeFalso::nuevo();
    let claude = h
        .binario(
            "bin/claude",
            format!(
                "#!/bin/sh\nsleep 1\ncat <<'FIN'\n{}\nFIN\n",
                VALIDA.replace('\n', "")
            )
            .as_bytes(),
        )
        .display()
        .to_string();
    let directa = Receta {
        repo: "https://github.com/ejemplo/repo".into(),
        doc: "receta directa".into(),
        pasos: vec![],
        requiere: vec![],
        verificar: None,
    };
    // La que va a Claude queda última en la lista: sin orden, sale primera de la fila.
    let trabajos = vec![
        Trabajo {
            id: "marketplace:directa".into(),
            prompt: String::new(),
            directa: Some(directa),
        },
        Trabajo {
            id: "skill:lenta".into(),
            prompt: "x".into(),
            directa: None,
        },
    ];
    let rx = en_paralelo(trabajos, claude, Duration::from_secs(10), 1);
    let ids: Vec<String> = rx.iter().map(|(id, _)| id).collect();
    assert_eq!(ids, ["marketplace:directa", "skill:lenta"]);
}

fn serena() -> (Recetario, &'static str) {
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil {
        nombre: "laburo".into(),
        dir: "~/.claude".into(),
    });
    let mut item = Item::nuevo("mcp:serena", Tipo::Mcp);
    item.perfiles = vec!["laburo".into()];
    item.pista = Some("servidor MCP: serena start-mcp-server".into());
    item.fuente = Some(Fuente {
        repo: "https://github.com/oraios/serena".into(),
        via: Via::Metadatos,
        doc: None,
    });
    r.items.push(item);
    (r, "mcp:serena")
}

#[test]
fn un_servidor_mcp_va_a_claude_y_sus_pasos_quedan_manuales() {
    let (r, id) = serena();
    let t = trabajo_para(&r, r.item(id).unwrap(), std::path::Path::new("/home/x"));
    assert!(t.directa.is_none());
    assert!(t.prompt.contains("servidor MCP: serena start-mcp-server"));
    assert!(t.prompt.contains("claude mcp add --scope user serena"));
    assert!(t.prompt.contains("No pongas CLAUDE_CONFIG_DIR"));

    let h = HomeFalso::nuevo();
    let claude = claude_falso(&h, VALIDA);
    let mut r = r;
    let receta = investigar(&t, &claude, Duration::from_secs(10));
    aplicar(&mut r, id, receta, "2026-10-17");
    let x = r.item(id).unwrap();
    assert_eq!(x.estado, Estado::PorRevisar);
    assert!(x.pasos.iter().all(|p| p.modo == Modo::Manual));
    assert_eq!(
        x.verificar.as_deref(),
        Some("claude mcp get 'serena' >/dev/null 2>&1")
    );
}

#[test]
fn un_servidor_mcp_sin_origen_no_se_investiga_en_lote() {
    let (mut r, id) = serena();
    r.item_mut(id).unwrap().fuente = None;
    let t = rebake::servicio::trabajos(&r, &[], std::path::Path::new("/home/x")).unwrap();
    assert!(t.is_empty());
    assert_eq!(rebake::servicio::sin_origen(&r), [id]);
}

fn origen(estructurada: &str) -> String {
    format!(
        r#"{{"type":"result","subtype":"success","is_error":false,"result":"ok","structured_output":{estructurada}}}"#
    )
}

#[test]
fn la_busqueda_de_origen_pide_solo_el_repo() {
    let (mut r, id) = serena();
    r.item_mut(id).unwrap().fuente = None;
    let prompt = prompt_origen(r.item(id).unwrap(), std::path::Path::new("/home/x"));
    assert!(prompt.contains("mcp:serena"));
    assert!(prompt.contains("servidor MCP: serena start-mcp-server"));
    assert!(prompt.contains("No armes la receta"));
    assert!(!prompt.contains("Reglas:"));

    let h = HomeFalso::nuevo();
    let claude = claude_falso(
        &h,
        &origen(
            r#"{"repo":"https://github.com/oraios/serena.git","motivo":"el paquete serena-agent es de oraios"}"#,
        ),
    );
    let posible = buscar_origen(&prompt, &claude, Duration::from_secs(10)).unwrap();
    assert_eq!(
        posible.repo.as_deref(),
        Some("https://github.com/oraios/serena")
    );
    aplicar_posible(&mut r, id, Ok(posible));
    let x = r.item(id).unwrap();
    // Solo queda propuesto: ni origen, ni receta, ni cambio de estado.
    assert!(x.fuente.is_none());
    assert!(x.pasos.is_empty());
    assert_eq!(x.estado, Estado::Pendiente);
    assert_eq!(
        x.posible.as_ref().unwrap().motivo,
        "el paquete serena-agent es de oraios"
    );
}

#[test]
fn un_repo_vacio_o_raro_no_se_propone() {
    let vacio = parsear_origen(&origen(r#"{"repo":"","motivo":"no hay nada claro"}"#)).unwrap();
    assert!(vacio.repo.is_none());
    assert_eq!(vacio.motivo, "no hay nada claro");
    let raro = parsear_origen(&origen(
        r#"{"repo":"https://github.com/x/y;rm -rf ~","motivo":"confiá"}"#,
    ))
    .unwrap();
    assert!(raro.repo.is_none());
    assert!(parsear_origen(&origen(r#"{"repo":"x"}"#)).is_err());
}

#[test]
fn un_posible_no_pisa_un_origen_ya_resuelto_ni_un_excluido() {
    let (mut r, id) = serena();
    let posible = || {
        Ok(Posible {
            repo: Some("https://github.com/otro/repo".into()),
            motivo: "x".into(),
            descartado: false,
        })
    };
    aplicar_posible(&mut r, id, posible());
    assert!(r.item(id).unwrap().posible.is_none());
    let x = r.item_mut(id).unwrap();
    x.fuente = None;
    x.estado = Estado::Excluida;
    aplicar_posible(&mut r, id, posible());
    assert!(r.item(id).unwrap().posible.is_none());
}

#[test]
fn un_requisito_con_forma_de_repo_usa_el_item_que_ya_tiene_ese_repo() {
    let mut r = recetario();
    let mut mkt = Item::nuevo("marketplace:openai-codex", Tipo::Marketplace);
    mkt.fuente = Some(Fuente {
        repo: "https://github.com/openai/codex-plugin-cc".into(),
        via: Via::Metadatos,
        doc: None,
    });
    r.items.push(mkt);
    let antes = r.items.len();
    let receta = Receta {
        repo: "https://github.com/ejemplo/house-rules".into(),
        doc: "https://github.com/ejemplo/house-rules#install".into(),
        pasos: vec![],
        requiere: vec![
            Requisito {
                tipo: "marketplace".into(),
                nombre: "openai/codex-plugin-cc".into(),
            },
            Requisito {
                tipo: "marketplace".into(),
                nombre: "https://github.com/OpenAI/codex-plugin-cc.git".into(),
            },
            // Claude Code es requisito de rebake, no una receta.
            Requisito {
                tipo: "herramienta".into(),
                nombre: "claude".into(),
            },
        ],
        verificar: None,
    };
    aplicar(&mut r, ID, Ok(receta), "2026-10-18");
    assert_eq!(r.item(ID).unwrap().requiere, ["marketplace:openai-codex"]);
    assert_eq!(r.items.len(), antes, "se creó un ítem de más");
}

#[test]
fn un_repo_sin_item_conocido_se_agrega_como_hasta_ahora() {
    let mut r = recetario();
    let receta = Receta {
        repo: "https://github.com/ejemplo/house-rules".into(),
        doc: "https://github.com/ejemplo/house-rules#install".into(),
        pasos: vec![],
        requiere: vec![Requisito {
            tipo: "marketplace".into(),
            nombre: "otro/repo".into(),
        }],
        verificar: None,
    };
    aplicar(&mut r, ID, Ok(receta), "2026-10-18");
    assert_eq!(r.item(ID).unwrap().requiere, ["marketplace:otro/repo"]);
    assert!(r.item("marketplace:otro/repo").is_some());
}

#[test]
fn un_repo_con_varios_plugins_no_elige_uno_al_azar() {
    let mut r = recetario();
    for id in ["plugin:a@oficial", "plugin:b@oficial"] {
        let mut p = Item::nuevo(id, Tipo::Plugin);
        p.fuente = Some(Fuente {
            repo: "https://github.com/anthropics/claude-plugins-official".into(),
            via: Via::Metadatos,
            doc: None,
        });
        r.items.push(p);
    }
    let receta = Receta {
        repo: "https://github.com/ejemplo/house-rules".into(),
        doc: "https://github.com/ejemplo/house-rules#install".into(),
        pasos: vec![],
        requiere: vec![Requisito {
            tipo: "plugin".into(),
            nombre: "anthropics/claude-plugins-official".into(),
        }],
        verificar: None,
    };
    aplicar(&mut r, ID, Ok(receta), "2026-10-18");
    let requiere = &r.item(ID).unwrap().requiere;
    assert!(
        !requiere.contains(&"plugin:a@oficial".to_string())
            && !requiere.contains(&"plugin:b@oficial".to_string()),
        "{requiere:?}"
    );
}
