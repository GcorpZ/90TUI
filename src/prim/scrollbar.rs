//! Barra de scroll vertical: `▲ ░ █ ▼`.
//!
//! Columna de 1 de ancho para paneles (árbol, listas, tablas): flechas en
//! los extremos, track punteado y thumb proporcional. Todo con glifos de
//! cobertura universal.

use crate::core::{Buffer, Cell, Color, Rect, Theme};

/// Dibuja el scrollbar en `col` (se usa `col.x`, `col.y`, `col.h`;
/// el ancho se fuerza a 1). `total` = items, `top` = primero visible,
/// `visible` = filas mostrables.
pub fn vscrollbar(
    buf: &mut Buffer,
    col: Rect,
    total: usize,
    top: usize,
    visible: usize,
    theme: Theme,
) {
    if col.h == 0 {
        return;
    }
    let x = col.x;
    let (track_fg, track_bg) = (Color::DarkGrey, theme.field_bg);
    let (thumb_fg, arrow_fg) = (Color::Black, Color::Black);
    // Flechas.
    buf.set(x, col.y, Cell::new('▲', arrow_fg, track_bg));
    buf.set(
        x,
        col.bottom().saturating_sub(1),
        Cell::new('▼', arrow_fg, track_bg),
    );
    if col.h <= 2 {
        return;
    }
    let track_y = col.y + 1;
    let track_h = (col.h - 2) as usize;
    // Track.
    for i in 0..track_h {
        buf.set(x, track_y + i as u16, Cell::new('░', track_fg, track_bg));
    }
    // Thumb proporcional.
    let vis = visible.max(1).min(total.max(1));
    let thumb_h = if total <= vis {
        track_h
    } else {
        (track_h * vis / total).max(1).min(track_h)
    };
    let thumb_y = if total <= vis {
        0
    } else {
        (track_h - thumb_h) * top.min(total - vis) / (total - vis).max(1)
    };
    for i in 0..thumb_h {
        buf.set(
            x,
            track_y + thumb_y as u16 + i as u16,
            Cell::new('█', thumb_fg, track_bg),
        );
    }
    let _ = theme;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrows_thumb_and_track() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(10, 10, t.desktop);
        vscrollbar(&mut b, Rect::new(8, 1, 1, 8), 20, 0, 5, t);
        assert_eq!(b.get(8, 1).unwrap().ch, '▲');
        assert_eq!(b.get(8, 8).unwrap().ch, '▼');
        // Thumb arriba del todo con top=0.
        assert_eq!(b.get(8, 2).unwrap().ch, '█');
        // Abajo queda track.
        assert_eq!(b.get(8, 7).unwrap().ch, '░');
    }

    #[test]
    fn thumb_follows_top() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(10, 10, t.desktop);
        vscrollbar(&mut b, Rect::new(8, 1, 1, 8), 20, 15, 5, t); // al final
        assert_eq!(b.get(8, 7).unwrap().ch, '█');
        assert_eq!(b.get(8, 2).unwrap().ch, '░');
    }
}
