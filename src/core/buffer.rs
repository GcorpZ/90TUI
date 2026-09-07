//! Back-buffer: la RAM espejo antes de la "VRAM".
//!
//! En DOS se dibujaba todo en RAM convencional y luego un `memcpy` a
//! `0xB8000`. Aquí se dibuja todo en `Buffer` y luego solo viaja por
//! ANSI el `diff()` contra el frame anterior. Mismo efecto: sin flicker
//! y rapidísimo, sin escribir carácter por carácter en la terminal.

use unicode_width::UnicodeWidthChar;

use super::cell::Cell;
use super::color::Color;
use super::rect::Rect;

/// Una operación de pintado: cambia una celda en (x, y).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawOp {
    pub x: u16,
    pub y: u16,
    pub cell: Cell,
}

/// Foto rectangular para `Savescreen()` (ver `screen.rs`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub rect: Rect,
    pub cells: Vec<Cell>, // row-major, `rect.w * rect.h`
}

/// Parrilla de celdas `w x h`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Buffer {
    w: u16,
    h: u16,
    cells: Vec<Cell>,
}

impl Buffer {
    pub fn new(w: u16, h: u16, fill: Cell) -> Self {
        Self {
            w,
            h,
            cells: vec![fill; w as usize * h as usize],
        }
    }

    pub fn blank(w: u16, h: u16, bg: Color) -> Self {
        Self::new(w, h, Cell::blank(bg))
    }

    pub fn width(&self) -> u16 {
        self.w
    }

    pub fn height(&self) -> u16 {
        self.h
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(0, 0, self.w, self.h)
    }

    #[inline]
    fn idx(&self, x: u16, y: u16) -> Option<usize> {
        if x < self.w && y < self.h {
            Some(y as usize * self.w as usize + x as usize)
        } else {
            None
        }
    }

    pub fn in_bounds(&self, x: u16, y: u16) -> bool {
        self.idx(x, y).is_some()
    }

    pub fn get(&self, x: u16, y: u16) -> Option<Cell> {
        self.idx(x, y).map(|i| self.cells[i])
    }

