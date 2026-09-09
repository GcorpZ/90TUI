//! Barra de progreso (`ProgressBar`): 0–100 con resolución sub-celular.
//!
//! Relleno de alta resolución (8x) con octavos de bloque sobre pista
//! `\u{2591}`, con los 4 parámetros globales (`foreground_color` =
//! relleno, `background_color` = pista, `border_color` opt-in,
//! `has_shadow`).
//!
//! Nota de tipos: colores como `Color` (16 ANSI), no `(u8, u8, u8)`.

use crate::core::{Attr, Buffer, Cell, Color, Rect};
use crate::prim::{draw_text, shadow, visible_len};

const FILL: char = '\u{2588}';
const TRACK: char = '\u{2591}';

/// Octavos de bloque Unicode: resolución 8x por celda (el índice es el
/// número de octavos pintados; 0 = vacío).
pub const SUB_BLOCKS: [char; 8] = [
    ' ', '\u{258f}', '\u{258e}', '\u{258d}', '\u{258c}', '\u{258b}', '\u{258a}', '\u{2589}',
];

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

/// Micro-pasos de relleno (octavos) para `pct` en un ancho `w`.
pub fn progress_eighths(pct: u8, w: u16) -> u32 {
    pct.min(100) as u32 * w as u32 * 8 / 100
}

