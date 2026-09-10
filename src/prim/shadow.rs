//! Sombras estilo VGA/modo texto: translúcidas, tramadas o sólidas.
//!
//! El truco de la época (visible en las capturas de referencia) no era
//! pintar un bloque negro opaco, sino **oscurecer lo que ya había** hacia
//! un neutro oscuro (nunca conservar tinte: el cian NO va a marino).
//! * **Translucent** (default ventanas): conserva el glifo y atenúa por
//!   luminancia con `darken_color` (fg ×0.50, bg ×0.40) más `dim`
//!   ANSI — el contenido se adivina debajo en gris oscuro, y eso
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

/// Aproximación RGB de cada color ANSI-16 (el motor no tiene `Rgb`:
/// la atenuación se calcula en este espacio y se cuantiza de vuelta
/// a neutro). Valores VGA estándar 0-255.
fn rgb_approx(c: Color) -> (f32, f32, f32) {
    use Color::*;
    match c {
        Black => (0.0, 0.0, 0.0),
        Navy => (0.0, 0.0, 128.0),
        Blue => (0.0, 0.0, 255.0),
        Teal => (0.0, 128.0, 128.0),
        Cyan => (0.0, 255.0, 255.0),
        DarkGrey => (128.0, 128.0, 128.0),
        Grey => (192.0, 192.0, 192.0),
        White => (255.0, 255.0, 255.0),
        Yellow => (255.0, 255.0, 0.0),
        Mint => (100.0, 255.0, 170.0),
        Red => (255.0, 0.0, 0.0),
        Green => (0.0, 128.0, 0.0),
    }
}

/// Oscurecimiento por escala de luminancia (nunca conserva tinte).
/// Luminancia Rec.601 `0.299R+0.587G+0.114B` × factor, cuantizada a
/// escala neutra: `>=110 → Grey`, `>=25 → DarkGrey`, si no `Black`.
/// El negro es el suelo. Jamás devuelve `Navy`/`Blue`/`Teal`/`Cyan`
/// (esos eran las franjas azuladas sobre cian).
pub fn darken_color(c: Color, factor: f32) -> Color {
    if c == Color::Black {
        return Color::Black;
    }
    let f = factor.clamp(0.0, 1.0);
    let (r, g, b) = rgb_approx(c);
    let lum = 0.299 * r + 0.587 * g + 0.114 * b;
    let scaled = lum * f;
    if scaled >= 110.0 {
        Color::Grey
    } else if scaled >= 25.0 {
        Color::DarkGrey
    } else {
        Color::Black
    }
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
        // La sombra cae en (3,2): glifo intacto, colores atenuados a neutro.
        let c = b.get(3, 2).unwrap();
        assert_eq!(c.ch, 'A');
        // Atenuación neutra, NO negro sólido ni tinte azul.
        assert_eq!(c.bg, darken_color(Color::White, 0.40));
        assert_ne!(c.bg, t.shadow);
        assert_eq!(c.fg, darken_color(Color::Black, 0.50));
        assert!(!c.bold);
        assert!(c.dim);
        // El cuerpo original no se toca.
        assert_eq!(b.get(1, 1).unwrap().bg, Color::White);
        // Cian brillante → gris neutro, jamás Navy/Teal.
        let mut b2 = Buffer::blank(10, 6, Color::Cyan);
        shadow(&mut b2, Rect::new(1, 1, 4, 2), t);
        let c2 = b2.get(3, 2).unwrap();
        assert_eq!(c2.bg, darken_color(Color::Cyan, 0.40));
        assert_eq!(c2.bg, Color::DarkGrey);
        assert_ne!(c2.bg, Color::Navy);
        assert_ne!(c2.bg, t.shadow);
    }

    #[test]
    fn darken_color_is_neutral_luminance() {
        // Blancos: fg 0.50 → Grey, bg 0.40 → DarkGrey (distintos).
        assert_eq!(darken_color(Color::White, 0.50), Color::Grey);
        assert_eq!(darken_color(Color::White, 0.40), Color::DarkGrey);
        // Colores brillantes → neutro, NUNCA tinte (el bug era Cyan→Navy).
        assert_eq!(darken_color(Color::Cyan, 0.50), Color::DarkGrey);
        assert_eq!(darken_color(Color::Cyan, 0.40), Color::DarkGrey);
        assert_eq!(darken_color(Color::Yellow, 0.50), Color::Grey);
        assert_eq!(darken_color(Color::Red, 0.50), Color::DarkGrey);
        assert_eq!(darken_color(Color::Green, 0.50), Color::DarkGrey);
        assert_eq!(darken_color(Color::Grey, 0.50), Color::DarkGrey);
        // Grises medios se quedan en DarkGrey (no colapsan a negro:
        // ese era el bloque opaco sobre ventanas).
        assert_eq!(darken_color(Color::Grey, 0.40), Color::DarkGrey);
        assert_eq!(darken_color(Color::DarkGrey, 0.50), Color::DarkGrey);
        assert_eq!(darken_color(Color::DarkGrey, 0.40), Color::DarkGrey);
        // Oscuros ya mínimos → suelo negro.
        assert_eq!(darken_color(Color::Black, 0.40), Color::Black);
        assert_eq!(darken_color(Color::Navy, 0.50), Color::Black);
        assert_eq!(darken_color(Color::Blue, 0.50), Color::Black);
        // Invariante global: jamás devuelve tinte frío.
        for c in [
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
        ] {
            let out = darken_color(c, 0.40);
            assert!(
                !matches!(out, Color::Navy | Color::Blue | Color::Teal | Color::Cyan),
                "tinte en {c:?} → {out:?}"
            );
        }
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
