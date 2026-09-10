//! Ventanas era DOS: bloque + barra de título + sombra + marcos CP437.
//!
//! `BorderStyle::None` es el bloque plano clásico (título en barra propia,
//! sin box-drawing). `Single`/`Double` visten el perímetro con caja CP437
//! (título incrustado en el marco superior) y `Bevel3D` le da bisel
//! mecánico estilo PC Tools (aristas claras/arriba-izquierda, oscuras
//! abajo-derecha). La geometría no cambia: el cuerpo siempre empieza en
//! `rect.y + 1`.

use crate::core::{Attr, Buffer, Cell, Color, Rect, Theme};

use super::label::{draw_text, fit_text, visible_len};
use super::shadow::{shadow_styled, ShadowStyle};

/// Estilo de borde de ventana (era DOS).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BorderStyle {
    /// Bloque plano clásico: título en barra propia, sin marcos.
    #[default]
    None,
    /// Caja simple CP437 (`┌─┐│└┘`).
    Single,
    /// Caja doble Norton/Clipper (`╔═╗║╚╝`).
    Double,
    /// Bisel mecánico PC Tools: arriba/izquierda brillante,
    /// abajo/derecha oscuro (ignora `border_color`).
    Bevel3D,
}

/// Glifos de una caja perimetral.
struct FrameGlyphs {
    tl: char,
    tr: char,
    bl: char,
    br: char,
    h: char,
    v: char,
}

const SINGLE: FrameGlyphs = FrameGlyphs {
    tl: '\u{250c}',
    tr: '\u{2510}',
    bl: '\u{2514}',
    br: '\u{2518}',
    h: '\u{2500}',
    v: '\u{2502}',
};

const DOUBLE: FrameGlyphs = FrameGlyphs {
    tl: '\u{2554}',
    tr: '\u{2557}',
    bl: '\u{255a}',
    br: '\u{255d}',
    h: '\u{2550}',
    v: '\u{2551}',
};

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
    /// `border_color` global: con `BorderStyle::None` pinta el marco de
    /// 1 celda macizo del borde; con `Single`/`Double` tiñe los glifos
    /// del marco (`None` = tinta del cuerpo). `Bevel3D` lo ignora.
    pub border_color: Option<Color>,
    /// Estilo de marco perimetral (los presets usan `Single`).
    pub border_style: BorderStyle,
    /// Pinta caja de cierre integrada al marco superior (`┤■├` en
    /// `Single`/`Bevel3D`, `╡■╞` en `Double`, `■` en `Yellow` bold):
    /// ocupa `x+3, x+4, x+5` relativos a la esquina, dejando 2 celdas
    /// de línea horizontal antes (`┌──┤■├─`, nunca `┌[■]─`).
    /// En `BorderStyle::None` (sin marco) sigue el flotante `[■]` ASCII.
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
            border_style: BorderStyle::Single,
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
            border_style: BorderStyle::Single,
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
            border_style: BorderStyle::Single,
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
    match opts.border_style {
        BorderStyle::None => draw_flat(buf, rect, opts),
        BorderStyle::Single => draw_framed(buf, rect, opts, &SINGLE),
        BorderStyle::Double => draw_framed(buf, rect, opts, &DOUBLE),
        BorderStyle::Bevel3D => draw_framed(buf, rect, opts, &SINGLE),
    }
}

