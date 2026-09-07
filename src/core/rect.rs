//! Rectángulos de pantalla y layout responsive.
//!
//! Toda la librería posiciona con `Rect` en coordenadas de celda (u16,
//! como crossterm). Nada de píxeles. `clamp_in` garantiza que ningún
//! popup se salga de la terminal aunque mida 80x25 o 200x60.

/// Rectángulo inclusivo-exclusivo: `[x, x+w) x [y, y+h)`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }

    pub fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }

    pub fn right(self) -> u16 {
        self.x.saturating_add(self.w)
    }

    pub fn bottom(self) -> u16 {
        self.y.saturating_add(self.h)
    }

    pub fn contains(self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Intersección con otro rect (para recorte). `None` si no se tocan.
    pub fn intersect(self, other: Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let r = self.right().min(other.right());
        let b = self.bottom().min(other.bottom());
        if r > x && b > y {
            Some(Rect {
                x,
                y,
                w: r - x,
                h: b - y,
            })
        } else {
            None
        }
    }

    /// Encoge por `n` celdas por lado (marco interior). Satura a 0.
    pub fn shrink(self, n: u16) -> Rect {
        let n2 = n.saturating_mul(2);
        Rect {
            x: self.x.saturating_add(n),
            y: self.y.saturating_add(n),
            w: self.w.saturating_sub(n2),
            h: self.h.saturating_sub(n2),
        }
    }

    /// Mueve+encoge `self` para que quepa dentro de `bounds`.
    /// Si es más grande que `bounds`, lo recorta al tamaño de `bounds`.
    pub fn clamp_in(self, bounds: Rect) -> Rect {
        if self.is_empty() || bounds.is_empty() {
            return Rect::new(bounds.x, bounds.y, 0, 0);
        }
        let w = self.w.min(bounds.w);
        let h = self.h.min(bounds.h);
        let max_x = bounds.x.saturating_add(bounds.w.saturating_sub(w));
        let max_y = bounds.y.saturating_add(bounds.h.saturating_sub(h));
        Rect {
            x: self.x.clamp(bounds.x, max_x),
            y: self.y.clamp(bounds.y, max_y),
            w,
            h,
        }
    }

    /// Rect de `w x h` centrado en `bounds` (con clamp de seguridad).
    pub fn centered_in(w: u16, h: u16, bounds: Rect) -> Rect {
        if bounds.is_empty() {
            return Rect::new(0, 0, 0, 0);
        }
        let w = w.min(bounds.w);
        let h = h.min(bounds.h);
        Rect {
            x: bounds.x + (bounds.w.saturating_sub(w)) / 2,
            y: bounds.y + (bounds.h.saturating_sub(h)) / 2,
            w,
            h,
        }
    }

    /// Popup anclado bajo un punto (ej. item de menubar): intenta abrir
    /// hacia abajo-derecha y lo mete en pantalla con `clamp_in`.
    pub fn popup_at(ax: u16, ay: u16, w: u16, h: u16, screen: Rect) -> Rect {
        Rect::new(ax, ay.saturating_add(1), w, h).clamp_in(screen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect_clips() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        assert_eq!(a.intersect(b), Some(Rect::new(5, 5, 5, 5)));
        assert_eq!(a.intersect(Rect::new(20, 20, 2, 2)), None);
    }

    #[test]
    fn centered_in_math() {
        let screen = Rect::new(0, 0, 80, 25);
        let r = Rect::centered_in(20, 10, screen);
        assert_eq!(r, Rect::new(30, 7, 20, 10));
    }

    #[test]
    fn clamp_keeps_popup_on_screen() {
        // Ancla pegada al borde derecho-inferior de 80x25.
        let screen = Rect::new(0, 0, 80, 25);
        let p = Rect::popup_at(70, 23, 30, 10, screen);
        assert!(p.right() <= 80 && p.bottom() <= 25);
        assert_eq!((p.w, p.h), (30, 10)); // cupo, solo se movió
    }

    #[test]
    fn oversize_popup_shrinks_to_screen() {
        let screen = Rect::new(0, 0, 80, 25);
        let p = Rect::new(0, 0, 200, 60).clamp_in(screen);
        assert_eq!(p, Rect::new(0, 0, 80, 25));
    }
}
