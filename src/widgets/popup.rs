//! Menú desplegable: popup azul sin marco + sombra dura.
//!
//! Hotkeys amarillas, separadores blancos finos, selección en barra gris
//! con texto negro (NO invertido). Salta separadores y deshabilitados.

use crossterm::event::KeyCode;

use crate::core::{Buffer, Cell, Rect, Theme};
use crate::prim::shadow::shadow;
use crate::prim::{draw_hot_label, hot_key_of, hsep, visible_len};

/// Un renglón del popup. `sep_before` pinta línea blanca encima.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PopupItem {
    pub label: String,
    pub sep_before: bool,
    pub disabled: bool,
}

impl PopupItem {
    pub fn item(label: &str) -> Self {
        Self {
            label: label.to_string(),
            sep_before: false,
            disabled: false,
        }
    }

    pub fn group(label: &str) -> Self {
        Self {
            label: label.to_string(),
            sep_before: true,
            disabled: false,
        }
    }

    pub fn disabled(label: &str) -> Self {
        Self {
            label: label.to_string(),
            sep_before: false,
            disabled: true,
        }
    }
}

fn selectable(it: &PopupItem) -> bool {
    !it.disabled && !it.label.is_empty()
}

/// Ancho necesario = etiqueta visible más larga + 4 de aire.
pub fn popup_size(items: &[PopupItem]) -> (u16, u16) {
    let w = items
        .iter()
        .map(|it| visible_len(&it.label))
        .max()
        .unwrap_or(0)
        .saturating_add(4);
    // +2 filas de aire + 1 por cada separador.
    let seps = items.iter().filter(|it| it.sep_before).count() as u16;
    (w.max(10), items.len() as u16 + 2 + seps)
}

/// Rect anclado bajo (x, y) y metido en pantalla.
pub fn popup_layout(x: u16, y: u16, items: &[PopupItem], screen: Rect) -> Rect {
    let (w, h) = popup_size(items);
    Rect::popup_at(x, y, w, h, screen)
}

/// Dibuja el popup con `selected` resaltado. No toca la pila.
pub fn popup_draw(
    buf: &mut Buffer,
    rect: Rect,
    items: &[PopupItem],
    selected: usize,
    theme: Theme,
) {
    if rect.is_empty() {
        return;
    }
    shadow(buf, rect, theme);
    buf.fill_rect(rect, Cell::new(' ', theme.popup_text, theme.popup));
    let mut y = rect.y.saturating_add(1);
    for (i, it) in items.iter().enumerate() {
        if y >= rect.bottom().saturating_sub(1) {
            break;
        }
        if it.sep_before {
            hsep(
                buf,
                y,
                rect.x.saturating_add(1),
                rect.right().saturating_sub(2),
                theme.popup_text,
                theme.popup,
            );
            y = y.saturating_add(1);
            if y >= rect.bottom().saturating_sub(1) {
                break;
            }
        }
        if it.label.is_empty() {
            y = y.saturating_add(1);
            continue;
        }
        let x = rect.x.saturating_add(2);
        if i == selected {
            buf.fill_rect(
                Rect::new(rect.x.saturating_add(1), y, rect.w.saturating_sub(2), 1),
                Cell::new(' ', theme.select_fg, theme.select_bg),
            );
            // En la barra gris todo va en negro (como en la referencia).
            draw_hot_label(
                buf,
                x,
                y,
                &it.label,
                theme.popup_sel_attr(),
                theme.popup_sel_attr(),
            );
        } else if it.disabled {
            let dim = crate::core::Attr::new(crate::core::Color::DarkGrey, theme.popup);
            draw_hot_label(buf, x, y, &it.label, dim, dim);
        } else {
            draw_hot_label(buf, x, y, &it.label, theme.popup_attr(), theme.hot_attr());
        }
        y = y.saturating_add(1);
    }
}

/// Resultado de una tecla sobre el popup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupKey {
    Stay,
    Move(usize),
    Choose(usize),
    Dismiss,
}

