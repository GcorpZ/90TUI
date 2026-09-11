//! Contenedor de pestañas tipo archivador (`TabControl`).
//!
//! Cada pestaña es una orejeta de carpeta en la tira superior (o lateral
//! en modo `Left`); la página activa se abre a una tarjeta con marco
//! CP437 de línea simple (`┌─┐│└┘`). La pestaña activa (blanco sobre
//! azul, alto contraste) queda conectada al interior: la línea superior
//! del marco se interrumpe bajo ella, mientras las inactivas
//! (`│ label │` tenue) se asientan con `┴` sobre el marco.
//!
//! Cada página expone un viewport de coordenadas locales: `tab_viewport()`
//! calcula el área interna útil del marco — en `Top` estrictamente
//! `(x+1, y+2, w-2, h-3)` — así los widgets se posicionan relativos a él
//! sin tocar las líneas de la caja. Expone los 4 parámetros globales
//! (`foreground_color`, `background_color`, `border_color`, `has_shadow`;
//! `border_color` tiñe el marco, `None` = tinta del primer plano).

use crossterm::event::KeyCode;

use crate::core::{Attr, Buffer, Cell, Color, Rect};
use crate::prim::{draw_text, fit_text, shadow, visible_len};

/// Dónde se renderiza la tira de pestañas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TabPosition {
    /// Arriba, horizontal.
    #[default]
    Top,
    /// A la izquierda, vertical.
    Left,
}

/// Una pestaña: etiqueta + colores propios (reservados; el render CUA
/// usa paleta fija: inactiva tenue, activa blanco/azul).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tab {
    pub label: String,
    pub fg: Color,
    pub bg: Color,
}

impl Tab {
    pub fn new(label: &str, fg: Color, bg: Color) -> Self {
        Self {
            label: label.to_string(),
            fg,
            bg,
        }
    }
}

/// Contenedor de pestañas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabControl {
    pub tabs: Vec<Tab>,
    pub active: usize,
    pub position: TabPosition,
    pub foreground_color: Color,
    pub background_color: Color,
    pub border_color: Option<Color>,
    pub has_shadow: bool,
}

impl TabControl {
    pub fn new(labels: &[&str], position: TabPosition, theme: crate::core::Theme) -> Self {
        Self {
            tabs: labels
                .iter()
                .map(|l| Tab::new(l, theme.popup_text, theme.popup))
                .collect(),
            active: 0,
            position,
            foreground_color: theme.popup_text,
            background_color: theme.popup,
            border_color: None,
            has_shadow: false,
        }
    }

    pub fn add_tab(&mut self, label: &str, fg: Color, bg: Color) -> usize {
        self.tabs.push(Tab::new(label, fg, bg));
        self.tabs.len() - 1
    }

    pub fn set_active(&mut self, idx: usize) {
        if !self.tabs.is_empty() {
            self.active = idx.min(self.tabs.len() - 1);
        }
    }

    /// Etiqueta de la página activa: nombre del ámbito de foco
    /// (`FocusManager::set_active`) por convención.
    pub fn active_scope(&self) -> &str {
        self.tabs
            .get(self.active)
            .map(|t| t.label.as_str())
            .unwrap_or("")
    }

    /// Ancho de la columna de pestañas en modo `Left`.
    pub fn strip_width(&self) -> u16 {
        self.tabs
            .iter()
            .map(|t| visible_len(&t.label))
            .max()
            .unwrap_or(0)
            .saturating_add(4)
    }
}