/// Ventana plana clásica: bloque + barra de título + marco macizo opt-in.
fn draw_flat(buf: &mut Buffer, rect: Rect, opts: &WindowOpts) {
    buf.fill_rect(rect, Cell::new(' ', opts.body_fg, opts.body_bg));
    // Barra de título: primera fila.
    if rect.h >= 1 {
        let bar = Rect::new(rect.x, rect.y, rect.w, 1);
        buf.fill_rect(bar, Cell::new(' ', opts.title_fg, opts.title_bg));
        draw_title_text(buf, rect, rect.w, opts);
        // Caja de cierre `[■]` (`\u{25a0}`) incrustada en el borde superior:
        // NO es un widget flotante; reemplaza los 3 primeros caracteres de
        // la línea horizontal en `x+1, x+2, x+3` (relativo a la esquina).
        if opts.controls && rect.w >= 8 {
            super::icons::win_close(
                buf,
                rect.x.saturating_add(1),
                rect.y,
                title_attr(opts),
                Color::Yellow,
            );
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

/// Ventana con marco CP437: el cuerpo se rellena primero y el marco se
/// pinta encima (nunca lo pisa). El título va incrustado y centrado en la
/// fila superior sin romper el recuadro; el cierre reemplaza marco
/// desde `x+3` con conectores (`┤■├`/`╡■╞`) tras 2 celdas de aire.
fn draw_framed(buf: &mut Buffer, rect: Rect, opts: &WindowOpts, g: &FrameGlyphs) {
    buf.fill_rect(rect, Cell::new(' ', opts.body_fg, opts.body_bg));
    // Tinta del marco: `border_color`, del cuerpo si no hay, o bisel
    // mecánico (arriba/izquierda brillante, abajo/derecha oscuro).
    let (bright, dark) = match opts.border_style {
        BorderStyle::Bevel3D => (Color::White, Color::DarkGrey),
        _ => {
            let ink = opts.border_color.unwrap_or(opts.body_fg);
            (ink, ink)
        }
    };
    let right = rect.right().saturating_sub(1);
    let bottom = rect.bottom().saturating_sub(1);
    // Horizontales (arriba sobre `title_bg`, abajo sobre `body_bg`).
    for x in rect.x.saturating_add(1)..rect.right().saturating_sub(1) {
        buf.set(
            x,
            rect.y,
            Cell::with_attr(g.h, Attr::new(bright, opts.title_bg)),
        );
        buf.set(
            x,
            bottom,
            Cell::with_attr(g.h, Attr::new(dark, opts.body_bg)),
        );
    }
    // Verticales (siempre sobre el cuerpo).
    for y in rect.y.saturating_add(1)..rect.bottom().saturating_sub(1) {
        buf.set(
            rect.x,
            y,
            Cell::with_attr(g.v, Attr::new(bright, opts.body_bg)),
        );
        buf.set(
            right,
            y,
            Cell::with_attr(g.v, Attr::new(dark, opts.body_bg)),
        );
    }
    // Esquinas (arriba con la barra, abajo con el cuerpo).
    buf.set(
        rect.x,
        rect.y,
        Cell::with_attr(g.tl, Attr::new(bright, opts.title_bg)),
    );
    buf.set(
        right,
        rect.y,
        Cell::with_attr(g.tr, Attr::new(bright, opts.title_bg)),
    );
    buf.set(
        rect.x,
        bottom,
        Cell::with_attr(g.bl, Attr::new(dark, opts.body_bg)),
    );
    buf.set(
        right,
        bottom,
        Cell::with_attr(g.br, Attr::new(dark, opts.body_bg)),
    );
    // Título centrado entre esquinas + cierre integrado al marco en
    // `x+3..x+5` (2 celdas de aire tras la esquina: `┌──┤■├─`).
    draw_title_text(buf, rect, rect.w.saturating_sub(2), opts);
    if opts.controls && rect.w >= 8 {
        let (left, right) = match opts.border_style {
            BorderStyle::Double => ('\u{2561}', '\u{255e}'),
            _ => ('\u{2524}', '\u{251c}'),
        };
        super::icons20::win_close_framed(
            buf,
            rect.x.saturating_add(3),
            rect.y,
            left,
            right,
            Attr::new(bright, opts.title_bg),
            Color::Yellow,
        );
    }
}

/// Título centrado en la fila superior (`title_fg`/`title_bg`).
fn draw_title_text(buf: &mut Buffer, rect: Rect, width: u16, opts: &WindowOpts) {
    let fit = fit_text(&opts.title, width);
    let tw = visible_len(&fit);
    let tx = rect.x.saturating_add(rect.w.saturating_sub(tw) / 2);
    draw_text(buf, tx, rect.y, &fit, title_attr(opts));
}

/// Atributo de la barra de título.
fn title_attr(opts: &WindowOpts) -> Attr {
    if opts.title_bold {
        Attr::bold(opts.title_fg, opts.title_bg)
    } else {
        Attr::new(opts.title_fg, opts.title_bg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prim::darken_color;

    #[test]
    fn modal_has_teal_title_grey_body_chromatic_shadow() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 15, t.desktop);
        let r = Rect::new(5, 3, 30, 9);
        window(&mut b, r, &WindowOpts::modal("ORDENAR", t), t);
        // Modal = caja simple por defecto: esquinas CP437 (arriba sobre teal).
        assert_eq!(b.get(5, 3).unwrap().ch, '\u{250c}'); // ┌
        assert_eq!(b.get(5 + 30 - 1, 3).unwrap().ch, '\u{2510}'); // ┐
        assert_eq!(b.get(5, 3 + 9 - 1).unwrap().ch, '\u{2514}'); // └
        assert_eq!(b.get(5 + 30 - 1, 3 + 9 - 1).unwrap().ch, '\u{2518}'); // ┘
        assert_eq!(b.get(5, 3).unwrap().bg, t.teal);
        // Título centrado e incrustado en el marco (misma x que en plano).
        assert_eq!(b.get(5 + (30 - 7) / 2, 3).unwrap().ch, 'O');
        assert_eq!(b.get(6, 3).unwrap().bg, t.teal);
        // Cuerpo gris.
        assert_eq!(b.get(6, 5).unwrap().bg, Color::Grey);
        // Sombra neutra en (x+2, y+h): el cian del escritorio baja a
        // gris oscuro por luminancia, no a azul ni a negro sólido.
        let sh = b.get(7, 3 + 9).unwrap();
        assert_eq!(sh.bg, darken_color(t.desktop, 0.40));
        assert_ne!(sh.bg, Color::Black);
        assert!(sh.dim);
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
        // `┌──┤■├─`: 2 celdas de línea tras la esquina + cierre integrado.
        assert_eq!(b.get(6, 3).unwrap().ch, '\u{2500}'); // ─ aire
        assert_eq!(b.get(7, 3).unwrap().ch, '\u{2500}'); // ─ aire
        assert_eq!(b.get(8, 3).unwrap().ch, '\u{2524}'); // ┤
        assert_eq!(b.get(9, 3).unwrap().ch, '\u{25a0}'); // ■
        assert_eq!(b.get(9, 3).unwrap().fg, Color::Yellow);
        assert!(b.get(9, 3).unwrap().bold);
        assert_eq!(b.get(10, 3).unwrap().ch, '\u{251c}'); // ├
                                                          // Doble: `╞/╡` en vez de `├/┤`.
        let mut dbl = WindowOpts::modal("D", t);
        dbl.border_style = BorderStyle::Double;
        dbl.controls = true;
        window(&mut b, r, &dbl, t);
        assert_eq!(b.get(8, 3).unwrap().ch, '\u{2561}'); // ╡
        assert_eq!(b.get(9, 3).unwrap().ch, '\u{25a0}');
        assert_eq!(b.get(10, 3).unwrap().ch, '\u{255e}'); // ╞
                                                          // Borde opt-in tiñe los glifos del marco (fondo sigue del cuerpo).
        let mut bordered = WindowOpts::modal("B", t);
        bordered.border_color = Some(Color::Red);
        window(&mut b, r, &bordered, t);
        assert_eq!(b.get(5, 3).unwrap().ch, '\u{250c}');
        assert_eq!(b.get(5, 3).unwrap().fg, Color::Red);
        assert_eq!(b.get(5, 3).unwrap().bg, t.teal);
        assert_eq!(b.get(5 + 30 - 1, 3 + 9 - 1).unwrap().fg, Color::Red);
        assert_eq!(b.get(5 + 30 - 1, 3 + 9 - 1).unwrap().bg, Color::Grey);
        // Sin borde por defecto: la esquina es glifo simple sobre el cuerpo.
        window(&mut b, r, &WindowOpts::modal("B", t), t);
        assert_eq!(b.get(5, 3 + 9 - 1).unwrap().ch, '\u{2514}');
        assert_eq!(b.get(5, 3 + 9 - 1).unwrap().bg, Color::Grey);
    }

    #[test]
    fn single_bevel_and_none_styles() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 15, t.desktop);
        let r = Rect::new(5, 3, 30, 9);
        // Single: esquinas finas, título incrustado.
        let mut single = WindowOpts::modal("S", t);
        single.border_style = BorderStyle::Single;
        window(&mut b, r, &single, t);
        assert_eq!(b.get(5, 3).unwrap().ch, '\u{250c}'); // ┌
        assert_eq!(b.get(5 + 30 - 1, 3 + 9 - 1).unwrap().ch, '\u{2518}'); // ┘
        assert_eq!(b.get(6, 3).unwrap().ch, '\u{2500}'); // ─
        assert_eq!(b.get(5, 4).unwrap().ch, '\u{2502}'); // │
                                                         // Bevel3D: arriba/izquierda brillante, abajo/derecha oscuro.
        let mut bevel = WindowOpts::modal("V", t);
        bevel.border_style = BorderStyle::Bevel3D;
        window(&mut b, r, &bevel, t);
        assert_eq!(b.get(5, 3).unwrap().fg, Color::White);
        assert_eq!(b.get(5, 4).unwrap().fg, Color::White);
        assert_eq!(b.get(5, 3 + 9 - 1).unwrap().fg, Color::DarkGrey);
        assert_eq!(b.get(5 + 30 - 1, 3 + 9 - 1).unwrap().fg, Color::DarkGrey);
        // None: bloque plano clásico, esquina del cuerpo.
        let mut flat = WindowOpts::modal("F", t);
        flat.border_style = BorderStyle::None;
        window(&mut b, r, &flat, t);
        assert_eq!(b.get(5, 3).unwrap().ch, ' ');
        assert_eq!(b.get(5, 3).unwrap().bg, t.teal);
        assert_eq!(b.get(5, 3 + 9 - 1).unwrap().bg, Color::Grey);
    }

    #[test]
    fn presets_default_to_single() {
        let t = Theme::clipper();
        assert_eq!(WindowOpts::modal("M", t).border_style, BorderStyle::Single);
        assert_eq!(WindowOpts::form("F", t).border_style, BorderStyle::Single);
        assert_eq!(WindowOpts::dialog("D", t).border_style, BorderStyle::Single);
        assert_eq!(BorderStyle::default(), BorderStyle::None);
    }
}
