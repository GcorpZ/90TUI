//! Gráficos analíticos de alta densidad en modo texto (`TuiChart`).
//!
//! Tres modos sobre la misma serie `(etiqueta, valor)`:
//! 1. **Barras** (`Bars3D`): relleno sólido uniforme + escala Y a la
//!    izquierda (Max, Max/2, 0) + etiqueta centrada en la base.
//! 2. **Líneas** (`Line`): polilínea Bresenham sobre el `VirtualCanvas`
//!    sub-celular (2x4, `U+2800` + bits) con contraste estricto sobre
//!    fondo claro, eje Y con etiquetas (`┤`/`┼`), base X (`─┼┴`) y badge
//!    invertido con el último valor.
//! 3. **Distribución** (`Pie`): barra horizontal 100% apilada de 2 filas
//!    (bloques sólidos por categoría, sin tramas) + leyenda
//!    `■ A (50%)` con el color de cada sección.
//!
//! Más `Candle`/`candle_draw`: velas japonesas OHLC para trading
//! (mecha `│`, cuerpo `█`, verde si cierra igual/arriba, rojo si baja).
//!
//! Expone los 4 parámetros globales (`foreground_color`,
//! `background_color`, `border_color`, `has_shadow`).

use crate::core::{Attr, Buffer, Cell, Color, Rect, Theme};
use crate::prim::{draw_text, fit_text, shadow, visible_len, VirtualCanvas};

const FULL: char = '\u{2588}';

/// Ancho reservado al eje Y: 5 de número + 1 aire + 1 conector.
const AXIS_W: u16 = 7;

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
    /// Color de barras / sectores / profundidad (la curva `Line` usa
    /// contraste estricto y lo ignora sobre fondo claro).
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

/// Etiqueta numérica de eje en exactamente 5 celdas (nunca rompe la
/// grilla): `100.0`, ` 50.0`, `  0.0`.
fn axis_num(v: f64) -> String {
    let s = format!("{v:.1}");
    let n = s.chars().count();
    if n > 5 {
        s.chars().skip(n - 5).collect()
    } else {
        format!("{s:>5}")
    }
}

// --- Barras sólidas con escala ---

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
    const BW: u16 = 3; // ancho uniforme de barra
    const GAP: u16 = 1;
    if plot.w < AXIS_W + BW + 1 || plot.h < 3 {
        return;
    }
    let max = chart.max_value();
    let fg = chart.foreground_color;
    let bg = chart.background_color;
    let label_row = plot.bottom().saturating_sub(1);
    let area_h = plot.h.saturating_sub(1);
    let mid_row = plot.y.saturating_add(area_h / 2);
    // Eje Y: Max arriba, Max/2 al medio, 0 en la base.
    draw_text(buf, plot.x, plot.y, &axis_num(max), Attr::new(fg, bg));
    buf.set(
        plot.x.saturating_add(6),
        plot.y,
        Cell::with_attr('\u{2524}', Attr::new(fg, bg)),
    );
    if mid_row != plot.y && mid_row != label_row {
        draw_text(
            buf,
            plot.x,
            mid_row,
            &axis_num(max / 2.0),
            Attr::new(fg, bg),
        );
        buf.set(
            plot.x.saturating_add(6),
            mid_row,
            Cell::with_attr('\u{2524}', Attr::new(fg, bg)),
        );
    }
    for y in plot.y..label_row {
        if y == plot.y || y == mid_row {
            continue;
        }
        buf.set(
            plot.x.saturating_add(6),
            y,
            Cell::with_attr('\u{2502}', Attr::new(fg, bg)),
        );
    }
    draw_text(buf, plot.x, label_row, &axis_num(0.0), Attr::new(fg, bg));
    buf.set(
        plot.x.saturating_add(6),
        label_row,
        Cell::with_attr('\u{253c}', Attr::new(fg, bg)),
    );
    if max <= 0.0 {
        return;
    }
    // Barras sólidas, sin tramas ni profundidad.
    let heights = bar_heights(&chart.series, area_h);
    let x0 = plot.x.saturating_add(AXIS_W);
    for (i, (pt, bh)) in chart.series.iter().zip(heights.iter()).enumerate() {
        let x = x0.saturating_add(i as u16 * (BW + GAP));
        if x.saturating_add(BW) > plot.right() {
            break;
        }
        let top = label_row.saturating_sub(*bh);
        for y in top..label_row {
            for dx in 0..BW {
                buf.set(x.saturating_add(dx), y, Cell::new(FULL, fg, bg));
            }
        }
        // Etiqueta centrada en la base (recortada al ancho de barra).
        let lab: String = pt.label.chars().take(BW as usize).collect();
        let lw = visible_len(&lab);
        draw_text(
            buf,
            x.saturating_add(BW.saturating_sub(lw) / 2),
            label_row,
            &lab,
            Attr::new(fg, bg),
        );
    }
}

