//! Menú desplegable (`Dropdown`): línea cerrada + popup/overlay.
//!
//! * **Cerrado:** una línea con la opción seleccionada y flecha
//!   `\u{25bc}` a la derecha.
//! * **Abierto:** overlay vertical con las opciones, scroll limitado por
//!   `max_height`, cursor (hover) con `active_bg_color`. Captura el foco:
//!   `\u{2191}`/`\u{2193}` navegan, `Enter` acepta y cierra, `Esc` cancela
//!   y cierra, letra salta a la primera opción que empiece con ella.
//!
//! Nota de tipos: la especificación pedía colores `(u8, u8, u8)`, pero el
//! motor es de 16 colores ANSI (`Color`); los campos obligatorios usan
//! `Color` con los mismos nombres (`bg_color`, `fg_color`,
//! `active_bg_color`). `cursor`/`scroll_top` son estado interno extra.

use crossterm::event::KeyCode;

use crate::core::{Attr, Buffer, Cell, Color, Rect, Theme};
use crate::prim::{draw_text, shadow, visible_len};

/// Flecha del desplegable (cerrado/abierto).
const ARROW_DOWN: char = '\u{25bc}';
const ARROW_UP: char = '\u{25b2}';

/// Widget desplegable. Ver docs del módulo para el contrato.
/// Widget desplegable. Ver docs del módulo para el contrato.
///
/// `bg_color`/`fg_color` son los parámetros globales `background_color`/
/// `foreground_color`; `border_color` (`None` = sin marco) y `has_shadow`
/// completan los 4 parámetros globales.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dropdown {
    pub options: Vec<String>,
    pub selected_index: usize,
    pub is_open: bool,
    pub label: String,
    /// Filas visibles máximas del overlay (scroll si hay más).
    pub max_height: u16,
    pub bg_color: Color,
    pub fg_color: Color,
    /// Fondo de la fila bajo el cursor (hover).
    pub active_bg_color: Color,
    pub border_color: Option<Color>,
    pub has_shadow: bool,
    /// Cursor de navegación cuando está abierto.
    pub cursor: usize,
    /// Primera opción visible del overlay (scroll).
    pub scroll_top: usize,
}

impl Dropdown {
    /// Constructor con paleta popup Clipper por defecto.
    pub fn new(label: &str, options: &[&str]) -> Self {
        let t = Theme::clipper();
        Self {
            options: options.iter().map(|s| s.to_string()).collect(),
            selected_index: 0,
            is_open: false,
            label: label.to_string(),
            max_height: 8,
            bg_color: t.popup,
            fg_color: t.popup_text,
            active_bg_color: t.select_bg,
            border_color: None,
            has_shadow: true,
            cursor: 0,
            scroll_top: 0,
        }
    }

    /// Texto de la opción seleccionada (vacío si no hay opciones).
    pub fn selected(&self) -> &str {
        self.options
            .get(self.selected_index)
            .map(String::as_str)
            .unwrap_or("")
    }

    /// Abre el overlay (cursor = selección actual).
    pub fn open(&mut self) {
        if self.options.is_empty() {
            return;
        }
        self.cursor = self.selected_index.min(self.options.len() - 1);
        self.is_open = true;
        self.ensure_visible();
    }

    /// Cierra aceptando el cursor como selección.
    pub fn accept(&mut self) {
        if !self.options.is_empty() {
            self.selected_index = self.cursor.min(self.options.len() - 1);
        }
        self.is_open = false;
    }

    /// Cierra descartando (cursor vuelve a la selección).
    pub fn cancel(&mut self) {
        self.cursor = self
            .selected_index
            .min(self.options.len().saturating_sub(1));
        self.is_open = false;
    }

    fn ensure_visible(&mut self) {
        let h = self.max_height.max(1) as usize;
        if self.cursor < self.scroll_top {
            self.scroll_top = self.cursor;
        } else if self.cursor >= self.scroll_top + h {
            self.scroll_top = self.cursor + 1 - h;
        }
    }
}

