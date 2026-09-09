//! Micro-diálogo de fechas (`CalendarPicker`).
//!
//! Cuadrícula clásica de 7 columnas (L M M J V S D) del mes, selección
//! rápida con flechas. Matemática civil propia (Hinnant), sin dependencias:
//! días julianos ↔ calendario + días bisiestos.
//! Expone los 4 parámetros globales (`foreground_color`,
//! `background_color`, `border_color`, `has_shadow`).

use crossterm::event::KeyCode;

use crate::core::{Attr, Buffer, Cell, Color, Rect};
use crate::prim::{draw_text, fit_text, shadow, visible_len};

const WEEKDAYS: [&str; 7] = ["L", "M", "M", "J", "V", "S", "D"];
const MONTHS: [&str; 12] = [
    "Enero",
    "Febrero",
    "Marzo",
    "Abril",
    "Mayo",
    "Junio",
    "Julio",
    "Agosto",
    "Septiembre",
    "Octubre",
    "Noviembre",
    "Diciembre",
];

/// Días desde 1970-01-01 (puede ser negativo).
pub fn days_from_civil(y: i32, m: u8, d: u8) -> i64 {
    let (mut y, m) = (y as i64, m as i64);
    y -= (m <= 2) as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Inversa de `days_from_civil`.
pub fn civil_from_days(z: i64) -> (i32, u8, u8) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
    ((y + (m <= 2) as i64) as i32, m, d)
}

/// Día de semana Lunes=0..Domingo=6.
pub fn weekday(y: i32, m: u8, d: u8) -> u8 {
    ((days_from_civil(y, m, d) + 3).rem_euclid(7)) as u8
}

pub fn is_leap(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

pub fn days_in_month(y: i32, m: u8) -> u8 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Micro-diálogo de fecha.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarPicker {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub foreground_color: Color,
    pub background_color: Color,
    pub header_fg: Color,
    pub selected_fg: Color,
    pub selected_bg: Color,
    pub border_color: Option<Color>,
    pub has_shadow: bool,
}

impl CalendarPicker {
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        let mut c = Self {
            year,
            month: month.clamp(1, 12),
            day: day.max(1),
            foreground_color: Color::Black,
            background_color: Color::White,
            header_fg: Color::White,
            selected_fg: Color::White,
            selected_bg: Color::Navy,
            border_color: None,
            has_shadow: true,
        };
        c.day = c.day.min(days_in_month(c.year, c.month));
        c
    }

    /// Mes actual del sistema (vía `SystemTime`, sin dependencias).
    pub fn current() -> Self {
        let days = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() / 86400)
            .unwrap_or(0) as i64;
        let (y, m, d) = civil_from_days(days);
        Self::new(y, m, d)
    }

    fn add_days(&mut self, delta: i64) {
        let (y, m, d) = civil_from_days(days_from_civil(self.year, self.month, self.day) + delta);
        self.year = y;
        self.month = m;
        self.day = d;
    }

    fn add_months(&mut self, delta: i32) {
        let total = (self.year * 12 + self.month as i32 - 1) + delta;
        self.year = total.div_euclid(12);
        self.month = (total.rem_euclid(12) + 1) as u8;
        self.day = self.day.min(days_in_month(self.year, self.month));
    }
}

/// Origen (x0, y0) de la cuadrícula dentro de `rect` (título + cabecera).
fn grid_origin(rect: Rect, bordered: bool) -> (u16, u16) {
    (
        rect.x.saturating_add(if bordered { 1 } else { 0 }),
        rect.y
            .saturating_add(2)
            .saturating_add(if bordered { 1 } else { 0 }),
    )
}

