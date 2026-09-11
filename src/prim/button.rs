//! Botones con foco por luminancia + hotkey CUA + sombra por celdas.
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
//! Hotkey (acelerador CUA): `&` marca la siguiente letra (`"&OK"` pinta
//! `OK` con la `O` en amarillo, el `&` jamás se dibuja); sin `&`, la
//! primera letra alfabética es el hotkey (ver `parse_hotkey`).
//!
//! Estados (el foco lo comunica el propio botón encendido, NUNCA marcas
//! externas como `►` ni manchas fuera de `x..x+w`):
//! * reposo: fondo `background_color`/`theme.button_bg`, texto normal,
//!   hotkey en amarillo atenuado.
//! * foco (`focused`): fondo `Blue` brillante, texto en negrita, hotkey
//!   en amarillo brillante + negrita.
//! * presionado (`pressed`): +1,+1 sin sombra, fondo `Navy` oscuro.
//!
//! Nota: el motor es ANSI-16 sin `Rgb`; este es el mapeo fiel de la
//! paleta TrueColor pedida (azul medio, celeste brillante, azul oscuro).
//!
//! Estilo global: `ButtonOpts` expone `foreground_color`,
//! `background_color`, `border_color` (`None` = tema), `has_shadow` y
//! `focused` (el `foreground_color` explícito se respeta en los 3 estados).

use crate::core::{Attr, Buffer, Cell, Color, Theme, WidgetStyle};

use super::label::{parse_hotkey, visible_len};

/// Ancho mínimo de un botón (texto corto centrado con relleno).
pub const BUTTON_MIN_WIDTH: u16 = 10;

/// Configuración de un botón (los 4 parámetros globales + foco;
/// `None` = tema). `border_color`: `Some` pinta corchetes `[ ]`
/// integrados en la única fila (sin filas extra); `None` = bloque sólido.
/// `focused`: el botón con foco del teclado (fondo brillante + negrita,
/// sin marcas externas).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonOpts {
    pub foreground_color: Option<Color>,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    /// Si es `false`, no se pinta sombra.
    pub has_shadow: bool,
    /// Botón con foco (luminancia + hotkey brillante).
    pub focused: bool,
}

impl Default for ButtonOpts {
    fn default() -> Self {
        Self {
            foreground_color: None,
            background_color: None,
            border_color: None,
            has_shadow: true,
            focused: false,
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
            focused: false,
        }
    }

