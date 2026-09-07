//! Sombras duras: rectángulo sólido desplazado, sin `▒▓`.
//!
//! La seña de la escuela Clipper: detrás de cada ventana/popup/botón
//! va un bloque del color sombra (normalmente negro) con offset.
//! Se pinta PRIMERO (debajo) y el cuerpo encima.

use crate::core::{Buffer, Cell, Rect, Theme};

/// Sombra clásica con offset (2, 1).
pub fn shadow(buf: &mut Buffer, rect: Rect, theme: Theme) {
    shadow_offset(buf, rect, 2, 1, theme);
}

/// Sombra con offset arbitrario (botones usan (1, 1)).
pub fn shadow_offset(buf: &mut Buffer, rect: Rect, dx: u16, dy: u16, theme: Theme) {
    if rect.is_empty() {
        return;
    }
    let r = Rect::new(
        rect.x.saturating_add(dx),
        rect.y.saturating_add(dy),
        rect.w,
        rect.h,
    );
    buf.fill_rect(r, Cell::new(' ', theme.shadow, theme.shadow));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Color;

    #[test]
    fn shadow_lands_offset_and_clips() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(10, 6, Color::White);
        let r = Rect::new(1, 1, 4, 2);
        shadow(&mut b, r, t);
        // Esquina de la sombra (x+2, y+1).
        assert_eq!(b.get(3, 2).unwrap().bg, t.shadow);
        // El cuerpo original no se toca.
        assert_eq!(b.get(1, 1).unwrap().bg, Color::White);
    }

    #[test]
    fn shadow_at_screen_edge_clips_silently() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(10, 6, Color::White);
        shadow(&mut b, Rect::new(7, 4, 5, 5), t); // sombra (9,5): solo 1 celda visible
        assert_eq!(b.get(9, 5).unwrap().bg, t.shadow);
        assert_eq!(b.get(0, 0).unwrap().bg, Color::White);
    }
}
