//! Tabla de datos (`GridTable`): inventario y listados.
//!
//! Encabezados + matriz de celdas, anchos automáticos (contenido) o fijos
//! por columna, fila seleccionada resaltada y scroll vertical nativo.
//! Expone los 4 parámetros globales (`foreground_color`,
//! `background_color`, `border_color`, `has_shadow`).

use crossterm::event::KeyCode;

use crate::core::{Attr, Buffer, Cell, Color, Rect};
use crate::prim::{draw_text, fit_text, shadow, visible_len};

/// Tabla de datos con scroll.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    /// Ancho por columna (`0` = automático según contenido).
    pub widths: Vec<u16>,
    pub selected: usize,
    pub top: usize,
    pub foreground_color: Color,
    pub background_color: Color,
    pub header_fg: Color,
    pub header_bg: Color,
    pub highlight_fg: Color,
    pub highlight_bg: Color,
    pub border_color: Option<Color>,
    pub has_shadow: bool,
}

impl GridTable {
    pub fn new(headers: &[&str], rows: Vec<Vec<String>>) -> Self {
        Self {
            headers: headers.iter().map(|s| s.to_string()).collect(),
            rows,
            widths: vec![0; headers.len()],
            selected: 0,
            top: 0,
            foreground_color: Color::Black,
            background_color: Color::White,
            header_fg: Color::White,
            header_bg: Color::Navy,
            highlight_fg: Color::White,
            highlight_bg: Color::Blue,
            border_color: None,
            has_shadow: false,
        }
    }

    pub fn ncols(&self) -> usize {
        self.headers.len()
    }