    /// Cadena con el botón con foco (el que manda el Tab del diálogo).
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

/// Ancho total del botón: padding de 2 por lado (`  TEXTO  `),
/// con mínimo `BUTTON_MIN_WIDTH` (10) para textos cortos.
pub fn button_width(label: &str) -> u16 {
    let text = visible_len(label);
    text.saturating_add(4).max(BUTTON_MIN_WIDTH)
}

/// Par de atributos según estado: texto base + hotkey.
struct ButtonPaint {
    base: Attr,
    hot: Attr,
}

fn resolve_paint(theme: Theme, opts: ButtonOpts, pressed: bool) -> ButtonPaint {
    // El estado manda sobre el fondo (el fg explícito se respeta siempre).
    let bg = if pressed {
        Color::Navy
    } else if opts.focused {
        Color::Blue
    } else {
        opts.background_color.unwrap_or(theme.button_bg)
    };
    let fg = opts.foreground_color.unwrap_or(theme.button_fg);
    let lit = opts.focused && !pressed;
    ButtonPaint {
        base: if lit {
            Attr::bold(fg, bg)
        } else {
            Attr::new(fg, bg)
        },
        hot: if lit {
            Attr::bold(Color::Yellow, bg)
        } else {
            Attr::new(Color::Yellow, bg)
        },
    }
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

    // Si está presionado se desplaza para el efecto de hundido
    let (bx, by) = if pressed {
        (x.saturating_add(1), y.saturating_add(1))
    } else {
        (x, y)
    };

    let paint = resolve_paint(theme, opts, pressed);

    // ----------------------------------------------------------------
    // FILA 1: CUERPO DEL BOTÓN + SOMBRA LATERAL MITAD INFERIOR
    // ----------------------------------------------------------------
    // Pinar el rectángulo del botón (1 fila de alto)
    buf.fill_rect(
        crate::core::Rect::new(bx, by, w, 1),
        Cell::with_attr(' ', paint.base),
    );

    // Texto limpio centrado (`&` = marca hotkey, jamás se dibuja).
    let (clean, hot) = parse_hotkey(label);
    let hot_idx = hot.map(|(i, _)| i);
    let text = visible_len(label);
    let start = bx.saturating_add(w.saturating_sub(text) / 2);
    let mut cx = start;
    for (i, ch) in clean.chars().enumerate() {
        if cx >= bx.saturating_add(w) {
            break;
        }
        let a = if Some(i) == hot_idx {
            paint.hot
        } else {
            paint.base
        };
        buf.set(cx, by, Cell::with_attr(ch, a));
        cx = cx.saturating_add(1);
    }

    // ELIMINAMOS los bucles for antiguos que inflaban el botón arriba y abajo.
    // 2. BORDE INTEGRADO OPCIONAL (Solo si se requiere una línea fina, NO celdas vacías extras)
    if let Some(_border) = opts.border_color {
        // Si necesitas un recuadro de línea fina estilo [ OK ], pinta los corchetes en los extremos
        buf.set(bx, by, Cell::with_attr('[', paint.base));
        buf.set(
            bx.saturating_add(w).saturating_sub(1),
            by,
            Cell::with_attr(']', paint.base),
        );
    }

    // SI NO ESTÁ PRESIONADO, DIBUJAMOS LA ILUSIÓN ÓPTICA DE LA SOMBRA DETECTANDO EL FONDO REAL
    if !pressed && opts.has_shadow {
        let x_end = bx.saturating_add(w);

        // A. SOMBRA LATERAL DER (Fila 1)
        if buf.in_bounds(x_end, by) {
            // DETECCIÓN: Leemos la celda que ya existe en el búfer en esa coordenada
            let current_cell = buf.get(x_end, by).unwrap_or(Cell::blank(theme.window_bg));
            let real_bg = current_cell.bg; // El verdadero color del fondo donde está el botón

            buf.set(
                x_end,
                by,
                Cell {
                    ch: '\u{2584}',
                    fg: Color::Black, // Negro puro para la mitad inferior
                    bg: real_bg,      // El color real detectado para la mitad superior
                    bold: false,
                    dim: false,
                },
            );
        }

        // B. SOMBRA INFERIOR DE MEDIA ALTURA (Fila 2)
        let shadow_y = by.saturating_add(1);
        for sx in bx.saturating_add(1)..=x_end {
            if buf.in_bounds(sx, shadow_y) {
                // DETECCIÓN: Leemos la celda de la fila de abajo antes de pisarla
                let current_cell = buf
                    .get(sx, shadow_y)
                    .unwrap_or(Cell::blank(theme.window_bg));
                let real_bg = current_cell.bg; // El color real debajo del botón

                buf.set(
                    sx,
                    shadow_y,
                    Cell {
                        ch: '\u{2580}',
                        fg: Color::Black, // Negro puro para la mitad superior
                        bg: real_bg,      // El color real detectado para la mitad inferior
                        bold: false,
                        dim: false,
                    },
                );
            }
        }
    }

    w
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
    fn draws_teal_block_with_pc_tools_shadow() {
        use crate::core::Color;
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 6, t.desktop);
        let (x, y) = (2u16, 1u16);
        let w = button(&mut b, x, y, "Salir", t);
        assert_eq!(w, BUTTON_MIN_WIDTH);
        let (x_start, x_end) = (x, x.saturating_add(w));
        assert_eq!(b.get(2, 1).unwrap().bg, t.button_bg);
        // A. Lateral `(x_end, y)`: `▄` negro sobre el fondo REAL detectado.
        let side = b.get(x_end, y).unwrap();
        assert_eq!(side.ch, '\u{2584}');
        assert_eq!(side.fg, Color::Black);
        assert_eq!(side.bg, t.desktop); // funde con el fondo existente
                                        // B. Inferior `(x_start+1)..=(x_end)` en `y+1`: `▀` sobre fondo real.
        for sx in x_start.saturating_add(1)..=x_end {
            let c = b.get(sx, y.saturating_add(1)).unwrap();
            assert_eq!(c.ch, '\u{2580}', "medio bloque sup en ({sx}, 2)");
            assert_eq!(c.fg, Color::Black);
            assert_eq!(c.bg, t.desktop); // detectado, no gris fijo
        }
        // Fuera del rango: fondo intacto (nada extra a la izquierda).
        assert_eq!(b.get(x_start, y.saturating_add(1)).unwrap().bg, t.desktop);
    }

