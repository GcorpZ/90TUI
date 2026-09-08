//! Caja de texto de una línea (`InputField`).
//!
//! Longitud máxima, color de texto, fondo distinto cuando está activa
//! (foco/cursor) y carácter de máscara optativo (`*` para contraseñas).
//! Expone los 4 parámetros globales (`foreground_color`,
//! `background_color`, `border_color`, `has_shadow`).
//!
//! Nota de tipos: colores como `Color` (16 ANSI), no `(u8, u8, u8)`,
//! porque el motor es de paleta fija (igual que `Dropdown`).

use crossterm::event::KeyCode;

use crate::core::{Attr, Buffer, Cell, Color, Rect};
use crate::prim::{draw_text, shadow};

/// Caja de texto de una línea.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputField {
    pub value: String,
    /// Longitud máxima en caracteres (`0` = vacío siempre).
    pub max_len: usize,
    /// Máscara de render (`Some('*')` = contraseña). Solo visual.
    pub mask: Option<char>,
    /// Posición del cursor en caracteres (0..=len).
    pub cursor: usize,
    pub foreground_color: Color,
    pub background_color: Color,
    /// Fondo cuando el campo tiene el foco.
    pub active_background_color: Color,
    pub border_color: Option<Color>,
    pub has_shadow: bool,
}

impl InputField {
    pub fn new(max_len: usize) -> Self {
        Self {
            value: String::new(),
            max_len,
            mask: None,
            cursor: 0,
            foreground_color: Color::Black,
            background_color: Color::White,
            active_background_color: Color::Grey,
            border_color: None,
            has_shadow: false,
        }
    }

    /// Texto visible (con máscara si hay).
    pub fn display(&self) -> String {
        match self.mask {
            Some(m) => std::iter::repeat(m)
                .take(self.value.chars().count())
                .collect(),
            None => self.value.clone(),
        }
    }

    fn len(&self) -> usize {
        self.value.chars().count()
    }

    fn clamp_cursor(&mut self) {
        self.cursor = self.cursor.min(self.len());
    }
}

/// Rect interior (descontando borde opt-in).
fn inner_of(rect: Rect, bordered: bool) -> Rect {
    if bordered && rect.w >= 3 && rect.h >= 3 {
        Rect::new(
            rect.x.saturating_add(1),
            rect.y.saturating_add(1),
            rect.w.saturating_sub(2),
            1,
        )
    } else {
        Rect::new(rect.x, rect.y, rect.w, 1)
    }
}

/// Dibuja el campo en `rect` (1 fila, o 3+ con borde). `focused` activa
/// el fondo de cursor y pinta el cursor invertido.
pub fn input_draw(buf: &mut Buffer, rect: Rect, field: &InputField, focused: bool) {
    if rect.is_empty() || rect.h < 1 {
        return;
    }
    let bordered = field.border_color.is_some();
    if field.has_shadow {
        shadow(
            buf,
            if bordered {
                rect
            } else {
                Rect::new(rect.x, rect.y, rect.w, 1)
            },
            crate::core::Theme::clipper(),
        );
    }
    if bordered {
        let b = field.border_color.unwrap_or(Color::Black);
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
    let inner = inner_of(rect, bordered);
    if inner.is_empty() {
        return;
    }
    let bg = if focused {
        field.active_background_color
    } else {
        field.background_color
    };
    buf.fill_rect(inner, Cell::new(' ', field.foreground_color, bg));
    // Scroll horizontal para que el cursor siempre se vea.
    let disp: Vec<char> = field.display().chars().collect();
    let w = inner.w as usize;
    let mut offset = 0usize;
    if field.cursor >= offset + w {
        offset = field.cursor + 1 - w;
    }
    let base = Attr::new(field.foreground_color, bg);
    for (i, ch) in disp.iter().skip(offset).take(w).enumerate() {
        draw_text(
            buf,
            inner.x.saturating_add(i as u16),
            inner.y,
            &ch.to_string(),
            base,
        );
    }
    if focused {
        // Cursor invertido sobre el char actual (o espacio al final).
        let cx = inner
            .x
            .saturating_add(field.cursor.saturating_sub(offset) as u16);
        if cx < inner.right() {
            let ch = disp.get(field.cursor).copied().unwrap_or(' ');
            buf.set(
                cx,
                inner.y,
                Cell::with_attr(ch, Attr::new(bg, field.foreground_color)),
            );
        }
    }
}

/// Resultado de una tecla sobre el campo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputKey {
    Stay,
    Moved,
    Changed,
}

