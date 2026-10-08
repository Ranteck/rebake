use recetario::archivo;
use recetario::modelo::*;
use std::fs;

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
    let ruta = dir.path().join("recetario.toml");
    let r = ejemplo();
    archivo::guardar(&ruta, &r).unwrap();
    assert_eq!(archivo::leer(&ruta).unwrap(), r);
}

#[test]
fn campo_desconocido_da_linea_y_columna() {
    let texto = "version = 1\n\n[[item]]\nid = \"claude\"\ntipo = \"claude\"\nestado = \"pendiente\"\ncolor = \"rojo\"\n";
    let err = archivo::parsear(texto, "recetario.toml")
        .unwrap_err()
        .to_string();
    assert!(err.starts_with("recetario.toml:"), "{err}");
    assert!(err.contains("color"), "{err}");
}

#[test]
fn guardar_a_traves_de_symlink_conserva_el_enlace() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("repo-privado").join("recetario.toml");
    fs::create_dir_all(real.parent().unwrap()).unwrap();
    fs::write(&real, "version = 1\n").unwrap();
    let enlace = dir.path().join("recetario.toml");
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
fn ruta_por_defecto_en_config() {
    let home = std::path::Path::new("/home/ejemplo");
    assert_eq!(
        recetario::rutas::archivo_por_defecto(home),
        home.join(".config/recetario/recetario.toml")
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
