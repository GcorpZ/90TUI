//! Sombras estilo VGA/modo texto: translúcidas, tramadas o sólidas.
//!
//! El truco de la época (visible en las capturas de referencia) no era
//! pintar un bloque negro opaco, sino **reutilizar la celda de fondo**:
//! * **Translucent** (estilo PCTools/gestión): conserva el glifo y lo
//!   aplasta a gris fantasma sobre negro — el contenido se adivina
//!   debajo, y eso el ojo lo lee como "sombra" en vez de "barra".
//! * **Stipple** (estilo Turbo): tramado ajedrez 50%: una celda sí, una
//!   no — el punteado clásico del IDE.
//! * **Solid**: bloque opaco (botones de 1px, como el OK/Cancel).
//!
//! Se pinta PRIMERO (debajo) y el cuerpo encima.

use crate::core::{Buffer, Cell, Color, Rect, Theme};

/// Estilo de sombra.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ShadowStyle {
    /// Fantasma gris: conserva glifo, oscurece. Default de ventanas.
    #[default]
    Translucent,
    /// Trama ajedrezada 50%.
    Stipple,
    /// Bloque opaco (botones).
    Solid,
}

/// Sombra clásica con offset (2, 1), estilo translúcido.
pub fn shadow(buf: &mut Buffer, rect: Rect, theme: Theme) {
    shadow_styled(buf, rect, 2, 1, ShadowStyle::Translucent, theme);
}

/// Sombra sólida (botones).
pub fn shadow_solid(buf: &mut Buffer, rect: Rect, theme: Theme) {
    shadow_styled(buf, rect, 2, 1, ShadowStyle::Solid, theme);
}

/// Sombra tramada estilo Turbo.
pub fn shadow_stipple(buf: &mut Buffer, rect: Rect, theme: Theme) {
    shadow_styled(buf, rect, 2, 1, ShadowStyle::Stipple, theme);
}

/// Sombra translúcida con offset arbitrario.
pub fn shadow_offset(buf: &mut Buffer, rect: Rect, dx: u16, dy: u16, theme: Theme) {
    shadow_styled(buf, rect, dx, dy, ShadowStyle::Translucent, theme);
}

/// Sombra con offset y estilo arbitrarios.
pub fn shadow_styled(
    buf: &mut Buffer,
    rect: Rect,
    dx: u16,
    dy: u16,
    style: ShadowStyle,
    theme: Theme,
) {
    if rect.is_empty() {
        return;
    }
    let r = Rect::new(
        rect.x.saturating_add(dx),
        rect.y.saturating_add(dy),
        rect.w,
        rect.h,
    );
    let Some(r) = r.intersect(buf.bounds()) else {
        return;
    };
    for y in r.y..r.bottom() {
        for x in r.x..r.right() {
            let old = buf.get(x, y).unwrap_or(Cell::blank(theme.shadow));
            let cell = match style {
                ShadowStyle::Solid => Cell::new(' ', theme.shadow, theme.shadow),
                ShadowStyle::Translucent => Cell {
                    ch: old.ch,
                    fg: Color::DarkGrey,
                    bg: theme.shadow,
                    bold: false,
                },
                ShadowStyle::Stipple => {
                    if (x + y) % 2 == 0 {
                        Cell {
                            ch: old.ch,
                            fg: Color::DarkGrey,
                            bg: theme.shadow,
                            bold: false,
                        }
                    } else {
                        continue; // celda intacta: el otro 50% del tramado
                    }
                }
            };
            buf.set(x, y, cell);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translucent_keeps_glyph_and_darkens() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(10, 6, Color::White);
        b.text(3, 2, "AB", Color::Black, Color::White);
        shadow(&mut b, Rect::new(1, 1, 4, 2), t);
        // La sombra cae en (3,2): el glifo sobrevive, aplastado a oscuro.
        let c = b.get(3, 2).unwrap();
        assert_eq!(c.ch, 'A');
        assert_eq!(c.bg, t.shadow);
        assert_eq!(c.fg, Color::DarkGrey);
        // El cuerpo original no se toca.
        assert_eq!(b.get(1, 1).unwrap().bg, Color::White);
    }

    #[test]
    fn stipple_is_checkerboard() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(10, 6, Color::White);
        b.text(0, 0, "0123456789", Color::Black, Color::White);
        shadow_stipple(&mut b, Rect::new(0, 0, 8, 1), t);
        // Sombra en (2..10, 1): pares tocadas, impares intactas.
        let mut dark = 0usize;
        let mut kept = 0usize;
        for x in 2..10u16 {
            if b.get(x, 1).unwrap().bg == t.shadow {
                dark += 1;
            } else {
                kept += 1;
            }
        }
        assert!(dark > 0 && kept > 0, "debe alternar");
        assert!(dark.abs_diff(kept) <= 1);
    }

    #[test]
    fn solid_stays_opaque() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(10, 6, Color::White);
        b.text(3, 2, "AB", Color::Black, Color::White);
        shadow_solid(&mut b, Rect::new(1, 1, 4, 2), t);
        assert_eq!(b.get(3, 2).unwrap(), Cell::new(' ', t.shadow, t.shadow));
    }

    #[test]
    fn shadow_at_screen_edge_clips_silently() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(10, 6, Color::White);
        shadow(&mut b, Rect::new(7, 4, 5, 5), t); // sombra (9,5): solo 1 celda visible
        assert_eq!(b.get(9, 5).unwrap().bg, t.shadow);
        assert_eq!(b.get(0, 0).unwrap().bg, Color::White);
    }
}
