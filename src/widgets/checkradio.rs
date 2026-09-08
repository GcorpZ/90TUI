//! Casillas y radios integrales: `☐/☑`, `○/◉`.
//!
//! Por defecto (moderno) se usan glifos Unicode integrales sin marcos:
//! checkbox `\u{2610}`/`\u{2611}`, radio `\u{25cb}`/`\u{25c9}`.
//! Requiere fuente con esos glifos (JetBrainsMono Nerd Font, Hack NF,
//! DejaVu, Cascadia o Consolas). `GlyphSet::ascii()` es el fallback
//! `[X]`/`[ ]`, `(*)`/`( )` para terminales sin Unicode.
//! `&` marca hotkey como siempre.

use crossterm::event::KeyCode;

use crate::core::{Attr, Buffer};
use crate::prim::{draw_hot_label, hot_key_of, visible_len, HotAttrs};

/// Una casilla con etiqueta (`&` = hotkey).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckItem {
    pub label: String,
    pub checked: bool,
}

/// Set de glifos: integrales Unicode o ASCII con marcos.
/// Moderno (default): `\u{2610}`/`\u{2611}` y `\u{25cb}`/`\u{25c9}`.
/// ASCII: `[X]`/`[ ]`, `(*)`/`( )`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlyphSet {
    pub check_on: char,
    pub check_off: char,
    pub radio_on: char,
    pub radio_off: char,
    /// Si `true`, envuelve con `[ ]` / `( )`; si `false`, glifo integral + espacio.
    pub framed: bool,
}

impl GlyphSet {
    /// Integrales: `\u{2611}` / `\u{2610}`, `\u{25c9}` / `\u{25cb}`.
    pub fn modern() -> Self {
        Self {
            check_on: '\u{2611}',
            check_off: '\u{2610}',
            radio_on: '\u{25c9}',
            radio_off: '\u{25cb}',
            framed: false,
        }
    }

    /// Solo ASCII: `[X]`/`[ ]`, `(*)`/`( )`.
    pub fn ascii() -> Self {
        Self {
            check_on: 'X',
            check_off: ' ',
            radio_on: '*',
            radio_off: ' ',
            framed: true,
        }
    }
}

impl Default for GlyphSet {
    fn default() -> Self {
        Self::modern()
    }
}

/// Estilo completo de casillas/radios: colores + glifos + parámetros globales.
/// `border_color` tiñe los marcos en modo ASCII (`[ ]`/`( )`); en modo
/// integral no hay marco que pintar. `has_shadow` proyecta sombra mezclada
/// de 1 celda (derecha + abajo) tras el glifo.
#[derive(Clone, Copy, Debug)]
pub struct CheckStyle {
    pub attrs: HotAttrs,
    pub glyphs: GlyphSet,
    pub border_color: Option<crate::core::Color>,
    pub has_shadow: bool,
}

impl CheckStyle {
    pub fn new(attrs: HotAttrs, glyphs: GlyphSet) -> Self {
        Self {
            attrs,
            glyphs,
            border_color: None,
            has_shadow: false,
        }
    }

    /// Estilo desde los 4 parámetros globales.
    pub fn styled(
        attrs: HotAttrs,
        glyphs: GlyphSet,
        border_color: Option<crate::core::Color>,
        has_shadow: bool,
    ) -> Self {
        Self {
            attrs,
            glyphs,
            border_color,
            has_shadow,
        }
    }
}

impl CheckItem {
    pub fn new(label: &str, checked: bool) -> Self {
        Self {
            label: label.to_string(),
            checked,
        }
    }
}