    #[test]
    fn shadow_detects_real_background() {
        use crate::core::Color;
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 6, t.desktop);
        // Fondo blanco bajo la futura sombra: debe detectarlo, no gris fijo.
        b.text(3, 2, "FFFFFFFFFF", Color::Black, Color::White);
        button(&mut b, 2, 1, "OK", t);
        for x in 3..=12u16 {
            let c = b.get(x, 2).unwrap();
            assert_eq!(c.ch, '\u{2580}');
            assert_eq!(c.bg, Color::White, "fondo detectado en ({x}, 2)");
        }
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
        // ...y tapó su propia sombra con fondo oscuro (ya no hay negro).
        assert_eq!(b.get(3, 2).unwrap().bg, Color::Navy);
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
            focused: false,
        };
        button_ex(&mut b, 2, 1, "OK", t, opts);
        assert_eq!(b.get(2, 1).unwrap().bg, Color::Navy);
        assert_eq!(b.get(2, 1).unwrap().fg, Color::Yellow);
        assert_eq!(b.get(3, 2).unwrap().bg, t.desktop); // sin sombra
        assert_eq!(b.get(12, 1).unwrap().bg, t.desktop);
    }

    #[test]
    fn border_is_inline_one_row_no_extra_cells() {
        use crate::core::Color;
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 8, t.desktop);
        let opts = ButtonOpts {
            border_color: Some(Color::Red),
            ..ButtonOpts::default()
        };
        let w = button_ex(&mut b, 4, 2, "OK", t, opts);
        // Corchetes integrados en la ÚNICA fila, sin filas extra arriba/abajo.
        assert_eq!(b.get(4, 2).unwrap().ch, '[');
        assert_eq!(b.get(4 + w - 1, 2).unwrap().ch, ']');
        assert_eq!(b.get(4, 1).unwrap().bg, t.desktop); // nada arriba
                                                        // Sombra `▀` abajo (by+1): tinta negra, fondo real detectado.
        for x in 5..=4 + w {
            let c = b.get(x, 3).unwrap();
            assert_eq!(c.ch, '\u{2580}', "medio bloque sup en ({x}, 3)");
            assert_eq!(c.fg, Color::Black);
            assert_eq!(c.bg, t.desktop);
        }
        // Lateral `▄` en la misma fila.
        assert_eq!(b.get(4 + w, 2).unwrap().ch, '\u{2584}');
    }

    #[test]
    fn border_draws_inline_brackets() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 6, t.desktop);
        let opts = ButtonOpts {
            border_color: Some(Color::Red),
            ..ButtonOpts::default()
        };
        button_ex(&mut b, 2, 2, "OK", t, opts);
        // 1 sola fila: `[` al inicio, `]` al final, texto intacto dentro.
        assert_eq!(b.get(2, 2).unwrap().ch, '[');
        assert_eq!(b.get(2 + 10 - 1, 2).unwrap().ch, ']');
        assert_eq!(b.get(2 + (10 - 2) / 2, 2).unwrap().ch, 'O');
        assert_eq!(b.get(2, 1).unwrap().bg, t.desktop); // sin fila extra
    }

    #[test]
    fn ampersand_marks_hotkey_and_is_never_drawn() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 6, t.desktop);
        let w = button(&mut b, 2, 1, "&Cancelar", t);
        // "Cancelar" (8) + 4 = 12; sin `&` en ninguna celda del botón.
        assert_eq!(w, 12);
        for x in 2..2 + w {
            assert_ne!(b.get(x, 1).unwrap().ch, '&', "& en {x}");
        }
        // Hotkey `C` en amarillo atenuado (reposo: sin negrita).
        let start = 2 + (12 - 8) / 2;
        let c = b.get(start, 1).unwrap();
        assert_eq!(c.ch, 'C');
        assert_eq!(c.fg, Color::Yellow);
        assert!(!c.bold);
        assert_eq!(c.bg, t.button_bg);
    }

    #[test]
    fn first_alpha_is_default_hotkey() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 6, t.desktop);
        button(&mut b, 2, 1, "OK", t);
        let start = 2 + (10 - 2) / 2;
        assert_eq!(b.get(start, 1).unwrap().ch, 'O');
        assert_eq!(b.get(start, 1).unwrap().fg, Color::Yellow);
        assert_eq!(b.get(start + 1, 1).unwrap().fg, t.button_fg);
    }

    #[test]
    fn focused_lights_up_blue_and_bold() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 6, t.desktop);
        let opts = ButtonOpts::default().focused(true);
        button_ex(&mut b, 2, 1, "&OK", t, opts);
        // Fondo brillante + texto en negrita + hotkey amarilla bold.
        assert_eq!(b.get(2, 1).unwrap().bg, Color::Blue);
        let start = 2 + (10 - 2) / 2;
        let o = b.get(start, 1).unwrap();
        assert_eq!((o.ch, o.fg, o.bg), ('O', Color::Yellow, Color::Blue));
        assert!(o.bold);
        let k = b.get(start + 1, 1).unwrap();
        assert_eq!(k.ch, 'K');
        assert!(k.bold);
        assert_eq!(k.bg, Color::Blue);
        // Reposo al lado: teal sin negrita (contraste visible).
        button(&mut b, 16, 1, "OK", t);
        assert_eq!(b.get(16, 1).unwrap().bg, t.button_bg);
        assert!(!b.get(16, 1).unwrap().bold);
    }

    #[test]
    fn pressed_darkens_navy_and_stays_inside() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 6, t.desktop);
        let w = button_draw(&mut b, 2, 1, "OK", t, true);
        // Hundido +1,+1 con fondo oscuro, sin sombra debajo.
        assert_eq!(b.get(3 + (10 - 2) / 2, 2).unwrap().ch, 'O');
        assert_eq!(b.get(3, 2).unwrap().bg, Color::Navy);
        // Geometría estricta: nada a la izquierda del botón (x-1 intacto).
        assert_eq!(b.get(1, 1).unwrap().bg, t.desktop);
        assert_eq!(b.get(1, 2).unwrap().bg, t.desktop);
        assert_eq!(w, BUTTON_MIN_WIDTH);
    }
}
