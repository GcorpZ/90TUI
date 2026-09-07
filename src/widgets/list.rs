//! Lista centrada con barra de selección azul (diálogos de trabajo).

use crossterm::event::KeyCode;

use crate::core::{Buffer, Cell, Rect, Theme};
use crate::prim::draw_text;

/// Dibuja `items` (centrados si `centered`) con `selected` en azul.
/// No pinta marco ni sombra: eso lo pone `window()` alrededor.
pub fn list_draw(
    buf: &mut Buffer,
    rect: Rect,
    items: &[String],
    selected: usize,
    centered: bool,
    theme: Theme,
) {
    if rect.is_empty() {
        return;
    }
    for (i, it) in items.iter().enumerate() {
        let y = rect.y.saturating_add(i as u16);
        if y >= rect.bottom() {
            break;
        }
        if i == selected {
            buf.fill_rect(
                Rect::new(rect.x, y, rect.w, 1),
                Cell::new(' ', theme.popup_text, theme.popup),
            );
            let x = text_x(rect, it, centered);
            draw_text(buf, x, y, it, theme.list_sel_attr());
        } else {
            let x = text_x(rect, it, centered);
            draw_text(
                buf,
                x,
                y,
                it,
                crate::core::Attr::new(theme.form_text, theme.window_bg),
            );
        }
    }
}

fn text_x(rect: Rect, s: &str, centered: bool) -> u16 {
    if centered {
        let w = crate::prim::visible_len(s);
        rect.x.saturating_add(rect.w.saturating_sub(w) / 2)
    } else {
        rect.x.saturating_add(2)
    }
}

/// Mueve la selección con recorte. `page` = salto de RePág/AvPág.
pub fn list_key(selected: usize, len: usize, code: KeyCode, page: u16) -> usize {
    if len == 0 {
        return 0;
    }
    let sel = selected.min(len - 1);
    match code {
        KeyCode::Up => sel.saturating_sub(1),
        KeyCode::Down => (sel + 1).min(len - 1),
        KeyCode::Home => 0,
        KeyCode::End => len - 1,
        KeyCode::PageUp => sel.saturating_sub(page.max(1) as usize),
        KeyCode::PageDown => (sel + page.max(1) as usize).min(len - 1),
        _ => sel,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_clamps_and_pages() {
        assert_eq!(list_key(0, 6, KeyCode::Up, 3), 0);
        assert_eq!(list_key(5, 6, KeyCode::Down, 3), 5);
        assert_eq!(list_key(0, 6, KeyCode::PageDown, 3), 3);
        assert_eq!(list_key(1, 6, KeyCode::PageUp, 3), 0);
        assert_eq!(list_key(2, 6, KeyCode::End, 3), 5);
        assert_eq!(list_key(2, 0, KeyCode::Down, 3), 0);
    }

    #[test]
    fn draws_blue_selection() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(50, 12, t.window_bg);
        let items = vec!["Proveedores".to_string(), "Clientes".to_string()];
        list_draw(&mut b, Rect::new(5, 2, 40, 6), &items, 0, true, t);
        assert_eq!(b.get(5, 2).unwrap().bg, t.popup);
        assert_eq!(b.get(6, 3).unwrap().bg, t.window_bg);
    }
}
