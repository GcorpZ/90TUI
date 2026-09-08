//! Campo memo multilínea (`TextArea`).
//!
//! Notas o descripciones largas: `Enter` inserta salto de línea, scroll
//! vertical automático, límite máximo de caracteres y navegación interna
//! con flechas (`↑↓←→`, `Home`/`End`, `PgUp`/`PgDn`).
//! Expone los 4 parámetros globales (`foreground_color`,
//! `background_color`, `border_color`, `has_shadow`).

use crossterm::event::KeyCode;

use crate::core::{Attr, Buffer, Cell, Color, Rect};
use crate::prim::{draw_text, shadow};

/// Memo multilínea. El texto vive en `lines` (sin `\n` incluidos).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextArea {
    pub lines: Vec<String>,
    /// Cursor en (fila, columna en caracteres).
    pub cursor_row: usize,
    pub cursor_col: usize,
    /// Máximo de caracteres totales (`usize::MAX` = sin límite).
    pub max_chars: usize,
    /// Primera fila visible (scroll vertical).
    pub scroll_top: usize,
    pub foreground_color: Color,
    pub background_color: Color,
    pub active_background_color: Color,
    pub border_color: Option<Color>,
    pub has_shadow: bool,
}

impl TextArea {
    pub fn new(max_chars: usize) -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            max_chars,
            scroll_top: 0,
            foreground_color: Color::Black,
            background_color: Color::White,
            active_background_color: Color::Grey,
            border_color: None,
            has_shadow: false,
        }
    }

    pub fn with_text(text: &str, max_chars: usize) -> Self {
        let mut a = Self::new(max_chars);
        a.lines = text.split('\n').map(|s| s.to_string()).collect();
        if a.lines.is_empty() {
            a.lines.push(String::new());
        }
        a
    }

    /// Texto completo con `\n`.
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn char_count(&self) -> usize {
        self.lines.iter().map(|l| l.chars().count()).sum::<usize>()
            + self.lines.len().saturating_sub(1)
    }

    fn clamp_cursor(&mut self) {
        self.cursor_row = self.cursor_row.min(self.lines.len().saturating_sub(1));
        let len = self.lines[self.cursor_row].chars().count();
        self.cursor_col = self.cursor_col.min(len);
    }

    fn ensure_visible(&mut self, visible: usize) {
        let vis = visible.max(1);
        if self.cursor_row < self.scroll_top {
            self.scroll_top = self.cursor_row;
        } else if self.cursor_row >= self.scroll_top + vis {
            self.scroll_top = self.cursor_row + 1 - vis;
        }
    }
}

/// Filas visibles de texto dentro de `rect`.
pub fn textarea_visible(rect: Rect, bordered: bool) -> usize {
    (if bordered {
        rect.h.saturating_sub(2)
    } else {
        rect.h
    }) as usize
}

