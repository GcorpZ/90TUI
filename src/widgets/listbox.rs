//! Caja de lista estática con scrollbar (`ListBox`).
//!
//! Colección visible con scroll y barra lateral derecha interactiva:
//! `\u{25b2}` arriba, `\u{25bc}` abajo, bloque `\u{2588}` proporcional,
//! pista `\u{2591}`. Navegación pura (`listbox_key`): flechas, `PgUp`/`PgDn`,
//! `Home`/`End` y filtrado rápido por letra.
//!
//! Expone los 4 parámetros globales (`foreground_color`,
//! `background_color`, `border_color`, `has_shadow`) más colores de
//! resaltado. Colores como `Color` (16 ANSI), no `(u8, u8, u8)`.

use crossterm::event::KeyCode;

use crate::core::{Attr, Buffer, Cell, Color, Rect};
use crate::prim::{draw_text, fit_text, shadow};

const ARROW_UP: char = '\u{25b2}';
const ARROW_DOWN: char = '\u{25bc}';
const THUMB: char = '\u{2588}';
const TRACK: char = '\u{2591}';

/// Caja de lista estática.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListBox {
    pub items: Vec<String>,
    pub selected: usize,
    /// Primera fila visible (scroll).
    pub top: usize,
    pub foreground_color: Color,
    pub background_color: Color,
    pub highlight_fg: Color,
    pub highlight_bg: Color,
    pub border_color: Option<Color>,
    pub has_shadow: bool,
    /// Ancho de la scrollbar (0 = sin barra).
    pub scrollbar_width: u16,
}

impl ListBox {
    pub fn new(items: &[&str]) -> Self {
        Self {
            items: items.iter().map(|s| s.to_string()).collect(),
            selected: 0,
            top: 0,
            foreground_color: Color::Black,
            background_color: Color::White,
            highlight_fg: Color::White,
            highlight_bg: Color::Navy,
            border_color: None,
            has_shadow: false,
            scrollbar_width: 1,
        }
    }

    fn clamp(&mut self) {
        if self.items.is_empty() {
            self.selected = 0;
            self.top = 0;
            return;
        }
        self.selected = self.selected.min(self.items.len() - 1);
        self.top = self.top.min(self.selected);
    }

    fn ensure_visible(&mut self, visible: usize) {
        let vis = visible.max(1);
        if self.selected < self.top {
            self.top = self.selected;
        } else if self.selected >= self.top + vis {
            self.top = self.selected + 1 - vis;
        }
    }
}

/// Filas de texto visibles dentro de `rect` (sin borde ni scrollbar).
pub fn listbox_visible(rect: Rect, bordered: bool) -> usize {
    let h = if bordered {
        rect.h.saturating_sub(2)
    } else {
        rect.h
    };
    h as usize
}

