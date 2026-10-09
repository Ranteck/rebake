use rebake::modelo::{Estado, Item, Recetario, Tipo};
use rebake::servicio::mensaje_commit;

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
