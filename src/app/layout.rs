//! Layout responsive del shell: barras + área de trabajo.
//!
//! Geometría fija y predecible en 80x25, degradación sin pánico en
//! terminales enanas (el área puede quedar vacía, nunca negativa).

use crate::core::Rect;

/// Zonas del shell clásico.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DesktopLayout {
    pub top: Rect,
    pub menu: Rect,
    pub work: Rect,
    pub fkeys: Rect,
    pub status: Rect,
}

/// Reparte `screen`: top(1) + menu(1) + work(resto-2) + fkeys(1) + status(1).
/// `menu`/`work`/`fkeys` llevan inset lateral de 1.
pub fn desktop(screen: Rect) -> DesktopLayout {
    let (x, y, w, h) = (screen.x, screen.y, screen.w, screen.h);
    // Insets con clamp: en pantallas enanas las zonas quedan vacías pero
    // siempre dentro (nunca coordenadas fuera ni anchos negativos).
    let ix = x.saturating_add(1).min(x.saturating_add(w));
    let iw = w.saturating_sub(2);
    let top = Rect::new(x, y, w, h.min(1));
    let menu = if h > 1 {
        Rect::new(ix, y.saturating_add(1), iw, 1)
    } else {
        Rect::new(x, y, 0, 0)
    };
    let status = if h > 1 {
        Rect::new(x, y.saturating_add(h).saturating_sub(1), w, 1)
    } else {
        Rect::new(x, y, 0, 0)
    };
    let fkeys = if h > 2 {
        Rect::new(ix, y.saturating_add(h).saturating_sub(2), iw, 1)
    } else {
        Rect::new(x, y, 0, 0)
    };
    let wy = y.saturating_add(2).min(y.saturating_add(h));
    let work = Rect::new(ix, wy, iw, h.saturating_sub(4));
    DesktopLayout {
        top,
        menu,
        work,
        fkeys,
        status,
    }
}

/// Ventana centrada (atajo a `Rect::centered_in`).
pub fn centered(w: u16, h: u16, screen: Rect) -> Rect {
    Rect::centered_in(w, h, screen)
}

/// Popup anclado (atajo a `Rect::popup_at`).
pub fn popup(x: u16, y: u16, w: u16, h: u16, screen: Rect) -> Rect {
    Rect::popup_at(x, y, w, h, screen)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_80x25() {
        let d = desktop(Rect::new(0, 0, 80, 25));
        assert_eq!(d.top, Rect::new(0, 0, 80, 1));
        assert_eq!(d.menu, Rect::new(1, 1, 78, 1));
        assert_eq!(d.work, Rect::new(1, 2, 78, 21));
        assert_eq!(d.fkeys, Rect::new(1, 23, 78, 1));
        assert_eq!(d.status, Rect::new(0, 24, 80, 1));
    }

    #[test]
    fn tiny_screens_dont_panic_or_overlap() {
        for (w, h) in [(40, 10), (20, 5), (10, 3), (5, 2), (1, 1)] {
            let d = desktop(Rect::new(0, 0, w, h));
            for r in [d.top, d.menu, d.work, d.fkeys, d.status] {
                assert!(r.right() <= w, "w={w} h={h} r={r:?}");
                assert!(r.bottom() <= h, "w={w} h={h} r={r:?}");
            }
        }
    }
}