/// Dibuja la lista + scrollbar derecha.
pub fn listbox_draw(buf: &mut Buffer, rect: Rect, lb: &ListBox) {
    if rect.is_empty() {
        return;
    }
    let bordered = lb.border_color.is_some();
    if lb.has_shadow {
        shadow(buf, rect, crate::core::Theme::clipper());
    }
    buf.fill_rect(
        rect,
        Cell::new(' ', lb.foreground_color, lb.background_color),
    );
    let bar_w = if lb.scrollbar_width > 0 && rect.w >= 4 {
        1
    } else {
        0
    };
    let text_w = rect
        .w
        .saturating_sub(if bordered { 2 } else { 0 })
        .saturating_sub(bar_w);
    let rows = listbox_visible(rect, bordered);
    let x0 = rect.x.saturating_add(if bordered { 1 } else { 0 });
    let y0 = rect.y.saturating_add(if bordered { 1 } else { 0 });
    for row in 0..rows {
        let idx = lb.top + row;
        if idx >= lb.items.len() {
            break;
        }
        let y = y0.saturating_add(row as u16);
        let sel = idx == lb.selected;
        let (fg, bg) = if sel {
            (lb.highlight_fg, lb.highlight_bg)
        } else {
            (lb.foreground_color, lb.background_color)
        };
        buf.fill_rect(Rect::new(x0, y, text_w, 1), Cell::new(' ', fg, bg));
        draw_text(
            buf,
            x0.saturating_add(1),
            y,
            &fit_text(&lb.items[idx], text_w.saturating_sub(1)),
            Attr::new(fg, bg),
        );
    }
    // Scrollbar: `▲` / pista `░` + bloque `█` / `▼`.
    if bar_w > 0 && rows >= 3 {
        let bx = rect
            .right()
            .saturating_sub(1)
            .saturating_sub(if bordered { 1 } else { 0 });
        let track_top = y0.saturating_add(1);
        let track_h = (rows as u16).saturating_sub(2);
        buf.set(
            bx,
            y0,
            Cell::new(ARROW_UP, lb.foreground_color, lb.background_color),
        );
        buf.set(
            bx,
            y0.saturating_add(rows as u16).saturating_sub(1),
            Cell::new(ARROW_DOWN, lb.foreground_color, lb.background_color),
        );
        let thumb = thumb_row(lb, track_h);
        for i in 0..track_h {
            let ch = if i == thumb { THUMB } else { TRACK };
            buf.set(
                bx,
                track_top.saturating_add(i),
                Cell::new(ch, lb.foreground_color, lb.background_color),
            );
        }
    }
    if bordered {
        let b = lb.border_color.unwrap_or(Color::Black);
        let bc = Cell::new(' ', b, b);
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

/// Fila del bloque `█` dentro de la pista (proporcional a `top`).
fn thumb_row(lb: &ListBox, track_h: u16) -> u16 {
    let n = lb.items.len() as u32;
    if n <= 1 || track_h == 0 {
        return 0;
    }
    (lb.top as u32 * track_h as u32 / n.max(1)) as u16 % track_h.max(1)
}

/// Resultado de tecla en la lista.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListNav {
    Stay,
    Move(usize),
}

/// Navegación pura (testeable sin terminal).
pub fn listbox_key(lb: &mut ListBox, visible: usize, code: KeyCode) -> ListNav {
    use ListNav::*;
    let n = lb.items.len();
    if n == 0 {
        return Stay;
    }
    lb.clamp();
    let vis = visible.max(1);
    match code {
        KeyCode::Up => {
            lb.selected = (lb.selected + n - 1) % n;
            lb.ensure_visible(vis);
            Move(lb.selected)
        }
        KeyCode::Down => {
            lb.selected = (lb.selected + 1) % n;
            lb.ensure_visible(vis);
            Move(lb.selected)
        }
        KeyCode::Home => {
            lb.selected = 0;
            lb.ensure_visible(vis);
            Move(0)
        }
        KeyCode::End => {
            lb.selected = n - 1;
            lb.ensure_visible(vis);
            Move(n - 1)
        }
        KeyCode::PageUp => {
            lb.selected = lb.selected.saturating_sub(vis);
            lb.ensure_visible(vis);
            Move(lb.selected)
        }
        KeyCode::PageDown => {
            lb.selected = (lb.selected + vis).min(n - 1);
            lb.ensure_visible(vis);
            Move(lb.selected)
        }
        KeyCode::Char(c) => {
            let lc = c.to_ascii_lowercase();
            for k in 0..n {
                let i = (lb.selected + 1 + k) % n;
                if lb.items[i]
                    .chars()
                    .next()
                    .map(|f| f.to_ascii_lowercase() == lc)
                    .unwrap_or(false)
                {
                    lb.selected = i;
                    lb.ensure_visible(vis);
                    return Move(i);
                }
            }
            Stay
        }
        _ => Stay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Theme;

    fn lb5() -> ListBox {
        ListBox::new(&["Alpha", "Beta", "Gamma", "Delta", "Epsilon"])
    }

    #[test]
    fn nav_moves_pages_and_jumps() {
        let mut lb = lb5();
        assert_eq!(listbox_key(&mut lb, 3, KeyCode::Down), ListNav::Move(1));
        assert_eq!(listbox_key(&mut lb, 3, KeyCode::PageDown), ListNav::Move(4));
        assert_eq!(listbox_key(&mut lb, 3, KeyCode::PageUp), ListNav::Move(1));
        assert_eq!(listbox_key(&mut lb, 3, KeyCode::End), ListNav::Move(4));
        assert_eq!(listbox_key(&mut lb, 3, KeyCode::Home), ListNav::Move(0));
        assert_eq!(
            listbox_key(&mut lb, 3, KeyCode::Char('g')),
            ListNav::Move(2)
        );
        assert_eq!(listbox_key(&mut lb, 3, KeyCode::Up), ListNav::Move(1));
    }

    #[test]
    fn scroll_follows_selection() {
        let mut lb = lb5();
        listbox_key(&mut lb, 2, KeyCode::End);
        assert_eq!(lb.top, 3); // 4 visible en ventana de 2
        listbox_key(&mut lb, 2, KeyCode::Home);
        assert_eq!(lb.top, 0);
    }

    #[test]
    fn draws_highlight_and_scrollbar() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 10, t.desktop);
        let mut lb = lb5();
        lb.selected = 1;
        let r = Rect::new(2, 1, 16, 7);
        listbox_draw(&mut b, r, &lb);
        // Fila seleccionada resaltada.
        assert_eq!(b.get(3, 2).unwrap().bg, lb.highlight_bg);
        assert_eq!(b.get(3, 2).unwrap().ch, 'B');
        // Scrollbar: `▲` arriba, `▼` abajo, bloque en la pista.
        assert_eq!(b.get(17, 1).unwrap().ch, '\u{25b2}');
        assert_eq!(b.get(17, 7).unwrap().ch, '\u{25bc}');
        let mut blocks = 0;
        for y in 2..7u16 {
            if b.get(17, y).unwrap().ch == '\u{2588}' {
                blocks += 1;
            }
        }
        assert_eq!(blocks, 1);
    }
}
