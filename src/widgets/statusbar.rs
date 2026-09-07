//! Barra de teclas de función: `F2 Grabar  Esc Salir`.
//!
//! Botones planos teal (sin sombra, van pegados al borde inferior).
//! Dibuja en la fila `y`; devuelve el ancho usado.

use crate::core::{Buffer, Theme};
use crate::prim::{draw_text, visible_len};

/// Dibuja pares `(tecla, acción)` separados por 2 espacios desde x=1.
pub fn fkey_bar(buf: &mut Buffer, y: u16, keys: &[(&str, &str)], theme: Theme) -> u16 {
    if y >= buf.height() {
        return 0;
    }
    let mut cx = 1u16;
    for (k, label) in keys {
        let s = format!(" {k} {label} ");
        let w = visible_len(&s);
        if cx.saturating_add(w) > buf.width() {
            break;
        }
        draw_text(buf, cx, y, &s, theme.button_attr());
        cx = cx.saturating_add(w).saturating_add(2);
    }
    cx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_keys_in_order() {
        let t = Theme::clipper();
        let mut buf = Buffer::blank(60, 25, t.desktop);
        fkey_bar(&mut buf, 24, &[("F2", "Grabar"), ("Esc", "Salir")], t);
        assert_eq!(buf.get(1, 24).unwrap().ch, ' ');
        assert_eq!(buf.get(2, 24).unwrap().ch, 'F');
        assert_eq!(buf.get(2, 24).unwrap().bg, t.button_bg);
    }
}
