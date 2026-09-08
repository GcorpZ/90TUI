//! Ventanas planas: bloque sólido + barra de título + sombra dura.
//!
//! Sin box-drawing. El título va centrado en la primera fila con su
//! propio fondo (teal en modales, menta en diálogos, negro en fichas).

use crate::core::{Buffer, Cell, Color, Rect, Theme};

use super::label::{draw_text, fit_text, visible_len};
use super::shadow::{shadow_styled, ShadowStyle};

/// Opciones de dibujo de una ventana.
#[derive(Clone, Debug)]
pub struct WindowOpts {
    pub body_bg: Color,
    pub body_fg: Color,
    pub title: String,
    pub title_bg: Color,
    pub title_fg: Color,
    pub title_bold: bool,
    pub shadow: bool,
    pub shadow_style: ShadowStyle,
    /// Pinta caja de cierre `[■]` a la izquierda del título.
    pub controls: bool,
}

impl WindowOpts {
    /// Modal gris con título teal (diálogos de trabajo).
    pub fn modal(title: &str, theme: Theme) -> Self {
        Self {
            body_bg: theme.window_bg,
            body_fg: Color::Black,
            title: title.to_string(),
            title_bg: theme.teal,
            title_fg: Color::White,
            title_bold: true,
            shadow: true,
            shadow_style: ShadowStyle::Translucent,
            controls: false,
        }
    }

    /// Ficha negra con título blanco (captura de datos).
    pub fn form(title: &str, theme: Theme) -> Self {
        Self {
            body_bg: theme.form_bg,
            body_fg: theme.form_text,
            title: title.to_string(),
            title_bg: theme.form_bg,
            title_fg: Color::White,
            title_bold: true,
            shadow: true,
            shadow_style: ShadowStyle::Translucent,
            controls: false,
        }
    }

    /// Diálogo menta con título teal (progresos y avisos).
    pub fn dialog(title: &str, theme: Theme) -> Self {
        Self {
            body_bg: theme.dialog,
            body_fg: theme.dialog_text,
            title: title.to_string(),
            title_bg: theme.teal,
            title_fg: Color::White,
            title_bold: true,
            shadow: true,
            shadow_style: ShadowStyle::Translucent,
            controls: false,
        }
    }
}

/// Dibuja la ventana (recortada al buffer). No toca la pila de pantallas:
/// eso lo hacen los widgets/`Screen::savescreen` (Fase 5/6).
pub fn window(buf: &mut Buffer, rect: Rect, opts: &WindowOpts, theme: Theme) {
    if rect.is_empty() {
        return;
    }
    if opts.shadow {
        shadow_styled(buf, rect, 2, 1, opts.shadow_style, theme);
    }
    buf.fill_rect(rect, Cell::new(' ', opts.body_fg, opts.body_bg));
    // Barra de título: primera fila.
    if rect.h >= 1 {
        let bar = Rect::new(rect.x, rect.y, rect.w, 1);
        buf.fill_rect(bar, Cell::new(' ', opts.title_fg, opts.title_bg));
        let fit = fit_text(&opts.title, rect.w.saturating_sub(2));
        let tw = visible_len(&fit);
        let tx = rect.x.saturating_add(rect.w.saturating_sub(tw) / 2);
        let attr = crate::core::Attr {
            fg: opts.title_fg,
            bg: opts.title_bg,
            bold: opts.title_bold,
            dim: false,
        };
        draw_text(buf, tx, rect.y, &fit, attr);
        // Caja de cierre sobre la barra (después del título para que se vea).
        if opts.controls && rect.w >= 8 {
            super::icons::win_close(buf, rect.x.saturating_add(1), rect.y, attr, Color::Yellow);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_has_teal_title_grey_body_black_shadow() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 15, t.desktop);
        let r = Rect::new(5, 3, 30, 9);
        window(&mut b, r, &WindowOpts::modal("ORDENAR", t), t);
        // Título centrado, fondo teal.
        assert_eq!(b.get(5 + (30 - 7) / 2, 3).unwrap().ch, 'O');
        assert_eq!(b.get(6, 3).unwrap().bg, t.teal);
        // Cuerpo gris.
        assert_eq!(b.get(6, 5).unwrap().bg, Color::Grey);
        // Sombra negra en (x+2, y+h) — primera fila bajo la ventana.
        assert_eq!(b.get(7, 3 + 9).unwrap().bg, Color::Black);
    }

    #[test]
    fn long_title_truncates_inside() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(20, 6, t.desktop);
        let r = Rect::new(2, 1, 12, 4);
        window(
            &mut b,
            r,
            &WindowOpts::modal("TITULO MUY LARGO QUE NO CABE", t),
            t,
        );
        // La última celda de la barra sigue siendo teal (nada se sale).
        assert_eq!(b.get(2 + 12 - 1, 1).unwrap().bg, t.teal);
    }
}