/// Dibuja título + semana + días (seleccionado invertido).
pub fn calendar_draw(buf: &mut Buffer, rect: Rect, cal: &CalendarPicker) {
    if rect.w < 22 || rect.h < 9 {
        return;
    }
    let bordered = cal.border_color.is_some();
    if cal.has_shadow {
        shadow(buf, rect, crate::core::Theme::clipper());
    }
    buf.fill_rect(
        rect,
        Cell::new(' ', cal.foreground_color, cal.background_color),
    );
    let title = format!("{} {}", MONTHS[(cal.month - 1) as usize], cal.year);
    draw_text(
        buf,
        rect.x
            .saturating_add(rect.w.saturating_sub(visible_len(&title)) / 2),
        rect.y.saturating_add(if bordered { 1 } else { 0 }),
        &fit_text(&title, rect.w.saturating_sub(2)),
        Attr::bold(cal.foreground_color, cal.background_color),
    );
    let (x0, y0) = grid_origin(rect, bordered);
    for (i, wd) in WEEKDAYS.iter().enumerate() {
        draw_text(
            buf,
            x0.saturating_add(1).saturating_add(i as u16 * 3),
            y0,
            wd,
            Attr::bold(cal.header_fg, cal.background_color),
        );
    }
    let first = weekday(cal.year, cal.month, 1);
    let ndays = days_in_month(cal.year, cal.month);
    for day in 1..=ndays {
        let pos = first as usize + (day - 1) as usize;
        let (col, row) = (pos % 7, pos / 7);
        let x = x0.saturating_add(1).saturating_add(col as u16 * 3);
        let y = y0.saturating_add(1).saturating_add(row as u16);
        let sel = day == cal.day;
        let (fg, bg) = if sel {
            (cal.selected_fg, cal.selected_bg)
        } else {
            (cal.foreground_color, cal.background_color)
        };
        buf.fill_rect(Rect::new(x, y, 2, 1), Cell::new(' ', fg, bg));
        draw_text(buf, x, y, &format!("{day:2}"), Attr::new(fg, bg));
    }
    if bordered {
        let b = cal.border_color.unwrap_or(Color::Black);
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
pub enum CalNav {
    Stay,
    Move(i32, u8, u8),
    Accept(i32, u8, u8),
}

/// Flechas mueven días (con cambio de mes), `PgUp/PgDn` meses,
/// `Enter` acepta, `Esc` se trata fuera (cancela el diálogo).
pub fn calendar_key(cal: &mut CalendarPicker, code: KeyCode) -> CalNav {
    use CalNav::*;
    match code {
        KeyCode::Left => {
            cal.add_days(-1);
            Move(cal.year, cal.month, cal.day)
        }
        KeyCode::Right => {
            cal.add_days(1);
            Move(cal.year, cal.month, cal.day)
        }
        KeyCode::Up => {
            cal.add_days(-7);
            Move(cal.year, cal.month, cal.day)
        }
        KeyCode::Down => {
            cal.add_days(7);
            Move(cal.year, cal.month, cal.day)
        }
        KeyCode::PageUp => {
            cal.add_months(-1);
            Move(cal.year, cal.month, cal.day)
        }
        KeyCode::PageDown => {
            cal.add_months(1);
            Move(cal.year, cal.month, cal.day)
        }
        KeyCode::Home => {
            cal.day = 1;
            Move(cal.year, cal.month, cal.day)
        }
        KeyCode::End => {
            cal.day = days_in_month(cal.year, cal.month);
            Move(cal.year, cal.month, cal.day)
        }
        KeyCode::Enter => Accept(cal.year, cal.month, cal.day),
        _ => Stay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Theme;

    #[test]
    fn civil_roundtrips_and_known_weekdays() {
        assert_eq!(civil_from_days(days_from_civil(2026, 9, 8)), (2026, 9, 8));
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // 2000-01-01 fue sábado (Lun=0 → 5).
        assert_eq!(weekday(2000, 1, 1), 5);
        // Septiembre 2026 empieza en martes.
        assert_eq!(weekday(2026, 9, 1), 1);
        assert_eq!(days_in_month(2026, 9), 30);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2026, 2), 28);
    }

    #[test]
    fn arrows_roll_months() {
        let mut c = CalendarPicker::new(2026, 9, 30);
        calendar_key(&mut c, KeyCode::Right);
        assert_eq!((c.year, c.month, c.day), (2026, 10, 1));
        calendar_key(&mut c, KeyCode::Left);
        assert_eq!((c.year, c.month, c.day), (2026, 9, 30));
        calendar_key(&mut c, KeyCode::PageDown);
        assert_eq!((c.year, c.month), (2026, 10));
        assert_eq!(c.day, 30);
        // Febrero recorta al día válido.
        let mut f = CalendarPicker::new(2026, 1, 31);
        calendar_key(&mut f, KeyCode::PageDown);
        assert_eq!((f.month, f.day), (2, 28));
    }

    #[test]
    fn draws_grid_with_selection() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 14, t.desktop);
        let c = CalendarPicker::new(2026, 9, 8);
        calendar_draw(&mut b, Rect::new(2, 1, 24, 10), &c);
        // Título centrado ("Septiembre 2026" en x=6) + cabecera + día 8.
        assert_eq!(b.get(6, 1).unwrap().ch, 'S'); // Septiembre
        assert_eq!(b.get(3, 3).unwrap().ch, 'L');
        // Día 1 (martes, col 1): x = 2+1+3 = 6, y = 3+1 = 4.
        assert_eq!(b.get(7, 4).unwrap().ch, '1');
        // Día 8 misma columna, resaltado navy.
        assert_eq!(b.get(7, 5).unwrap().bg, c.selected_bg);
    }
}
