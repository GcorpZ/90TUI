//! Barra de teclas de función: plana, con estilo o apilada estilo PCTools.
//!
//! * `fkey_bar`: 1 fila, ` F2 Grabar ` teal (clásica de la lib).
//! * `fkey_bar_styled`: 1 fila con colores a gusto (`FKeyStyle`).
//! * `fkey_bar_stacked`: 2 filas con la **F sobre el número**,
//!   alineados verticalmente como en la referencia:
//!   `F  F   F` / `1 Help  2 Qview  10 Menu`.
//!
//! Qué hace cada tecla (punto c) lo decide la app con `app::match_fkey`
//! (mapea `F(n)`/`Esc` → índice); la barra solo pinta.

use crate::core::{Attr, Buffer, Color, Theme};
use crate::prim::{draw_text, visible_len};

/// Colores de la botonera, por partes:
/// a) cantidad = `keys.len()`, b) etiquetas = `keys[i].1`,
/// d) color Fx = `key_*`, e) color etiqueta = `label_*`.
#[derive(Clone, Copy, Debug)]
pub struct FKeyStyle {
    pub key_fg: Color,
    pub key_bg: Color,
    pub label_fg: Color,
    pub label_bg: Color,
}

impl FKeyStyle {
    /// Estilo clásico de la lib (bloque teal + texto blanco).
    pub fn classic(theme: Theme) -> Self {
        Self {
            key_fg: theme.button_fg,
            key_bg: theme.button_bg,
            label_fg: theme.button_fg,
            label_bg: theme.button_bg,
        }
    }

    /// Estilo referencia: Fx amarilla + etiqueta blanca sobre el fondo dado.
    pub fn highlight(fx: Color, label: Color, bg: Color) -> Self {
        Self {
            key_fg: fx,
            key_bg: bg,
            label_fg: label,
            label_bg: bg,
        }
    }
}

/// Dibuja pares `(tecla, acción)` separados por 2 espacios desde x=1.
pub fn fkey_bar(buf: &mut Buffer, y: u16, keys: &[(&str, &str)], theme: Theme) -> u16 {
    fkey_bar_styled(buf, y, keys, FKeyStyle::classic(theme))
}

/// 1 fila con estilo propio.
pub fn fkey_bar_styled(buf: &mut Buffer, y: u16, keys: &[(&str, &str)], style: FKeyStyle) -> u16 {
    if y >= buf.height() {
        return 0;
    }
    let mut cx = 1u16;
    for (k, label) in keys {
        let s = format!(" {k} {label} ");
        let w = visible_len(&s);
        if cx.saturating_add(w) > buf.width() {
            break;
        }
        draw_text(buf, cx, y, &format!(" {k}"), style_key(style));
        draw_text(
            buf,
            cx.saturating_add(visible_len(k)).saturating_add(1),
            y,
            &format!(" {label} "),
            style_label(style),
        );
        cx = cx.saturating_add(w).saturating_add(2);
    }
    cx
}

/// 2 filas estilo referencia: la F sobre el número, alineados.
/// Ocupa las filas `y` (letras) y `y+1` (número + etiqueta).
/// `F2` → `F`/`2 Grabar`; `Esc` → `Esc`/`Salir`; `F10` → `F`/`10 Menu`.
pub fn fkey_bar_stacked(buf: &mut Buffer, y: u16, keys: &[(&str, &str)], style: FKeyStyle) -> u16 {
    if y.saturating_add(1) >= buf.height() {
        return 0;
    }
    let ka = style_key(style);
    let la = style_label(style);
    let mut cx = 1u16;
    for (k, label) in keys {
        // Parte superior: "F" (o la tecla tal cual si no es Fn).
        // Parte inferior: "2 Grabar" (número con color Fx + etiqueta).
        let (top, num) = split_fkey(k);
        let w = visible_len(top).max(
            visible_len(num)
                .saturating_add(visible_len(label))
                .saturating_add(1),
        );
        if cx.saturating_add(w) > buf.width() {
            break;
        }
        draw_text(buf, cx, y, top, ka);
        draw_text(buf, cx, y.saturating_add(1), num, ka);
        draw_text(
            buf,
            cx.saturating_add(visible_len(num)),
            y.saturating_add(1),
            &format!(" {label}"),
            la,
        );
        cx = cx.saturating_add(w).saturating_add(2);
    }
    cx
}

/// `"F10"` → `("F", "10")`; `"Esc"` → `("Esc", "")`.
fn split_fkey(k: &str) -> (&str, &str) {
    if (k.len() == 2 || k.len() == 3)
        && (k.as_bytes()[0] == b'F' || k.as_bytes()[0] == b'f')
        && k[1..].chars().all(|c| c.is_ascii_digit())
    {
        (&k[..1], &k[1..])
    } else {
        (k, "")
    }
}

