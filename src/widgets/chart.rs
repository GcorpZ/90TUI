//! Gráficos retro-modernos en modo texto (`TuiChart`).
//!
//! Tres modos sobre la misma serie `(etiqueta, valor)`:
//! 1. **Barras 3D** (`Bars3D`): bloques `\u{2588}` con profundidad simulada
//!    en `\u{2592}` (borde lateral derecho + borde superior, 1 celda).
//! 2. **Líneas** (`Line`): puntos Braille 2x4 por celda (alta precisión
//!    aparente: `\u{2800}` + bits) sobre la polilínea interpolada.
//! 3. **Tarta** (`Pie`): disco geométrico (`x²+y²<=r²` con aspecto 1:2)
//!    por sectores, cada porción con un bloque sectorial
//!    (`\u{2588}\u{2593}\u{2592}\u{2591}` rotando) + leyenda con `%`.
//!
//! Expone los 4 parámetros globales (`foreground_color`,
//! `background_color`, `border_color`, `has_shadow`).

use crate::core::{Attr, Buffer, Cell, Color, Rect, Theme};
use crate::prim::{draw_text, fit_text, shadow, visible_len};

const FULL: char = '\u{2588}';
const SHADE_3D: char = '\u{2592}';
const SECTOR_GLYPHS: [char; 4] = ['\u{2588}', '\u{2593}', '\u{2592}', '\u{2591}'];

/// Modo de render del gráfico.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ChartKind {
    #[default]
    Bars3D,
    Line,
    Pie,
}

/// Un punto de la serie.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartPoint {
    pub label: String,
    pub value: f64,
}

impl ChartPoint {
    pub fn new(label: &str, value: f64) -> Self {
        Self {
            label: label.to_string(),
            value,
        }
    }
}

/// Gráfico completo.
#[derive(Clone, Debug, PartialEq)]
pub struct TuiChart {
    pub kind: ChartKind,
    pub title: String,
    pub series: Vec<ChartPoint>,
    pub foreground_color: Color,
    pub background_color: Color,
    /// Color de la profundidad 3D / curva braille / sectores.
    pub accent_color: Color,
    pub border_color: Option<Color>,
    pub has_shadow: bool,
}

impl TuiChart {
    pub fn new(kind: ChartKind, title: &str, series: Vec<ChartPoint>) -> Self {
        Self {
            kind,
            title: title.to_string(),
            series,
            foreground_color: Color::Navy,
            background_color: Color::White,
            accent_color: Color::DarkGrey,
            border_color: None,
            has_shadow: false,
        }
    }

    pub fn max_value(&self) -> f64 {
        self.series
            .iter()
            .map(|p| p.value.max(0.0))
            .fold(0.0, f64::max)
    }

    pub fn total(&self) -> f64 {
        self.series.iter().map(|p| p.value.max(0.0)).sum()
    }
}