/// Ancho sugerido = etiqueta + selección más larga + flecha + aire.
pub fn dropdown_size(dd: &Dropdown) -> (u16, u16) {
    let longest = dd.options.iter().map(|o| visible_len(o)).max().unwrap_or(0);
    let label_w = if dd.label.is_empty() {
        0
    } else {
        visible_len(&dd.label).saturating_add(2) // "Lbl: "
    };
    (label_w.saturating_add(longest).saturating_add(4).max(10), 1)
}

/// Rect del overlay abierto bajo `closed` (clampado a pantalla).
pub fn dropdown_overlay_rect(closed: Rect, dd: &Dropdown, screen: Rect) -> Rect {
    let visible = (dd.options.len() as u16).min(dd.max_height.max(1));
    let h = visible.saturating_add(2); // aire arriba/abajo
    Rect::popup_at(closed.x, closed.y.saturating_add(1), closed.w, h, screen)
}

/// Dibuja la línea cerrada y, si `is_open`, el overlay encima.
/// No toca la pila de pantallas (lo hace el llamante con `savescreen`).
pub fn dropdown_draw(buf: &mut Buffer, rect: Rect, dd: &Dropdown, theme: Theme) {
    if rect.is_empty() {
        return;
    }
    // Línea cerrada: fondo propio, texto izquierda, flecha derecha.
    buf.fill_rect(
        Rect::new(rect.x, rect.y, rect.w, 1),
        Cell::new(' ', dd.fg_color, dd.bg_color),
    );
    let arrow = if dd.is_open { ARROW_UP } else { ARROW_DOWN };
    let base = Attr::new(dd.fg_color, dd.bg_color);
    let inner_w = rect.w.saturating_sub(3); // aire + flecha + aire
    let mut line = String::new();
    if !dd.label.is_empty() {
        line.push_str(&dd.label);
        line.push_str(": ");
    }
    line.push_str(dd.selected());
    let fit = crate::prim::fit_text(&line, inner_w);
    draw_text(buf, rect.x.saturating_add(1), rect.y, &fit, base);
    buf.set(
        rect.x.saturating_add(rect.w).saturating_sub(2),
        rect.y,
        Cell::with_attr(arrow, base),
    );

    if !dd.is_open || dd.options.is_empty() {
        return;
    }
    // Overlay: lista vertical con scroll + sombra + borde opt-in.
    let ov = dropdown_overlay_rect(rect, dd, buf.bounds());
    if ov.is_empty() {
        return;
    }
    if dd.has_shadow {
        shadow(buf, ov, theme);
    }
    buf.fill_rect(ov, Cell::new(' ', dd.fg_color, dd.bg_color));
    let h = ov.h.saturating_sub(2);
    for row in 0..h {
        let idx = dd.scroll_top + row as usize;
        if idx >= dd.options.len() {
            break;
        }
        let y = ov.y.saturating_add(1).saturating_add(row);
        let hover = idx == dd.cursor;
        let bg = if hover {
            dd.active_bg_color
        } else {
            dd.bg_color
        };
        let fg = if hover { Color::Black } else { dd.fg_color };
        buf.fill_rect(
            Rect::new(ov.x.saturating_add(1), y, ov.w.saturating_sub(2), 1),
            Cell::new(' ', fg, bg),
        );
        let fit = crate::prim::fit_text(&dd.options[idx], ov.w.saturating_sub(4));
        draw_text(buf, ov.x.saturating_add(2), y, &fit, Attr::new(fg, bg));
    }
    if let Some(border) = dd.border_color {
        let bc = Cell::new(' ', border, border);
        for x in ov.x..ov.right() {
            buf.set(x, ov.y, bc);
            buf.set(x, ov.bottom().saturating_sub(1), bc);
        }
        for y in ov.y..ov.bottom() {
            buf.set(ov.x, y, bc);
            buf.set(ov.right().saturating_sub(1), y, bc);
        }
    }
}

