//! Botones teal con sombra CUA por celdas + estado presionado intrínseco.
//!
//! Geometría de sombra (todo en celdas, botón en `(x, y)`, `W` x `H=1`):
//! * columna derecha: `(x+W, y)` .. `(x+W, y+H)` (incluye la esquina),
//!   fila inferior desplazada: `(x+1, y+H)` .. `(x+W, y+H)`.
//!
//! Cada celda de sombra **mezcla**: conserva el glifo subyacente y solo
//! fuerza `bg` a negro (+ `dim`), nunca borra el fondo.
//!
//! Ancho: mínimo `BUTTON_MIN_WIDTH` (10) con texto centrado; si el texto
//! lo supera, 2 celdas de margen por lado. Al presionarse, el botón
//! **baja 1 y se corre 1 a la derecha**, tapando su sombra.
//!
//! Estilo global: `ButtonOpts` expone `foreground_color`,
//! `background_color`, `border_color` (`None` = tema) y `has_shadow`.

use crate::core::{Attr, Buffer, Cell, Color, Theme, WidgetStyle};

use super::label::visible_len;

/// Ancho mínimo de un botón (texto corto centrado con relleno).
pub const BUTTON_MIN_WIDTH: u16 = 10;

/// Configuración de un botón (los 4 parámetros globales; `None` = tema).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonOpts {
    pub foreground_color: Option<Color>,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    /// Si es `false`, no se pinta sombra.
    pub has_shadow: bool,
}

impl Default for ButtonOpts {
    fn default() -> Self {
        Self {
            foreground_color: None,
            background_color: None,
            border_color: None,
            has_shadow: true,
        }
    }
}

impl ButtonOpts {
    pub fn with_shadow(has_shadow: bool) -> Self {
        Self {
            has_shadow,
            ..Self::default()
        }
    }

    /// Estilo explícito completo (convierte `WidgetStyle`).
    pub fn styled(style: WidgetStyle) -> Self {
        Self {
            foreground_color: Some(style.foreground_color),
            background_color: Some(style.background_color),
            border_color: Some(style.border_color),
            has_shadow: style.has_shadow,
        }
    }
}

/// Ancho total del botón: padding de 2 por lado (`  TEXTO  `),
/// con mínimo `BUTTON_MIN_WIDTH` (10) para textos cortos.
pub fn button_width(label: &str) -> u16 {
    let text = visible_len(label);
    text.saturating_add(4).max(BUTTON_MIN_WIDTH)
}

fn resolve_attr(theme: Theme, opts: ButtonOpts) -> Attr {
    Attr::bold(
        opts.foreground_color.unwrap_or(theme.button_fg),
        opts.background_color.unwrap_or(theme.button_bg),
    )
}

/// Dibuja el botón en reposo en (x, y). Devuelve el ancho ocupado.
/// El recorte al buffer es silencioso (no panic en bordes).
/// Con sombra CUA por defecto (`has_shadow = true`).
pub fn button(buf: &mut Buffer, x: u16, y: u16, label: &str, theme: Theme) -> u16 {
    button_draw(buf, x, y, label, theme, false)
}

/// Variante con opciones (p.ej. `has_shadow = false`).
pub fn button_ex(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    label: &str,
    theme: Theme,
    opts: ButtonOpts,
) -> u16 {
    button_draw_opts(buf, x, y, label, theme, false, opts)
}

/// Dibuja el botón; con `pressed = true` se renderiza hundido
/// (desplazado +1,+1 y sin sombra, como si tapara la suya).
/// Devuelve el ancho ocupado (el mismo en ambos estados).
/// Con sombra CUA por defecto.
pub fn button_draw(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    label: &str,
    theme: Theme,
    pressed: bool,
) -> u16 {
    button_draw_ex(buf, x, y, label, theme, pressed, true)
}

/// Dibujo completo con control de sombra.
///
/// * `pressed` — hundido (+1,+1, sin sombra).
/// * `has_shadow` — si `false`, no proyecta sombra ni en reposo.
pub fn button_draw_ex(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    label: &str,
    theme: Theme,
    pressed: bool,
    has_shadow: bool,
) -> u16 {
    button_draw_opts(
        buf,
        x,
        y,
        label,
        theme,
        pressed,
        ButtonOpts {
            has_shadow,
            ..ButtonOpts::default()
        },
    )
}

