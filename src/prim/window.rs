//! Ventanas planas: bloque sólido + barra de título + sombra dura.
//!
//! Sin box-drawing. El título va centrado en la primera fila con su
//! propio fondo (teal en modales, menta en diálogos, negro en fichas).

use crate::core::{Buffer, Cell, Color, Rect, Theme};

use super::label::{draw_text, fit_text, visible_len};
use super::shadow::{shadow_styled, ShadowStyle};

/// Opciones de dibujo de una ventana (incluye los 4 parámetros globales:
/// cuerpo = fg/bg, `border_color` opt-in, `shadow` = `has_shadow`).
#[derive(Clone, Debug)]
pub struct WindowOpts {
    /// `foreground_color` global: texto del cuerpo.
    pub body_fg: Color,
    /// `background_color` global: fondo del cuerpo.
    pub body_bg: Color,
    pub title: String,
    pub title_bg: Color,
    pub title_fg: Color,
    pub title_bold: bool,
    /// `has_shadow`: si `false`, no proyecta sombra.
    pub shadow: bool,
    pub shadow_style: ShadowStyle,
    /// `border_color` global: `Some` pinta marco de 1 celda en el borde
    /// del rect; `None` = bloque sin marco (look Clipper por defecto).
    pub border_color: Option<Color>,
    /// Pinta caja de cierre `[■]` (`\u{25a0}`) en relativo `(x=2, y=0)`.
    pub controls: bool,
}

impl WindowOpts {
    /// Alias explícito del flag de sombra (`shadow`).
    pub fn has_shadow(&self) -> bool {
        self.shadow
    }

    /// Ajusta `has_shadow` en cadena.
    pub fn with_shadow(mut self, has_shadow: bool) -> Self {
        self.shadow = has_shadow;
        self
    }
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
            border_color: None,
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
            border_color: None,
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
            border_color: None,
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
        // Caja de cierre `[■]` (`\u{25a0}`) incrustada en la línea superior,
        // coordenada relativa interna `(x=2, y=0)`: reemplaza esa porción
        // de la línea horizontal del borde/título.
        if opts.controls && rect.w >= 8 {
            super::icons::win_close(buf, rect.x.saturating_add(2), rect.y, attr, Color::Yellow);
        }
    }
    // Borde opt-in: marco de 1 celda en el perímetro (no cambia el tamaño).
    if let Some(border) = opts.border_color {
        let bc = Cell::new(' ', border, border);
        for x in rect.x..rect.right() {
            buf.set(x, rect.y, bc);
            buf.set(x, rect.bottom().saturating_sub(1), bc);
        }
        for y in rect.y..rect.bottom() {
            buf.set(rect.x, y, bc);
            buf.set(rect.right().saturating_sub(1), y, bc);
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

    #[test]
    fn close_box_and_border() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 15, t.desktop);
        let r = Rect::new(5, 3, 30, 9);
        let mut opts = WindowOpts::modal("ORDENAR", t);
        opts.controls = true;
        window(&mut b, r, &opts, t);
        // `[■]` en relativo (x=2, y=0): corchetes + `\u{25a0}` amarillo.
        assert_eq!(b.get(7, 3).unwrap().ch, '[');
        assert_eq!(b.get(8, 3).unwrap().ch, '\u{25a0}');
        assert_eq!(b.get(8, 3).unwrap().fg, Color::Yellow);
        assert_eq!(b.get(9, 3).unwrap().ch, ']');
        // Borde opt-in: perímetro en el color pedido.
        let mut bordered = WindowOpts::modal("B", t);
        bordered.border_color = Some(Color::Red);
        window(&mut b, r, &bordered, t);
        assert_eq!(b.get(5, 3).unwrap().bg, Color::Red);
        assert_eq!(b.get(5 + 30 - 1, 3 + 9 - 1).unwrap().bg, Color::Red);
        // Sin borde por defecto: la esquina es del cuerpo.
        window(&mut b, r, &WindowOpts::modal("B", t), t);
        assert_eq!(b.get(5, 3 + 9 - 1).unwrap().bg, Color::Grey);
    }
}