/// Dibuja `☐ etiqueta` / `☑ etiqueta` (o `[X]` en ASCII según `style`).
/// `focused` pone la etiqueta en negrita. Devuelve el ancho ocupado.
pub fn checkbox_draw(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    item: &CheckItem,
    focused: bool,
    style: CheckStyle,
) -> u16 {
    let g = if item.checked {
        style.glyphs.check_on
    } else {
        style.glyphs.check_off
    };
    let glyph = if style.glyphs.framed {
        format!("[{g}] ")
    } else {
        format!("{g} ")
    };
    let gw = if style.glyphs.framed && style.border_color.is_some() {
        draw_framed_glyph(buf, x, y, &glyph, style.attrs.base, style.border_color)
    } else {
        draw_box_glyph(buf, x, y, &glyph, style.attrs.base)
    };
    let label_attr = Attr {
        bold: focused,
        ..style.attrs.base
    };
    let hot_attr = Attr {
        bold: focused,
        ..style.attrs.hot
    };
    let total = gw + draw_hot_label(buf, x + gw, y, &item.label, label_attr, hot_attr);
    if style.has_shadow {
        row_shadow(buf, x, y, total);
    }
    total
}

/// Dibuja `◉ etiqueta` si `selected` (círculo lleno), `○` si no.
/// En ASCII: `(*)` / `( )`.
pub fn radio_draw(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    label: &str,
    selected: bool,
    focused: bool,
    style: CheckStyle,
) -> u16 {
    let g = if selected {
        style.glyphs.radio_on
    } else {
        style.glyphs.radio_off
    };
    let glyph = if style.glyphs.framed {
        format!("({g}) ")
    } else {
        format!("{g} ")
    };
    let gw = if style.glyphs.framed && style.border_color.is_some() {
        draw_framed_glyph(buf, x, y, &glyph, style.attrs.base, style.border_color)
    } else {
        draw_box_glyph(buf, x, y, &glyph, style.attrs.base)
    };
    let label_attr = Attr {
        bold: focused || selected,
        ..style.attrs.base
    };
    let hot_attr = Attr {
        bold: focused || selected,
        ..style.attrs.hot
    };
    let total = gw + draw_hot_label(buf, x + gw, y, label, label_attr, hot_attr);
    if style.has_shadow {
        row_shadow(buf, x, y, total);
    }
    total
}

fn draw_box_glyph(buf: &mut Buffer, x: u16, y: u16, glyph: &str, attr: Attr) -> u16 {
    crate::prim::draw_text(buf, x, y, glyph, attr)
}

/// Marcos ASCII (`[ ]`/`( )`) con los bordes en `border_color`.
fn draw_framed_glyph(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    glyph: &str,
    attr: Attr,
    border: Option<crate::core::Color>,
) -> u16 {
    let Some(border) = border else {
        return draw_box_glyph(buf, x, y, glyph, attr);
    };
    let battr = Attr { fg: border, ..attr };
    // Formato `"[X] "` / `"(•) "`: abre y cierra con borde, resto en base.
    let chars: Vec<char> = glyph.chars().collect();
    if chars.len() >= 4 {
        let mut cx = x;
        crate::prim::draw_text(buf, cx, y, &chars[0].to_string(), battr);
        cx = cx.saturating_add(1);
        for ch in &chars[1..chars.len() - 2] {
            crate::prim::draw_text(buf, cx, y, &ch.to_string(), attr);
            cx = cx.saturating_add(1);
        }
        let close = chars[chars.len() - 2];
        crate::prim::draw_text(buf, cx, y, &close.to_string(), battr);
        cx = cx.saturating_add(1);
        crate::prim::draw_text(buf, cx, y, " ", attr);
        cx = cx.saturating_add(1);
        return cx.saturating_sub(x);
    }
    draw_box_glyph(buf, x, y, glyph, attr)
}

/// Sombra CUA de una fila de controles (`H = 1`): lateral + abajo + esquina.
fn row_shadow(buf: &mut Buffer, x: u16, y: u16, w: u16) {
    use crate::core::{Cell, Color};
    let mut blend = |sx: u16, sy: u16| {
        if !buf.in_bounds(sx, sy) {
            return;
        }
        let old = buf.get(sx, sy).unwrap_or(Cell::blank(Color::Black));
        buf.set(
            sx,
            sy,
            Cell {
                ch: old.ch,
                fg: old.fg,
                bg: Color::Black,
                bold: false,
                dim: true,
            },
        );
    };
    blend(x.saturating_add(w), y);
    for i in 0..w {
        blend(x.saturating_add(i).saturating_add(1), y.saturating_add(1));
    }
    blend(x.saturating_add(w), y.saturating_add(1));
}

