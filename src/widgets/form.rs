//! Formularios `@...GET`: fichas negras con campos editables.
//!
//! `FormState` guarda valores + foco. `form_key` edita puro (sin TTY):
//! letras, Backspace, Tab/↑↓ para moverse, Enter acepta, Esc cancela.

use crossterm::event::KeyCode;

use crate::core::{Attr, Buffer, Cell, Rect, Theme};
use crate::prim::{draw_text, visible_len, window, WindowOpts};

/// Tipo de campo (valida lo tecleado).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Number,
    YesNo,
}

/// Un campo `Nombre : valor`.
#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub value: String,
    pub width: u16,
    pub kind: FieldKind,
}

impl Field {
    pub fn text(name: &str, value: &str, width: u16) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            width,
            kind: FieldKind::Text,
        }
    }

    pub fn number(name: &str, value: &str, width: u16) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            width,
            kind: FieldKind::Number,
        }
    }

    pub fn yesno(name: &str, value: bool) -> Self {
        Self {
            name: name.to_string(),
            value: if value {
                "Si".to_string()
            } else {
                "No".to_string()
            },
            width: 2,
            kind: FieldKind::YesNo,
        }
    }

    fn accepts(&self, ch: char) -> bool {
        match self.kind {
            FieldKind::Text => !ch.is_control(),
            FieldKind::Number => ch.is_ascii_digit(),
            FieldKind::YesNo => false, // se cambia con Espacio, no tecleando
        }
    }
}

/// Estado del formulario.
#[derive(Clone, Debug)]
pub struct FormState {
    pub fields: Vec<Field>,
    pub focus: usize,
}

impl FormState {
    pub fn new(fields: Vec<Field>) -> Self {
        Self { fields, focus: 0 }
    }
}

/// Dibuja la ficha (título + filas `Nombre : valor`). El campo con foco
/// lleva fondo gris (como el cursor de captura de la época).
pub fn form_draw(buf: &mut Buffer, rect: Rect, title: &str, state: &FormState, theme: Theme) {
    if rect.is_empty() {
        return;
    }
    window(buf, rect, &WindowOpts::form(title, theme), theme);
    let name_w: u16 = state
        .fields
        .iter()
        .map(|f| visible_len(&f.name))
        .max()
        .unwrap_or(0);
    for (i, f) in state.fields.iter().enumerate() {
        let y = rect.y.saturating_add(2 + i as u16);
        if y >= rect.bottom().saturating_sub(1) {
            break;
        }
        let nx = rect.x.saturating_add(2);
        draw_text(buf, nx, y, &f.name, theme.form_attr());
        draw_text(
            buf,
            nx.saturating_add(name_w).saturating_add(1),
            y,
            ":",
            theme.form_attr(),
        );
        let vx = nx.saturating_add(name_w).saturating_add(3);
        let shown: String = f.value.chars().take(f.width as usize).collect();
        let pad = f.width.saturating_sub(visible_len(&shown));
        let cell_attr: Attr = if i == state.focus {
            theme.field_attr()
        } else {
            Attr::new(theme.form_text, theme.field_bg)
        };
        draw_text(buf, vx, y, &shown, cell_attr);
        // Relleno del campo hasta su ancho.
        for p in 0..pad {
            buf.set(
                vx.saturating_add(visible_len(&shown)).saturating_add(p),
                y,
                Cell::with_attr(' ', cell_attr),
            );
        }
    }
}

/// Resultado de tecla en el formulario.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormKey {
    Stay,
    Moved,
    Accept,
    Cancel,
}

/// Edita el estado. `Enter` en el último campo acepta; `Esc` cancela.
pub fn form_key(state: &mut FormState, code: KeyCode) -> FormKey {
    use FormKey::*;
    let n = state.fields.len();
    if n == 0 {
        return Cancel;
    }
    let focus = state.focus.min(n - 1);
    state.focus = focus;
    match code {
        KeyCode::Esc => Cancel,
        KeyCode::Tab | KeyCode::Down => {
            state.focus = (focus + 1) % n;
            Moved
        }
        KeyCode::Up => {
            state.focus = (focus + n - 1) % n;
            Moved
        }
        KeyCode::Enter => {
            if focus + 1 >= n {
                Accept
            } else {
                state.focus = focus + 1;
                Moved
            }
        }
        KeyCode::Backspace => {
            state.fields[focus].value.pop();
            Stay
        }
        KeyCode::Char(' ') if state.fields[focus].kind == FieldKind::YesNo => {
            let f = &mut state.fields[focus];
            f.value = if f.value == "Si" {
                "No".to_string()
            } else {
                "Si".to_string()
            };
            Stay
        }
        KeyCode::Char(c) => {
            let f = &mut state.fields[focus];
            if f.accepts(c) && visible_len(&f.value) < f.width {
                f.value.push(c);
            }
            Stay
        }
        _ => Stay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> FormState {
        FormState::new(vec![
            Field::text("Código", "", 5),
            Field::number("Clase", "", 3),
            Field::yesno("Seriales", false),
        ])
    }

    #[test]
    fn typing_respects_kind_and_width() {
        let mut s = state();
        form_key(&mut s, KeyCode::Char('A'));
        form_key(&mut s, KeyCode::Char('B'));
        assert_eq!(s.fields[0].value, "AB");
        // Número rechaza letras.
        form_key(&mut s, KeyCode::Tab);
        form_key(&mut s, KeyCode::Char('X'));
        form_key(&mut s, KeyCode::Char('5'));
        assert_eq!(s.fields[1].value, "5");
        // YesNo alterna con espacio.
        form_key(&mut s, KeyCode::Tab);
        assert_eq!(s.fields[2].value, "No");
        form_key(&mut s, KeyCode::Char(' '));
        assert_eq!(s.fields[2].value, "Si");
    }

    #[test]
    fn enter_accepts_on_last_esc_cancels() {
        let mut s = state();
        assert_eq!(form_key(&mut s, KeyCode::Esc), FormKey::Cancel);
        assert_eq!(form_key(&mut s, KeyCode::Enter), FormKey::Moved);
        assert_eq!(form_key(&mut s, KeyCode::Enter), FormKey::Moved);
        assert_eq!(form_key(&mut s, KeyCode::Enter), FormKey::Accept);
    }

    #[test]
    fn draws_focused_field_grey() {
        let t = Theme::clipper();
        let mut buf = Buffer::blank(60, 12, t.desktop);
        let r = Rect::new(5, 2, 50, 8);
        let s = state();
        form_draw(&mut buf, r, "DEPARTAMENTOS", &s, t);
        // Ficha negra con título blanco.
        assert_eq!(buf.get(6, 4).unwrap().bg, t.form_bg);
        // Valor del campo con foco (0) en gris.
        let name_w = visible_len("Seriales");
        let vx = 5 + 2 + name_w + 3;
        assert_eq!(buf.get(vx, 2 + 2).unwrap().bg, t.field_bg);
    }
}
