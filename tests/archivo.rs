use rebake::archivo;
use rebake::modelo::*;
use std::fs;

#[test]
fn responsable_de_hook_se_conserva_y_se_valida() {
    let texto = "version = 1\n[[item]]\nid = 'herramienta:tool'\ntipo = 'herramienta'\nestado = 'pendiente'\n[[item]]\nid = 'hook:tool'\ntipo = 'hook'\nestado = 'pendiente'\ninstalado_por = { tipo = 'item', nombre = 'herramienta:tool' }\n";
    let r = archivo::parsear(texto, "cookbook.toml").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let ruta = dir.path().join("cookbook.toml");
    archivo::guardar(&ruta, &r).unwrap();
    assert_eq!(archivo::leer(&ruta).unwrap(), r);
    assert!(fs::read_to_string(ruta).unwrap().contains("instalado_por"));
    assert!(
        archivo::parsear(
            &texto.replace("nombre = 'herramienta:tool'", "nombre = 'no-existe'"),
            "cookbook.toml"
        )
        .is_err()
    );
    assert!(
        archivo::parsear(
            &texto.replace("tipo = 'hook'", "tipo = 'skill'"),
            "cookbook.toml"
        )
        .is_err()
    );
}

#[test]
fn campo_desconocido_de_cookbook_nuevo_incluye_como_actualizar() {
    let texto = "version = 1\n[[item]]\nid = 'hook:tool'\ntipo = 'hook'\nestado = 'pendiente'\ncampo_de_otra_version = 'nuevo'\n";
    let err = format!(
        "{:#}",
        archivo::parsear(texto, "cookbook.toml").unwrap_err()
    );
    assert!(err.contains("unknown field"), "{err}");
    assert!(err.contains("actualizá con: curl -fsSL https://github.com/Ranteck/rebake/releases/latest/download/install.sh | sh"), "{err}");
}

fn ejemplo() -> Recetario {
    let mut r = Recetario::nuevo();
    r.perfiles.push(Perfil {
        nombre: "laburo".into(),
        dir: "~/.claude".into(),
    });
    let mut item = Item::nuevo("plugin:codex@openai-codex", Tipo::Plugin);
    item.perfiles = vec!["laburo".into()];
    item.estado = Estado::Aprobada;
    item.fuente = Some(Fuente {
        repo: "https://github.com/openai/codex-plugin-cc".into(),
        via: Via::Metadatos,
        doc: Some("https://github.com/openai/codex-plugin-cc#install".into()),
    });
    item.requiere = vec!["marketplace:openai-codex".into()];
    item.pasos.push(Paso {
        cmd: "claude plugin install codex@openai-codex".into(),
        modo: Modo::Auto,
        por_perfil: true,
        cita: Some("/plugin install codex@openai-codex".into()),
        nota: None,
    });
    r.items.push(item);
    let mut mcp = Item::nuevo("mcp:serena", Tipo::Mcp);
    mcp.pista = Some("servidor MCP: serena start-mcp-server".into());
    mcp.posible = Some(Posible {
        repo: Some("https://github.com/oraios/serena".into()),
        motivo: "lo dice el paquete".into(),
        descartado: true,
    });
    r.items.push(mcp);
    r.checklist.ajustes.push(Ajuste {
        perfil: "laburo".into(),
        clave: "theme".into(),
        valor: "dark".into(),
    });
    r
}

#[test]
fn ida_y_vuelta_sin_perdidas() {
    let dir = tempfile::tempdir().unwrap();
    let ruta = dir.path().join("cookbook.toml");
    let r = ejemplo();
    archivo::guardar(&ruta, &r).unwrap();
    assert_eq!(archivo::leer(&ruta).unwrap(), r);
    let texto = fs::read_to_string(&ruta).unwrap();
    assert!(texto.contains("tipo = \"mcp\""), "{texto}");
    assert!(texto.contains("descartado = true"), "{texto}");
}

#[test]
fn campo_desconocido_da_linea_y_columna() {
    let texto = "version = 1\n\n[[item]]\nid = \"claude\"\ntipo = \"claude\"\nestado = \"pendiente\"\ncolor = \"rojo\"\n";
    let err = archivo::parsear(texto, "cookbook.toml")
        .unwrap_err()
        .to_string();
    assert!(err.starts_with("cookbook.toml:"), "{err}");
    assert!(err.contains("color"), "{err}");
}