/// Marco común: sombra + fondo + título + borde. Devuelve el área de plot.
fn frame(buf: &mut Buffer, rect: Rect, chart: &TuiChart) -> Rect {
    if chart.has_shadow {
        shadow(buf, rect, Theme::clipper());
    }
    buf.fill_rect(
        rect,
        Cell::new(' ', chart.foreground_color, chart.background_color),
    );
    if !chart.title.is_empty() && rect.h >= 1 {
        let fit = fit_text(&chart.title, rect.w.saturating_sub(2));
        let tx = rect
            .x
            .saturating_add(rect.w.saturating_sub(visible_len(&fit)) / 2);
        draw_text(
            buf,
            tx,
            rect.y,
            &fit,
            Attr::bold(chart.foreground_color, chart.background_color),
        );
    }
    if let Some(border) = chart.border_color {
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
    let inset = if chart.border_color.is_some() { 1 } else { 0 };
    Rect::new(
        rect.x.saturating_add(inset).saturating_add(1),
        rect.y.saturating_add(1).saturating_add(1),
        rect.w.saturating_sub(inset * 2).saturating_sub(2),
        rect.h.saturating_sub(2).saturating_sub(inset * 2),
    )
}

/// Dibuja el gráfico según su `kind`.
pub fn tuichart_draw(buf: &mut Buffer, rect: Rect, chart: &TuiChart) {
    if rect.w < 6 || rect.h < 4 {
        return;
    }
    let plot = frame(buf, rect, chart);
    if plot.is_empty() {
        return;
    }
    if chart.series.is_empty() {
        draw_text(
            buf,
            plot.x,
            plot.y,
            &fit_text("(sin datos)", plot.w),
            Attr::new(chart.foreground_color, chart.background_color),
        );
        return;
    }
    match chart.kind {
        ChartKind::Bars3D => draw_bars(buf, plot, chart),
        ChartKind::Line => draw_line(buf, plot, chart),
        ChartKind::Pie => draw_pie(buf, plot, chart),
    }
}

// --- Barras 3D ---

/// Alturas en celdas para cada valor (0..=h).
pub fn bar_heights(series: &[ChartPoint], h: u16) -> Vec<u16> {
    let max = series.iter().map(|p| p.value.max(0.0)).fold(0.0, f64::max);
    if max <= 0.0 {
        return vec![0; series.len()];
    }
    series
        .iter()
        .map(|p| (p.value.max(0.0) * h as f64 / max).round() as u16)
        .collect()
}

fn draw_bars(buf: &mut Buffer, plot: Rect, chart: &TuiChart) {
    const BW: u16 = 3; // ancho de barra
    const GAP: u16 = 1;
    let label_h = 1u16; // fila de etiquetas
    let h = plot.h.saturating_sub(label_h).saturating_sub(1); // -1: aire 3D sup
    if h == 0 {
        return;
    }
    let heights = bar_heights(&chart.series, h);
    let base = plot.bottom().saturating_sub(label_h);
    for (i, (pt, bh)) in chart.series.iter().zip(heights.iter()).enumerate() {
        let x = plot.x.saturating_add(i as u16 * (BW + GAP));
        if x.saturating_add(BW) > plot.right() {
            break;
        }
        let top = base.saturating_sub(*bh);
        // Cuerpo `\u{2588}`.
        for y in top..base {
            for dx in 0..BW {
                buf.set(
                    x.saturating_add(dx),
                    y,
                    Cell::new(FULL, chart.foreground_color, chart.background_color),
                );
            }
        }
        // Profundidad 3D en `\u{2592}`: lateral derecho + borde superior.
        for y in top..base {
            buf.set(
                x.saturating_add(BW),
                y,
                Cell::new(SHADE_3D, chart.accent_color, chart.background_color),
            );
        }
        if top > plot.y {
            for dx in 0..BW {
                buf.set(
                    x.saturating_add(dx).saturating_add(1),
                    top.saturating_sub(1),
                    Cell::new(SHADE_3D, chart.accent_color, chart.background_color),
                );
            }
        }
        // Etiqueta (primer char).
        if let Some(ch) = pt.label.chars().next() {
            buf.set(
                x.saturating_add(1),
                base,
                Cell::new(ch, chart.foreground_color, chart.background_color),
            );
        }
    }
}

// --- Líneas braille ---

/// Bit braille para (dx 0..=1, dy 0..=3, dy=0 arriba).
fn braille_bit(dx: usize, dy: usize) -> u8 {
    const MAP: [[u8; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];
    MAP[dy.min(3)][dx.min(1)]
}

fn braille_char(mask: u8) -> char {
    char::from_u32(0x2800 + mask as u32).unwrap_or(' ')
}

/// Puntos (col de celda, fila de celda, máscara) de la polilínea.
pub fn line_dots(values: &[f64], cols: u16, rows: u16) -> Vec<(u16, u16, u8)> {
    if values.is_empty() || cols == 0 || rows == 0 {
        return Vec::new();
    }
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let span = (max - min).max(1e-9);
    let dots_w = cols as usize * 2;
    let dots_h = rows as usize * 4;
    // Posición en dots de cada valor.
    let pts: Vec<(usize, usize)> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let dx = if values.len() == 1 {
                0
            } else {
                i * (dots_w - 1) / (values.len() - 1)
            };
            let dy = ((max - v) * (dots_h - 1) as f64 / span).round() as usize;
            (dx, dy.min(dots_h - 1))
        })
        .collect();
    // Rasteriza segmentos con interpolación lineal.
    let mut cells = vec![vec![0u8; cols as usize]; rows as usize];
    let mut dot = |dx: usize, dy: usize| {
        let cx = dx / 2;
        let cy = dy / 4;
        if cx < cols as usize && cy < rows as usize {
            cells[cy][cx] |= braille_bit(dx % 2, dy % 4);
        }
    };
    for w in pts.windows(2) {
        let (x0, y0) = (w[0].0 as i32, w[0].1 as i32);
        let (x1, y1) = (w[1].0 as i32, w[1].1 as i32);
        let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
        for s in 0..=steps {
            let x = x0 + (x1 - x0) * s / steps;
            let y = y0 + (y1 - y0) * s / steps;
            dot(x as usize, y as usize);
        }
    }
    if pts.len() == 1 {
        dot(pts[0].0, pts[0].1);
    }
    let mut out = Vec::new();
    for (cy, row) in cells.iter().enumerate() {
        for (cx, mask) in row.iter().enumerate() {
            if *mask != 0 {
                out.push((cx as u16, cy as u16, *mask));
            }
        }
    }
    out
}

