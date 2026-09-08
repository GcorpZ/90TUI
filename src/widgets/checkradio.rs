//! Casillas y radios: `[X] Compresión`, `(•) Rápido`.
//!
//! Glifos ASCII seguros (`[X]`/`[ ]`, `(•)`/`( )`) en vez de ☐/◉: en la
//! época eran glifos VGA a medida; aquí priorizamos que se vean en
//! cualquier fuente. `&` marca hotkey como siempre.

use crossterm::event::KeyCode;

use crate::core::{Attr, Buffer};
use crate::prim::{draw_hot_label, hot_key_of, visible_len, HotAttrs};

/// Una casilla con etiqueta (`&` = hotkey).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckItem {
    pub label: String,
    pub checked: bool,
}

impl CheckItem {
    pub fn new(label: &str, checked: bool) -> Self {
        Self {
            label: label.to_string(),
            checked,
        }
    }
}

/// Dibuja `[X] etiqueta`. `focused` pone la etiqueta en negrita.
/// Devuelve el ancho ocupado.
pub fn checkbox_draw(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    item: &CheckItem,
    focused: bool,
    attrs: HotAttrs,
) -> u16 {
    let glyph = if item.checked { "[X] " } else { "[ ] " };
    let gw = draw_box_glyph(buf, x, y, glyph, attrs.base);
    let label_attr = Attr {
        bold: focused,
        ..attrs.base
    };
    let hot_attr = Attr {
        bold: focused,
        ..attrs.hot
    };
    gw + draw_hot_label(buf, x + gw, y, &item.label, label_attr, hot_attr)
}

/// Dibuja `(•) etiqueta` si `selected`, `( )` si no.
pub fn radio_draw(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    label: &str,
    selected: bool,
    focused: bool,
    attrs: HotAttrs,
) -> u16 {
    let glyph = if selected { "(•) " } else { "( ) " };
    let gw = draw_box_glyph(buf, x, y, glyph, attrs.base);
    let label_attr = Attr {
        bold: focused || selected,
        ..attrs.base
    };
    let hot_attr = Attr {
        bold: focused || selected,
        ..attrs.hot
    };
    gw + draw_hot_label(buf, x + gw, y, label, label_attr, hot_attr)
}

fn draw_box_glyph(buf: &mut Buffer, x: u16, y: u16, glyph: &str, attr: Attr) -> u16 {
    crate::prim::draw_text(buf, x, y, glyph, attr)
}

/// Resultado de tecla en un grupo de casillas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckNav {
    Stay,
    Move(usize),
    Toggled(usize),
}

/// ↑↓ mueven foco, Espacio/Enter alterna, letra = hotkey directa.
pub fn check_key(items: &[CheckItem], focus: usize, code: KeyCode) -> CheckNav {
    use CheckNav::*;
    let n = items.len();
    if n == 0 {
        return Stay;
    }
    let f = focus.min(n - 1);
    match code {
        KeyCode::Up | KeyCode::Left => Move((f + n - 1) % n),
        KeyCode::Down | KeyCode::Right | KeyCode::Tab => Move((f + 1) % n),
        KeyCode::Home => Move(0),
        KeyCode::End => Move(n - 1),
        KeyCode::Char(' ') | KeyCode::Enter => Toggled(f),
        KeyCode::Char(c) => {
            let lc = c.to_ascii_lowercase();
            for (i, it) in items.iter().enumerate() {
                if hot_key_of(&it.label).map(|h| h.to_ascii_lowercase()) == Some(lc) {
                    return Toggled(i);
                }
            }
            Stay
        }
        _ => Stay,
    }
}

/// Resultado de tecla en un grupo de radios.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadioNav {
    Stay,
    Select(usize),
}

/// Las flechas eligen directamente (comportamiento clásico de radio).
pub fn radio_key(n: usize, selected: usize, code: KeyCode) -> RadioNav {
    use RadioNav::*;
    if n == 0 {
        return Stay;
    }
    let s = selected.min(n - 1);
    match code {
        KeyCode::Up | KeyCode::Left => Select((s + n - 1) % n),
        KeyCode::Down | KeyCode::Right | KeyCode::Tab => Select((s + 1) % n),
        KeyCode::Home => Select(0),
        KeyCode::End => Select(n - 1),
        KeyCode::Enter | KeyCode::Char(' ') => Select(s),
        KeyCode::Char(c) => {
            // La letra salta por hotkey si el llamante resuelve etiquetas;
            // aquí sin etiquetas: Stay (el showroom usa números).
            let _ = c;
            Stay
        }
        _ => Stay,
    }
}

/// Ancho visible de una fila de casilla (para layout).
pub fn check_width(item: &CheckItem) -> u16 {
    4 + visible_len(&item.label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Buffer, Color, Theme};

    fn base() -> (Attr, Attr) {
        let t = Theme::clipper();
        (t.dialog_attr(), Attr::bold(Color::Red, t.dialog))
    }

    #[test]
    fn draws_box_and_label() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 5, t.dialog);
        let (base, hot) = base();
        let attrs = HotAttrs { base, hot };
        let w = checkbox_draw(
            &mut b,
            2,
            1,
            &CheckItem::new("&Compresión", true),
            false,
            attrs,
        );
        assert_eq!(b.get(2, 1).unwrap().ch, '[');
        assert_eq!(b.get(3, 1).unwrap().ch, 'X');
        assert_eq!(b.get(6, 1).unwrap().fg, Color::Red); // hotkey C
        assert!(w >= 4 + visible_len("Compresión"));
        let w2 = radio_draw(&mut b, 2, 2, "Rápido", true, false, attrs);
        assert_eq!(b.get(3, 2).unwrap().ch, '•');
        assert!(w2 > 4);
    }

    #[test]
    fn check_nav_moves_and_toggles() {
        let items = vec![CheckItem::new("A", false), CheckItem::new("B", true)];
        assert_eq!(check_key(&items, 0, KeyCode::Down), CheckNav::Move(1));
        assert_eq!(check_key(&items, 1, KeyCode::Down), CheckNav::Move(0));
        assert_eq!(
            check_key(&items, 0, KeyCode::Char(' ')),
            CheckNav::Toggled(0)
        );
        assert_eq!(
            check_key(&items, 0, KeyCode::Char('b')),
            CheckNav::Toggled(1)
        );
    }

    #[test]
    fn radio_nav_selects_directly() {
        assert_eq!(radio_key(3, 1, KeyCode::Up), RadioNav::Select(0));
        assert_eq!(radio_key(3, 0, KeyCode::Up), RadioNav::Select(2));
        assert_eq!(radio_key(3, 2, KeyCode::Down), RadioNav::Select(0));
        assert_eq!(radio_key(0, 0, KeyCode::Down), RadioNav::Stay);
    }
}
