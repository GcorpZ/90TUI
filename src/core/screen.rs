//! Pila de pantallas (`Savescreen()`/`Restscreen()`) + doble buffer.
//!
//! En Clipper cada ventana guardaba lo que había debajo y lo restauraba
//! al cerrarse, permitiendo N modales apilados (en las capturas de referencia
//! se ven hasta 3 niveles: base + diálogo + progreso). Aquí igual:
//! `savescreen(rect)` devuelve un id LIFO; `restscreen(id)` restaura.
//! `Screen` además guarda `back` (donde dibujan los widgets) y `front`
//! (lo último enviado a la terminal) para el `diff`.

use super::buffer::{Buffer, DrawOp, Snapshot};
use super::rect::Rect;
use super::theme::Theme;

/// Pila LIFO de fotos. El `id` es la posición; solo se puede restaurar
/// la cima (igual que cerrar ventanas en orden inverso).
#[derive(Debug, Default)]
pub struct ScreenStack {
    stack: Vec<Snapshot>,
}

impl ScreenStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Fotografía `rect` del `buf` y la apila. Devuelve el id.
    pub fn save(&mut self, buf: &Buffer, rect: Rect) -> usize {
        self.stack.push(buf.snapshot(rect));
        self.stack.len() - 1
    }

    /// Restaura el snapshot `id` sobre `buf`. Solo la cima (`id ==
    /// depth-1`); en otro caso devuelve `false` sin tocar nada.
    pub fn restore(&mut self, buf: &mut Buffer, id: usize) -> bool {
        if id + 1 != self.stack.len() {
            return false;
        }
        let snap = self.stack.pop().expect("depth checked");
        buf.restore(&snap);
        true
    }

    /// Atajo: restaura la cima.
    pub fn pop(&mut self, buf: &mut Buffer) -> bool {
        if self.stack.is_empty() {
            return false;
        }
        self.restore(buf, self.stack.len() - 1)
    }

    /// Vacía sin restaurar (ej. tras un `resize` que invalida fotos).
    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

/// Pantalla completa: doble buffer + pila + tema.
#[derive(Debug)]
pub struct Screen {
    pub back: Buffer,
    pub front: Buffer,
    pub stack: ScreenStack,
    pub theme: Theme,
}

impl Screen {
    pub fn new(w: u16, h: u16, theme: Theme) -> Self {
        let bg = theme.desktop;
        Self {
            back: Buffer::blank(w.max(1), h.max(1), bg),
            // Front "desconocido" (0x0): el primer `present_ops` devuelve
            // el frame completo, como el primer `memcpy` a 0xB800.
            front: Buffer::blank(0, 0, bg),
            stack: ScreenStack::new(),
            theme,
        }
    }

    pub fn size(&self) -> (u16, u16) {
        (self.back.width(), self.back.height())
    }

    pub fn bounds(&self) -> Rect {
        self.back.bounds()
    }

    /// Acceso al back-buffer para dibujar el frame.
    pub fn frame(&mut self) -> &mut Buffer {
        &mut self.back
    }

    /// Guarda la zona `rect` del back-buffer. Ver `ScreenStack::save`.
    pub fn savescreen(&mut self, rect: Rect) -> usize {
        self.stack.save(&self.back, rect)
    }

    /// Restaura el snapshot `id` sobre el back-buffer.
    pub fn restscreen(&mut self, id: usize) -> bool {
        let (back, stack) = (&mut self.back, &mut self.stack);
        stack.restore(back, id)
    }

    /// Calcula el diff back-vs-front y promueve back a front.
    /// El resultado viaja al `Backend::present`.
    pub fn present_ops(&mut self) -> Vec<DrawOp> {
        let ops = self.back.diff(&self.front);
        self.front = self.back.clone();
        ops
    }

    /// Resize conservando la esquina superior-izquierda común.
    /// Invalida la pila (las fotos viejas ya no encajan) y fuerza
    /// repintado total en el próximo `present_ops`.
    pub fn resize(&mut self, w: u16, h: u16) {
        let (w, h) = (w.max(1), h.max(1));
        let old = std::mem::replace(&mut self.back, Buffer::blank(w, h, self.theme.desktop));
        // Copia solapada.
        let cw = old.width().min(w);
        let ch = old.height().min(h);
        for y in 0..ch {
            for x in 0..cw {
                if let Some(c) = old.get(x, y) {
                    self.back.set(x, y, c);
                }
            }
        }
        self.front = Buffer::blank(0, 0, self.theme.desktop);
        self.stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::super::cell::Cell;
    use super::super::color::Color;
    use super::*;

    #[test]
    fn save_restore_roundtrip_over_back() {
        let mut s = Screen::new(20, 10, Theme::clipper());
        s.frame().text(2, 2, "FONDO", Color::Black, Color::White);
        let id = s.savescreen(Rect::new(0, 0, 20, 10));
        assert_eq!(id, 0);
        s.frame().fill_rect(
            Rect::new(0, 0, 20, 10),
            Cell::new('X', Color::White, Color::Blue),
        );
        assert!(s.restscreen(id));
        assert_eq!(s.frame().get(2, 2).unwrap().ch, 'F');
    }

    #[test]
    fn restore_is_strict_lifo() {
        let mut s = Screen::new(20, 10, Theme::clipper());
        let a = s.savescreen(Rect::new(0, 0, 5, 5));
        let _b = s.savescreen(Rect::new(5, 5, 5, 5));
        // Intentar restaurar `a` sin haber cerrado `b` falla.
        assert!(!s.restscreen(a));
        assert!(s.stack.pop(&mut s.back));
        assert!(s.restscreen(a));
    }

    #[test]
    fn present_ops_then_noop() {
        let mut s = Screen::new(10, 5, Theme::clipper());
        let ops = s.present_ops();
        assert_eq!(ops.len(), 10 * 5); // primer frame: todo es nuevo
        let ops2 = s.present_ops();
        assert!(ops2.is_empty()); // sin cambios: silencio (sin flicker)
    }

    #[test]
    fn resize_keeps_top_left_and_clears_stack() {
        let mut s = Screen::new(10, 5, Theme::clipper());
        s.frame().text(0, 0, "AB", Color::Black, Color::White);
        s.savescreen(Rect::new(0, 0, 4, 4));
        s.resize(20, 10);
        assert_eq!(s.frame().get(0, 0).unwrap().ch, 'A');
        assert_eq!(s.size(), (20, 10));
        assert!(s.stack.is_empty());
    }
}