/// Área interna útil para los widgets de la página activa (viewport local:
/// el origen `(0,0)` de la página equivale a su esquina superior-izquierda).
/// En `Top` es estrictamente `(x+1, y+2, w-2, h-3)`: tras la tira (fila 0)
/// y la línea superior del marco (fila 1), entre paredes y sobre el fondo.
pub fn tab_viewport(container: Rect, tabs: &TabControl) -> Rect {
    match tabs.position {
        TabPosition::Top => Rect::new(
            container.x.saturating_add(1),
            container.y.saturating_add(2),
            container.w.saturating_sub(2),
            container.h.saturating_sub(3),
        ),
        TabPosition::Left => {
            let sw = tabs.strip_width().min(container.w);
            Rect::new(
                container.x.saturating_add(sw).saturating_add(1),
                container.y.saturating_add(1),
                container.w.saturating_sub(sw).saturating_sub(2),
                container.h.saturating_sub(2),
            )
        }
    }
}

/// Dibuja tira + tarjeta con marco (los widgets los pinta el llamante en
/// el viewport). Devuelve el viewport. Sin bordes planos: el marco es
/// siempre caja CP437 de línea simple.
pub fn tab_draw(buf: &mut Buffer, rect: Rect, tabs: &TabControl) -> Rect {
    if rect.is_empty() || tabs.tabs.is_empty() {
        return Rect::new(rect.x, rect.y, 0, 0);
    }
    if tabs.has_shadow {
        shadow(buf, rect, crate::core::Theme::clipper());
    }
    let ink = Attr::new(
        tabs.border_color.unwrap_or(tabs.foreground_color),
        tabs.background_color,
    );
    let active = tabs.active.min(tabs.tabs.len() - 1);
    match tabs.position {
        TabPosition::Top => draw_top(buf, rect, tabs, active, ink),
        TabPosition::Left => draw_left(buf, rect, tabs, active, ink),
    }
    tab_viewport(rect, tabs)
}

/// Tira horizontal + tarjeta. La activa (`│ label │` blanco/azul) abre el
/// marco bajo ella; las inactivas (`│ label │` tenue) asientan con `┴`.
fn draw_top(buf: &mut Buffer, rect: Rect, tabs: &TabControl, active: usize, ink: Attr) {
    let bg = tabs.background_color;
    // Base: tira + interior en el fondo de la tarjeta.
    buf.fill_rect(rect, Cell::new(' ', tabs.foreground_color, bg));
    // Orejetas desde x+1 (última celda libre por simetría).
    let mut cx = rect.x.saturating_add(1);
    let mut active_span: Option<(u16, u16)> = None;
    let mut idle_edges: Vec<u16> = Vec::new();
    for (i, tab) in tabs.tabs.iter().enumerate() {
        let inner = format!(" {} ", tab.label);
        let w = visible_len(&inner).saturating_add(2);
        if cx.saturating_add(w) > rect.right().saturating_sub(1) {
            break;
        }
        let a = if i == active {
            active_span = Some((cx, w));
            Attr::bold(Color::Navy, Color::White)
        } else {
            idle_edges.push(cx);
            idle_edges.push(cx.saturating_add(w).saturating_sub(1));
            Attr::new(Color::Grey, Color::DarkGrey)
        };
        buf.set(cx, rect.y, Cell::with_attr('\u{2502}', a));
        draw_text(
            buf,
            cx.saturating_add(1),
            rect.y,
            &fit_text(&inner, w.saturating_sub(2)),
            a,
        );
        buf.set(
            cx.saturating_add(w).saturating_sub(1),
            rect.y,
            Cell::with_attr('\u{2502}', a),
        );
        cx = cx.saturating_add(w).saturating_add(1);
    }
    if rect.h < 2 {
        return;
    }
    // Línea superior del marco con la apertura de la activa.
    let by = rect.y.saturating_add(1);
    buf.set(rect.x, by, Cell::with_attr('\u{250c}', ink));
    buf.set(
        rect.right().saturating_sub(1),
        by,
        Cell::with_attr('\u{2510}', ink),
    );
    for x in rect.x.saturating_add(1)..rect.right().saturating_sub(1) {
        let g = if let Some((ax, aw)) = active_span {
            if x == ax || x == ax.saturating_add(aw).saturating_sub(1) {
                '\u{2502}' // paredes de la apertura
            } else if x > ax && x < ax.saturating_add(aw).saturating_sub(1) {
                continue; // interior abierto al contenido
            } else if idle_edges.contains(&x) {
                '\u{2534}'
            } else {
                '\u{2500}'
            }
        } else if idle_edges.contains(&x) {
            '\u{2534}'
        } else {
            '\u{2500}'
        };
        buf.set(x, by, Cell::with_attr(g, ink));
    }
    draw_card_body(buf, rect, by, ink);
}

