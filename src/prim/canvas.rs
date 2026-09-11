//! Rasterizado sub-celular Braille (2x4): el motor de alta resolución.
//!
//! Una celda de texto mide ~el doble de alto que de ancho (ratio 1:2),
//! así que dibujar curvas con caracteres completos deja huecos y deforma
//! círculos. La macro-técnica: cada celda se divide en una cuadrícula
//! virtual de 2 columnas x 4 filas (8 micro-píxeles). Un área de 60x15
//! celdas pasa a 120x60 píxeles virtuales — y como 2x4 cancela el 1:2
//! de la celda, el micro-píxel sale ~cuadrado (corrección de aspecto
//! gratis, sin factores de escala por ningún lado).
//!
//! Codificación de bits Braille estándar (`U+2800 + byte`):
//! ```text
//! px(0,0)=0x01  px(1,0)=0x08
//! px(0,1)=0x02  px(1,1)=0x10
//! px(0,2)=0x04  px(1,2)=0x20
//! px(0,3)=0x40  px(1,3)=0x80
//! ```
//! El trazado entre puntos usa Bresenham en el espacio virtual: la
//! curva sale continua, sin huecos. Todo recorta en silencio fuera del
//! área (como `buf.set`).

use crate::core::{Buffer, Cell, Color, Rect};

/// Lienzo virtual de micro-píxeles sobre un área de celdas.
pub struct VirtualCanvas {
    /// Ancho virtual = cols * 2.
    pub width: usize,
    /// Alto virtual = rows * 4.
    pub height: usize,
    pixels: Vec<bool>,
}

impl VirtualCanvas {
    /// Crea el lienzo para `cols` x `rows` celdas (todo apagado).
    pub fn new(cols: u16, rows: u16) -> Self {
        let w = cols as usize * 2;
        let h = rows as usize * 4;
        Self {
            width: w,
            height: h,
            pixels: vec![false; w * h],
        }
    }

