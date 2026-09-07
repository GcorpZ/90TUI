//! Separadores finos entre grupos de menú (`─` en el fg del popup).

use crate::core::{Buffer, Color};

/// Línea horizontal fina de x0..=x1 en la fila y (recortada al buffer).
pub fn hsep(buf: &mut Buffer, y: u16, x0: u16, x1: u16, fg: Color, bg: Color) {
    buf.hline(y, x0, x1, '─', fg, bg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Theme;

    #[test]
    fn draws_between_bounds() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(20, 5, t.popup);
        hsep(&mut b, 2, 3, 8, t.popup_text, t.popup);
        assert_eq!(b.get(3, 2).unwrap().ch, '─');
        assert_eq!(b.get(8, 2).unwrap().ch, '─');
        assert_eq!(b.get(2, 2).unwrap().ch, ' ');
    }
}
