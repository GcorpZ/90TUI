//! Progreso con barras dobles: diálogo menta + track gris + fill azul.
//!
//! Cada barra ocupa 3 filas: `0 ... 100 %` / barra / `etiqueta  cur/total`.

use crate::core::{Buffer, Cell, Rect, Theme};
use crate::prim::{draw_text, visible_len, window, WindowOpts};

/// Una barra de avance.
#[derive(Clone, Debug)]
pub struct BarInfo {
    pub label: String,
    pub file: String,
    pub pct: u8,
    pub cur: u64,
    pub total: u64,
}

impl BarInfo {
    pub fn new(label: &str, file: &str, pct: u8, cur: u64, total: u64) -> Self {
        Self {
            label: label.to_string(),
            file: file.to_string(),
            pct: pct.min(100),
            cur,
            total,
        }
    }
}

/// Diálogo completo de progreso.
#[derive(Clone, Debug)]
pub struct ProgressInfo {
    pub title: String,
    pub space: String,
    pub elapsed: String,
    pub bars: Vec<BarInfo>,
}

impl ProgressInfo {
    pub fn new(title: &str, space: &str, elapsed: &str, bars: Vec<BarInfo>) -> Self {
        Self {
            title: title.to_string(),
            space: space.to_string(),
            elapsed: elapsed.to_string(),
            bars,
        }
    }
}

/// Ancho de relleno para `pct` en un track de `inner` celdas.
pub fn bar_fill_width(inner: u16, pct: u8) -> u16 {
    (inner as u32 * pct.min(100) as u32 / 100) as u16
}

/// Dibuja el diálogo centrado (recortado a pantalla). Solo dibujo.
pub fn progress_draw(buf: &mut Buffer, screen: Rect, info: &ProgressInfo, theme: Theme) {
    let header = format!("Espacio: {}   Tiempo: {}", info.space, info.elapsed);
    let content_w = visible_len(&header)
        .max(
            info.bars
                .iter()
                .map(|b| visible_len(&b.label) + visible_len(&b.file) + 12)
                .max()
                .unwrap_or(0),
        )
        .max(40);
    let w = (content_w + 4).min(screen.w.saturating_sub(2).max(10));
    let h = (4 + 3 * info.bars.len() as u16 + 1).min(screen.h.max(1));
    let r = Rect::centered_in(w, h, screen);
    if r.is_empty() {
        return;
    }
    window(buf, r, &WindowOpts::dialog(&info.title, theme), theme);
    let inner_x = r.x.saturating_add(2);
    let inner_w = r.w.saturating_sub(4);
    draw_text(buf, inner_x, r.y + 1, &header, theme.dialog_attr());

    for (i, bar) in info.bars.iter().enumerate() {
        let base = r.y + 3 + i as u16 * 3;
        if base + 2 >= r.y + r.h {
            break;
        }
        // Fila 1: `0` a la izq, `100 %` a la der.
        draw_text(buf, inner_x, base, "0", theme.dialog_attr());
        let pct_s = "100 %";
        draw_text(
            buf,
            inner_x
                .saturating_add(inner_w)
                .saturating_sub(visible_len(pct_s)),
            base,
            pct_s,
            theme.dialog_attr(),
        );
        // Fila 2: track + fill + pct centrado encima.
        let fill = bar_fill_width(inner_w, bar.pct);
        for x in 0..inner_w {
            let c = if x < fill {
                Cell::new(' ', theme.popup_text, theme.popup)
            } else {
                Cell::new(' ', theme.dialog_text, theme.field_bg)
            };
            buf.set(inner_x.saturating_add(x), base + 1, c);
        }
        let mid = format!("{}%", bar.pct);
        draw_text(
            buf,
            inner_x.saturating_add(inner_w.saturating_sub(visible_len(&mid)) / 2),
            base + 1,
            &mid,
            theme.dialog_attr(),
        );
        // Fila 3: `Etiqueta: ARCHIVO   cur/total`.
        let desc = format!("{}: {}", bar.label, bar.file);
        draw_text(buf, inner_x, base + 2, &desc, theme.dialog_attr());
        let cnt = format!("{}/{}", bar.cur, bar.total);
        draw_text(
            buf,
            inner_x
                .saturating_add(inner_w)
                .saturating_sub(visible_len(&cnt)),
            base + 2,
            &cnt,
            theme.dialog_attr(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_math() {
        assert_eq!(bar_fill_width(50, 0), 0);
        assert_eq!(bar_fill_width(50, 50), 25);
        assert_eq!(bar_fill_width(50, 100), 50);
        assert_eq!(bar_fill_width(50, 200), 50); // clamp
        assert_eq!(bar_fill_width(0, 50), 0);
    }

    #[test]
    fn draws_track_and_fill() {
        let t = Theme::clipper();
        let mut buf = Buffer::blank(80, 25, t.desktop);
        let info = ProgressInfo::new(
            "ORDENAR INDICES",
            "999848KB",
            "00:00:00",
            vec![BarInfo::new("Inventario", "APROD.DAT", 12, 60, 546)],
        );
        progress_draw(&mut buf, Rect::new(0, 0, 80, 25), &info, t);
        // Hay al menos una celda azul (fill) y una gris (track).
        let mut found_fill = false;
        let mut found_track = false;
        for x in 0..80 {
            for y in 0..25 {
                let bg = buf.get(x, y).unwrap().bg;
                if bg == t.popup {
                    found_fill = true;
                }
                if bg == t.field_bg {
                    found_track = true;
                }
            }
        }
        assert!(found_fill && found_track);
        // Contador visible en alguna fila.
        let found = (0..25).any(|y| {
            let row: String = (0..80).map(|x| buf.get(x, y).unwrap().ch).collect();
            row.contains("60/546")
        });
        assert!(found, "contador 60/546 no encontrado");
    }
}
