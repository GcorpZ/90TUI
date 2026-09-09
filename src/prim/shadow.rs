//! Sombras estilo VGA/modo texto: translúcidas, tramadas o sólidas.
//!
//! El truco de la época (visible en las capturas de referencia) no era
//! pintar un bloque negro opaco, sino **oscurecer lo que ya había**:
//! * **Translucent** (estilo PCTools/gestión/Norton): conserva el glifo y
//!   atenúa sus colores con `darken_color` (fg ×0.50, bg ×0.40) más `dim`
//!   ANSI — el contenido se adivina debajo con su tinte original, y eso
//!   el ojo lo lee como "sombra" en vez de "barra". El fondo negro sólido
//!   (`theme.shadow`) solo queda para `Solid`/`Stipple`.
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

/// Un paso de oscurecimiento sobre la paleta fija de 12 colores ANSI:
/// brillantes → su equivalente oscuro; oscuros → el mínimo de su tinte
/// (o negro si ya es mínimo); blanco/gris claro → gris oscuro.
/// (Nota: el motor es ANSI-16 sin `Rgb`/`AnsiValue`, así que la
/// atenuación es por tabla discreta en vez de multiplicar canales.)
fn darken_step(c: Color) -> Color {
    use Color::*;
    match c {
        Black => Black,
        Navy => Black,
        Blue => Navy,
        Teal => Navy,
        Cyan => Teal,
        DarkGrey => Black,
        Grey => DarkGrey,
        White => Grey,
        Yellow => Green,
        Mint => Green,
        Red => Black,
        Green => Black,
    }
}

/// Oscurecimiento cromático natural (estilo Norton/Clipper).
/// `factor >= 0.5` baja 1 escalón, `< 0.5` baja 2; el negro es el suelo
/// (nunca se sale de la paleta). El tinte se conserva siempre que la
/// paleta tenga un escalón más profundo para él.
pub fn darken_color(c: Color, factor: f32) -> Color {
    let steps = if factor < 0.5 { 2 } else { 1 };
    let mut out = c;
    for _ in 0..steps {
        out = darken_step(out);
    }
    out
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
                    fg: darken_color(old.fg, 0.50),
                    bg: darken_color(old.bg, 0.40),
                    bold: false,
                    dim: true,
                },
                ShadowStyle::Stipple => {
                    if (x + y) % 2 == 0 {
                        Cell {
                            ch: old.ch,
                            fg: old.fg,
                            bg: theme.shadow,
                            bold: false,
                            dim: true,
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
    use crate::core::Color;

    #[test]
    fn translucent_keeps_glyph_and_darkens() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(10, 6, Color::White);
        b.text(3, 2, "AB", Color::Black, Color::White);
        shadow(&mut b, Rect::new(1, 1, 4, 2), t);
        // La sombra cae en (3,2): glifo intacto, colores atenuados.
        let c = b.get(3, 2).unwrap();
        assert_eq!(c.ch, 'A');
        // Oscurecimiento cromático, NO negro sólido.
        assert_eq!(c.bg, darken_color(Color::White, 0.40));
        assert_ne!(c.bg, t.shadow);
        assert_eq!(c.fg, darken_color(Color::Black, 0.50));
        assert!(!c.bold);
        assert!(c.dim);
        // El cuerpo original no se toca.
        assert_eq!(b.get(1, 1).unwrap().bg, Color::White);
        // Cian brillante → tinte frío profundo, tampoco negro.
        let mut b2 = Buffer::blank(10, 6, Color::Cyan);
        shadow(&mut b2, Rect::new(1, 1, 4, 2), t);
        let c2 = b2.get(3, 2).unwrap();
        assert_eq!(c2.bg, darken_color(Color::Cyan, 0.40));
        assert_ne!(c2.bg, t.shadow);
    }

    #[test]
    fn darken_color_steps_bright_to_dark() {
        // factor >= 0.5 = 1 escalón, < 0.5 = 2 escalones.
        assert_eq!(darken_color(Color::White, 0.50), Color::Grey);
        assert_eq!(darken_color(Color::White, 0.40), Color::DarkGrey);
        assert_eq!(darken_color(Color::Cyan, 0.50), Color::Teal);
        assert_eq!(darken_color(Color::Cyan, 0.40), Color::Navy);
        assert_eq!(darken_color(Color::Blue, 0.50), Color::Navy);
        assert_eq!(darken_color(Color::Yellow, 0.50), Color::Green);
        // El negro es el suelo: nunca se sale de la paleta.
        assert_eq!(darken_color(Color::Black, 0.40), Color::Black);
        assert_eq!(darken_color(Color::Red, 0.50), Color::Black);
        assert_eq!(darken_color(Color::Green, 0.40), Color::Black);
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
        assert_eq!(b.get(9, 5).unwrap().bg, darken_color(Color::White, 0.40));
        assert_eq!(b.get(0, 0).unwrap().bg, Color::White);
    }
}