/// Dibujo completo con estilo global (`ButtonOpts`).
pub fn button_draw_opts(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    label: &str,
    theme: Theme,
    pressed: bool,
    opts: ButtonOpts,
) -> u16 {
    let w = button_width(label);
    if w == 0 {
        return 0;
    }
    // Hundido: corre el bloque y no pintes sombra (queda tapada).
    let (bx, by) = if pressed {
        (x.saturating_add(1), y.saturating_add(1))
    } else {
        (x, y)
    };
    let attr = resolve_attr(theme, opts);
    // Bloque + texto centrado.
    buf.fill_rect(
        crate::core::Rect::new(bx, by, w, 1),
        Cell::with_attr(' ', attr),
    );
    let text = visible_len(label);
    let start = bx.saturating_add(w.saturating_sub(text) / 2);
    let mut cx = start;
    for ch in label.chars() {
        if cx >= bx.saturating_add(w) {
            break;
        }
        buf.set(cx, by, Cell::with_attr(ch, attr));
        cx = cx.saturating_add(1);
    }
    // Borde opt-in: marco de 1 celda alrededor (no cambia el ancho devuelto).
    if let Some(border) = opts.border_color {
        let battr = Attr::new(border, border);
        for i in 0..w.saturating_add(2) {
            buf.set(
                bx.saturating_sub(1).saturating_add(i),
                by.saturating_sub(1),
                Cell::with_attr(' ', battr),
            );
        }
        for i in 0..w.saturating_add(2) {
            buf.set(
                bx.saturating_sub(1).saturating_add(i),
                by.saturating_add(1),
                Cell::with_attr(' ', battr),
            );
        }
        buf.set(bx.saturating_sub(1), by, Cell::with_attr(' ', battr));
        buf.set(bx.saturating_add(w), by, Cell::with_attr(' ', battr));
    }
    if !pressed && opts.has_shadow {
        // Sombra CUA exacta por celdas (botón en `(bx, by)`, `W` x `H=1`):
        // - lateral derecha: EXACTAMENTE `(x_end, y)` = `(bx+w, by)`,
        // - inferior: `(x_start+1, y+1)` .. `(x_end, y+1)` inclusive
        //   (el rango ya incluye la esquina `(bx+w, by+1)` que cierra la L).
        // Sin `\n` ni celdas extra: mezcla del fondo existente.
        blend_shadow(buf, bx.saturating_add(w), by, theme);
        for i in 0..w {
            blend_shadow(
                buf,
                bx.saturating_add(i).saturating_add(1),
                by.saturating_add(1),
                theme,
            );
        }
    }
    w
}

