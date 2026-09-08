//! Barra de progreso (`ProgressBar`): 0–100 con bloques Unicode.
//!
//! Relleno `\u{2588}` (lleno) sobre `\u{2591}` (tenue), con los 4
//! parámetros globales (`foreground_color` = relleno,
//! `background_color` = pista, `border_color` opt-in, `has_shadow`).
//!
//! Nota de tipos: colores como `Color` (16 ANSI), no `(u8, u8, u8)`.

use crate::core::{Attr, Buffer, Cell, Color, Rect};
use crate::prim::{draw_text, shadow, visible_len};

const FILL: char = '\u{2588}';
const TRACK: char = '\u{2591}';

/// Barra de progreso 0–100.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressBar {
    /// Porcentaje (se recorta a 0..=100 al dibujar).
    pub pct: u8,
    /// Color del relleno (`\u{2588}`).
    pub foreground_color: Color,
    /// Color de la pista (`\u{2591}`).
    pub background_color: Color,
    pub border_color: Option<Color>,
    pub has_shadow: bool,
    /// Pinta `NN%` centrado si cabe.
    pub show_pct: bool,
}

impl ProgressBar {
    pub fn new(pct: u8) -> Self {
        Self {
            pct: pct.min(100),
            foreground_color: Color::Navy,
            background_color: Color::Grey,
            border_color: None,
            has_shadow: false,
            show_pct: true,
        }
    }

    pub fn set_pct(&mut self, pct: u8) {
        self.pct = pct.min(100);
    }
}

/// Celdas de relleno para `pct` en un ancho `w`.
pub fn progress_fill(pct: u8, w: u16) -> u16 {
    (pct.min(100) as u32 * w as u32 / 100) as u16
}

/// Dibuja la barra en `rect` (usa la primera fila; borde opt-in).
pub fn progressbar_draw(buf: &mut Buffer, rect: Rect, bar: &ProgressBar) {
    if rect.is_empty() || rect.h < 1 {
        return;
    }
    let bordered = bar.border_color.is_some();
    if bar.has_shadow {
        shadow(
            buf,
            if bordered {
                rect
            } else {
                Rect::new(rect.x, rect.y, rect.w, 1)
            },
            crate::core::Theme::clipper(),
        );
    }
    if bordered {
        let b = bar.border_color.unwrap_or(Color::Black);
        let bc = Cell::new(' ', b, b);
        for x in rect.x..rect.right() {
            buf.set(x, rect.y, bc);
            buf.set(x, rect.bottom().saturating_sub(1), bc);
        }
        for y in rect.y..rect.bottom() {
            buf.set(rect.x, y, bc);
            buf.set(rect.right().saturating_sub(1), y, bc);
        }
    }
    let inner = if bordered && rect.w >= 3 && rect.h >= 3 {
        Rect::new(
            rect.x.saturating_add(1),
            rect.y.saturating_add(1),
            rect.w.saturating_sub(2),
            1,
        )
    } else {
        Rect::new(rect.x, rect.y, rect.w, 1)
    };
    if inner.is_empty() {
        return;
    }
    let fill = progress_fill(bar.pct, inner.w);
    for i in 0..inner.w {
        let filled = i < fill;
        let (ch, fg) = if filled {
            (FILL, bar.foreground_color)
        } else {
            (TRACK, bar.background_color)
        };
        buf.set(
            inner.x.saturating_add(i),
            inner.y,
            Cell::new(ch, fg, bar.background_color),
        );
    }
    if bar.show_pct {
        let label = format!("{}%", bar.pct.min(100));
        if visible_len(&label).saturating_add(2) <= inner.w {
            let lx = inner
                .x
                .saturating_add(inner.w.saturating_sub(visible_len(&label)) / 2);
            // Texto legible sobre ambos tramos: invierte según el fondo.
            for (i, ch) in label.chars().enumerate() {
                let x = lx.saturating_add(i as u16);
                let filled = x < inner.x.saturating_add(fill);
                let attr = if filled {
                    Attr::bold(bar.background_color, bar.foreground_color)
                } else {
                    Attr::bold(bar.foreground_color, bar.background_color)
                };
                draw_text(buf, x, inner.y, &ch.to_string(), attr);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Theme;

    #[test]
    fn fill_math() {
        assert_eq!(progress_fill(0, 10), 0);
        assert_eq!(progress_fill(50, 10), 5);
        assert_eq!(progress_fill(100, 10), 10);
        assert_eq!(progress_fill(200, 10), 10); // recorta
    }

    #[test]
    fn draws_fill_and_track() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 5, t.desktop);
        let mut bar = ProgressBar::new(50);
        bar.show_pct = false;
        progressbar_draw(&mut b, Rect::new(2, 1, 10, 1), &bar);
        assert_eq!(b.get(2, 1).unwrap().ch, '\u{2588}');
        assert_eq!(b.get(6, 1).unwrap().ch, '\u{2588}');
        assert_eq!(b.get(7, 1).unwrap().ch, '\u{2591}');
        assert_eq!(b.get(2, 1).unwrap().fg, bar.foreground_color);
    }

    #[test]
    fn shows_pct_when_it_fits() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 5, t.desktop);
        let bar = ProgressBar::new(42);
        progressbar_draw(&mut b, Rect::new(2, 1, 20, 1), &bar);
        // "42%" centrado: x = 2 + (20-3)/2 = 10.
        assert_eq!(b.get(10, 1).unwrap().ch, '4');
        assert_eq!(b.get(11, 1).unwrap().ch, '2');
        assert_eq!(b.get(12, 1).unwrap().ch, '%');
    }
}
