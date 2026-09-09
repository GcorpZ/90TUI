//! Contenedor de pestañas (`TabControl`).
//!
//! Pestañas arriba (`Top`, horizontal) o al lado (`Left`, vertical), con
//! colores propios por pestaña (`fg`/`bg`). Cada página expone un viewport
//! de coordenadas locales: `tab_viewport()` calcula el área interna útil
//! restante del contenedor, así los widgets de la página se posicionan
//! relativos a él sin matemática manual.
//! Expone los 4 parámetros globales (`foreground_color`,
//! `background_color`, `border_color`, `has_shadow`).

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

/// Una pestaña: etiqueta + colores propios.
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
pub fn tab_viewport(container: Rect, tabs: &TabControl) -> Rect {
    let inset = if tabs.border_color.is_some() { 1 } else { 0 };
    match tabs.position {
        TabPosition::Top => Rect::new(
            container.x.saturating_add(inset),
            container.y.saturating_add(1).saturating_add(inset),
            container.w.saturating_sub(inset * 2),
            container.h.saturating_sub(1).saturating_sub(inset * 2),
        ),
        TabPosition::Left => {
            let sw = tabs.strip_width().min(container.w);
            Rect::new(
                container.x.saturating_add(sw).saturating_add(inset),
                container.y.saturating_add(inset),
                container.w.saturating_sub(sw).saturating_sub(inset * 2),
                container.h.saturating_sub(inset * 2),
            )
        }
    }
}

/// Dibuja tira + fondo de página (los widgets los pinta el llamante en el
/// viewport). Devuelve el viewport.
pub fn tab_draw(buf: &mut Buffer, rect: Rect, tabs: &TabControl) -> Rect {
    if rect.is_empty() || tabs.tabs.is_empty() {
        return Rect::new(rect.x, rect.y, 0, 0);
    }
    if tabs.has_shadow {
        shadow(buf, rect, crate::core::Theme::clipper());
    }
    buf.fill_rect(
        rect,
        Cell::new(' ', tabs.foreground_color, tabs.background_color),
    );
    match tabs.position {
        TabPosition::Top => {
            let mut cx = rect.x.saturating_add(1);
            for (i, tab) in tabs.tabs.iter().enumerate() {
                let label = format!(" {} ", tab.label);
                let w = visible_len(&label).saturating_add(2);
                if cx.saturating_add(w) > rect.right() {
                    break;
                }
                let (fg, bg) = if i == tabs.active {
                    (tab.bg, tab.fg) // activa invertida
                } else {
                    (tab.fg, tab.bg)
                };
                buf.fill_rect(Rect::new(cx, rect.y, w, 1), Cell::new(' ', fg, bg));
                draw_text(
                    buf,
                    cx.saturating_add(1),
                    rect.y,
                    &fit_text(&label, w.saturating_sub(2)),
                    Attr::bold(fg, bg),
                );
                cx = cx.saturating_add(w).saturating_add(1);
            }
        }
        TabPosition::Left => {
            let sw = tabs.strip_width().min(rect.w);
            let mut cy = rect.y.saturating_add(1);
            for (i, tab) in tabs.tabs.iter().enumerate() {
                if cy >= rect.bottom() {
                    break;
                }
                let (fg, bg) = if i == tabs.active {
                    (tab.bg, tab.fg)
                } else {
                    (tab.fg, tab.bg)
                };
                buf.fill_rect(Rect::new(rect.x, cy, sw, 1), Cell::new(' ', fg, bg));
                draw_text(
                    buf,
                    rect.x.saturating_add(2),
                    cy,
                    &fit_text(&tab.label, sw.saturating_sub(3)),
                    Attr::bold(fg, bg),
                );
                cy = cy.saturating_add(1);
            }
        }
    }
    if let Some(border) = tabs.border_color {
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
    tab_viewport(rect, tabs)
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
        let v = tab_viewport(r, &c);
        assert_eq!((v.x, v.y, v.w, v.h), (10, 6, 50, 14)); // fila de tira arriba
        let mut left = tc();
        left.position = TabPosition::Left;
        let sw = left.strip_width();
        let v2 = tab_viewport(r, &left);
        assert_eq!((v2.x, v2.w), (10 + sw, 50 - sw));
        assert_eq!((v2.y, v2.h), (5, 15));
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
    fn draws_active_inverted() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(60, 10, t.desktop);
        let mut c = TabControl::new(&["Uno", "Dos"], TabPosition::Top, t);
        c.active = 1;
        let v = tab_draw(&mut b, Rect::new(2, 1, 40, 8), &c);
        assert_eq!((v.y, v.h), (2, 7)); // viewport bajo la tira
                                        // Activa invertida: fondo = fg del tab.
        assert_eq!(b.get(3, 1).unwrap().bg, t.popup); // "Uno" normal
        let x2 = 3 + visible_len(" Uno ") + 2 + 1;
        assert_eq!(b.get(x2, 1).unwrap().bg, t.popup_text); // "Dos" invertida
    }
}