/// Tira vertical + tarjeta a la derecha. La fila activa (blanco/azul)
/// se prolonga 1 celda hacia el marco como apertura.
fn draw_left(buf: &mut Buffer, rect: Rect, tabs: &TabControl, active: usize, ink: Attr) {
    let bg = tabs.background_color;
    let sw = tabs.strip_width().min(rect.w);
    // Banda de tira apagada + interior en fondo de tarjeta.
    buf.fill_rect(
        Rect::new(rect.x, rect.y, sw, rect.h),
        Cell::new(' ', Color::Grey, Color::DarkGrey),
    );
    if sw < rect.w {
        buf.fill_rect(
            Rect::new(
                rect.x.saturating_add(sw),
                rect.y,
                rect.w.saturating_sub(sw),
                rect.h,
            ),
            Cell::new(' ', tabs.foreground_color, bg),
        );
    }
    let mut ay: Option<u16> = None;
    let mut cy = rect.y.saturating_add(1);
    for (i, tab) in tabs.tabs.iter().enumerate() {
        if cy >= rect.bottom().saturating_sub(1) {
            break;
        }
        if i == active {
            ay = Some(cy);
            buf.fill_rect(
                Rect::new(rect.x, cy, sw.saturating_add(1).min(rect.w), 1),
                Cell::new(' ', Color::Navy, Color::White),
            );
            draw_text(
                buf,
                rect.x.saturating_add(2),
                cy,
                &fit_text(&tab.label, sw.saturating_sub(3)),
                Attr::bold(Color::Navy, Color::White),
            );
        } else {
            draw_text(
                buf,
                rect.x.saturating_add(2),
                cy,
                &fit_text(&tab.label, sw.saturating_sub(3)),
                Attr::new(Color::Grey, Color::DarkGrey),
            );
        }
        cy = cy.saturating_add(1);
    }
    // Marco desde x+sw (con apertura en la fila activa).
    let fx = rect.x.saturating_add(sw);
    if rect.w.saturating_sub(sw) < 2 || rect.h < 2 {
        return;
    }
    let right = rect.right().saturating_sub(1);
    let bottom = rect.bottom().saturating_sub(1);
    buf.set(
        rect.x.saturating_add(sw),
        rect.y,
        Cell::with_attr('\u{250c}', ink),
    );
    buf.set(right, rect.y, Cell::with_attr('\u{2510}', ink));
    for x in fx.saturating_add(1)..right {
        buf.set(x, rect.y, Cell::with_attr('\u{2500}', ink));
    }
    for y in rect.y.saturating_add(1)..bottom {
        if Some(y) == ay {
            continue; // apertura: el blanco activo entra a la tarjeta
        }
        buf.set(fx, y, Cell::with_attr('\u{2502}', ink));
        buf.set(right, y, Cell::with_attr('\u{2502}', ink));
    }
    buf.set(fx, bottom, Cell::with_attr('\u{2514}', ink));
    buf.set(right, bottom, Cell::with_attr('\u{2518}', ink));
    for x in fx.saturating_add(1)..right {
        buf.set(x, bottom, Cell::with_attr('\u{2500}', ink));
    }
}