    /// Escritura con recorte silencioso (como Clipper: lo de fuera se ignora).
    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if let Some(i) = self.idx(x, y) {
            self.cells[i] = cell;
        }
    }

    /// Rellena un rect (recortado a este buffer).
    pub fn fill_rect(&mut self, rect: Rect, cell: Cell) {
        let Some(r) = rect.intersect(self.bounds()) else {
            return;
        };
        for y in r.y..r.bottom() {
            for x in r.x..r.right() {
                // `idx` ya validado por la intersección.
                let i = y as usize * self.w as usize + x as usize;
                self.cells[i] = cell;
            }
        }
    }

    /// Texto de una línea con (fg, bg). Recorta al borde, ignora `\n`,
    /// salta combinantes (ancho 0) y avanza según ancho Unicode.
    pub fn text(&mut self, x: u16, y: u16, s: &str, fg: Color, bg: Color) {
        if y >= self.h {
            return;
        }
        let mut cx = x;
        for ch in s.chars() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            let w = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
            if w == 0 {
                continue; // combinante o control: no ocupa celda
            }
            if cx >= self.w {
                break;
            }
            self.set(cx, y, Cell::new(ch, fg, bg));
            cx = cx.saturating_add(w);
        }
    }

    /// Texto en negrita (títulos SAINT).
    pub fn text_bold(&mut self, x: u16, y: u16, s: &str, fg: Color, bg: Color) {
        if y >= self.h {
            return;
        }
        let mut cx = x;
        for ch in s.chars() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            let w = UnicodeWidthChar::width(ch).unwrap_or(0) as u16;
            if w == 0 {
                continue;
            }
            if cx >= self.w {
                break;
            }
            self.set(cx, y, Cell::bold(ch, fg, bg));
            cx = cx.saturating_add(w);
        }
    }

    /// Línea horizontal de un glifo (separadores de grupos SAINT).
    pub fn hline(&mut self, y: u16, x0: u16, x1: u16, ch: char, fg: Color, bg: Color) {
        if y >= self.h {
            return;
        }
        let (a, b) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
        for x in a..=b {
            self.set(x, y, Cell::new(ch, fg, bg));
        }
    }

    /// Copia otro buffer con offset (recortado). Como `Blit` de VGA.
    pub fn blit(&mut self, src: &Buffer, dx: i32, dy: i32) {
        for y in 0..src.h {
            for x in 0..src.w {
                let tx = x as i32 + dx;
                let ty = y as i32 + dy;
                if tx >= 0 && ty >= 0 {
                    if let Some(c) = src.get(x, y) {
                        self.set(tx as u16, ty as u16, c);
                    }
                }
            }
        }
    }

    /// Foto de una zona para apilar (recortada al buffer).
    pub fn snapshot(&self, rect: Rect) -> Snapshot {
        let r = rect
            .intersect(self.bounds())
            .unwrap_or(Rect::new(rect.x, rect.y, 0, 0));
        let mut cells = Vec::with_capacity(r.w as usize * r.h as usize);
        for y in r.y..r.bottom() {
            for x in r.x..r.right() {
                cells.push(self.get(x, y).unwrap_or(Cell::blank(Color::Black)));
            }
        }
        Snapshot { rect: r, cells }
    }

    /// Restaura una foto.
    pub fn restore(&mut self, snap: &Snapshot) {
        let mut k = 0;
        for y in snap.rect.y..snap.rect.bottom() {
            for x in snap.rect.x..snap.rect.right() {
                if let Some(c) = snap.cells.get(k) {
                    self.set(x, y, *c);
                }
                k += 1;
            }
        }
    }

    /// Celdas que difieren de `other` (mismo tamaño). Si los tamaños
    /// difieren, se devuelve el frame completo (resize = repintar todo).
    pub fn diff(&self, other: &Buffer) -> Vec<DrawOp> {
        if self.w != other.w || self.h != other.h {
            let mut all = Vec::with_capacity(self.w as usize * self.h as usize);
            for y in 0..self.h {
                for x in 0..self.w {
                    all.push(DrawOp {
                        x,
                        y,
                        cell: self.cells[y as usize * self.w as usize + x as usize],
                    });
                }
            }
            return all;
        }
        let mut ops = Vec::new();
        for (i, (a, b)) in self.cells.iter().zip(other.cells.iter()).enumerate() {
            if a != b {
                ops.push(DrawOp {
                    x: (i % self.w as usize) as u16,
                    y: (i / self.w as usize) as u16,
                    cell: *a,
                });
            }
        }
        ops
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_clips_silently() {
        let mut b = Buffer::blank(5, 3, Color::Black);
        b.set(99, 99, Cell::new('X', Color::White, Color::Black));
        assert!(b.get(99, 99).is_none());
        b.set(1, 1, Cell::new('X', Color::White, Color::Black));
        assert_eq!(b.get(1, 1).unwrap().ch, 'X');
    }

    #[test]
    fn text_clips_at_edge() {
        let mut b = Buffer::blank(5, 1, Color::Black);
        b.text(3, 0, "abcdef", Color::White, Color::Black);
        assert_eq!(b.get(3, 0).unwrap().ch, 'a');
        assert_eq!(b.get(4, 0).unwrap().ch, 'b');
    }

    #[test]
    fn fill_rect_respects_bounds() {
        let mut b = Buffer::blank(4, 4, Color::Black);
        b.fill_rect(
            Rect::new(2, 2, 10, 10),
            Cell::new('F', Color::White, Color::Blue),
        );
        assert_eq!(b.get(3, 3).unwrap().bg, Color::Blue);
        assert_eq!(b.get(0, 0).unwrap().bg, Color::Black);
    }

    #[test]
    fn diff_detects_only_changes() {
        let a = Buffer::blank(10, 5, Color::Black);
        let mut b = a.clone();
        b.set(2, 1, Cell::new('X', Color::White, Color::Black));
        b.set(7, 3, Cell::new('Y', Color::White, Color::Black));
        let d = b.diff(&a);
        assert_eq!(d.len(), 2);
        assert_eq!((d[0].x, d[0].y), (2, 1));
        assert_eq!((d[1].x, d[1].y), (7, 3));
    }

    #[test]
    fn diff_on_resize_returns_full_frame() {
        let a = Buffer::blank(4, 4, Color::Black);
        let b = Buffer::blank(5, 5, Color::Black);
        assert_eq!(b.diff(&a).len(), 25);
    }

    #[test]
    fn snapshot_restore_roundtrip() {
        let mut b = Buffer::blank(10, 5, Color::White);
        b.text(1, 1, "SAINT", Color::Black, Color::White);
        let snap = b.snapshot(Rect::new(0, 0, 10, 5));
        b.fill_rect(
            Rect::new(0, 0, 10, 5),
            Cell::new(' ', Color::White, Color::Blue),
        );
        b.restore(&snap);
        assert_eq!(b.get(1, 1).unwrap().ch, 'S');
        assert_eq!(b.get(1, 1).unwrap().bg, Color::White);
    }
}
