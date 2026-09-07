//! Tests de integración del núcleo (Fase 3): lo que la FASE 1 exige.
//!
//! Rectángulos con recorte, diff del back-buffer, pila LIFO y backend
//! de pruebas. Todo sin TTY real.

use tui90::{Backend, Buffer, Cell, Color, Rect, Screen, ScreenStack, TestBackend, Theme};

#[test]
fn rect_clip_keeps_popup_inside_80x25() {
    let screen = Rect::new(0, 0, 80, 25);
    // Popup de 30x10 anclado casi en la esquina: debe caber moviéndose.
    let p = Rect::popup_at(70, 23, 30, 10, screen);
    assert!(p.right() <= 80, "se sale por la derecha: {p:?}");
    assert!(p.bottom() <= 25, "se sale por abajo: {p:?}");
    assert_eq!((p.w, p.h), (30, 10));
}

#[test]
fn buffer_diff_reports_only_changed_cells() {
    let mut back = Buffer::blank(80, 25, Color::Black);
    let front = Buffer::blank(80, 25, Color::Black);
    back.set(10, 5, Cell::new('X', Color::White, Color::Black));
    back.set(11, 5, Cell::new('Y', Color::White, Color::Black));
    let ops = back.diff(&front);
    assert_eq!(ops.len(), 2);
}

#[test]
fn screenstack_save_restore_is_lifo() {
    let theme = Theme::clipper();
    let mut s = Screen::new(40, 12, theme);
    s.frame().text(1, 1, "BASE", Color::Black, Color::White);
    let id_base = s.savescreen(Rect::new(0, 0, 40, 12));
    s.frame().fill_rect(
        Rect::new(5, 2, 20, 6),
        Cell::new(' ', Color::White, Color::Blue),
    );
    let id_top = s.savescreen(Rect::new(5, 2, 20, 6));

    // Cerrar fuera de orden falla (protege el apilado de modales).
    assert!(!s.restscreen(id_base));
    // Cerrar la cima sí funciona, dos veces hasta vaciar.
    assert!(s.restscreen(id_top));
    assert!(s.restscreen(id_base));
    assert!(s.stack.is_empty());
    // Y el fondo sigue ahí.
    assert_eq!(s.frame().get(1, 1).unwrap().ch, 'B');
}

#[test]
fn stack_standalone_push_pop() {
    let mut buf = Buffer::blank(10, 10, Color::White);
    let mut st = ScreenStack::new();
    let id = st.save(&buf, Rect::new(2, 2, 4, 4));
    buf.fill_rect(
        Rect::new(0, 0, 10, 10),
        Cell::new('Z', Color::White, Color::Black),
    );
    assert_eq!(st.depth(), 1);
    assert!(st.restore(&mut buf, id));
    assert!(st.is_empty());
    // Dentro de la foto: restaurado al blanco original.
    assert_eq!(buf.get(2, 2).unwrap().ch, ' ');
    // Fuera de la foto: el relleno 'Z' sobrevive (recorte correcto).
    assert_eq!(buf.get(0, 0).unwrap().ch, 'Z');
}

#[test]
fn full_frame_flow_into_test_backend() {
    let theme = Theme::clipper();
    let mut s = Screen::new(80, 25, theme);
    // Base estilo Clipper: escritorio + área de trabajo blanca.
    let bounds = s.bounds();
    s.frame().fill_rect(bounds, Cell::blank(theme.desktop));
    let work = Rect::new(2, 3, 76, 18);
    s.frame().fill_rect(work, Cell::blank(theme.work));
    s.frame()
        .text(4, 4, "EMPRESA DEMO", Color::White, theme.navy);

    let mut be = TestBackend::new(80, 25);
    let ops = s.present_ops();
    be.present(&ops).unwrap();
    assert!(!be.take().is_empty());
    // Segundo frame sin cambios: silencio total (cero flicker).
    let ops2 = s.present_ops();
    assert!(ops2.is_empty());
}
