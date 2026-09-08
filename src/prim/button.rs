//! Botones teal con sombra solo-abajo + estado presionado intrínseco.
//!
//! Bloque sólido (1 espacio de aire por lado) + sombra de 1px **debajo**
//! (desplazada +1 a la derecha). Al presionarse, el botón **baja 1 y se
//! corre 1 a la derecha**, tapando su sombra: el hundimiento clásico.
//! Sin corchetes, sin box-drawing.

use crate::core::{Buffer, Cell, Theme};

use super::label::visible_len;

/// Ancho total que ocupará `button()` (texto + 2 de aire).
pub fn button_width(label: &str) -> u16 {
    visible_len(label).saturating_add(2)
}

/// Dibuja el botón en reposo en (x, y). Devuelve el ancho ocupado.
/// El recorte al buffer es silencioso (no panic en bordes).
pub fn button(buf: &mut Buffer, x: u16, y: u16, label: &str, theme: Theme) -> u16 {
    button_draw(buf, x, y, label, theme, false)
}

/// Dibuja el botón; con `pressed = true` se renderiza hundido
/// (desplazado +1,+1 y sin sombra, como si tapara la suya).
/// Devuelve el ancho ocupado (el mismo en ambos estados).
pub fn button_draw(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    label: &str,
    theme: Theme,
    pressed: bool,
) -> u16 {
    let w = button_width(label);
    if w == 0 {
        return 0;
    }
    // Hundido: corre el bloque y no pintes sombra (queda tapada).
    let (bx, by) = if pressed {
        (x.saturating_add(1), y.saturating_add(1))
    } else {
        (x, y)
    };
    let attr = theme.button_attr();
    // Texto con aire: ` label `.
    buf.set(bx, by, Cell::with_attr(' ', attr));
    let mut cx = bx.saturating_add(1);
    for ch in label.chars() {
        if cx >= bx.saturating_add(w).saturating_sub(1) {
            break;
        }
        buf.set(cx, by, Cell::with_attr(ch, attr));
        cx = cx.saturating_add(1);
    }
    buf.set(
        bx.saturating_add(w).saturating_sub(1),
        by,
        Cell::with_attr(' ', attr),
    );
    if !pressed {
        // Sombra solo debajo, corrida +1 (nada al costado).
        let sh = Cell::new(' ', theme.shadow, theme.shadow);
        for i in 0..w {
            buf.set(
                bx.saturating_add(i).saturating_add(1),
                by.saturating_add(1),
                sh,
            );
        }
    }
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
    fn draws_teal_block_with_bottom_only_shadow() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(20, 5, t.desktop);
        let w = button(&mut b, 2, 1, "Salir", t);
        assert_eq!(w, 7);
        assert_eq!(b.get(2, 1).unwrap().bg, t.button_bg);
        assert_eq!(b.get(3, 1).unwrap().ch, 'S');
        assert_eq!(b.get(3, 1).unwrap().fg, t.button_fg);
        // Sombra solo debajo (corrida +1), nada al costado derecho.
        assert_eq!(b.get(3, 2).unwrap().bg, t.shadow);
        assert_eq!(b.get(2 + 7, 1).unwrap().bg, t.desktop);
    }

    #[test]
    fn pressed_moves_down_right_and_covers_shadow() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(20, 5, t.desktop);
        // Primero en reposo (pinta sombra), luego presionado encima.
        button(&mut b, 2, 1, "Salir", t);
        button_draw(&mut b, 2, 1, "Salir", t, true);
        // El bloque bajó: la S ahora está en (4,2)...
        assert_eq!(b.get(4, 2).unwrap().ch, 'S');
        // ...y tapó su propia sombra (ya no hay negro debajo del bloque).
        assert_eq!(b.get(3, 2).unwrap().bg, t.button_bg);
    }
}