    fn clamp(&mut self) {
        if self.rows.is_empty() {
            self.selected = 0;
            self.top = 0;
            return;
        }
        self.selected = self.selected.min(self.rows.len() - 1);
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

/// Anchos finales: fijos donde se piden, automáticos (contenido, tope 24)
/// en el resto; si sobran, se recortan desde la más ancha (mínimo 3).
pub fn grid_column_widths(grid: &GridTable, avail: u16) -> Vec<u16> {
    let n = grid.ncols();
    if n == 0 {
        return Vec::new();
    }
    let mut widths: Vec<u16> = (0..n)
        .map(|c| {
            if let Some(&fixed) = grid.widths.get(c) {
                if fixed > 0 {
                    return fixed;
                }
            }
            let mut w = visible_len(&grid.headers[c]);
            for row in &grid.rows {
                if let Some(cell) = row.get(c) {
                    w = w.max(visible_len(cell));
                }
            }
            w.saturating_add(2).clamp(3, 24)
        })
        .collect();
    // Gaps de 1 celda entre columnas.
    let total: u16 = widths
        .iter()
        .sum::<u16>()
        .saturating_add(n.saturating_sub(1) as u16);
    let mut over = total.saturating_sub(avail);
    while over > 0 {
        let mut shrunk = false;
        for w in widths.iter_mut() {
            if over == 0 {
                break;
            }
            if *w > 4 {
                *w -= 1;
                over -= 1;
                shrunk = true;
            }
        }
        if !shrunk {
            break;
        }
    }
    widths
}

/// Filas de datos visibles (cabecera aparte).
pub fn grid_visible(rect: Rect, bordered: bool) -> usize {
    rect.h
        .saturating_sub(1) // cabecera
        .saturating_sub(if bordered { 2 } else { 0 }) as usize
}

/// Dibuja cabecera + filas con resaltado + scroll.
pub fn grid_draw(buf: &mut Buffer, rect: Rect, grid: &GridTable) {
    if rect.is_empty() || rect.h < 2 {
        return;
    }
    let bordered = grid.border_color.is_some();
    if grid.has_shadow {
        shadow(buf, rect, crate::core::Theme::clipper());
    }
    buf.fill_rect(
        rect,
        Cell::new(' ', grid.foreground_color, grid.background_color),
    );
    let x0 = rect.x.saturating_add(if bordered { 1 } else { 0 });
    let y0 = rect.y.saturating_add(if bordered { 1 } else { 0 });
    let avail = rect.w.saturating_sub(if bordered { 2 } else { 0 });
    let widths = grid_column_widths(grid, avail);
    // Cabecera.
    let mut cx = x0;
    for (c, h) in grid.headers.iter().enumerate() {
        let w = widths.get(c).copied().unwrap_or(0);
        if w == 0 || cx.saturating_add(w) > x0.saturating_add(avail) {
            break;
        }
        buf.fill_rect(
            Rect::new(cx, y0, w, 1),
            Cell::new(' ', grid.header_fg, grid.header_bg),
        );
        draw_text(
            buf,
            cx.saturating_add(1),
            y0,
            &fit_text(h, w.saturating_sub(1)),
            Attr::bold(grid.header_fg, grid.header_bg),
        );
        cx = cx.saturating_add(w).saturating_add(1);
    }
    // Filas.
    let rows = grid_visible(rect, bordered);
    for row in 0..rows {
        let idx = grid.top + row;
        if idx >= grid.rows.len() {
            break;
        }
        let y = y0.saturating_add(1).saturating_add(row as u16);
        let sel = idx == grid.selected;
        let (fg, bg) = if sel {
            (grid.highlight_fg, grid.highlight_bg)
        } else {
            (grid.foreground_color, grid.background_color)
        };
        let mut cx = x0;
        for c in 0..grid.ncols() {
            let w = widths.get(c).copied().unwrap_or(0);
            if w == 0 || cx.saturating_add(w) > x0.saturating_add(avail) {
                break;
            }
            buf.fill_rect(Rect::new(cx, y, w, 1), Cell::new(' ', fg, bg));
            if let Some(cell) = grid.rows[idx].get(c) {
                draw_text(
                    buf,
                    cx.saturating_add(1),
                    y,
                    &fit_text(cell, w.saturating_sub(1)),
                    Attr::new(fg, bg),
                );
            }
            cx = cx.saturating_add(w).saturating_add(1);
        }
    }
    if bordered {
        let b = grid.border_color.unwrap_or(Color::Black);
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

/// Navegación pura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridNav {
    Stay,
    Move(usize),
}

impl super::focus::HandleEvent for GridTable {
    type Out = GridNav;
    /// `↑↓` (y `PgUp/PgDn/Home/End`) alteran `selected` con scroll y el
    /// llamante redibuja para resaltar la fila al instante.
    fn handle_event(&mut self, ctx: &super::focus::EventCtx, code: KeyCode) -> GridNav {
        grid_key(self, ctx.visible, code)
    }
}

pub fn grid_key(grid: &mut GridTable, visible: usize, code: KeyCode) -> GridNav {
    use GridNav::*;
    let n = grid.rows.len();
    if n == 0 {
        return Stay;
    }
    grid.clamp();
    let vis = visible.max(1);
    match code {
        KeyCode::Up => {
            grid.selected = (grid.selected + n - 1) % n;
            grid.ensure_visible(vis);
            Move(grid.selected)
        }
        KeyCode::Down => {
            grid.selected = (grid.selected + 1) % n;
            grid.ensure_visible(vis);
            Move(grid.selected)
        }
        KeyCode::Home => {
            grid.selected = 0;
            grid.ensure_visible(vis);
            Move(0)
        }
        KeyCode::End => {
            grid.selected = n - 1;
            grid.ensure_visible(vis);
            Move(n - 1)
        }
        KeyCode::PageUp => {
            grid.selected = grid.selected.saturating_sub(vis);
            grid.ensure_visible(vis);
            Move(grid.selected)
        }
        KeyCode::PageDown => {
            grid.selected = (grid.selected + vis).min(n - 1);
            grid.ensure_visible(vis);
            Move(grid.selected)
        }
        _ => Stay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Theme;

    fn inv() -> GridTable {
        GridTable::new(
            &["Ref", "Descripción"],
            vec![
                vec!["A1".to_string(), "Tornillo".to_string()],
                vec!["B22".to_string(), "Tuerca larga".to_string()],
                vec!["C333".to_string(), "Arandela".to_string()],
            ],
        )
    }

    #[test]
    fn widths_auto_from_content() {
        let g = inv();
        let w = grid_column_widths(&g, 60);
        // "Descripción" (11) +2 = 13; "Tuerca larga" (12)+2 = 14 → 14.
        assert_eq!(w[0], 4 + 2); // "Ref"/"C333" = 4
        assert_eq!(w[1], 12 + 2);
        // Fijo se respeta.
        let mut fixed = inv();
        fixed.widths = vec![10, 0];
        let w2 = grid_column_widths(&fixed, 60);
        assert_eq!(w2[0], 10);
        // Desborde: recorta sin romper.
        let w3 = grid_column_widths(&g, 10);
        assert!(w3.iter().sum::<u16>() < 10);
    }

    #[test]
    fn nav_scrolls_with_selection() {
        let mut g = inv();
        assert_eq!(grid_key(&mut g, 2, KeyCode::Down), GridNav::Move(1));
        assert_eq!(grid_key(&mut g, 2, KeyCode::End), GridNav::Move(2));
        assert_eq!(g.top, 1); // scroll con ventana de 2
        assert_eq!(grid_key(&mut g, 2, KeyCode::Home), GridNav::Move(0));
        assert_eq!(g.top, 0);
        assert_eq!(grid_key(&mut g, 2, KeyCode::PageDown), GridNav::Move(2));
    }

    #[test]
    fn draws_header_and_highlight() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(50, 10, t.desktop);
        let mut g = inv();
        g.selected = 1;
        grid_draw(&mut b, Rect::new(2, 1, 40, 6), &g);
        // Cabecera navy + fila 1 resaltada en azul.
        assert_eq!(b.get(2, 1).unwrap().bg, g.header_bg);
        assert_eq!(b.get(3, 1).unwrap().ch, 'R');
        assert_eq!(b.get(3, 3).unwrap().bg, g.highlight_bg);
        assert_eq!(b.get(3, 3).unwrap().ch, 'B'); // "B22"
    }
}
