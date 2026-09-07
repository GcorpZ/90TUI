//! Barras superior e inferior navy (título de app + estado/ayuda).

use crate::core::{Buffer, Theme};

use super::label::{draw_text, visible_len};

/// Fila 0: `left` a la izquierda, `right` a la derecha, fondo navy.
pub fn top_bar(buf: &mut Buffer, left: &str, right: &str, theme: Theme) {
    if buf.height() == 0 {
        return;
    }
    let w = buf.width();
    buf.fill_rect(
        crate::core::Rect::new(0, 0, w, 1),
        crate::core::Cell::new(' ', theme.status_fg, theme.navy),
    );
    let a = theme.top_attr();
    draw_text(buf, 1, 0, left, a);
    let rw = visible_len(right);
    if rw + 2 < w {
        draw_text(buf, w.saturating_sub(rw).saturating_sub(1), 0, right, a);
    }
}

/// Última fila: igual que arriba (estado + ayuda).
pub fn status_bar(buf: &mut Buffer, left: &str, right: &str, theme: Theme) {
    if buf.height() == 0 {
        return;
    }
    let (w, h) = (buf.width(), buf.height());
    buf.fill_rect(
        crate::core::Rect::new(0, h.saturating_sub(1), w, 1),
        crate::core::Cell::new(' ', theme.status_fg, theme.status_bg),
    );
    let a = theme.status_attr();
    draw_text(buf, 1, h.saturating_sub(1), left, a);
    let rw = visible_len(right);
    if rw + 2 < w {
        draw_text(
            buf,
            w.saturating_sub(rw).saturating_sub(1),
            h.saturating_sub(1),
            right,
            a,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Color;

    #[test]
    fn bars_paint_first_and_last_rows() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 10, Color::White);
        top_bar(&mut b, "EMPRESA", "FECHA", t);
        status_bar(&mut b, "v0.0.1", "Ayuda", t);
        assert_eq!(b.get(0, 0).unwrap().bg, t.navy);
        assert_eq!(b.get(1, 0).unwrap().ch, 'E');
        assert_eq!(b.get(0, 9).unwrap().bg, t.status_bg);
        assert_eq!(b.get(1, 9).unwrap().ch, 'v');
        // El medio no se toca.
        assert_eq!(b.get(1, 5).unwrap().bg, Color::White);
        // Texto derecho anclado al borde.
        assert_eq!(b.get(30 - 1 - 5, 0).unwrap().ch, 'F'); // "FECHA"
    }
}