/// Mezcla de sombra: conserva glifo+fg, fuerza bg negro + `dim`.
fn blend_shadow(buf: &mut Buffer, x: u16, y: u16, theme: Theme) {
    if !buf.in_bounds(x, y) {
        return;
    }
    let old = buf.get(x, y).unwrap_or(Cell::blank(theme.shadow));
    buf.set(
        x,
        y,
        Cell {
            ch: old.ch,
            fg: old.fg,
            bg: theme.shadow,
            bold: false,
            dim: true,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_has_minimum_and_margins() {
        assert_eq!(button_width("OK"), BUTTON_MIN_WIDTH); // enano → 10
        assert_eq!(button_width(""), BUTTON_MIN_WIDTH);
        assert_eq!(button_width("Salir"), BUTTON_MIN_WIDTH); // 5+4 < 10
        assert_eq!(button_width("Ordenar"), 7 + 4); // padding 2 por lado
        assert_eq!(button_width("123456789"), 9 + 4);
    }

    #[test]
    fn short_label_is_centered() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 5, t.desktop);
        let w = button(&mut b, 2, 1, "OK", t);
        assert_eq!(w, BUTTON_MIN_WIDTH);
        // "OK" centrado en 10: inicio = 2 + (10-2)/2 = 6.
        assert_eq!(b.get(6, 1).unwrap().ch, 'O');
        assert_eq!(b.get(7, 1).unwrap().ch, 'K');
        assert_eq!(b.get(2, 1).unwrap().ch, ' ');
        assert_eq!(b.get(2 + 10 - 1, 1).unwrap().ch, ' ');
    }

    #[test]
    fn draws_teal_block_with_cua_shadow() {
        use crate::core::Color;
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 6, t.desktop);
        // Fondo con glifo para verificar mezcla (no borra).
        b.text(12, 1, "Z", Color::Black, t.desktop);
        b.text(12, 2, "Y", Color::Black, t.desktop);
        let w = button(&mut b, 2, 1, "Salir", t);
        assert_eq!(w, BUTTON_MIN_WIDTH);
        assert_eq!(b.get(2, 1).unwrap().bg, t.button_bg);
        // Sombra inferior corrida +1, mezclada.
        let c = b.get(3, 2).unwrap();
        assert_eq!(c.bg, t.shadow);
        assert!(c.dim);
        // Sombra lateral CUA: última columna +1, misma fila, mezclada.
        let side = b.get(2 + 10, 1).unwrap();
        assert_eq!(side.ch, 'Z'); // conserva glifo
        assert_eq!(side.bg, t.shadow);
        assert!(side.dim);
        // Esquina (x+W, y+H): cierra la L, también mezclada.
        let corner = b.get(2 + 10, 2).unwrap();
        assert_eq!(corner.ch, 'Y');
        assert_eq!(corner.bg, t.shadow);
        assert!(corner.dim);
    }

    #[test]
    fn has_shadow_false_draws_no_shadow() {
        use crate::core::Color;
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 6, t.desktop);
        b.text(12, 1, "Z", Color::Black, t.desktop);
        button_draw_ex(&mut b, 2, 1, "Salir", t, false, false);
        // Sin sombra: fondo intacto al costado, debajo y en la esquina.
        assert_eq!(b.get(12, 1).unwrap().ch, 'Z');
        assert_eq!(b.get(12, 1).unwrap().bg, t.desktop);
        assert_eq!(b.get(3, 2).unwrap().bg, t.desktop);
        assert_eq!(b.get(12, 2).unwrap().bg, t.desktop);
    }

    #[test]
    fn pressed_moves_down_right_and_covers_shadow() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 6, t.desktop);
        // Primero en reposo (pinta sombra), luego presionado encima.
        button(&mut b, 2, 1, "OK", t);
        button_draw(&mut b, 2, 1, "OK", t, true);
        // El bloque bajó: la O ahora está en (3+4?...): centrada en (3,2).
        assert_eq!(b.get(3 + (10 - 2) / 2, 2).unwrap().ch, 'O');
        // ...y tapó su propia sombra (ya no hay negro debajo del bloque).
        assert_eq!(b.get(3, 2).unwrap().bg, t.button_bg);
    }

    #[test]
    fn style_overrides_theme_colors() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 6, t.desktop);
        let opts = ButtonOpts {
            foreground_color: Some(Color::Yellow),
            background_color: Some(Color::Navy),
            border_color: None,
            has_shadow: false,
        };
        button_ex(&mut b, 2, 1, "OK", t, opts);
        assert_eq!(b.get(2, 1).unwrap().bg, Color::Navy);
        assert_eq!(b.get(2, 1).unwrap().fg, Color::Yellow);
        assert_eq!(b.get(3, 2).unwrap().bg, t.desktop); // sin sombra
        assert_eq!(b.get(12, 1).unwrap().bg, t.desktop);
    }

    #[test]
    fn border_draws_frame_around() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 6, t.desktop);
        let opts = ButtonOpts {
            border_color: Some(Color::Red),
            ..ButtonOpts::default()
        };
        button_ex(&mut b, 2, 2, "OK", t, opts);
        assert_eq!(b.get(1, 1).unwrap().bg, Color::Red); // arriba-izq
        assert_eq!(b.get(1, 2).unwrap().bg, Color::Red); // lateral
        assert_eq!(b.get(2, 2).unwrap().bg, t.button_bg); // botón intacto
    }
}