/// Edición pura: inserta/borra/mueve. Respeta `max_len` (en caracteres).
pub fn input_key(field: &mut InputField, code: KeyCode) -> InputKey {
    use InputKey::*;
    field.clamp_cursor();
    match code {
        KeyCode::Left => {
            if field.cursor > 0 {
                field.cursor -= 1;
                Moved
            } else {
                Stay
            }
        }
        KeyCode::Right => {
            if field.cursor < field.len() {
                field.cursor += 1;
                Moved
            } else {
                Stay
            }
        }
        KeyCode::Home => {
            field.cursor = 0;
            Moved
        }
        KeyCode::End => {
            field.cursor = field.len();
            Moved
        }
        KeyCode::Backspace => {
            if field.cursor == 0 {
                return Stay;
            }
            let idx = byte_idx(&field.value, field.cursor - 1);
            field.value.remove(idx);
            field.cursor -= 1;
            Changed
        }
        KeyCode::Delete => {
            if field.cursor >= field.len() {
                return Stay;
            }
            let idx = byte_idx(&field.value, field.cursor);
            field.value.remove(idx);
            Changed
        }
        KeyCode::Char(c) => {
            if field.len() >= field.max_len {
                return Stay;
            }
            let idx = byte_idx(&field.value, field.cursor);
            field.value.insert(idx, c);
            field.cursor += 1;
            Changed
        }
        _ => Stay,
    }
}

/// Índice en bytes del carácter n-ésimo.
fn byte_idx(s: &str, n: usize) -> usize {
    s.char_indices().map(|(i, _)| i).nth(n).unwrap_or(s.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Theme;

    #[test]
    fn insert_respects_max_len() {
        let mut f = InputField::new(3);
        assert_eq!(input_key(&mut f, KeyCode::Char('a')), InputKey::Changed);
        assert_eq!(input_key(&mut f, KeyCode::Char('b')), InputKey::Changed);
        assert_eq!(input_key(&mut f, KeyCode::Char('c')), InputKey::Changed);
        assert_eq!(input_key(&mut f, KeyCode::Char('d')), InputKey::Stay);
        assert_eq!(f.value, "abc");
    }

    #[test]
    fn backspace_delete_and_move() {
        let mut f = InputField::new(8);
        for c in "hi".chars() {
            input_key(&mut f, KeyCode::Char(c));
        }
        input_key(&mut f, KeyCode::Left);
        assert_eq!(input_key(&mut f, KeyCode::Backspace), InputKey::Changed);
        assert_eq!(f.value, "i");
        input_key(&mut f, KeyCode::End);
        input_key(&mut f, KeyCode::Char('!'));
        assert_eq!(f.value, "i!");
        input_key(&mut f, KeyCode::Home);
        assert_eq!(input_key(&mut f, KeyCode::Delete), InputKey::Changed);
        assert_eq!(f.value, "!");
    }

    #[test]
    fn mask_only_affects_display() {
        let mut f = InputField::new(8);
        f.mask = Some('*');
        for c in "sec".chars() {
            input_key(&mut f, KeyCode::Char(c));
        }
        assert_eq!(f.display(), "***");
        assert_eq!(f.value, "sec");
    }

    #[test]
    fn draws_value_and_focused_cursor() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 5, t.desktop);
        let mut f = InputField::new(12);
        f.background_color = Color::White;
        f.active_background_color = Color::Grey;
        for c in "AB".chars() {
            input_key(&mut f, KeyCode::Char(c));
        }
        input_draw(&mut b, Rect::new(2, 1, 10, 1), &f, false);
        assert_eq!(b.get(2, 1).unwrap().ch, 'A');
        assert_eq!(b.get(2, 1).unwrap().bg, Color::White);
        // Con foco: fondo activo + cursor invertido al final.
        input_draw(&mut b, Rect::new(2, 3, 10, 1), &f, true);
        assert_eq!(b.get(2, 3).unwrap().bg, Color::Grey);
        let cur = b.get(4, 3).unwrap();
        assert_eq!(cur.bg, f.foreground_color); // invertido
    }
}
