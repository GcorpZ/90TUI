//! Progreso con barras dobles: diálogo menta + `ProgressBar` real.
//!
//! Cada barra ocupa 3 filas: `0 ... 100 %` / barra / `etiqueta  cur/total`.
//! La fila 2 la pinta `progressbar_draw` (octavos de bloque + `NN%`).

use super::progressbar::{progressbar_draw, ProgressBar};
use crate::core::{Buffer, Rect, Theme};
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
        // Fila 2: un `ProgressBar` real (octavos + `NN%` integrado).
        let mut pbar = ProgressBar::new(bar.pct);
        pbar.foreground_color = theme.popup;
        pbar.background_color = theme.field_bg;
        progressbar_draw(buf, Rect::new(inner_x, base + 1, inner_w, 1), &pbar);
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
        // Diálogo 44×8 centrado: fila 2 en y=12, x=20..60.
        // 12% de 40 = 38 octavos: 4 llenas + frontera ▊(6/8) + pista.
        assert_eq!(buf.get(20, 12).unwrap().ch, '\u{2588}');
        assert_eq!(buf.get(20, 12).unwrap().fg, t.popup);
        assert_eq!(buf.get(24, 12).unwrap().ch, '\u{258a}');
        assert_eq!(buf.get(25, 12).unwrap().ch, '\u{2591}');
        // Porcentaje integrado + contador en sus filas.
        let row12: String = (0..80).map(|x| buf.get(x, 12).unwrap().ch).collect();
        assert!(row12.contains("12%"), "12% no encontrado");
        let found = (0..25).any(|y| {
            let row: String = (0..80).map(|x| buf.get(x, y).unwrap().ch).collect();
            row.contains("60/546")
        });
        assert!(found, "contador 60/546 no encontrado");
    }
}
