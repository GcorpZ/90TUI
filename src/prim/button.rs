//! Botones teal con sombra: ` Esc Salir `.
//!
//! Bloque sólido (1 espacio de aire por lado) + sombra de 1px abajo y
//! a la derecha. Sin corchetes, sin box-drawing: el color + la sombra
//! ya dicen "apretable".

use crate::core::{Buffer, Cell, Theme};

use super::label::visible_len;

/// Ancho total que ocupará `button()` (texto + 2 de aire).
pub fn button_width(label: &str) -> u16 {
    visible_len(label).saturating_add(2)
}

/// Dibuja el botón en (x, y). Devuelve el ancho ocupado.
/// El recorte al buffer es silencioso (no panic en bordes).
pub fn button(buf: &mut Buffer, x: u16, y: u16, label: &str, theme: Theme) -> u16 {
    let w = button_width(label);
    if w == 0 {
        return 0;
    }
    let (bbg, bfg) = (theme.button_bg, theme.button_fg);
    let attr = theme.button_attr();
    // Texto con aire: ` label `.
    buf.set(x, y, Cell::with_attr(' ', attr));
    let mut cx = x.saturating_add(1);
    for ch in label.chars() {
        if cx >= x.saturating_add(w).saturating_sub(1) {
            break;
        }
        buf.set(cx, y, Cell::with_attr(ch, attr));
        cx = cx.saturating_add(1);
    }
    buf.set(
        x.saturating_add(w).saturating_sub(1),
        y,
        Cell::with_attr(' ', attr),
    );
    // Sombra: fila inferior + columna derecha.
    let sh = Cell::new(' ', theme.shadow, theme.shadow);
    for i in 0..w {
        buf.set(
            x.saturating_add(i).saturating_add(1),
            y.saturating_add(1),
            sh,
        );
    }
    buf.set(x.saturating_add(w), y, sh);
    let _ = (bbg, bfg);
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_counts_air() {
        assert_eq!(button_width("Salir"), 7); // " Salir "
        assert_eq!(button_width(""), 2);
    }

    #[test]
    fn draws_teal_block_with_shadow() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(20, 5, t.desktop);
        let w = button(&mut b, 2, 1, "Salir", t);
        assert_eq!(w, 7);
        assert_eq!(b.get(2, 1).unwrap().bg, t.button_bg);
        assert_eq!(b.get(3, 1).unwrap().ch, 'S');
        assert_eq!(b.get(3, 1).unwrap().fg, t.button_fg);
        // Sombra debajo y a la derecha.
        assert_eq!(b.get(3, 2).unwrap().bg, t.shadow);
        assert_eq!(b.get(2 + 7, 1).unwrap().bg, t.shadow);
    }
}