    /// Enciende un micro-píxel (fuera de rango = no-op).
    pub fn set_pixel(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = true;
        }
    }

    /// Línea de Bresenham entre micro-píxeles (continua, sin huecos).
    pub fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize) {
        let (mut x0, mut y0) = (x0 as isize, y0 as isize);
        let (x1, y1) = (x1 as isize, y1 as isize);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            if x0 >= 0 && y0 >= 0 {
                self.set_pixel(x0 as usize, y0 as usize);
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Máscara braille (0..=255) de la celda (`col`, `row`). Fuera de
    /// rango = 0.
    pub fn cell_mask(&self, col: usize, row: usize) -> u8 {
        let px = col * 2;
        let py = row * 4;
        let mut byte = 0u8;
        if self.get(px, py) {
            byte |= 0x01;
        }
        if self.get(px, py + 1) {
            byte |= 0x02;
        }
        if self.get(px, py + 2) {
            byte |= 0x04;
        }
        if self.get(px + 1, py) {
            byte |= 0x08;
        }
        if self.get(px + 1, py + 1) {
            byte |= 0x10;
        }
        if self.get(px + 1, py + 2) {
            byte |= 0x20;
        }
        if self.get(px, py + 3) {
            byte |= 0x40;
        }
        if self.get(px + 1, py + 3) {
            byte |= 0x80;
        }
        byte
    }

    /// Vuelca el lienzo al buffer: cada celda = `U+2800 + máscara`
    /// (máscara 0 = espacio). Celdas fuera del buffer se recortan.
    pub fn render_to_buffer(&self, buf: &mut Buffer, area: Rect, fg: Color, bg: Color) {
        for row in 0..area.h as usize {
            for col in 0..area.w as usize {
                let byte = self.cell_mask(col, row);
                let ch = if byte == 0 {
                    ' '
                } else {
                    char::from_u32(0x2800 + byte as u32).unwrap_or(' ')
                };
                buf.set(
                    area.x.saturating_add(col as u16),
                    area.y.saturating_add(row as u16),
                    Cell::new(ch, fg, bg),
                );
            }
        }
    }

    fn get(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x]
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dims_are_subcell() {
        let c = VirtualCanvas::new(60, 15);
        assert_eq!((c.width, c.height), (120, 60));
        assert_eq!(VirtualCanvas::new(0, 0).pixels.len(), 0);
    }

    #[test]
    fn pixels_out_of_range_are_ignored() {
        let mut c = VirtualCanvas::new(2, 1);
        c.set_pixel(99, 99); // no panic
        c.set_pixel(0, 0);
        assert_eq!(c.cell_mask(0, 0), 0x01);
        assert_eq!(c.cell_mask(5, 5), 0); // fuera del lienzo
    }

    #[test]
    fn bit_mapping_matches_braille_standard() {
        // (0,0)->0x01 (1,0)->0x08 (0,1)->0x02 (1,1)->0x10
        // (0,2)->0x04 (1,2)->0x20 (0,3)->0x40 (1,3)->0x80.
        let mut c = VirtualCanvas::new(1, 1);
        c.set_pixel(0, 0);
        c.set_pixel(1, 0);
        assert_eq!(c.cell_mask(0, 0), 0x01 | 0x08);
        let mut c = VirtualCanvas::new(1, 1);
        c.set_pixel(0, 3);
        c.set_pixel(1, 3);
        assert_eq!(c.cell_mask(0, 0), 0x40 | 0x80);
        let mut c = VirtualCanvas::new(1, 1);
        for y in 0..4 {
            c.set_pixel(0, y);
        }
        assert_eq!(c.cell_mask(0, 0), 0x01 | 0x02 | 0x04 | 0x40);
    }

    #[test]
    fn lines_are_continuous_without_gaps() {
        // Horizontal: todas las celdas de la fila tienen algún dot.
        let mut c = VirtualCanvas::new(10, 2);
        c.draw_line(0, 2, 19, 2);
        for col in 0..10 {
            assert_ne!(c.cell_mask(col, 0), 0, "hueco en col {col}");
        }
        // Diagonal empinada: toca todas las filas (una por celda al menos).
        let mut c = VirtualCanvas::new(4, 6);
        c.draw_line(0, 23, 7, 0);
        for row in 0..6 {
            let row_lit = (0..4).any(|col| c.cell_mask(col, row) != 0);
            assert!(row_lit, "fila {row} vacía en diagonal");
        }
        // Punto degenerado (origen == destino) enciende uno.
        let mut c = VirtualCanvas::new(3, 3);
        c.draw_line(2, 5, 2, 5);
        assert_ne!(c.cell_mask(1, 1), 0);
    }

    #[test]
    fn render_emits_braille_or_space() {
        let t = crate::core::Theme::clipper();
        // Un solo micro-píxel (0,0) -> U+2801 exacto.
        let mut b = Buffer::blank(8, 4, t.desktop);
        let mut one = VirtualCanvas::new(8, 4);
        one.set_pixel(0, 0);
        one.render_to_buffer(&mut b, Rect::new(0, 0, 8, 4), Color::Yellow, t.desktop);
        assert_eq!(b.get(0, 0).unwrap().ch, '\u{2801}');
        assert_eq!(b.get(0, 0).unwrap().fg, Color::Yellow);
        // Diagonal: celdas tocadas con braille en rango, resto espacios.
        let mut b = Buffer::blank(8, 4, t.desktop);
        let mut c = VirtualCanvas::new(8, 4);
        c.draw_line(0, 0, 15, 15);
        c.render_to_buffer(&mut b, Rect::new(0, 0, 8, 4), Color::Yellow, t.desktop);
        let mut lit = 0;
        for y in 0..4u16 {
            for x in 0..8u16 {
                let ch = b.get(x, y).unwrap().ch;
                assert!(ch == ' ' || (0x2800..=0x28FF).contains(&(ch as u32)));
                if ch != ' ' {
                    lit += 1;
                }
            }
        }
        assert!(lit > 2, "diagonal visible");
        // Lienzo vacío = espacios (no U+2800 en blanco).
        let mut b2 = Buffer::blank(4, 2, t.desktop);
        VirtualCanvas::new(4, 2).render_to_buffer(
            &mut b2,
            Rect::new(0, 0, 4, 2),
            Color::Yellow,
            t.desktop,
        );
        for y in 0..2u16 {
            for x in 0..4u16 {
                assert_eq!(b2.get(x, y).unwrap().ch, ' ');
            }
        }
    }
}
