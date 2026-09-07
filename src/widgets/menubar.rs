//! Barra de menús (fila teal) + navegación.
//!
//! Dibuja los títulos; el activo va en negro (como en la referencia).
//! `&` en títulos marca hotkey; sin marca, primera letra.

use crossterm::event::KeyCode;

use crate::core::{Buffer, Cell, Rect, Theme};
use crate::prim::{draw_text, hot_key_of, visible_len};

/// Un item de menú desplegable (`&` = hotkey).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
    pub label: String,
}

impl MenuItem {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }
}

/// Una columna del menubar con sus items.
#[derive(Clone, Debug)]
pub struct MenuDef {
    pub title: String,
    pub items: Vec<MenuItem>,
}

impl MenuDef {
    pub fn new(title: &str, items: &[&str]) -> Self {
        Self {
            title: title.to_string(),
            items: items.iter().map(|s| MenuItem::new(s)).collect(),
        }
    }
}

/// Dibuja la barra en `bar` (normalmente fila 1, ancho de trabajo).
pub fn menubar_draw(buf: &mut Buffer, bar: Rect, menus: &[MenuDef], active: usize, theme: Theme) {
    if bar.is_empty() {
        return;
    }
    buf.fill_rect(bar, Cell::new(' ', theme.status_fg, theme.teal));
    let mut cx = bar.x.saturating_add(2);
    for (i, m) in menus.iter().enumerate() {
        let label = format!(" {} ", m.title);
        let w = visible_len(&label);
        if cx.saturating_add(w) > bar.right() {
            break;
        }
        draw_text(buf, cx, bar.y, &label, theme.menu_attr(i == active));
        cx = cx.saturating_add(w).saturating_add(2);
    }
}

/// Resultado de una tecla sobre el menubar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuBarKey {
    Stay,
    Move(usize),
    Open(usize),
    Dismiss,
}

/// Navegación pura (sin dibujar): el App loop de Fase 6 la cablea.
pub fn menubar_key(menus: &[MenuDef], active: usize, code: KeyCode) -> MenuBarKey {
    use MenuBarKey::*;
    let n = menus.len();
    if n == 0 {
        return Stay;
    }
    let active = active.min(n - 1);
    match code {
        KeyCode::Left => Move((active + n - 1) % n),
        KeyCode::Right => Move((active + 1) % n),
        KeyCode::Home => Move(0),
        KeyCode::End => Move(n - 1),
        KeyCode::Down | KeyCode::Enter => Open(active),
        KeyCode::Esc => Dismiss,
        KeyCode::Char(c) => {
            let lc = c.to_ascii_lowercase();
            for (i, m) in menus.iter().enumerate() {
                if hot_key_of(&m.title).map(|h| h.to_ascii_lowercase()) == Some(lc) {
                    return if i == active { Open(i) } else { Move(i) };
                }
            }
            Stay
        }
        _ => Stay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defs() -> Vec<MenuDef> {
        vec![
            MenuDef::new("Archivos", &["Proveedores", "Clientes"]),
            MenuDef::new("Varios", &["Finalizar"]),
        ]
    }

    #[test]
    fn left_right_wrap() {
        let m = defs();
        assert_eq!(menubar_key(&m, 0, KeyCode::Left), MenuBarKey::Move(1));
        assert_eq!(menubar_key(&m, 1, KeyCode::Right), MenuBarKey::Move(0));
        assert_eq!(menubar_key(&m, 0, KeyCode::Down), MenuBarKey::Open(0));
        assert_eq!(menubar_key(&m, 0, KeyCode::Esc), MenuBarKey::Dismiss);
    }

    #[test]
    fn letter_jumps_to_menu() {
        let m = defs();
        // 'v' → Varios (índice 1), distinto del activo → Move.
        assert_eq!(menubar_key(&m, 0, KeyCode::Char('v')), MenuBarKey::Move(1));
        // Sobre el activo, la letra lo abre.
        assert_eq!(menubar_key(&m, 1, KeyCode::Char('V')), MenuBarKey::Open(1));
        assert_eq!(menubar_key(&m, 0, KeyCode::Char('z')), MenuBarKey::Stay);
    }

    #[test]
    fn draws_active_in_black() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(80, 25, t.desktop);
        let m = defs();
        menubar_draw(&mut b, Rect::new(0, 1, 80, 1), &m, 1, t);
        // " Varios " empieza en x=2+len(" Archivos ")+2 = 2+10+2.
        let x = 2 + 10 + 2;
        assert_eq!(b.get(x, 1).unwrap().bg, t.menu_attr(true).bg);
        assert_eq!(b.get(2, 1).unwrap().bg, t.teal);
    }
}
