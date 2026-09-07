//! Tablas `browse`: header gris + cursor gris + scroll.
//!
//! Como las pantallas de movimientos de la época: columnas de ancho fijo,
//! fila actual resaltada, `TableState{row, top}` para el viewport.

use crossterm::event::KeyCode;

use crate::core::{Attr, Buffer, Cell, Rect, Theme};
use crate::prim::draw_text;

/// Definición de columnas + filas (todo texto ya formateado).
#[derive(Clone, Debug)]
pub struct TableDef {
    pub headers: Vec<(String, u16)>,
    pub rows: Vec<Vec<String>>,
}

impl TableDef {
    pub fn new(headers: &[(&str, u16)], rows: Vec<Vec<String>>) -> Self {
        Self {
            headers: headers.iter().map(|(s, w)| (s.to_string(), *w)).collect(),
            rows,
        }
    }

    pub fn col_x(&self, col: usize) -> u16 {
        self.headers.iter().take(col).map(|(_, w)| w + 1).sum()
    }
}

/// Fila actual + primera visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableState {
    pub row: usize,
    pub top: usize,
}

impl TableState {
    pub fn new() -> Self {
        Self { row: 0, top: 0 }
    }
}

impl Default for TableState {
    fn default() -> Self {
        Self::new()
    }
}

/// Dibuja header + viewport + `footer` abajo. `rect` incluye todo.
pub fn table_draw(
    buf: &mut Buffer,
    rect: Rect,
    def: &TableDef,
    state: &TableState,
    footer: &str,
    theme: Theme,
) {
    if rect.is_empty() || rect.h < 3 {
        return;
    }
    // Header gris.
    buf.fill_rect(
        Rect::new(rect.x, rect.y, rect.w, 1),
        Cell::new(' ', crate::core::Color::Black, theme.field_bg),
    );
    for (c, (h, _)) in def.headers.iter().enumerate() {
        draw_text(
            buf,
            rect.x.saturating_add(def.col_x(c)),
            rect.y,
            h,
            Attr::bold(crate::core::Color::Black, theme.field_bg),
        );
    }
    // Filas visibles.
    let visible = rect.h.saturating_sub(2) as usize;
    for i in 0..visible {
        let y = rect.y + 1 + i as u16;
        let Some(row) = def.rows.get(state.top + i) else {
            break;
        };
        let selected = state.top + i == state.row;
        if selected {
            buf.fill_rect(
                Rect::new(rect.x, y, rect.w, 1),
                Cell::new(' ', theme.select_fg, theme.select_bg),
            );
        }
        for (c, cell) in row.iter().enumerate() {
            let Some((_, w)) = def.headers.get(c) else {
                continue;
            };
            let text: String = cell.chars().take(*w as usize).collect();
            let a = if selected {
                theme.popup_sel_attr()
            } else {
                theme.form_attr()
            };
            draw_text(buf, rect.x.saturating_add(def.col_x(c)), y, &text, a);
        }
    }
    // Footer.
    draw_text(
        buf,
        rect.x.saturating_add(1),
        rect.bottom().saturating_sub(1),
        footer,
        theme.form_attr(),
    );
}

/// Mueve el cursor ajustando el viewport. `visible` = filas mostrables.
pub fn table_key(state: &mut TableState, n_rows: usize, visible: usize, code: KeyCode) {
    if n_rows == 0 {
        state.row = 0;
        state.top = 0;
        return;
    }
    let vis = visible.max(1);
    let mut row = state.row.min(n_rows - 1);
    match code {
        KeyCode::Up => row = row.saturating_sub(1),
        KeyCode::Down => row = (row + 1).min(n_rows - 1),
        KeyCode::Home => row = 0,
        KeyCode::End => row = n_rows - 1,
        KeyCode::PageUp => row = row.saturating_sub(vis),
        KeyCode::PageDown => row = (row + vis).min(n_rows - 1),
        _ => {}
    }
    state.row = row;
    // Mantener el cursor visible.
    if row < state.top {
        state.top = row;
    } else if row >= state.top + vis {
        state.top = row + 1 - vis;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def() -> TableDef {
        TableDef::new(
            &[("Ref", 8), ("Costo", 10)],
            (0..20)
                .map(|i| vec![format!("R{i:04}"), format!("{}.00", i * 10)])
                .collect(),
        )
    }

    #[test]
    fn scrolls_viewport() {
        let mut s = TableState::new();
        table_key(&mut s, 20, 5, KeyCode::PageDown);
        assert_eq!((s.row, s.top), (5, 1));
        table_key(&mut s, 20, 5, KeyCode::End);
        assert_eq!((s.row, s.top), (19, 15));
        table_key(&mut s, 20, 5, KeyCode::Home);
        assert_eq!((s.row, s.top), (0, 0));
    }

    #[test]
    fn draws_header_and_cursor() {
        let t = Theme::clipper();
        let mut buf = Buffer::blank(60, 10, t.form_bg);
        let r = Rect::new(0, 0, 40, 8);
        let mut s = TableState::new();
        s.row = 1;
        table_draw(&mut buf, r, &def(), &s, "Linea: 2/20", t);
        assert_eq!(buf.get(0, 0).unwrap().bg, t.field_bg);
        assert_eq!(buf.get(0, 2).unwrap().bg, t.select_bg); // cursor
        assert_eq!(buf.get(0, 1).unwrap().bg, t.form_bg); // normal
        assert_eq!(buf.get(1, 7).unwrap().ch, 'L'); // footer
    }
}