/// Resultado de una tecla sobre el desplegable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropdownKey {
    Stay,
    Opened,
    Moved(usize),
    Accepted(usize),
    Cancelled,
}

/// Navegación pura (testeable sin terminal).
///
/// Cerrado: `Abajo`/`Enter`/`Espacio` abre; letra salta la selección.
/// Abierto: `Arriba`/`Abajo` mueve cursor (con scroll), `Enter` acepta y
/// cierra, `Esc` cancela y cierra, letra salta el cursor.
pub fn dropdown_key(dd: &mut Dropdown, code: KeyCode) -> DropdownKey {
    use DropdownKey::*;
    if dd.options.is_empty() {
        return Stay;
    }
    let n = dd.options.len();
    let clamp = |i: usize| i.min(n - 1);
    if !dd.is_open {
        match code {
            KeyCode::Down | KeyCode::Enter | KeyCode::Char(' ') => {
                dd.open();
                Opened
            }
            KeyCode::Char(c) => {
                let lc = c.to_ascii_lowercase();
                if let Some(i) = dd.options.iter().position(|o| {
                    o.chars()
                        .next()
                        .map(|f| f.to_ascii_lowercase() == lc)
                        .unwrap_or(false)
                }) {
                    dd.selected_index = i;
                    dd.cursor = i;
                    Moved(i)
                } else {
                    Stay
                }
            }
            _ => Stay,
        }
    } else {
        dd.cursor = clamp(dd.cursor);
        match code {
            KeyCode::Up => {
                // Circular: 0 → última. (cursor ya < n por el clamp de arriba;
                // sin clamp intermedio: `clamp(c + n - 1) % n` se congelaba).
                dd.cursor = (dd.cursor + n - 1) % n;
                dd.ensure_visible();
                Moved(dd.cursor)
            }
            KeyCode::Down => {
                dd.cursor = (dd.cursor + 1) % n;
                dd.ensure_visible();
                Moved(dd.cursor)
            }
            KeyCode::Home => {
                dd.cursor = 0;
                dd.ensure_visible();
                Moved(0)
            }
            KeyCode::End => {
                dd.cursor = n - 1;
                dd.ensure_visible();
                Moved(n - 1)
            }
            KeyCode::Enter => {
                dd.accept();
                Accepted(dd.selected_index)
            }
            KeyCode::Esc => {
                dd.cancel();
                Cancelled
            }
            KeyCode::Char(c) => {
                let lc = c.to_ascii_lowercase();
                // Busca desde el siguiente (rotando) para filtrado rápido.
                for k in 0..n {
                    let i = (dd.cursor + 1 + k) % n;
                    if dd.options[i]
                        .chars()
                        .next()
                        .map(|f| f.to_ascii_lowercase() == lc)
                        .unwrap_or(false)
                    {
                        dd.cursor = i;
                        dd.ensure_visible();
                        return Moved(i);
                    }
                }
                Stay
            }
            _ => Stay,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dd() -> Dropdown {
        let mut d = Dropdown::new("Depto", &["Ventas", "Compras", "Gerencia"]);
        d.max_height = 8;
        d
    }

    #[test]
    fn closed_draws_selected_and_arrow() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 10, t.desktop);
        let d = dd();
        let r = Rect::new(2, 1, 20, 1);
        dropdown_draw(&mut b, r, &d, t);
        // Etiqueta + selección a la izquierda, flecha a la derecha.
        assert_eq!(b.get(3, 1).unwrap().ch, 'D');
        assert_eq!(b.get(2 + 20 - 2, 1).unwrap().ch, '\u{25bc}');
        assert_eq!(b.get(3, 1).unwrap().bg, d.bg_color);
    }

    #[test]
    fn enter_opens_and_accepts() {
        let mut d = dd();
        assert_eq!(dropdown_key(&mut d, KeyCode::Enter), DropdownKey::Opened);
        assert!(d.is_open);
        assert_eq!(dropdown_key(&mut d, KeyCode::Down), DropdownKey::Moved(1));
        assert_eq!(
            dropdown_key(&mut d, KeyCode::Enter),
            DropdownKey::Accepted(1)
        );
        assert!(!d.is_open);
        assert_eq!(d.selected_index, 1);
        assert_eq!(d.selected(), "Compras");
    }

    #[test]
    fn up_moves_and_wraps_circularly() {
        let mut d = dd(); // 3 opciones
        dropdown_key(&mut d, KeyCode::Enter); // abre, cursor = 0
        assert_eq!(dropdown_key(&mut d, KeyCode::Down), DropdownKey::Moved(1));
        // Arriba desde 1 → 0 (antes: bug clamp saltaba a n-1 y se congelaba).
        assert_eq!(dropdown_key(&mut d, KeyCode::Up), DropdownKey::Moved(0));
        // Arriba desde 0 → última (circular).
        assert_eq!(dropdown_key(&mut d, KeyCode::Up), DropdownKey::Moved(2));
        // Y sigue navegando (no se congela).
        assert_eq!(dropdown_key(&mut d, KeyCode::Up), DropdownKey::Moved(1));
        assert_eq!(dropdown_key(&mut d, KeyCode::Down), DropdownKey::Moved(2));
    }

    #[test]
    fn esc_cancels_without_changing_selection() {
        let mut d = dd();
        dropdown_key(&mut d, KeyCode::Enter);
        dropdown_key(&mut d, KeyCode::Down);
        assert_eq!(dropdown_key(&mut d, KeyCode::Esc), DropdownKey::Cancelled);
        assert!(!d.is_open);
        assert_eq!(d.selected_index, 0);
    }

    #[test]
    fn letter_jumps_to_match() {
        let mut d = dd();
        dropdown_key(&mut d, KeyCode::Enter); // abre
        assert_eq!(
            dropdown_key(&mut d, KeyCode::Char('g')),
            DropdownKey::Moved(2)
        );
        // Cerrado también filtra la selección directa.
        let mut d2 = dd();
        assert_eq!(
            dropdown_key(&mut d2, KeyCode::Char('c')),
            DropdownKey::Moved(1)
        );
        assert_eq!(d2.selected_index, 1);
    }

    #[test]
    fn open_draws_overlay_with_hover_and_scroll() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 12, t.desktop);
        let mut d = Dropdown::new("", &["A", "B", "C", "D", "E"]);
        d.max_height = 2;
        d.bg_color = Color::Blue;
        d.fg_color = Color::White;
        d.active_bg_color = Color::Grey;
        dropdown_key(&mut d, KeyCode::Enter);
        dropdown_key(&mut d, KeyCode::Down); // cursor 1
        dropdown_key(&mut d, KeyCode::Down); // cursor 2 → scroll
        assert_eq!(d.scroll_top, 1);
        let r = Rect::new(2, 1, 16, 1);
        dropdown_draw(&mut b, r, &d, t);
        // Overlay bajo la línea: filas visibles con hover en cursor.
        let ov = dropdown_overlay_rect(r, &d, Rect::new(0, 0, 40, 12));
        assert_eq!(ov.h, 4); // 2 visibles + aire
        assert_eq!(b.get(ov.x + 2, ov.y + 2).unwrap().bg, d.active_bg_color);
    }

    #[test]
    fn empty_stays_quiet() {
        let mut d = Dropdown::new("X", &[]);
        assert_eq!(dropdown_key(&mut d, KeyCode::Enter), DropdownKey::Stay);
        assert_eq!(dropdown_key(&mut d, KeyCode::Down), DropdownKey::Stay);
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 8, t.desktop);
        dropdown_draw(&mut b, Rect::new(1, 1, 12, 1), &d, t);
        assert_eq!(b.get(11, 1).unwrap().ch, '\u{25bc}');
    }
}