/// Dibuja la barra en `rect` (usa la primera fila; borde opt-in):
/// corchetes `[`/`]` si hay `border_color` y altura < 3, o recuadro de
/// línea simple si la altura es >= 3.
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
    // Riel: recuadro simple si cabe en alto, si no corchetes laterales.
    let inner = if bordered && rect.w >= 3 && rect.h >= 3 {
        draw_box(
            buf,
            rect,
            bar.border_color.unwrap_or(Color::Black),
            bar.background_color,
        );
        Rect::new(
            rect.x.saturating_add(1),
            rect.y.saturating_add(1),
            rect.w.saturating_sub(2),
            1,
        )
    } else if bordered && rect.w >= 3 {
        let bc = Attr::new(
            bar.border_color.unwrap_or(Color::Black),
            bar.background_color,
        );
        buf.set(rect.x, rect.y, Cell::with_attr('[', bc));
        buf.set(
            rect.right().saturating_sub(1),
            rect.y,
            Cell::with_attr(']', bc),
        );
        Rect::new(
            rect.x.saturating_add(1),
            rect.y,
            rect.w.saturating_sub(2),
            1,
        )
    } else {
        Rect::new(rect.x, rect.y, rect.w, 1)
    };
    if inner.is_empty() {
        return;
    }
    // Relleno sub-celular: celdas llenas + celda frontera con octavos.
    let total = progress_eighths(bar.pct, inner.w);
    let full_cells = (total / 8) as u16;
    let rem = (total % 8) as usize;
    let frontier = inner.x.saturating_add(full_cells);
    for i in 0..inner.w {
        let x = inner.x.saturating_add(i);
        let (ch, fg) = if i < full_cells {
            (FILL, bar.foreground_color)
        } else if i == full_cells && full_cells < inner.w {
            (SUB_BLOCKS[rem], bar.foreground_color)
        } else {
            (TRACK, bar.background_color)
        };
        buf.set(x, inner.y, Cell::new(ch, fg, bar.background_color));
    }
    if bar.show_pct {
        let label = format!("{}%", bar.pct.min(100));
        if visible_len(&label).saturating_add(2) <= inner.w {
            let lx = inner
                .x
                .saturating_add(inner.w.saturating_sub(visible_len(&label)) / 2);
            // Texto legible sobre ambos tramos: invierte según el fondo.
            // La celda frontera no se pisa (conserva su octavo legible).
            for (i, ch) in label.chars().enumerate() {
                let x = lx.saturating_add(i as u16);
                if x == frontier {
                    continue;
                }
                let filled = x < frontier;
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

/// Recuadro de línea simple (`┌─┐│└┘`) sobre el fondo de la pista.
fn draw_box(buf: &mut Buffer, rect: Rect, border: Color, bg: Color) {
    let bc = Attr::new(border, bg);
    let right = rect.right().saturating_sub(1);
    let bottom = rect.bottom().saturating_sub(1);
    for x in rect.x.saturating_add(1)..rect.right().saturating_sub(1) {
        buf.set(x, rect.y, Cell::with_attr('\u{2500}', bc));
        buf.set(x, bottom, Cell::with_attr('\u{2500}', bc));
    }
    for y in rect.y.saturating_add(1)..rect.bottom().saturating_sub(1) {
        buf.set(rect.x, y, Cell::with_attr('\u{2502}', bc));
        buf.set(right, y, Cell::with_attr('\u{2502}', bc));
    }
    buf.set(rect.x, rect.y, Cell::with_attr('\u{250c}', bc));
    buf.set(right, rect.y, Cell::with_attr('\u{2510}', bc));
    buf.set(rect.x, bottom, Cell::with_attr('\u{2514}', bc));
    buf.set(right, bottom, Cell::with_attr('\u{2518}', bc));
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
        // 50% de 10 = 40 octavos: 5 llenas + frontera vacía + pista.
        assert_eq!(b.get(2, 1).unwrap().ch, '\u{2588}');
        assert_eq!(b.get(6, 1).unwrap().ch, '\u{2588}');
        assert_eq!(b.get(7, 1).unwrap().ch, ' ');
        assert_eq!(b.get(8, 1).unwrap().ch, '\u{2591}');
        assert_eq!(b.get(2, 1).unwrap().fg, bar.foreground_color);
    }

    #[test]
    fn fractional_eighths() {
        assert_eq!(SUB_BLOCKS.len(), 8);
        assert_eq!(progress_eighths(0, 10), 0);
        assert_eq!(progress_eighths(100, 10), 80);
        assert_eq!(progress_eighths(200, 10), 80); // recorta
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 5, t.desktop);
        // 13% de 10 = 10 octavos: 1 llena + frontera ▎(2/8).
        let mut bar = ProgressBar::new(13);
        bar.show_pct = false;
        progressbar_draw(&mut b, Rect::new(2, 1, 10, 1), &bar);
        assert_eq!(b.get(2, 1).unwrap().ch, '\u{2588}');
        assert_eq!(b.get(3, 1).unwrap().ch, '\u{258e}');
        assert_eq!(b.get(4, 1).unwrap().ch, '\u{2591}');
        // 5% de 10 = 4 octavos: solo frontera ▌(4/8).
        let mut bar2 = ProgressBar::new(5);
        bar2.show_pct = false;
        progressbar_draw(&mut b, Rect::new(2, 3, 10, 1), &bar2);
        assert_eq!(b.get(2, 3).unwrap().ch, '\u{258c}');
        assert_eq!(b.get(3, 3).unwrap().ch, '\u{2591}');
    }

    #[test]
    fn shows_pct_when_it_fits() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 5, t.desktop);
        let bar = ProgressBar::new(42);
        progressbar_draw(&mut b, Rect::new(2, 1, 20, 1), &bar);
        // "42%" centrado en x=10, pero el '4' cae en la frontera (▍) y se
        // respeta el octavo: la celda conserva su glifo legible.
        assert_eq!(b.get(10, 1).unwrap().ch, '\u{258d}');
        assert_eq!(b.get(11, 1).unwrap().ch, '2');
        assert_eq!(b.get(12, 1).unwrap().ch, '%');
    }

    #[test]
    fn brackets_and_box_borders() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 8, t.desktop);
        // 1 fila: corchetes laterales.
        let mut bar = ProgressBar::new(50);
        bar.show_pct = false;
        bar.border_color = Some(Color::Black);
        progressbar_draw(&mut b, Rect::new(2, 1, 12, 1), &bar);
        assert_eq!(b.get(2, 1).unwrap().ch, '[');
        assert_eq!(b.get(13, 1).unwrap().ch, ']');
        assert_eq!(b.get(3, 1).unwrap().ch, '\u{2588}');
        // Alto >= 3: recuadro de línea simple.
        progressbar_draw(&mut b, Rect::new(2, 3, 12, 3), &bar);
        assert_eq!(b.get(2, 3).unwrap().ch, '\u{250c}');
        assert_eq!(b.get(13, 3).unwrap().ch, '\u{2510}');
        assert_eq!(b.get(2, 5).unwrap().ch, '\u{2514}');
        assert_eq!(b.get(13, 5).unwrap().ch, '\u{2518}');
        assert_eq!(b.get(3, 4).unwrap().ch, '\u{2588}');
    }
}