#[test]
fn guardar_a_traves_de_symlink_conserva_el_enlace() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("repo-privado").join("cookbook.toml");
    fs::create_dir_all(real.parent().unwrap()).unwrap();
    fs::write(&real, "version = 1\n").unwrap();
    let enlace = dir.path().join("cookbook.toml");
    std::os::unix::fs::symlink(&real, &enlace).unwrap();

    archivo::guardar(&enlace, &ejemplo()).unwrap();

    assert!(
        fs::symlink_metadata(&enlace)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(
        fs::read_to_string(&real)
            .unwrap()
            .contains("codex@openai-codex")
    );
}

#[test]
fn ruta_por_defecto_en_documentos_del_sistema() {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join(".config")).unwrap();
    fs::write(
        home.path().join(".config/user-dirs.dirs"),
        "# generado por xdg-user-dirs-update\nXDG_DESKTOP_DIR=\"$HOME/Escritorio\"\nXDG_DOCUMENTS_DIR=\"$HOME/Documentos\"\n",
    )
    .unwrap();
    assert_eq!(
        rebake::rutas::archivo_por_defecto(home.path()),
        home.path().join("Documentos/rebake/cookbook.toml")
    );
}

#[test]
fn sin_user_dirs_usa_documents() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(
        rebake::rutas::archivo_por_defecto(home.path()),
        home.path().join("Documents/rebake/cookbook.toml")
    );
}

#[test]
fn inexistente_da_recetario_vacio() {
    let dir = tempfile::tempdir().unwrap();
    let r = archivo::leer(&dir.path().join("no-existe.toml")).unwrap();
    assert_eq!(r, Recetario::nuevo());
}

#[test]
fn ids_repetidos_son_invalidos() {
    let mut r = ejemplo();
    r.items.push(r.items[0].clone());
    assert!(validar(&r).is_err());
}

#[test]
fn crear_privado_es_0600_y_no_sigue_symlinks() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let ruta = dir.path().join("editar.toml");
    archivo::crear_privado(&ruta, "contenido").unwrap();
    assert_eq!(
        fs::metadata(&ruta).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(fs::read_to_string(&ruta).unwrap(), "contenido");

    let victima = dir.path().join("victima");
    fs::write(&victima, "original").unwrap();
    let trampa = dir.path().join("trampa.toml");
    std::os::unix::fs::symlink(&victima, &trampa).unwrap();
    assert!(archivo::crear_privado(&trampa, "malicioso").is_err());
    assert_eq!(fs::read_to_string(&victima).unwrap(), "original");
}

#[test]
fn algo_desconocido_dice_como_actualizar_y_lo_mal_escrito_no() {
    let nuevo =
        "version = 1\n\n[[item]]\nid = \"x:y\"\ntipo = \"futuro\"\nestado = \"pendiente\"\n";
    let err = format!(
        "{:#}",
        archivo::parsear(nuevo, "cookbook.toml").unwrap_err()
    );
    assert!(err.starts_with("cookbook.toml:5:"), "{err}");
    assert!(err.contains("futuro"), "{err}");
    assert!(
        err.contains("si lo escribió un rebake más nuevo, actualizá con: curl -fsSL https://github.com/Ranteck/rebake/releases/latest/download/install.sh | sh"),
        "{err}"
    );
    // Un error de sintaxis no es de otra versión: no se sugiere actualizar.
    let roto = "version = 1\n[[item]\n";
    let err = format!("{:#}", archivo::parsear(roto, "cookbook.toml").unwrap_err());
    assert!(!err.contains("actualizá"), "{err}");
}

#[test]
fn sincronizacion_por_perfil_se_conserva_en_otra_pc_sin_archivos() {
    let texto = r#"version = 1
[[perfil]]
nombre = "cuenta"
dir = "~/.claude"
[[item]]
id = "skill:pdf"
tipo = "skill"
estado = "aprobada"
perfiles = ["cuenta"]
sincronizado_en = ["cuenta"]
"#;
    let r = archivo::parsear(texto, "prueba").unwrap();
    let home = tempfile::tempdir().unwrap();
    let destino = home.path().join("cookbook.toml");
    archivo::guardar(&destino, &r).unwrap();
    let leido = archivo::leer(&destino).unwrap();
    assert_eq!(
        serde_json::to_value(&leido.items[0]).unwrap()["sincronizado_en"],
        serde_json::json!(["cuenta"])
    );
    assert!(rebake::instalador::planificar(&leido).unwrap().is_empty());
}
