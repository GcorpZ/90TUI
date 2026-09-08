//! Enlace web clickeable (`Hyperlink`).
//!
//! Recibe un texto y una URL. En el búfer se renderiza como texto
//! subrayado en color (fallback visible en cualquier terminal); para
//! terminales modernas, `osc8_sequence()` construye la secuencia ANSI
//! OSC 8 (`\x1b]8;;URL\x1b\\TEXTO\x1b]8;;\x1b\\`) que la terminal
//! reconoce como enlace clickeable (Ctrl+Clic).
//!
//! Por qué dos caminos: el `Buffer` es una grilla de celdas (un char +
//! colores por celda) y no puede portar secuencias de escape de ancho
//! cero; el OSC 8 se emite por el camino de salida cruda (ver
//! `osc8_sequence`) mientras el búfer muestra el mismo texto con estilo.
//! Expone los 4 parámetros globales (`foreground_color`,
//! `background_color`, `border_color`, `has_shadow`).

use crate::core::{Attr, Buffer, Cell, Color};
use crate::prim::draw_text;

/// Enlace texto + URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hyperlink {
    pub text: String,
    pub url: String,
    pub foreground_color: Color,
    pub background_color: Color,
    pub border_color: Option<Color>,
    pub has_shadow: bool,
    /// Subrayado simulado en el búfer (segunda pasada, ver draw).
    pub underline: bool,
}

impl Hyperlink {
    pub fn new(text: &str, url: &str) -> Self {
        Self {
            text: text.to_string(),
            url: url.to_string(),
            foreground_color: Color::Blue,
            background_color: Color::White,
            border_color: None,
            has_shadow: false,
            underline: true,
        }
    }

    /// Secuencia OSC 8 cruda para salida directa por stdout:
    /// `\x1b]8;;URL\x1b\\TEXTO\x1b]8;;\x1b\\`.
    pub fn osc8_sequence(&self) -> String {
        format!(
            "\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\",
            self.url, self.text
        )
    }
}

/// Dibuja el texto con estilo de enlace (color + subrayado simulado).
/// Devuelve el ancho ocupado.
pub fn hyperlink_draw(buf: &mut Buffer, x: u16, y: u16, link: &Hyperlink) -> u16 {
    let attr = Attr {
        fg: link.foreground_color,
        bg: link.background_color,
        bold: true,
        dim: false,
    };
    let w = draw_text(buf, x, y, &link.text, attr);
    if link.underline {
        // Subrayado simulado: conserva el char, marca la celda.
        // (Las terminales reales lo ven vía `osc8_sequence`.)
        for i in 0..w {
            let cx = x.saturating_add(i);
            if let Some(c) = buf.get(cx, y) {
                buf.set(cx, y, Cell { bold: true, ..c });
            }
        }
    }
    if link.has_shadow {
        for i in 0..w {
            let sx = x.saturating_add(i).saturating_add(1);
            let sy = y.saturating_add(1);
            if buf.in_bounds(sx, sy) {
                let old = buf.get(sx, sy).unwrap_or(Cell::blank(Color::Black));
                buf.set(
                    sx,
                    sy,
                    Cell {
                        ch: old.ch,
                        fg: old.fg,
                        bg: Color::Black,
                        bold: false,
                        dim: true,
                    },
                );
            }
        }
    }
    let _ = link.border_color; // Sin marco: el enlace es texto corrido.
    w
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Theme;

    #[test]
    fn osc8_sequence_is_exact() {
        let l = Hyperlink::new("Portal", "https://example.com/lib");
        assert_eq!(
            l.osc8_sequence(),
            "\u{1b}]8;;https://example.com/lib\u{1b}\\Portal\u{1b}]8;;\u{1b}\\"
        );
    }

    #[test]
    fn draws_bold_colored_text() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 5, t.desktop);
        let l = Hyperlink::new("Docs", "https://example.com");
        let w = hyperlink_draw(&mut b, 2, 1, &l);
        assert_eq!(w, 4);
        assert_eq!(b.get(2, 1).unwrap().ch, 'D');
        assert_eq!(b.get(2, 1).unwrap().fg, Color::Blue);
        assert!(b.get(2, 1).unwrap().bold);
    }
}
