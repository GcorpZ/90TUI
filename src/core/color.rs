//! Color VGA + atributo (fg/bg/bold).
//!
//! Equivalente moderno al byte de atributo de `0xB800`:
//! en DOS era un byte (4 bits fondo + 4 bits frente);
//! aquí es un `enum` seguro que se traduce a ANSI vía crossterm.

use crossterm::style::Color as Cc;

/// Paleta base. Nombres pensados para el tema pastel estilo Clipper;
/// `Theme::turbo()` los reutiliza con otra asignación.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    Black,
    Navy, // azul oscuro de barras (VGA DarkBlue)
    Blue, // azul medio de popups
    Teal, // teal oscuro de menús/botones (VGA DarkCyan)
    Cyan, // cian brillante del escritorio de la época
    DarkGrey,
    Grey, // gris de selección / ventanas
    White,
    Yellow, // hotkeys
    Mint,   // verde menta de diálogos de progreso
    Red,
    Green,
}

impl Color {
    /// Traducción a color crossterm (ANSI 16 colores).
    pub fn to_crossterm(self) -> Cc {
        match self {
            Color::Black => Cc::Black,
            Color::Navy => Cc::DarkBlue,
            Color::Blue => Cc::Blue,
            Color::Teal => Cc::DarkCyan,
            Color::Cyan => Cc::Cyan,
            Color::DarkGrey => Cc::DarkGrey,
            Color::Grey => Cc::Grey,
            Color::White => Cc::White,
            Color::Yellow => Cc::Yellow,
            Color::Mint => Cc::Green,
            Color::Red => Cc::Red,
            Color::Green => Cc::DarkGreen,
        }
    }
}

/// Par fg/bg/negrita/atenuado listo para pintar una celda o un span.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Attr {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub dim: bool,
}

impl Attr {
    pub fn new(fg: Color, bg: Color) -> Self {
        Self {
            fg,
            bg,
            bold: false,
            dim: false,
        }
    }

    pub fn bold(fg: Color, bg: Color) -> Self {
        Self {
            fg,
            bg,
            bold: true,
            dim: false,
        }
    }

    /// Atenuado ANSI (`\x1b[2m`): sombras fantasma.
    pub fn faint(fg: Color, bg: Color) -> Self {
        Self {
            fg,
            bg,
            bold: false,
            dim: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_color_maps_to_something() {
        let all = [
            Color::Black,
            Color::Navy,
            Color::Blue,
            Color::Teal,
            Color::Cyan,
            Color::DarkGrey,
            Color::Grey,
            Color::White,
            Color::Yellow,
            Color::Mint,
            Color::Red,
            Color::Green,
        ];
        for c in all {
            let _ = c.to_crossterm();
        }
    }

    #[test]
    fn attr_constructors() {
        let a = Attr::new(Color::White, Color::Blue);
        assert!(!a.bold);
        let b = Attr::bold(Color::Yellow, Color::Blue);
        assert!(b.bold);
        assert_ne!(a, b);
    }
}
