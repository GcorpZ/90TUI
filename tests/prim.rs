//! Tests de integración de primitivas (Fase 4).
//!
//! Composición real sobre `Buffer`: ventana + sombra + botón + hotlabel,
//! como un popup de menú de la época.

use tui90::{
    button, draw_hot_label, hsep, status_bar, top_bar, window, Buffer, Color, Rect, Theme,
    WindowOpts,
};

#[test]
fn popup_like_composition() {
    let t = Theme::clipper();
    let mut b = Buffer::blank(80, 25, t.desktop);

    // Popup azul con sombra.
    let r = Rect::new(50, 2, 26, 10);
    tui90::shadow(&mut b, r, t);
    b.fill_rect(r, tui90::Cell::new(' ', t.popup_text, t.popup));

    // Items con hotkeys amarillas.
    let items = ["&Respaldo de datos", "Rec&uperar datos", "&Ordenar índices"];
    for (i, it) in items.iter().enumerate() {
        draw_hot_label(&mut b, 52, 3 + i as u16, it, t.popup_attr(), t.hot_attr());
    }
    // Separador + selección gris en la fila 2.
    hsep(&mut b, 6, 51, 74, t.popup_text, t.popup);
    b.fill_rect(
        Rect::new(51, 4, 24, 1),
        tui90::Cell::new(' ', t.select_fg, t.select_bg),
    );
    draw_hot_label(&mut b, 52, 4, items[1], t.popup_sel_attr(), t.hot_attr());

    // Verificaciones estilo captura de referencia.
    assert_eq!(b.get(52, 3).unwrap().fg, t.hot); // R amarilla
    assert_eq!(b.get(51, 6).unwrap().ch, '─'); // separador
    assert_eq!(b.get(51, 4).unwrap().bg, t.select_bg); // selección gris
    assert_eq!(b.get(76, 3).unwrap().bg, t.shadow); // sombra derecha (x+w)
    assert_eq!(b.get(52, 12).unwrap().bg, t.shadow); // sombra abajo (y+h)
}

#[test]
fn modal_window_with_buttons() {
    let t = Theme::clipper();
    let mut b = Buffer::blank(80, 25, t.desktop);
    let r = Rect::centered_in(40, 12, Rect::new(0, 0, 80, 25));
    window(&mut b, r, &WindowOpts::modal("ORDENAR INDICES", t), t);

    // Título centrado en barra teal.
    let title_x = r.x + (r.w - 15) / 2;
    assert_eq!(b.get(title_x, r.y).unwrap().ch, 'O');
    assert_eq!(b.get(r.x, r.y).unwrap().bg, t.teal);

    // Botonera inferior (padding 2 por lado: 7 + 4 = 11).
    let bw = button(&mut b, r.x + 2, r.y + r.h - 2, "Ordenar", t);
    assert_eq!(bw, 7 + 4);
    assert_eq!(b.get(r.x + 4, r.y + r.h - 2).unwrap().ch, 'O');

    // Cuerpo gris, sombra negra.
    assert_eq!(b.get(r.x + 1, r.y + 2).unwrap().bg, t.window_bg);
    assert_eq!(b.get(r.x + 2, r.y + r.h).unwrap().bg, Color::Black);
}

#[test]
fn shell_bars_and_work_area() {
    let t = Theme::clipper();
    let mut b = Buffer::blank(80, 25, Color::White);
    top_bar(&mut b, "EMPRESA DEMO", "FECHA", t);
    status_bar(&mut b, "TUI90 Demo", "Ayuda", t);
    assert_eq!(b.get(0, 0).unwrap().bg, t.navy);
    assert_eq!(b.get(0, 24).unwrap().bg, t.navy);
    assert_eq!(b.get(1, 12).unwrap().bg, Color::White);
}