/// Dibuja el memo (cursor invertido con foco).
pub fn textarea_draw(buf: &mut Buffer, rect: Rect, area: &TextArea, focused: bool) {
    if rect.is_empty() {
        return;
    }
    let bordered = area.border_color.is_some();
    if area.has_shadow {
        shadow(buf, rect, crate::core::Theme::clipper());
    }
    let bg = if focused {
        area.active_background_color
    } else {
        area.background_color
    };
    buf.fill_rect(rect, Cell::new(' ', area.foreground_color, bg));
    let x0 = rect.x.saturating_add(if bordered { 1 } else { 0 });
    let y0 = rect.y.saturating_add(if bordered { 1 } else { 0 });
    let w = rect.w.saturating_sub(if bordered { 2 } else { 0 });
    let rows = textarea_visible(rect, bordered);
    let base = Attr::new(area.foreground_color, bg);
    for row in 0..rows {
        let idx = area.scroll_top + row;
        if idx >= area.lines.len() {
            break;
        }
        draw_text(
            buf,
            x0.saturating_add(1),
            y0.saturating_add(row as u16),
            &crate::prim::fit_text(&area.lines[idx], w.saturating_sub(1)),
            base,
        );
    }
    if focused {
        let cx = x0.saturating_add(1).saturating_add(area.cursor_col as u16);
        let cy = y0.saturating_add(area.cursor_row.saturating_sub(area.scroll_top) as u16);
        if cx < rect.right() && cy < rect.bottom() {
            let ch = area.lines[area.cursor_row]
                .chars()
                .nth(area.cursor_col)
                .unwrap_or(' ');
            buf.set(
                cx,
                cy,
                Cell::with_attr(ch, Attr::new(bg, area.foreground_color)),
            );
        }
    }
    if bordered {
        let b = area.border_color.unwrap_or(Color::Black);
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

/// Resultado de tecla en el memo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAreaKey {
    Stay,
    Moved,
    Changed,
}

/// Edición pura multilínea.
pub fn textarea_key(area: &mut TextArea, visible: usize, code: KeyCode) -> TextAreaKey {
    use TextAreaKey::*;
    if area.lines.is_empty() {
        area.lines.push(String::new());
    }
    area.clamp_cursor();
    match code {
        KeyCode::Left => {
            if area.cursor_col > 0 {
                area.cursor_col -= 1;
            } else if area.cursor_row > 0 {
                area.cursor_row -= 1;
                area.cursor_col = area.lines[area.cursor_row].chars().count();
            } else {
                return Stay;
            }
            area.ensure_visible(visible);
            Moved
        }
        KeyCode::Right => {
            let len = area.lines[area.cursor_row].chars().count();
            if area.cursor_col < len {
                area.cursor_col += 1;
            } else if area.cursor_row + 1 < area.lines.len() {
                area.cursor_row += 1;
                area.cursor_col = 0;
            } else {
                return Stay;
            }
            area.ensure_visible(visible);
            Moved
        }
        KeyCode::Up => {
            if area.cursor_row == 0 {
                return Stay;
            }
            area.cursor_row -= 1;
            area.clamp_cursor();
            area.ensure_visible(visible);
            Moved
        }
        KeyCode::Down => {
            if area.cursor_row + 1 >= area.lines.len() {
                return Stay;
            }
            area.cursor_row += 1;
            area.clamp_cursor();
            area.ensure_visible(visible);
            Moved
        }
        KeyCode::Home => {
            area.cursor_col = 0;
            Moved
        }
        KeyCode::End => {
            area.cursor_col = area.lines[area.cursor_row].chars().count();
            Moved
        }
        KeyCode::PageUp => {
            area.cursor_row = area.cursor_row.saturating_sub(visible.max(1));
            area.clamp_cursor();
            area.ensure_visible(visible);
            Moved
        }
        KeyCode::PageDown => {
            area.cursor_row = (area.cursor_row + visible.max(1)).min(area.lines.len() - 1);
            area.clamp_cursor();
            area.ensure_visible(visible);
            Moved
        }
        KeyCode::Backspace => {
            if area.cursor_col > 0 {
                let idx = byte_idx(&area.lines[area.cursor_row], area.cursor_col - 1);
                area.lines[area.cursor_row].remove(idx);
                area.cursor_col -= 1;
            } else if area.cursor_row > 0 {
                // Une con la línea anterior.
                let cur = area.lines.remove(area.cursor_row);
                area.cursor_row -= 1;
                area.cursor_col = area.lines[area.cursor_row].chars().count();
                area.lines[area.cursor_row].push_str(&cur);
            } else {
                return Stay;
            }
            area.ensure_visible(visible);
            Changed
        }
        KeyCode::Delete => {
            let len = area.lines[area.cursor_row].chars().count();
            if area.cursor_col < len {
                let idx = byte_idx(&area.lines[area.cursor_row], area.cursor_col);
                area.lines[area.cursor_row].remove(idx);
            } else if area.cursor_row + 1 < area.lines.len() {
                let next = area.lines.remove(area.cursor_row + 1);
                area.lines[area.cursor_row].push_str(&next);
            } else {
                return Stay;
            }
            Changed
        }
        KeyCode::Enter => {
            if area.char_count() >= area.max_chars {
                return Stay;
            }
            let idx = byte_idx(&area.lines[area.cursor_row], area.cursor_col);
            let tail = area.lines[area.cursor_row].split_off(idx);
            area.lines.insert(area.cursor_row + 1, tail);
            area.cursor_row += 1;
            area.cursor_col = 0;
            area.ensure_visible(visible);
            Changed
        }
        KeyCode::Char(c) => {
            if area.char_count() >= area.max_chars {
                return Stay;
            }
            let idx = byte_idx(&area.lines[area.cursor_row], area.cursor_col);
            area.lines[area.cursor_row].insert(idx, c);
            area.cursor_col += 1;
            Changed
        }
        _ => Stay,
    }
}

fn byte_idx(s: &str, n: usize) -> usize {
    s.char_indices().map(|(i, _)| i).nth(n).unwrap_or(s.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Theme;

    #[test]
    fn enter_splits_and_backspace_joins() {
        let mut a = TextArea::new(100);
        for c in "hola".chars() {
            textarea_key(&mut a, 5, KeyCode::Char(c));
        }
        textarea_key(&mut a, 5, KeyCode::Enter);
        for c in "bye".chars() {
            textarea_key(&mut a, 5, KeyCode::Char(c));
        }
        assert_eq!(a.text(), "hola\nbye");
        // Backspace al inicio de la 2da une.
        textarea_key(&mut a, 5, KeyCode::Home);
        assert_eq!(
            textarea_key(&mut a, 5, KeyCode::Backspace),
            TextAreaKey::Changed
        );
        assert_eq!(a.text(), "holabye");
    }

    #[test]
    fn max_chars_blocks_input() {
        let mut a = TextArea::new(3);
        for c in "abcd".chars() {
            textarea_key(&mut a, 5, KeyCode::Char(c));
        }
        assert_eq!(a.text(), "abc");
        assert_eq!(textarea_key(&mut a, 5, KeyCode::Enter), TextAreaKey::Stay);
    }

    #[test]
    fn arrows_move_and_scroll() {
        let mut a = TextArea::with_text("l1\nl2\nl3\nl4", 100);
        textarea_key(&mut a, 2, KeyCode::End);
        textarea_key(&mut a, 2, KeyCode::Down);
        textarea_key(&mut a, 2, KeyCode::Down);
        textarea_key(&mut a, 2, KeyCode::Down);
        assert_eq!((a.cursor_row, a.scroll_top), (3, 2));
        textarea_key(&mut a, 2, KeyCode::Up);
        assert_eq!(a.cursor_row, 2);
    }

    #[test]
    fn draws_lines_and_cursor() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 8, t.desktop);
        let a = TextArea::with_text("ab\ncd", 100);
        textarea_draw(&mut b, Rect::new(2, 1, 12, 4), &a, true);
        assert_eq!(b.get(3, 1).unwrap().ch, 'a');
        assert_eq!(b.get(3, 2).unwrap().ch, 'c');
    }
}
