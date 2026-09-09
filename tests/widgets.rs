//! Tests de integración de widgets (Fase 5): composición apilada.
//!
//! Menubar + popup + confirm sobre `Screen`, con savescreen/restscreen
//! anidados como en la época (3 niveles).

use crossterm::event::KeyCode;

use g90tui::{
    confirm_draw, confirm_key, darken_color, menubar_draw, menubar_key, popup_draw, popup_key,
    popup_layout, progress_draw, BarInfo, Buffer, Cell, DialogKey, MenuBarKey, MenuDef, PopupItem,
    PopupKey, ProgressInfo, Rect, Screen, Theme,
};

fn menus() -> Vec<MenuDef> {
    vec![
        MenuDef::new("Archivos", &["Proveedores", "Clientes"]),
        MenuDef::new(
            "Varios",
            &["&Respaldo de datos", "&Ordenar índices", "&Finalizar"],
        ),
    ]
}

#[test]
fn popup_opens_under_active_menu() {
    let t = Theme::clipper();
    let mut buf = Buffer::blank(80, 25, t.desktop);
    let m = menus();
    menubar_draw(&mut buf, Rect::new(0, 1, 80, 1), &m, 1, t);
    // El popup de "Varios" cuelga de su título (x aprox 2+10+2+2=...). Lo
    // anclamos donde caiga el título: calculamos como el draw.
    let r = popup_layout(
        14,
        1,
        &m[1]
            .items
            .iter()
            .map(|i| PopupItem::item(&i.label))
            .collect::<Vec<_>>(),
        Rect::new(0, 0, 80, 25),
    );
    popup_draw(
        &mut buf,
        r,
        &m[1]
            .items
            .iter()
            .map(|i| PopupItem::item(&i.label))
            .collect::<Vec<_>>(),
        1,
        t,
    );
    // Fila 0 sin seleccionar: popup azul con hotkey amarilla.
    assert_eq!(buf.get(r.x + 2, r.y + 1).unwrap().bg, t.popup);
    assert_eq!(buf.get(r.x + 2, r.y + 1).unwrap().fg, t.hot);
    // Fila 1 seleccionada: barra gris.
    assert_eq!(buf.get(r.x + 1, r.y + 2).unwrap().bg, t.select_bg);
    assert_eq!(
        buf.get(r.right(), r.y + 1).unwrap().bg,
        darken_color(t.desktop, 0.40)
    );
}

#[test]
fn stacked_dialogs_save_and_restore() {
    let t = Theme::clipper();
    let mut s = Screen::new(80, 25, t);
    let bounds = s.bounds();
    s.frame().fill_rect(bounds, Cell::blank(t.desktop));
    let base = s.frame().get(10, 10).unwrap();

    // Nivel 1: popup de menú.
    let id1 = s.savescreen(Rect::new(40, 2, 30, 10));
    s.frame().fill_rect(
        Rect::new(40, 2, 30, 10),
        Cell::new(' ', t.popup_text, t.popup),
    );
    // Nivel 2: confirm encima.
    let id2 = s.savescreen(Rect::new(20, 10, 40, 7));
    confirm_draw(
        s.frame(),
        bounds,
        "AVISO",
        "¿ Está conforme ?",
        &["&Si".to_string(), "&No".to_string()],
        0,
        t,
    );
    // Cierran en orden inverso y el fondo vuelve intacto.
    assert!(s.restscreen(id2));
    assert!(s.restscreen(id1));
    assert_eq!(s.frame().get(10, 10).unwrap(), base);
}

#[test]
fn full_key_flows() {
    let m = menus();
    // Abrir Varios con su letra.
    assert_eq!(menubar_key(&m, 0, KeyCode::Char('v')), MenuBarKey::Move(1));
    // En el popup, 'o' elige Ordenar (índice 1).
    let items: Vec<PopupItem> = m[1]
        .items
        .iter()
        .map(|i| PopupItem::item(&i.label))
        .collect();
    assert_eq!(
        popup_key(&items, 0, KeyCode::Char('o')),
        PopupKey::Choose(1)
    );
    // En el confirm, 'n' acepta No (índice 1).
    assert_eq!(
        confirm_key(
            &["&Si".to_string(), "&No".to_string()],
            0,
            KeyCode::Char('n')
        ),
        DialogKey::Accept(1)
    );
}

#[test]
fn progress_over_modal() {
    let t = Theme::clipper();
    let mut s = Screen::new(80, 25, t);
    let bounds = s.bounds();
    s.frame().fill_rect(bounds, Cell::blank(t.work));
    let id = s.savescreen(bounds);
    progress_draw(
        s.frame(),
        bounds,
        &ProgressInfo::new(
            "ORDENAR INDICES",
            "999848KB",
            "00:00:00",
            vec![BarInfo::new("Inventario", "APROD.DAT", 12, 60, 546)],
        ),
        t,
    );
    // Menta + azul presentes; al restaurar vuelve el blanco.
    let mut found_dialog = false;
    for x in 0..80 {
        for y in 0..25 {
            if s.frame().get(x, y).unwrap().bg == t.dialog {
                found_dialog = true;
            }
        }
    }
    assert!(found_dialog);
    assert!(s.restscreen(id));
    assert!(s.frame().get(40, 12).unwrap().bg == t.work);
}