fn step(items: &[PopupItem], from: usize, dir: i32) -> usize {
    let n = items.len();
    if n == 0 {
        return 0;
    }
    let mut i = from as i32;
    for _ in 0..n {
        i = (i + dir).rem_euclid(n as i32);
        if selectable(&items[i as usize]) {
            return i as usize;
        }
    }
    from
}

/// Navegación pura: salta separadores/deshabilitados, `Enter` elige.
pub fn popup_key(items: &[PopupItem], selected: usize, code: KeyCode) -> PopupKey {
    use PopupKey::*;
    if items.is_empty() {
        return Dismiss;
    }
    let sel = selected.min(items.len() - 1);
    match code {
        KeyCode::Up => Move(step(items, sel, -1)),
        KeyCode::Down => Move(step(items, sel, 1)),
        KeyCode::Home => Move(
            (0..items.len())
                .find(|&i| selectable(&items[i]))
                .unwrap_or(sel),
        ),
        KeyCode::End => Move(
            (0..items.len())
                .rev()
                .find(|&i| selectable(&items[i]))
                .unwrap_or(sel),
        ),
        KeyCode::Enter => {
            if selectable(&items[sel]) {
                Choose(sel)
            } else {
                Move(step(items, sel, 1))
            }
        }
        KeyCode::Esc => Dismiss,
        KeyCode::Char(c) => {
            let lc = c.to_ascii_lowercase();
            for (i, it) in items.iter().enumerate() {
                if !selectable(it) {
                    continue;
                }
                if hot_key_of(&it.label).map(|h| h.to_ascii_lowercase()) == Some(lc) {
                    return Choose(i);
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

    fn items() -> Vec<PopupItem> {
        vec![
            PopupItem::item("&Respaldo de datos"),
            PopupItem::item("Rec&uperar datos"),
            PopupItem::group("&Ordenar índices"),
            PopupItem::disabled("Cambio de &fecha"),
        ]
    }

    #[test]
    fn size_counts_air_and_seps() {
        let (w, h) = popup_size(&items());
        assert!(w >= visible_len("Recuperar datos") + 4);
        assert_eq!(h, 4 + 2 + 1); // items + aire + 1 sep
    }

    #[test]
    fn layout_clamps_near_edges() {
        let screen = Rect::new(0, 0, 80, 25);
        let r = popup_layout(79, 24, &items(), screen);
        assert!(r.right() <= 80 && r.bottom() <= 25);
    }

    #[test]
    fn nav_skips_sep_and_disabled() {
        let it = items();
        // Desde 1, Down salta... 2 es seleccionable (grupo, no sep).
        assert_eq!(popup_key(&it, 1, KeyCode::Down), PopupKey::Move(2));
        // Desde 2, Down salta el deshabilitado y envuelve al 0.
        assert_eq!(popup_key(&it, 2, KeyCode::Down), PopupKey::Move(0));
        assert_eq!(popup_key(&it, 0, KeyCode::Up), PopupKey::Move(2));
        assert_eq!(popup_key(&it, 1, KeyCode::Enter), PopupKey::Choose(1));
        assert_eq!(popup_key(&it, 1, KeyCode::Esc), PopupKey::Dismiss);
    }

    #[test]
    fn letter_chooses_by_hotkey() {
        let it = items();
        assert_eq!(popup_key(&it, 0, KeyCode::Char('o')), PopupKey::Choose(2));
        // 'f' de "fecha" está deshabilitada → Stay.
        assert_eq!(popup_key(&it, 0, KeyCode::Char('f')), PopupKey::Stay);
    }

    #[test]
    fn draws_grey_selection_and_shadow() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(80, 25, t.desktop);
        let r = popup_layout(50, 2, &items(), Rect::new(0, 0, 80, 25));
        popup_draw(&mut b, r, &items(), 1, t);
        // Selección gris en la fila del item 1 (y = r.y+1+1).
        assert_eq!(b.get(r.x + 1, r.y + 2).unwrap().bg, t.select_bg);
        // Hotkey amarilla del item 0.
        assert_eq!(b.get(r.x + 2, r.y + 1).unwrap().fg, t.hot);
        // Sombra a la derecha.
        assert_eq!(b.get(r.right(), r.y + 1).unwrap().bg, t.shadow);
    }
}