fn draw_line(buf: &mut Buffer, plot: Rect, chart: &TuiChart) {
    let values: Vec<f64> = chart.series.iter().map(|p| p.value).collect();
    for (cx, cy, mask) in line_dots(&values, plot.w, plot.h) {
        buf.set(
            plot.x.saturating_add(cx),
            plot.y.saturating_add(cy),
            Cell::new(
                braille_char(mask),
                chart.accent_color,
                chart.background_color,
            ),
        );
    }
}

// --- Tarta ---

/// Sector (índice) de cada ángulo según valores proporcionales.
pub fn pie_sectors(values: &[f64], steps: usize) -> Vec<usize> {
    let total: f64 = values.iter().map(|v| v.max(0.0)).sum();
    if total <= 0.0 || steps == 0 {
        return vec![0; steps];
    }
    (0..steps)
        .map(|s| {
            let frac = s as f64 / steps as f64;
            let mut acc = 0.0;
            for (i, v) in values.iter().enumerate() {
                acc += v.max(0.0) / total;
                if frac < acc {
                    return i.min(values.len() - 1);
                }
            }
            values.len() - 1
        })
        .collect()
}

fn draw_pie(buf: &mut Buffer, plot: Rect, chart: &TuiChart) {
    let values: Vec<f64> = chart.series.iter().map(|p| p.value.max(0.0)).collect();
    let total = chart.total().max(1e-9);
    // Disco: la mitad superior para el círculo, resto leyenda si cabe.
    let legend_rows = (chart.series.len() as u16).min(plot.h.saturating_sub(4));
    let disc_h = plot.h.saturating_sub(legend_rows);
    if disc_h < 3 || plot.w < 6 {
        return;
    }
    let cx = plot.x as f64 + plot.w as f64 / 2.0;
    let cy = plot.y as f64 + disc_h as f64 / 2.0;
    let rx = plot.w as f64 / 2.0 - 1.0;
    let ry = disc_h as f64 / 2.0 - 0.5;
    for dy in 0..disc_h {
        for dx in 0..plot.w {
            let nx = (plot.x as f64 + dx as f64 + 0.5 - cx) / rx.max(0.5);
            let ny = (plot.y as f64 + dy as f64 + 0.5 - cy) * 2.0 / ry.max(0.5);
            if nx * nx + ny * ny > 1.0 {
                continue;
            }
            let mut ang = ny.atan2(nx); // -PI..PI
            if ang < 0.0 {
                ang += 2.0 * std::f64::consts::PI;
            }
            let frac = ang / (2.0 * std::f64::consts::PI);
            let mut acc = 0.0;
            let mut sector = values.len() - 1;
            for (i, v) in values.iter().enumerate() {
                acc += v / total;
                if frac < acc {
                    sector = i;
                    break;
                }
            }
            let ch = SECTOR_GLYPHS[sector % SECTOR_GLYPHS.len()];
            buf.set(
                plot.x.saturating_add(dx),
                plot.y.saturating_add(dy),
                Cell::new(ch, chart.foreground_color, chart.background_color),
            );
        }
    }
    // Leyenda `etiqueta NN%` bajo el disco.
    for (i, pt) in chart.series.iter().enumerate().take(legend_rows as usize) {
        let pct = (pt.value.max(0.0) * 100.0 / total).round() as u16;
        let line = format!("{} {}%", pt.label, pct.min(100));
        draw_text(
            buf,
            plot.x,
            plot.y.saturating_add(disc_h).saturating_add(i as u16),
            &fit_text(&line, plot.w),
            Attr::new(chart.foreground_color, chart.background_color),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<ChartPoint> {
        vec![
            ChartPoint::new("Ene", 10.0),
            ChartPoint::new("Feb", 30.0),
            ChartPoint::new("Mar", 20.0),
        ]
    }

    #[test]
    fn bars_scale_with_max() {
        assert_eq!(bar_heights(&sample(), 10), vec![3, 10, 7]);
        assert_eq!(bar_heights(&[], 10), vec![]);
        assert_eq!(bar_heights(&sample(), 0), vec![0, 0, 0]);
    }

    #[test]
    fn bars_draw_body_and_3d_shade() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 14, t.desktop);
        let chart = TuiChart::new(ChartKind::Bars3D, "Vtas", sample());
        tuichart_draw(&mut b, Rect::new(1, 1, 30, 12), &chart);
        // Cuerpo `\u{2588}` y profundidad `\u{2592}` presentes.
        let mut body = 0;
        let mut shade = 0;
        for y in 0..14u16 {
            for x in 0..40u16 {
                match b.get(x, y).unwrap().ch {
                    '\u{2588}' => body += 1,
                    '\u{2592}' => shade += 1,
                    _ => {}
                }
            }
        }
        assert!(body > 10, "cuerpo visible");
        assert!(shade > 2, "profundidad 3D visible");
    }

    #[test]
    fn line_produces_braille_dots() {
        let dots = line_dots(&[1.0, 5.0, 3.0, 8.0], 10, 4);
        assert!(!dots.is_empty());
        for (_, _, m) in &dots {
            let ch = braille_char(*m);
            assert!((0x2800..=0x28FF).contains(&(ch as u32)));
        }
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 12, t.desktop);
        let chart = TuiChart::new(
            ChartKind::Line,
            "Tendencia",
            vec![
                ChartPoint::new("a", 1.0),
                ChartPoint::new("b", 5.0),
                ChartPoint::new("c", 3.0),
            ],
        );
        tuichart_draw(&mut b, Rect::new(1, 1, 30, 10), &chart);
        let n = (0..40u16)
            .flat_map(|x| (0..12u16).map(move |y| (x, y)))
            .filter(|(x, y)| (0x2800..=0x28FF).contains(&(b.get(*x, *y).unwrap().ch as u32)))
            .count();
        assert!(n > 2);
    }

    #[test]
    fn pie_draws_disc_and_legend() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 16, t.desktop);
        let chart = TuiChart::new(
            ChartKind::Pie,
            "Cuota",
            vec![ChartPoint::new("A", 50.0), ChartPoint::new("B", 50.0)],
        );
        tuichart_draw(&mut b, Rect::new(1, 1, 24, 14), &chart);
        // Disco con glifos sectoriales + leyenda con %.
        let mut disc = 0;
        let mut pct = false;
        for y in 0..16u16 {
            for x in 0..40u16 {
                let ch = b.get(x, y).unwrap().ch;
                if SECTOR_GLYPHS.contains(&ch) {
                    disc += 1;
                }
                if ch == '%' {
                    pct = true;
                }
            }
        }
        assert!(disc > 8);
        assert!(pct, "leyenda con %");
        // Sectores proporcionales: mitad y mitad.
        let s = pie_sectors(&[50.0, 50.0], 100);
        assert_eq!(s.iter().filter(|&&i| i == 0).count(), 50);
    }

    #[test]
    fn empty_series_shows_placeholder() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(30, 8, t.desktop);
        let chart = TuiChart::new(ChartKind::Bars3D, "V", vec![]);
        tuichart_draw(&mut b, Rect::new(1, 1, 20, 6), &chart);
        // plot = (2,3): título en y=1, área desde y=3.
        assert_eq!(b.get(2, 3).unwrap().ch, '(');
    }
}
