//! Tests de integración del App shell (Fase 6).
//!
//! Eventos mapeados, foco por capas y shell presentable sin TTY.

use std::time::Duration;

use crossterm::event::{KeyCode as C, KeyModifiers as M};

use tui90::app::{poll_event, test_app};
use tui90::{
    app_desktop, map_key, menu_title_x, AppKey, DesktopLayout, FKeyDef, Focus, Layer, MenuDef,
    Rect, Theme,
};

#[test]
fn key_mapping_covers_arrows_f_and_alt() {
    assert_eq!(map_key(C::Up, M::empty()), tui90::AppEvent::key(AppKey::Up));
    assert_eq!(
        map_key(C::F(1), M::empty()),
        tui90::AppEvent::key(AppKey::F(1))
    );
    let alt = map_key(C::Char('f'), M::ALT);
    assert!(matches!(alt, tui90::AppEvent::Key { alt: true, .. }));
}

#[test]
fn poll_without_tty_or_input_returns_none() {
    // Sin entrada pendiente y timeout 0 no bloquea (en CI no hay TTY;
    // si falla por consola ausente, el test lo indica y se revisa).
    let r = poll_event(Duration::from_millis(0));
    assert!(r.is_ok());
}

#[test]
fn focus_layers_route() {
    let mut f = Focus::new();
    f.push(Layer::MenuBar);
    f.push(Layer::Popup);
    assert_eq!(f.top(), Layer::Popup);
    f.pop();
    assert_eq!(f.top(), Layer::MenuBar);
}

#[test]
fn shell_geometry_and_menu_anchor() {
    let d: DesktopLayout = app_desktop(Rect::new(0, 0, 80, 25));
    assert_eq!(d.work, Rect::new(1, 2, 78, 21));
    let menus = vec![MenuDef::new("Archivos", &[]), MenuDef::new("Varios", &[])];
    assert_eq!(menu_title_x(&menus, 1), 2 + 10 + 2);
    let _ = Theme::clipper();
}

#[test]
fn app_shell_flow_with_fkeys() {
    let mut app = test_app();
    let menus = vec![
        MenuDef::new("Archivos", &["A"]),
        MenuDef::new("Varios", &["B"]),
    ];
    let fkeys = vec![FKeyDef::new("F2", "Grabar"), FKeyDef::new("Esc", "Salir")];
    app.paint_shell(&tui90::ShellContent {
        company: "E",
        date: "D",
        status: "S",
        help: "H",
        menus: &menus,
        active: 0,
        fkeys: &fkeys,
    });
    app.present().unwrap();
    assert!(!app.backend.take().is_empty());
    // F2 dispara el botón 0, Esc el botón 1.
    assert_eq!(tui90::match_fkey(&fkeys, AppKey::F(2)), Some(0));
    assert_eq!(tui90::match_fkey(&fkeys, AppKey::Esc), Some(1));
}