// --- Líneas braille (motor: VirtualCanvas + Bresenham) ---

/// Color de curva con contraste estricto: sobre fondo blanco siempre
/// `Navy` (el amarillo del showroom queda anulado); si no, el accent.
fn curve_fg(chart: &TuiChart) -> Color {
    if chart.background_color == Color::White || chart.accent_color == Color::Yellow {
        Color::Navy
    } else {
        chart.accent_color
    }
}

/// Lienzo con la serie rasterizada: valores → micro-píxeles (2x4 por
/// celda, píxel virtual ~cuadrado) unidos con Bresenham.
fn line_canvas(values: &[f64], cols: u16, rows: u16) -> VirtualCanvas {
    let mut canvas = VirtualCanvas::new(cols, rows);
    if values.is_empty() || cols == 0 || rows == 0 {
        return canvas;
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
    // Segmentos con Bresenham (continuo, sin huecos).
    for w in pts.windows(2) {
        canvas.draw_line(w[0].0, w[0].1, w[1].0, w[1].1);
    }
    if pts.len() == 1 {
        canvas.set_pixel(pts[0].0, pts[0].1);
    }
    canvas
}

/// Puntos (col de celda, fila de celda, máscara) de la polilínea.
pub fn line_dots(values: &[f64], cols: u16, rows: u16) -> Vec<(u16, u16, u8)> {
    let canvas = line_canvas(values, cols, rows);
    let mut out = Vec::new();
    for cy in 0..rows as usize {
        for cx in 0..cols as usize {
            let mask = canvas.cell_mask(cx, cy);
            if mask != 0 {
                out.push((cx as u16, cy as u16, mask));
            }
        }
    }
    out
}

/// Columna de celda del punto i (para ticks y badge).
fn point_col(i: usize, n: usize, cw: u16) -> u16 {
    if n <= 1 {
        0
    } else {
        (i * (cw as usize - 1) / (n - 1)) as u16
    }
}

fn draw_line(buf: &mut Buffer, plot: Rect, chart: &TuiChart) {
    let values: Vec<f64> = chart.series.iter().map(|p| p.value).collect();
    let fg = curve_fg(chart);
    let pfg = chart.foreground_color;
    let bg = chart.background_color;
    if plot.w < AXIS_W + 3 || plot.h < 3 {
        // Sin lugar para ejes: curva pelada con contraste igual.
        line_canvas(&values, plot.w, plot.h).render_to_buffer(buf, plot, fg, bg);
        return;
    }
    let cw = plot.w.saturating_sub(AXIS_W);
    let ch = plot.h.saturating_sub(1); // última fila = base X
    let area = Rect::new(plot.x.saturating_add(AXIS_W), plot.y, cw, ch);
    line_canvas(&values, cw, ch).render_to_buffer(buf, area, fg, bg);
    // Eje Y: max arriba, mid al medio, min en la base.
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mid = (max + min) / 2.0;
    let mid_row = (ch - 1) / 2;
    let ay = |r: u16| plot.y.saturating_add(r);
    draw_text(buf, plot.x, ay(0), &axis_num(max), Attr::new(pfg, bg));
    buf.set(
        plot.x.saturating_add(6),
        ay(0),
        Cell::with_attr('\u{2524}', Attr::new(pfg, bg)),
    );
    if mid_row != 0 && mid_row != ch - 1 {
        draw_text(buf, plot.x, ay(mid_row), &axis_num(mid), Attr::new(pfg, bg));
        buf.set(
            plot.x.saturating_add(6),
            ay(mid_row),
            Cell::with_attr('\u{2524}', Attr::new(pfg, bg)),
        );
    }
    for r in 0..ch {
        if r == 0 || r == mid_row {
            continue;
        }
        buf.set(
            plot.x.saturating_add(6),
            ay(r),
            Cell::with_attr('\u{2502}', Attr::new(pfg, bg)),
        );
    }
    // Base X: min + `┼`, línea `─` con ticks `┴` bajo cada dato.
    let by = plot.y.saturating_add(ch);
    draw_text(buf, plot.x, by, &axis_num(min), Attr::new(pfg, bg));
    buf.set(
        plot.x.saturating_add(6),
        by,
        Cell::with_attr('\u{253c}', Attr::new(pfg, bg)),
    );
    let n = values.len();
    let mut tick = vec![false; cw as usize];
    for i in 0..n {
        let c = point_col(i, n, cw).min(cw.saturating_sub(1));
        tick[c as usize] = true;
    }
    for dx in 0..cw {
        let g = if tick[dx as usize] {
            '\u{2534}'
        } else {
            '\u{2500}'
        };
        buf.set(
            area.x.saturating_add(dx),
            by,
            Cell::with_attr(g, Attr::new(pfg, bg)),
        );
    }
    // Badge invertido con el último valor, al final de la curva.
    if let Some(last) = values.last() {
        let text = format!("{last:.1}");
        let w = visible_len(&text);
        if w > 0 && w <= cw {
            let lcx = point_col(n - 1, n, cw);
            let col = if lcx.saturating_add(1).saturating_add(w) <= cw {
                lcx.saturating_add(1)
            } else {
                lcx.saturating_sub(w)
            };
            let span = (max - min).max(1e-9);
            let dy = ((max - last) * (ch as usize * 4 - 1) as f64 / span).round() as usize;
            let lrow = (dy / 4).min(ch as usize - 1) as u16;
            draw_text(
                buf,
                area.x.saturating_add(col),
                ay(lrow),
                &text,
                Attr::new(Color::White, Color::Navy),
            );
        }
    }
}

// --- Distribución 100% apilada (slot `Pie`) ---

/// Paleta de secciones sobre fondo claro.
const DIST_COLORS: [Color; 8] = [
    Color::Blue,
    Color::Red,
    Color::Green,
    Color::Teal,
    Color::Yellow,
    Color::Navy,
    Color::Cyan,
    Color::Grey,
];

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

/// Anchos en celdas por sección (suman `bar_w`, resto mayor).
fn dist_widths(values: &[f64], bar_w: u16) -> Vec<u16> {
    let total: f64 = values.iter().map(|v| v.max(0.0)).sum();
    if total <= 0.0 || bar_w == 0 {
        return vec![0; values.len()];
    }
    let quotas: Vec<f64> = values
        .iter()
        .map(|v| v.max(0.0) * bar_w as f64 / total)
        .collect();
    let mut out: Vec<u16> = quotas.iter().map(|q| q.floor() as u16).collect();
    let mut rest = bar_w.saturating_sub(out.iter().sum());
    let mut frac: Vec<(usize, f64)> = quotas
        .iter()
        .enumerate()
        .map(|(i, q)| (i, q - q.floor()))
        .collect();
    frac.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (i, _) in frac {
        if rest == 0 {
            break;
        }
        out[i] = out[i].saturating_add(1);
        rest = rest.saturating_sub(1);
    }
    out
}

fn draw_pie(buf: &mut Buffer, plot: Rect, chart: &TuiChart) {
    let values: Vec<f64> = chart.series.iter().map(|p| p.value.max(0.0)).collect();
    let total = chart.total().max(0.0);
    const BAR_H: u16 = 2;
    if plot.h < BAR_H + 1 || plot.w < 4 {
        return;
    }
    // Barra continua de 2 filas, bloques sólidos por categoría.
    let widths = dist_widths(&values, plot.w);
    let mut x = plot.x;
    for (i, w) in widths.iter().enumerate() {
        let c = DIST_COLORS[i % DIST_COLORS.len()];
        for _ in 0..*w {
            for dy in 0..BAR_H {
                buf.set(x, plot.y.saturating_add(dy), Cell::new(' ', c, c));
            }
            x = x.saturating_add(1);
        }
    }
    // Leyenda `■ A (50%)` con el color de su sección, fluyendo por filas.
    let mut lx = plot.x;
    let mut ly = plot.y.saturating_add(BAR_H + 1);
    for (i, pt) in chart.series.iter().enumerate() {
        let c = DIST_COLORS[i % DIST_COLORS.len()];
        let pct = if total > 0.0 {
            (pt.value.max(0.0) * 100.0 / total).round() as u16
        } else {
            0
        };
        let entry = format!("{} ({}%)", pt.label, pct.min(100));
        let w = visible_len(&entry).saturating_add(2); // ■ + aire
        if lx.saturating_add(w) > plot.right() && lx != plot.x {
            lx = plot.x;
            ly = ly.saturating_add(1);
        }
        if ly >= plot.bottom() {
            break;
        }
        buf.set(
            lx,
            ly,
            Cell::with_attr('\u{25a0}', Attr::new(c, chart.background_color)),
        );
        draw_text(
            buf,
            lx.saturating_add(2),
            ly,
            &fit_text(&entry, plot.right().saturating_sub(lx.saturating_add(2))),
            Attr::new(chart.foreground_color, chart.background_color),
        );
        lx = lx.saturating_add(w).saturating_add(3);
    }
}

// --- Velas japonesas OHLC ---

/// Una vela financiera: apertura, máximo, mínimo, cierre.
#[derive(Clone, Debug, PartialEq)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl Candle {
    pub fn new(open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            open,
            high,
            low,
            close,
        }
    }

    /// Verde si cierra igual o arriba, rojo si baja.
    pub fn up(&self) -> bool {
        self.close >= self.open
    }
}