fn style_key(s: FKeyStyle) -> Attr {
    Attr::bold(s.key_fg, s.key_bg)
}

fn style_label(s: FKeyStyle) -> Attr {
    Attr::new(s.label_fg, s.label_bg)
}

/// Fila compacta estilo referencia: número en superíndice + etiqueta.
/// `F1 Help` → `¹Help`, `F10 Menu` → `¹⁰Menu`, `Esc Salir` tal cual.
/// El número va en colores Fx (típico: invertidos o amarillos).
pub fn fkey_bar_compact(buf: &mut Buffer, y: u16, keys: &[(&str, &str)], style: FKeyStyle) -> u16 {
    if y >= buf.height() {
        return 0;
    }
    let ka = style_key(style);
    let la = style_label(style);
    let mut cx = 1u16;
    for (k, label) in keys {
        let num = compact_num(k);
        let w = visible_len(&num).saturating_add(visible_len(label));
        if cx.saturating_add(w) > buf.width() {
            break;
        }
        draw_text(buf, cx, y, &num, ka);
        draw_text(buf, cx.saturating_add(visible_len(&num)), y, label, la);
        cx = cx.saturating_add(w).saturating_add(2);
    }
    cx
}

/// `"F2"` → `"²"`, `"F10"` → `"¹⁰"`; lo demás tal cual.
fn compact_num(k: &str) -> String {
    match k.strip_prefix(&['F', 'f'][..]) {
        Some(n) if !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) => superscript_digits(n),
        _ => k.to_string(),
    }
}

fn superscript_digits(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_keys_in_order() {
        let t = Theme::clipper();
        let mut buf = Buffer::blank(60, 25, t.desktop);
        fkey_bar(&mut buf, 24, &[("F2", "Grabar"), ("Esc", "Salir")], t);
        assert_eq!(buf.get(1, 24).unwrap().ch, ' ');
        assert_eq!(buf.get(2, 24).unwrap().ch, 'F');
        assert_eq!(buf.get(2, 24).unwrap().bg, t.button_bg);
    }

    #[test]
    fn styled_applies_both_colors() {
        let t = Theme::clipper();
        let mut buf = Buffer::blank(60, 25, t.desktop);
        let style = FKeyStyle::highlight(Color::Yellow, Color::White, t.desktop);
        fkey_bar_styled(&mut buf, 0, &[("F2", "Grabar")], style);
        assert_eq!(buf.get(2, 0).unwrap().fg, Color::Yellow); // F
        assert_eq!(buf.get(4, 0).unwrap().fg, Color::White); // G
    }

    #[test]
    fn stacked_puts_f_over_number() {
        let t = Theme::clipper();
        let mut buf = Buffer::blank(60, 25, t.desktop);
        let style = FKeyStyle::highlight(Color::Yellow, Color::White, t.desktop);
        fkey_bar_stacked(&mut buf, 22, &[("F2", "Grabar"), ("F10", "Menu")], style);
        // F sobre el 2, alineados en x=1.
        assert_eq!(buf.get(1, 22).unwrap().ch, 'F');
        assert_eq!(buf.get(1, 23).unwrap().ch, '2');
        assert_eq!(buf.get(1, 22).unwrap().fg, Color::Yellow);
        // Segundo botón: F sobre "10".
        let x2 = 1 + "2 Grabar".len() as u16 + 2;
        assert_eq!(buf.get(x2, 22).unwrap().ch, 'F');
        assert_eq!(buf.get(x2, 23).unwrap().ch, '1');
        assert_eq!(buf.get(x2 + 1, 23).unwrap().ch, '0');
    }

    #[test]
    fn compact_uses_superscripts() {
        let t = Theme::clipper();
        let mut buf = Buffer::blank(60, 25, t.desktop);
        let style = FKeyStyle::highlight(Color::Yellow, Color::White, t.desktop);
        fkey_bar_compact(
            &mut buf,
            24,
            &[("F2", "Help"), ("F10", "Menu"), ("Esc", "Salir")],
            style,
        );
        assert_eq!(buf.get(1, 24).unwrap().ch, '²');
        assert_eq!(buf.get(1, 24).unwrap().fg, Color::Yellow);
        assert_eq!(buf.get(2, 24).unwrap().ch, 'H');
        let x2 = 1 + 1 + 4 + 2; // ²Help + aire
        assert_eq!(buf.get(x2, 24).unwrap().ch, '¹');
        assert_eq!(buf.get(x2 + 1, 24).unwrap().ch, '⁰');
    }

    #[test]
    fn split_fkey_cases() {
        assert_eq!(split_fkey("F2"), ("F", "2"));
        assert_eq!(split_fkey("F10"), ("F", "10"));
        assert_eq!(split_fkey("Esc"), ("Esc", ""));
        assert_eq!(split_fkey("Enter"), ("Enter", ""));
    }
}