/// Resultado de tecla en un grupo de casillas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckNav {
    Stay,
    Move(usize),
    Toggled(usize),
}

/// ↑↓ mueven foco, Espacio/Enter alterna, letra = hotkey directa.
pub fn check_key(items: &[CheckItem], focus: usize, code: KeyCode) -> CheckNav {
    use CheckNav::*;
    let n = items.len();
    if n == 0 {
        return Stay;
    }
    let f = focus.min(n - 1);
    match code {
        KeyCode::Up | KeyCode::Left => Move((f + n - 1) % n),
        KeyCode::Down | KeyCode::Right | KeyCode::Tab => Move((f + 1) % n),
        KeyCode::Home => Move(0),
        KeyCode::End => Move(n - 1),
        KeyCode::Char(' ') | KeyCode::Enter => Toggled(f),
        KeyCode::Char(c) => {
            let lc = c.to_ascii_lowercase();
            for (i, it) in items.iter().enumerate() {
                if hot_key_of(&it.label).map(|h| h.to_ascii_lowercase()) == Some(lc) {
                    return Toggled(i);
                }
            }
            Stay
        }
        _ => Stay,
    }
}

/// Resultado de tecla en un grupo de radios.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadioNav {
    Stay,
    Select(usize),
}

/// Las flechas eligen directamente (comportamiento clásico de radio).
pub fn radio_key(n: usize, selected: usize, code: KeyCode) -> RadioNav {
    use RadioNav::*;
    if n == 0 {
        return Stay;
    }
    let s = selected.min(n - 1);
    match code {
        KeyCode::Up | KeyCode::Left => Select((s + n - 1) % n),
        KeyCode::Down | KeyCode::Right | KeyCode::Tab => Select((s + 1) % n),
        KeyCode::Home => Select(0),
        KeyCode::End => Select(n - 1),
        KeyCode::Enter | KeyCode::Char(' ') => Select(s),
        KeyCode::Char(c) => {
            // La letra salta por hotkey si el llamante resuelve etiquetas;
            // aquí sin etiquetas: Stay (el showroom usa números).
            let _ = c;
            Stay
        }
        _ => Stay,
    }
}

/// Ancho visible de una fila de casilla con marcos (cota superior, para layout).
pub fn check_width(item: &CheckItem) -> u16 {
    4 + visible_len(&item.label)
}