/// Dibuja velas (una columna por vela): mecha `│` en todo el rango y
/// cuerpo `█` entre apertura y cierre, verde/rojo según `up()`.
pub fn candle_draw(buf: &mut Buffer, rect: Rect, candles: &[Candle], bg: Color) {
    if candles.is_empty() || rect.is_empty() || rect.h == 0 {
        return;
    }
    let lo = candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
    let hi = candles
        .iter()
        .map(|c| c.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let span = (hi - lo).max(1e-9);
    let row_for = |v: f64| ((hi - v) * (rect.h - 1) as f64 / span).round() as u16;
    for (i, c) in candles.iter().enumerate() {
        let x = rect.x.saturating_add(i as u16);
        if x >= rect.right() {
            break;
        }
        let col = if c.up() { Color::Green } else { Color::Red };
        let r_hi = row_for(c.high).min(rect.h - 1);
        let r_lo = row_for(c.low).min(rect.h - 1);
        for r in r_hi..=r_lo {
            buf.set(x, rect.y.saturating_add(r), Cell::new('\u{2502}', col, bg));
        }
        let r_o = row_for(c.open).min(rect.h - 1);
        let r_c = row_for(c.close).min(rect.h - 1);
        let (top, bot) = (r_o.min(r_c), r_o.max(r_c));
        for r in top..=bot {
            buf.set(x, rect.y.saturating_add(r), Cell::new(FULL, col, bg));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn braille_char(mask: u8) -> char {
        char::from_u32(0x2800 + mask as u32).unwrap_or(' ')
    }

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
    fn bars_are_solid_with_axis_and_labels() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 14, t.desktop);
        let chart = TuiChart::new(ChartKind::Bars3D, "Vtas", sample());
        tuichart_draw(&mut b, Rect::new(1, 1, 30, 12), &chart);
        // Cuerpo sólido, CERO tramas `▒` en todo el buffer.
        let mut body = 0;
        for y in 0..14u16 {
            for x in 0..40u16 {
                match b.get(x, y).unwrap().ch {
                    '\u{2588}' => body += 1,
                    '\u{2592}' => panic!("trama 3D en ({x}, {y})"),
                    _ => {}
                }
            }
        }
        assert!(body > 10, "cuerpo visible");
        // Eje Y: Max arriba + `┤`, 0 en la base + `┼`.
        // plot = (2,3,28,10): eje en x=2, " 30.0" en y=3, base en y=12.
        assert_eq!(b.get(2, 3).unwrap().ch, ' ');
        assert_eq!(b.get(3, 3).unwrap().ch, '3');
        assert_eq!(b.get(8, 3).unwrap().ch, '\u{2524}');
        assert_eq!(b.get(8, 12).unwrap().ch, '\u{253c}');
        // Etiqueta completa centrada en la base (x0=9, "Ene").
        assert_eq!(b.get(9, 12).unwrap().ch, 'E');
        assert_eq!(b.get(10, 12).unwrap().ch, 'n');
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
        // plot = (2,3,28,8): curva Navy (contraste, jamás amarilla),
        // eje Y con `┤`, base X con `─`.
        let mut navy_curve = 0;
        let mut has_tick = false;
        let mut has_base = false;
        for y in 0..12u16 {
            for x in 0..40u16 {
                let c = b.get(x, y).unwrap();
                if (0x2800..=0x28FF).contains(&(c.ch as u32)) {
                    assert_eq!(c.fg, Color::Navy, "curva siempre Navy");
                    navy_curve += 1;
                }
                if c.ch == '\u{2524}' {
                    has_tick = true;
                }
                if c.ch == '\u{2500}' {
                    has_base = true;
                }
            }
        }
        assert!(navy_curve > 2);
        assert!(has_tick, "eje Y");
        assert!(has_base, "base X");
        // Badge invertido del último valor ("3.0" sobre Navy).
        let mut badge = false;
        for y in 0..12u16 {
            for x in 0..38u16 {
                let cells = [
                    b.get(x, y).unwrap(),
                    b.get(x + 1, y).unwrap(),
                    b.get(x + 2, y).unwrap(),
                ];
                if cells[0].ch == '3'
                    && cells[1].ch == '.'
                    && cells[2].ch == '0'
                    && cells.iter().all(|c| c.bg == Color::Navy)
                {
                    badge = true;
                }
            }
        }
        assert!(badge, "badge del último valor");
    }

    #[test]
    fn line_never_paints_yellow_on_white() {
        // El caso del showroom F4: accent amarillo + fondo blanco.
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 12, t.desktop);
        let mut chart = TuiChart::new(
            ChartKind::Line,
            "Tendencia",
            vec![
                ChartPoint::new("1", 2.0),
                ChartPoint::new("2", 8.0),
                ChartPoint::new("3", 5.0),
                ChartPoint::new("4", 9.0),
                ChartPoint::new("5", 6.0),
            ],
        );
        chart.accent_color = Color::Yellow;
        tuichart_draw(&mut b, Rect::new(1, 1, 30, 10), &chart);
        for y in 0..12u16 {
            for x in 0..40u16 {
                let c = b.get(x, y).unwrap();
                if (0x2800..=0x28FF).contains(&(c.ch as u32)) {
                    assert_ne!(c.fg, Color::Yellow, "amarillo sobre blanco");
                    assert_eq!(c.fg, Color::Navy);
                }
            }
        }
    }

    #[test]
    fn line_is_continuous_without_gaps() {
        // Diagonal empinada: Bresenham toca todas las filas del plot.
        let dots = line_dots(&[0.0, 100.0], 6, 6);
        for row in 0..6u16 {
            assert!(
                dots.iter().any(|(_, cy, _)| *cy == row),
                "fila {row} vacía en diagonal"
            );
        }
        // Un solo valor = un solo dot.
        assert_eq!(line_dots(&[5.0], 6, 6).len(), 1);
        assert!(line_dots(&[], 6, 6).is_empty());
    }

    #[test]
    fn pie_is_stacked_bar_with_legend() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 16, t.desktop);
        let chart = TuiChart::new(
            ChartKind::Pie,
            "Cuota",
            vec![
                ChartPoint::new("A", 50.0),
                ChartPoint::new("B", 30.0),
                ChartPoint::new("C", 20.0),
            ],
        );
        tuichart_draw(&mut b, Rect::new(1, 1, 24, 14), &chart);
        // plot = (2,3,22,12): barra de 2 filas, A = mitad azul sólida.
        let mut blue = 0;
        let mut shade = false;
        for y in 3..5u16 {
            for x in 2..24u16 {
                let c = b.get(x, y).unwrap();
                if c.bg == Color::Blue {
                    blue += 1;
                }
                if matches!(c.ch, '\u{2591}' | '\u{2592}' | '\u{2593}') {
                    shade = true;
                }
            }
        }
        assert_eq!(blue, 2 * 11, "A ocupa la mitad sólida");
        assert!(!shade, "sin tramas");
        // Leyenda con ■ de colores + %.
        let mut squares = 0;
        let mut pct = false;
        for y in 0..16u16 {
            for x in 0..40u16 {
                let c = b.get(x, y).unwrap();
                if c.ch == '\u{25a0}' {
                    squares += 1;
                }
                if c.ch == '%' {
                    pct = true;
                }
            }
        }
        assert_eq!(squares, 3);
        assert!(pct, "leyenda con %");
        // Sectores proporcionales: mitad y mitad.
        let s = pie_sectors(&[50.0, 50.0], 100);
        assert_eq!(s.iter().filter(|&&i| i == 0).count(), 50);
    }

    #[test]
    fn candles_use_wick_body_and_trend_color() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(20, 10, t.desktop);
        let cs = vec![
            Candle::new(10.0, 15.0, 5.0, 14.0), // sube: verde
            Candle::new(14.0, 16.0, 8.0, 9.0),  // baja: rojo
        ];
        assert!(cs[0].up());
        assert!(!cs[1].up());
        candle_draw(&mut b, Rect::new(2, 1, 6, 8), &cs, t.desktop);
        // Cuerpo verde en col 2, rojo en col 3, mechas `│` presentes.
        let mut green_body = false;
        let mut red_body = false;
        let mut wick = false;
        for y in 1..9u16 {
            let a = b.get(2, y).unwrap();
            let c = b.get(3, y).unwrap();
            if a.ch == '\u{2588}' && a.fg == Color::Green {
                green_body = true;
            }
            if c.ch == '\u{2588}' && c.fg == Color::Red {
                red_body = true;
            }
            if a.ch == '\u{2502}' || c.ch == '\u{2502}' {
                wick = true;
            }
        }
        assert!(green_body && red_body && wick);
        // Vacío = no-op sin pánico.
        candle_draw(&mut b, Rect::new(2, 1, 6, 8), &[], t.desktop);
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