/// Paredes laterales + fondo para la tarjeta `Top` (la superior ya va).
fn draw_card_body(buf: &mut Buffer, rect: Rect, top_row: u16, ink: Attr) {
    let bottom = rect.bottom().saturating_sub(1);
    let right = rect.right().saturating_sub(1);
    for y in top_row.saturating_add(1)..bottom {
        buf.set(rect.x, y, Cell::with_attr('\u{2502}', ink));
        buf.set(right, y, Cell::with_attr('\u{2502}', ink));
    }
    if bottom > top_row {
        buf.set(rect.x, bottom, Cell::with_attr('\u{2514}', ink));
        buf.set(right, bottom, Cell::with_attr('\u{2518}', ink));
        for x in rect.x.saturating_add(1)..right {
            buf.set(x, bottom, Cell::with_attr('\u{2500}', ink));
        }
    }
}

/// Navegación pura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabNav {
    Stay,
    Move(usize),
}

/// `←→` (Top) o `↑↓` (Left) cambian de página (circular); `Home/End` van a
/// los extremos. `Tab` también avanza (circular).
pub fn tab_key(tabs: &mut TabControl, code: KeyCode) -> TabNav {
    use TabNav::*;
    let n = tabs.tabs.len();
    if n == 0 {
        return Stay;
    }
    tabs.active = tabs.active.min(n - 1);
    let fwd = (tabs.active + 1) % n;
    let back = (tabs.active + n - 1) % n;
    match (tabs.position, code) {
        (_, KeyCode::Home) => {
            tabs.active = 0;
            Move(0)
        }
        (_, KeyCode::End) => {
            tabs.active = n - 1;
            Move(n - 1)
        }
        (TabPosition::Top, KeyCode::Left) => {
            tabs.active = back;
            Move(back)
        }
        (TabPosition::Top, KeyCode::Right) | (_, KeyCode::Tab) => {
            tabs.active = fwd;
            Move(fwd)
        }
        (TabPosition::Left, KeyCode::Up) => {
            tabs.active = back;
            Move(back)
        }
        (TabPosition::Left, KeyCode::Down) => {
            tabs.active = fwd;
            Move(fwd)
        }
        _ => Stay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Theme;

    fn tc() -> TabControl {
        let t = Theme::clipper();
        let mut c = TabControl::new(&["General", "Inventario"], TabPosition::Top, t);
        c.add_tab("Ayuda", Color::Yellow, Color::Navy);
        c
    }

    #[test]
    fn viewport_top_and_left() {
        let c = tc();
        let r = Rect::new(10, 5, 50, 15);
        // Top estricto: (x+1, y+2, w-2, h-3).
        let v = tab_viewport(r, &c);
        assert_eq!((v.x, v.y, v.w, v.h), (11, 7, 48, 12));
        let mut left = tc();
        left.position = TabPosition::Left;
        let sw = left.strip_width(); // "Inventario" (10) + 4
        assert_eq!(sw, 14);
        let v2 = tab_viewport(r, &left);
        assert_eq!((v2.x, v2.y, v2.w, v2.h), (10 + 14 + 1, 6, 50 - 14 - 2, 13));
    }

    #[test]
    fn nav_respects_position_and_wraps() {
        let mut c = tc(); // Top
        assert_eq!(tab_key(&mut c, KeyCode::Right), TabNav::Move(1));
        assert_eq!(tab_key(&mut c, KeyCode::Up), TabNav::Stay); // Top ignora Up
        assert_eq!(tab_key(&mut c, KeyCode::Left), TabNav::Move(0));
        assert_eq!(tab_key(&mut c, KeyCode::Left), TabNav::Move(2)); // circular
        c.position = TabPosition::Left;
        assert_eq!(tab_key(&mut c, KeyCode::Down), TabNav::Move(0));
        assert_eq!(tab_key(&mut c, KeyCode::Down), TabNav::Move(1));
        assert_eq!(tab_key(&mut c, KeyCode::End), TabNav::Move(2));
        assert_eq!(tab_key(&mut c, KeyCode::Home), TabNav::Move(0));
    }

    #[test]
    fn card_has_ears_frame_and_opening() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(60, 10, t.desktop);
        let mut c = TabControl::new(&["Uno", "Dos"], TabPosition::Top, t);
        c.active = 1;
        let v = tab_draw(&mut b, Rect::new(2, 1, 40, 8), &c);
        // Viewport estricto (x+1, y+2, w-2, h-3).
        assert_eq!((v.x, v.y, v.w, v.h), (3, 3, 38, 5));
        // Orejeta inactiva tenue: `│ Uno │` en gris sobre apagado.
        assert_eq!(b.get(3, 1).unwrap().ch, '\u{2502}');
        assert_eq!(b.get(5, 1).unwrap().ch, 'U');
        assert_eq!(b.get(5, 1).unwrap().fg, Color::Grey);
        assert_eq!(b.get(3, 1).unwrap().bg, Color::DarkGrey);
        // Orejeta activa en alto contraste: `│ Dos │` azul sobre blanco.
        assert_eq!(b.get(11, 1).unwrap().ch, '\u{2502}');
        assert_eq!(b.get(11, 1).unwrap().bg, Color::White);
        assert_eq!(b.get(13, 1).unwrap().ch, 'D');
        assert_eq!(b.get(13, 1).unwrap().fg, Color::Navy);
        assert!(b.get(13, 1).unwrap().bold);
        // Marco: `┌` en (2,2), `┐` en (41,2), `└` en (2,8).
        assert_eq!(b.get(2, 2).unwrap().ch, '\u{250c}');
        assert_eq!(b.get(41, 2).unwrap().ch, '\u{2510}');
        assert_eq!(b.get(2, 8).unwrap().ch, '\u{2514}');
        // Apertura bajo la activa (│ en bordes, aire dentro)...
        assert_eq!(b.get(11, 2).unwrap().ch, '\u{2502}');
        assert_eq!(b.get(17, 2).unwrap().ch, '\u{2502}');
        assert_eq!(b.get(14, 2).unwrap().ch, ' ');
        // ...y `┴` bajo la inactiva.
        assert_eq!(b.get(3, 2).unwrap().ch, '\u{2534}');
        assert_eq!(b.get(9, 2).unwrap().ch, '\u{2534}');
        // Pared lateral e interior en fondo de tarjeta.
        assert_eq!(b.get(2, 4).unwrap().ch, '\u{2502}');
        assert_eq!(b.get(20, 5).unwrap().bg, t.popup);
    }

    #[test]
    fn frame_uses_border_color_as_ink() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(60, 10, t.desktop);
        let mut c = TabControl::new(&["Uno"], TabPosition::Top, t);
        c.border_color = Some(Color::Red);
        tab_draw(&mut b, Rect::new(2, 1, 30, 6), &c);
        assert_eq!(b.get(2, 2).unwrap().fg, Color::Red);
        assert_eq!(b.get(2, 2).unwrap().ch, '\u{250c}');
    }

    #[test]
    fn left_strip_opens_active_row() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(60, 12, t.desktop);
        let mut c = TabControl::new(&["Uno", "Dos"], TabPosition::Left, t);
        c.active = 0;
        let sw = c.strip_width(); // "Uno"(3)/"Dos"(3) + 4 = 7
        assert_eq!(sw, 7);
        let v = tab_draw(&mut b, Rect::new(2, 1, 30, 8), &c);
        assert_eq!((v.x, v.y, v.w, v.h), (2 + sw + 1, 2, 30 - sw - 2, 6));
        // Fila activa (y=2) en blanco hasta la columna del marco...
        assert_eq!(b.get(2 + sw, 2).unwrap().bg, Color::White);
        // ...y la fila inactiva (y=3) conserva la pared.
        assert_eq!(b.get(2 + sw, 3).unwrap().ch, '\u{2502}');
        assert_eq!(b.get(3, 2).unwrap().fg, Color::Navy);
    }
}
