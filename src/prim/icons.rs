//! Iconos de 1–4 celdas: carpetas, unidades y cajas de ventana.
//!
//! En modo texto cada "píxel" es una celda (carácter + colores), así que
//! los iconos se arman con glifos de cobertura universal (`■ ► ▼ ▲ -`,
//! presentes hasta en Consolas) + color. Nada de emoji ni fuentes raras:
//! si la celda lo puede pintar, el icono existe.

use crate::core::{Attr, Buffer, Cell, Color};

/// Carpeta: `►■` cerrada / `▼■` abierta (marcador + cuadro en acento).
/// Devuelve el ancho ocupado (2).
pub fn folder(buf: &mut Buffer, x: u16, y: u16, open: bool, attr: Attr, accent: Color) -> u16 {
    let mark = if open { '▼' } else { '►' };
    buf.set(x, y, Cell::with_attr(mark, attr));
    buf.set(x.saturating_add(1), y, Cell::new('■', accent, attr.bg));
    2
}

/// Unidad de disco: `[C:]` con la letra en acento. Devuelve 4.
pub fn drive(buf: &mut Buffer, x: u16, y: u16, letter: char, attr: Attr, accent: Color) -> u16 {
    let up = letter.to_ascii_uppercase();
    buf.set(x, y, Cell::with_attr('[', attr));
    buf.set(
        x.saturating_add(1),
        y,
        Cell::with_attr(up, Attr { fg: accent, ..attr }),
    );
    buf.set(x.saturating_add(2), y, Cell::with_attr(':', attr));
    buf.set(x.saturating_add(3), y, Cell::with_attr(']', attr));
    4
}

/// Caja de cierre `[■]` para barras de título. Devuelve 3.
pub fn win_close(buf: &mut Buffer, x: u16, y: u16, attr: Attr, accent: Color) -> u16 {
    buf.set(x, y, Cell::with_attr('[', attr));
    buf.set(
        x.saturating_add(1),
        y,
        Cell::with_attr('■', Attr { fg: accent, ..attr }),
    );
    buf.set(x.saturating_add(2), y, Cell::with_attr(']', attr));
    3
}

/// Caja de minimizar `[-]` para barras de título. Devuelve 3.
pub fn win_min(buf: &mut Buffer, x: u16, y: u16, attr: Attr) -> u16 {
    buf.set(x, y, Cell::with_attr('[', attr));
    buf.set(x.saturating_add(1), y, Cell::with_attr('-', attr));
    buf.set(x.saturating_add(2), y, Cell::with_attr(']', attr));
    3
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Buffer, Theme};

    fn attr() -> (Attr, Theme) {
        let t = Theme::clipper();
        (t.popup_attr(), t)
    }

    #[test]
    fn folder_marks_open_state() {
        let (a, t) = attr();
        let mut b = Buffer::blank(20, 4, t.desktop);
        folder(&mut b, 1, 1, false, a, Color::Yellow);
        folder(&mut b, 1, 2, true, a, Color::Yellow);
        assert_eq!(b.get(1, 1).unwrap().ch, '►');
        assert_eq!(b.get(2, 1).unwrap().ch, '■');
        assert_eq!(b.get(2, 1).unwrap().fg, Color::Yellow);
        assert_eq!(b.get(1, 2).unwrap().ch, '▼');
    }

    #[test]
    fn drive_spells_letter() {
        let (a, t) = attr();
        let mut b = Buffer::blank(20, 4, t.desktop);
        assert_eq!(drive(&mut b, 1, 1, 'c', a, Color::Yellow), 4);
        assert_eq!(b.get(2, 1).unwrap().ch, 'C');
        assert_eq!(b.get(2, 1).unwrap().fg, Color::Yellow);
    }

    #[test]
    fn window_boxes() {
        let (a, t) = attr();
        let mut b = Buffer::blank(20, 4, t.desktop);
        assert_eq!(win_close(&mut b, 1, 1, a, Color::Yellow), 3);
        assert_eq!(win_min(&mut b, 5, 1, a), 3);
        assert_eq!(b.get(2, 1).unwrap().ch, '■');
        assert_eq!(b.get(6, 1).unwrap().ch, '-');
    }
}
