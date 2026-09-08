//! Etiquetas de texto + hotkeys amarillas + recorte con elipsis.
//!
//! Convención (igual que en los menús de la época): `&` delante de una
//! letra la marca como hotkey (`"&Respaldo"` → **R**espaldo, la R en
//! amarillo). `&&` pinta un `&` literal. Sin marca, la hotkey es la
//! primera letra alfabética automáticamente.

use unicode_width::UnicodeWidthChar;

use crate::core::{Attr, Buffer, Color};

/// Ancho visible en celdas (sin contar marcas `&`).
pub fn visible_len(s: &str) -> u16 {
    let (clean, _) = parse_hotkey(s);
    let mut w = 0u16;
    for ch in clean.chars() {
        w = w.saturating_add(UnicodeWidthChar::width(ch).unwrap_or(0) as u16);
    }
    w
}

/// Separa texto limpio + hotkey.
/// Devuelve `(texto_sin_marcas, Option<(índice_char, letra)>)`.
pub fn parse_hotkey(s: &str) -> (String, Option<(usize, char)>) {
    let mut clean = String::with_capacity(s.len());
    let mut hot: Option<(usize, char)> = None;
    let mut marked = false; // el `&` anterior marca al siguiente
    let mut idx = 0usize; // índice en chars del texto limpio
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '&' {
            if marked {
                // `&&` → literal. Si aún no hay hotkey, este `&`
                // cuenta como primera opción alfabética? No: es símbolo.
                clean.push('&');
                idx += 1;
                marked = false;
            } else if chars.peek() == Some(&'&') {
                // Consumir el segundo `&` como literal es más simple aquí:
                // lo tratamos en la próxima iteración vía `marked`.
                marked = true;
            } else {
                marked = true;
            }
            continue;
        }
        if marked {
            marked = false;
            // Un espacio marcado no sirve como hotkey; se ignora la marca.
            if hot.is_none() && !ch.is_whitespace() {
                hot = Some((idx, ch));
            }
        }
        clean.push(ch);
        idx += 1;
    }
    // `&` final solitario → literal.
    if marked {
        clean.push('&');
    }
    // Sin marca: primera letra alfabética.
    if hot.is_none() {
        for (i, ch) in clean.chars().enumerate() {
            if ch.is_alphabetic() {
                hot = Some((i, ch));
                break;
            }
        }
    }
    (clean, hot)
}

/// Solo la letra hotkey, si hay.
pub fn hot_key_of(s: &str) -> Option<char> {
    parse_hotkey(s).1.map(|(_, c)| c)
}

/// Recorta `s` a `max_w` celdas, con `…` final si recorta.
/// Conserva la marca `&` de la hotkey si la letra sobrevive al recorte.
pub fn fit_text(s: &str, max_w: u16) -> String {
    if max_w == 0 {
        return String::new();
    }
    let (clean, hot) = parse_hotkey(s);
    let total: u16 = clean
        .chars()
        .map(|c| UnicodeWidthChar::width(c).unwrap_or(0) as u16)
        .sum();
    if total <= max_w {
        return s.to_string();
    }
    let budget = max_w.saturating_sub(1); // 1 celda para `…`
    let hot_idx = hot.map(|(i, _)| i);
    let mut out = String::new();
    let mut w = 0u16;
    for (i, ch) in clean.chars().enumerate() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
        if cw == 0 {
            continue;
        }
        if w.saturating_add(cw) > budget {
            break;
        }
        if Some(i) == hot_idx {
            out.push('&');
        }
        out.push(ch);
        w = w.saturating_add(cw);
    }
    out.push('…');
    out
}

/// Dibuja texto simple con un atributo. Devuelve ancho pintado.
pub fn draw_text(buf: &mut Buffer, x: u16, y: u16, s: &str, attr: Attr) -> u16 {
    let mut cx = x;
    for ch in s.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
        if w == 0 {
            continue;
        }
        if cx >= buf.width() {
            break;
        }
        buf.set(cx, y, crate::core::Cell::with_attr(ch, attr));
        cx = cx.saturating_add(w);
    }
    cx.saturating_sub(x)
}

/// Dibuja etiqueta con hotkey (`base` normal, `hot` amarilla).
/// Devuelve ancho pintado en celdas.
pub fn draw_hot_label(buf: &mut Buffer, x: u16, y: u16, s: &str, base: Attr, hot: Attr) -> u16 {
    let (clean, hot_idx) = parse_hotkey(s);
    let mut cx = x;
    for (i, ch) in clean.chars().enumerate() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
        if w == 0 {
            continue;
        }
        if cx >= buf.width() {
            break;
        }
        let a = if Some(i) == hot_idx.map(|(j, _)| j) {
            hot
        } else {
            base
        };
        buf.set(cx, y, crate::core::Cell::with_attr(ch, a));
        cx = cx.saturating_add(w);
    }
    cx.saturating_sub(x)
}

/// Par base+hotkey para etiquetas (evita funciones de 8 parámetros).
#[derive(Clone, Copy, Debug)]
pub struct HotAttrs {
    pub base: Attr,
    pub hot: Attr,
}

/// Atributo base sobre un fondo dado (texto blanco normal).
pub fn base_on(bg: Color) -> Attr {
    Attr::new(Color::White, bg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Theme;

    #[test]
    fn parse_marks_hotkey() {
        let (t, h) = parse_hotkey("&Respaldo");
        assert_eq!(t, "Respaldo");
        assert_eq!(h, Some((0, 'R')));
    }

    #[test]
    fn parse_double_amp_is_literal() {
        let (t, h) = parse_hotkey("A && B");
        assert_eq!(t, "A & B");
        // Sin marca: primera letra alfabética.
        assert_eq!(h, Some((0, 'A')));
    }

    #[test]
    fn parse_auto_hotkey_first_alpha() {
        let (t, h) = parse_hotkey("Ordenar índices");
        assert_eq!(t, "Ordenar índices");
        assert_eq!(h, Some((0, 'O')));
    }

    #[test]
    fn draw_paints_hot_yellow() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(20, 3, t.popup);
        draw_hot_label(&mut b, 1, 1, "&Respaldo", t.popup_attr(), t.hot_attr());
        assert_eq!(
            b.get(1, 1).unwrap(),
            crate::core::Cell::with_attr('R', t.hot_attr())
        );
        assert_eq!(
            b.get(2, 1).unwrap(),
            crate::core::Cell::with_attr('e', t.popup_attr())
        );
    }

    #[test]
    fn fit_text_truncates_with_ellipsis() {
        let f = fit_text("Proveedores importantes", 10);
        assert!(visible_len(&f) <= 10, "f={f}");
        assert!(f.ends_with('…'), "f={f}");
        assert_eq!(fit_text("Hola", 10), "Hola");
        assert_eq!(fit_text("abc", 0), "");
    }
}
