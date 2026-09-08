//! Celda de la VRAM virtual: 1 carácter + atributo.
//!
//! En DOS real eran 2 bytes en `0xB800:0000` (char + attr).
//! Aquí es un struct `Copy` para que el doble buffer sea barato de clonar.

use super::color::{Attr, Color};

/// Una posición de la parrilla 80x25 (o cualquier tamaño moderno).
/// `dim` = atenuado ANSI (`\x1b[2m`): lo usan las sombras fantasma.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub dim: bool,
}

impl Cell {
    pub fn new(ch: char, fg: Color, bg: Color) -> Self {
        Self {
            ch,
            fg,
            bg,
            bold: false,
            dim: false,
        }
    }

    pub fn bold(ch: char, fg: Color, bg: Color) -> Self {
        Self {
            ch,
            fg,
            bg,
            bold: true,
            dim: false,
        }
    }

    /// Celda atenuada (sombras): mismo color, con `dim`.
    pub fn faint(ch: char, fg: Color, bg: Color) -> Self {
        Self {
            ch,
            fg,
            bg,
            bold: false,
            dim: true,
        }
    }

    pub fn with_attr(ch: char, attr: Attr) -> Self {
        Self {
            ch,
            fg: attr.fg,
            bg: attr.bg,
            bold: attr.bold,
            dim: attr.dim,
        }
    }

    /// Celda vacía (espacio del color de fondo). fg=bg para no dejar
    /// residuos visibles si el terminal falla en algo.
    pub fn blank(bg: Color) -> Self {
        Self {
            ch: ' ',
            fg: bg,
            bg,
            bold: false,
            dim: false,
        }
    }

    pub fn attr(&self) -> Attr {
        Attr {
            fg: self.fg,
            bg: self.bg,
            bold: self.bold,
            dim: self.dim,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_uses_bg_as_fg() {
        let c = Cell::blank(Color::Cyan);
        assert_eq!(c.ch, ' ');
        assert_eq!(c.fg, Color::Cyan);
        assert_eq!(c.bg, Color::Cyan);
    }

    #[test]
    fn with_attr_roundtrip() {
        let a = Attr::bold(Color::Yellow, Color::Blue);
        let c = Cell::with_attr('R', a);
        assert_eq!(c.attr(), a);
    }
}