/// Ancho real según el set de glifos (2 + etiqueta si integral, 4 + si ASCII).
pub fn check_width_styled(item: &CheckItem, glyphs: GlyphSet) -> u16 {
    let prefix = if glyphs.framed { 4 } else { 2 };
    prefix + visible_len(&item.label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Buffer, Color, Theme};

    fn base() -> (Attr, Attr) {
        let t = Theme::clipper();
        (t.dialog_attr(), Attr::bold(Color::Red, t.dialog))
    }

    #[test]
    fn draws_box_and_label() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 5, t.dialog);
        let (base, hot) = base();
        let style = CheckStyle::new(HotAttrs { base, hot }, GlyphSet::modern());
        let w = checkbox_draw(
            &mut b,
            2,
            1,
            &CheckItem::new("&Compresión", true),
            false,
            style,
        );
        // Integrales sin marcos: `☑` + espacio, hotkey en x=4.
        assert_eq!(b.get(2, 1).unwrap().ch, '\u{2611}');
        assert_eq!(b.get(4, 1).unwrap().fg, Color::Red); // hotkey C
        assert!(w >= 2 + visible_len("Compresión"));
        checkbox_draw(&mut b, 2, 0, &CheckItem::new("Off", false), false, style);
        assert_eq!(b.get(2, 0).unwrap().ch, '\u{2610}');
        let w2 = radio_draw(&mut b, 2, 2, "Rápido", true, false, style);
        assert_eq!(b.get(2, 2).unwrap().ch, '\u{25c9}');
        assert!(w2 >= 2);
        radio_draw(&mut b, 2, 3, "Lento", false, false, style);
        assert_eq!(b.get(2, 3).unwrap().ch, '\u{25cb}');
        // Fallback ASCII con marcos.
        let ascii = CheckStyle::new(HotAttrs { base, hot }, GlyphSet::ascii());
        checkbox_draw(&mut b, 10, 3, &CheckItem::new("X", true), false, ascii);
        assert_eq!(b.get(10, 3).unwrap().ch, '[');
        assert_eq!(b.get(11, 3).unwrap().ch, 'X');
        radio_draw(&mut b, 10, 4, "R", true, false, ascii);
        assert_eq!(b.get(10, 4).unwrap().ch, '(');
        assert_eq!(b.get(11, 4).unwrap().ch, '*');
        // Anchos por estilo.
        assert_eq!(
            check_width_styled(&CheckItem::new("AB", false), GlyphSet::modern()),
            2 + 2
        );
        assert_eq!(
            check_width_styled(&CheckItem::new("AB", false), GlyphSet::ascii()),
            4 + 2
        );
    }

    #[test]
    fn check_nav_moves_and_toggles() {
        let items = vec![CheckItem::new("A", false), CheckItem::new("B", true)];
        assert_eq!(check_key(&items, 0, KeyCode::Down), CheckNav::Move(1));
        assert_eq!(check_key(&items, 1, KeyCode::Down), CheckNav::Move(0));
        assert_eq!(
            check_key(&items, 0, KeyCode::Char(' ')),
            CheckNav::Toggled(0)
        );
        assert_eq!(
            check_key(&items, 0, KeyCode::Char('b')),
            CheckNav::Toggled(1)
        );
    }

    #[test]
    fn radio_nav_selects_directly() {
        assert_eq!(radio_key(3, 1, KeyCode::Up), RadioNav::Select(0));
        assert_eq!(radio_key(3, 0, KeyCode::Up), RadioNav::Select(2));
        assert_eq!(radio_key(3, 2, KeyCode::Down), RadioNav::Select(0));
        assert_eq!(radio_key(0, 0, KeyCode::Down), RadioNav::Stay);
    }

    #[test]
    fn border_tints_ascii_frames_and_shadow_blends() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 8, t.dialog);
        let (base, hot) = base();
        let style = CheckStyle::styled(
            HotAttrs { base, hot },
            GlyphSet::ascii(),
            Some(Color::Yellow),
            true,
        );
        let w = checkbox_draw(&mut b, 2, 1, &CheckItem::new("X", true), false, style);
        // Marcos en borde amarillo, interior en base.
        assert_eq!(b.get(2, 1).unwrap().ch, '[');
        assert_eq!(b.get(2, 1).unwrap().fg, Color::Yellow);
        assert_eq!(b.get(3, 1).unwrap().ch, 'X');
        assert_eq!(b.get(4, 1).unwrap().ch, ']');
        assert_eq!(b.get(4, 1).unwrap().fg, Color::Yellow);
        // Sombra CUA de la fila: lateral + abajo + esquina.
        assert_eq!(b.get(2 + w, 1).unwrap().bg, Color::Black);
        assert_eq!(b.get(3, 2).unwrap().bg, Color::Black);
        assert_eq!(b.get(2 + w, 2).unwrap().bg, Color::Black);
        // Sin sombra por defecto.
        let plain = CheckStyle::new(HotAttrs { base, hot }, GlyphSet::modern());
        let mut b2 = Buffer::blank(40, 8, t.dialog);
        let w2 = checkbox_draw(&mut b2, 2, 1, &CheckItem::new("X", true), false, plain);
        assert_eq!(b2.get(2 + w2, 1).unwrap().bg, t.dialog);
    }
}
