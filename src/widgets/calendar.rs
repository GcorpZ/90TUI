//! Selector de fecha (`CalendarPicker`): campo contraído + popup flotante.
//!
//! * **Cerrado:** 1 fila con máscara ` [ DD/MM/AAAA ] [ <icono> ]`
//!   (icono Nerd Font `\u{f073}`). `Enter`/`Espacio` abre el popup.
//!   La máscara la elige el programador (`datemask`: `YYYY`/`YY`/`MM`/`M`/
//!   `DD`/`D`, resto literal); sin ella, estándar `DD/MM/AAAA`.
//! * **Abierto:** overlay de 23×10 con caja CP437 de doble línea,
//!   cabecera `«◄ MES AAAA ►»`, semana `Lu..Do`, cursor Clipper en el
//!   día y **hoy en negrita roja** (`today_fg`). Flechas mueven días,
//!   `PgUp`/`PgDn` meses, `Shift`/`Ctrl`+`PgUp`/`PgDn` años,
//!   `Enter` acepta y cierra, `Esc` restaura y cierra.
//! * Matemática civil propia (Hinnant), sin dependencias: días julianos
//!   ↔ calendario + bisiestos.
//! * Expone los 4 parámetros globales (`foreground_color`,
//!   `background_color`, `border_color`, `has_shadow`).

use crossterm::event::{KeyCode, KeyModifiers};

use crate::core::{Attr, Buffer, Cell, Color, Rect, Theme};
use crate::prim::{draw_text, fit_text, shadow, visible_len};

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

/// Icono del campo contraído (Nerd Font, calendario).
const CAL_ICON: char = '\u{f073}';
/// Tamaño fijo del popup: caja + cabecera + semana + 6 filas + caja.
const POPUP_W: u16 = 23;
const POPUP_H: u16 = 10;
/// Cabecera de columnas (20 celdas, centrada en el interior de 21).
const WEEK_HEAD: &str = "Lu Ma Mi Ju Vi Sa Do";

/// Caja CP437 de doble línea.
const BOX_TL: char = '\u{2554}'; // ╔
const BOX_TR: char = '\u{2557}'; // ╗
const BOX_BL: char = '\u{255a}'; // ╚
const BOX_BR: char = '\u{255d}'; // ╝
const BOX_H: char = '\u{2550}'; // ═
const BOX_V: char = '\u{2551}'; // ║
/// Navegación de la cabecera: `«`/`»` años rápidos, `◄`/`►` meses.
const HDR_PREV_YEAR: char = '\u{ab}'; // «
const HDR_PREV_MONTH: char = '\u{25c4}'; // ◄
const HDR_NEXT_MONTH: char = '\u{25ba}'; // ►
const HDR_NEXT_YEAR: char = '\u{bb}'; // »

/// Máscara estándar cuando `datemask` es `None`.
pub const DEFAULT_DATEMASK: &str = "DD/MM/YYYY";

/// Expande una máscara de fecha (`YYYY`/`YY`/`MM`/`M`/`DD`/`D`,
/// resto literal) sin dependencias.
pub fn apply_datemask(mask: &str, y: i32, m: u8, d: u8) -> String {
    const TOKENS: [&str; 6] = ["YYYY", "YY", "MM", "DD", "M", "D"];
    let mut out = String::new();
    let mut i = 0;
    while i < mask.len() {
        let rest = &mask[i..];
        if let Some(tok) = TOKENS.iter().find(|t| rest.starts_with(*t)) {
            match *tok {
                "YYYY" => out.push_str(&format!("{y:04}")),
                "YY" => out.push_str(&format!("{:02}", y.rem_euclid(100))),
                "MM" => out.push_str(&format!("{m:02}")),
                "M" => out.push_str(&format!("{m}")),
                "DD" => out.push_str(&format!("{d:02}")),
                _ => out.push_str(&format!("{d}")),
            }
            i += tok.len();
        } else {
            let ch = rest.chars().next().unwrap_or('?');
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// Fecha de hoy del sistema como `(año, mes, día)`.
fn today_ymd() -> (i32, u8, u8) {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() / 86400)
        .unwrap_or(0) as i64;
    civil_from_days(days)
}

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

/// Selector de fecha con popup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarPicker {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub is_open: bool,
    pub foreground_color: Color,
    pub background_color: Color,
    pub header_fg: Color,
    pub selected_fg: Color,
    pub selected_bg: Color,
    /// Tinta del día de hoy (siempre en negrita).
    pub today_fg: Color,
    /// Máscara del campo (`YYYY`/`YY`/`MM`/`M`/`DD`/`D`); `None` = estándar.
    pub datemask: Option<String>,
    pub border_color: Option<Color>,
    pub has_shadow: bool,
    /// Valor confirmado al abrir (para restaurar con `Esc`).
    saved: (i32, u8, u8),
}

impl CalendarPicker {
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        let mut c = Self {
            year,
            month: month.clamp(1, 12),
            day: day.max(1),
            is_open: false,
            foreground_color: Color::Black,
            background_color: Color::White,
            header_fg: Color::White,
            selected_fg: Color::White,
            selected_bg: Color::Navy,
            today_fg: Color::Red,
            datemask: None,
            border_color: None,
            has_shadow: true,
            saved: (year, month.clamp(1, 12), day.max(1)),
        };
        c.day = c.day.min(days_in_month(c.year, c.month));
        c.saved = (c.year, c.month, c.day);
        c
    }

    /// Mes actual del sistema (vía `SystemTime`, sin dependencias).
    pub fn current() -> Self {
        let (y, m, d) = today_ymd();
        Self::new(y, m, d)
    }

    /// Fecha confirmada según `datemask` (o estándar `DD/MM/AAAA`).
    pub fn formatted(&self) -> String {
        let mask = self.datemask.as_deref().unwrap_or(DEFAULT_DATEMASK);
        apply_datemask(mask, self.year, self.month, self.day)
    }

    /// Abre el popup (guarda el valor para un posible `Esc`).
    pub fn open(&mut self) {
        self.saved = (self.year, self.month, self.day);
        self.is_open = true;
    }

    /// Cierra aceptando la fecha visible.
    pub fn accept(&mut self) {
        self.is_open = false;
    }

    /// Cierra descartando (restaura lo que había al abrir).
    pub fn cancel(&mut self) {
        (self.year, self.month, self.day) = self.saved;
        self.is_open = false;
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

    fn add_years(&mut self, delta: i32) {
        self.add_months(delta.saturating_mul(12));
    }
}

/// Ancho de la fila contraída (1 fila de alto): ` [ fecha ] [ X ]`.
pub fn calendar_field_width(cal: &CalendarPicker) -> u16 {
    cal.formatted().chars().count() as u16 + 11
}

/// Rect del popup abierto: debajo del campo si cabe, si no encima.
/// Siempre clampado a pantalla.
pub fn calendar_popup_rect(closed: Rect, screen: Rect) -> Rect {
    let below = closed.y.saturating_add(1);
    let y = if below.saturating_add(POPUP_H) <= screen.bottom() {
        below
    } else {
        closed.y.saturating_sub(POPUP_H)
    };
    Rect::new(closed.x, y, POPUP_W, POPUP_H).clamp_in(screen)
}

/// Dibuja el campo contraído y, si `is_open`, el popup encima.
/// El llamante lo invoca al final (el overlay queda sobre todo).
pub fn calendar_draw(buf: &mut Buffer, rect: Rect, cal: &CalendarPicker) {
    if rect.is_empty() {
        return;
    }
    let base = Attr::new(cal.foreground_color, cal.background_color);
    buf.fill_rect(
        Rect::new(rect.x, rect.y, rect.w, 1),
        Cell::new(' ', cal.foreground_color, cal.background_color),
    );
    let line = format!(" [ {} ] [ {} ]", cal.formatted(), CAL_ICON);
    draw_text(buf, rect.x, rect.y, &fit_text(&line, rect.w), base);
    if !cal.is_open {
        return;
    }
    let pop = calendar_popup_rect(rect, buf.bounds());
    if pop.w < POPUP_W || pop.h < POPUP_H {
        return;
    }
    if cal.has_shadow {
        shadow(buf, pop, Theme::clipper());
    }
    buf.fill_rect(
        pop,
        Cell::new(' ', cal.foreground_color, cal.background_color),
    );
    // Caja CP437 de doble línea.
    let bc = Attr::new(
        cal.border_color.unwrap_or(cal.foreground_color),
        cal.background_color,
    );
    for x in pop.x.saturating_add(1)..pop.right().saturating_sub(1) {
        buf.set(x, pop.y, Cell::with_attr(BOX_H, bc));
        buf.set(
            x,
            pop.bottom().saturating_sub(1),
            Cell::with_attr(BOX_H, bc),
        );
    }
    for y in pop.y.saturating_add(1)..pop.bottom().saturating_sub(1) {
        buf.set(pop.x, y, Cell::with_attr(BOX_V, bc));
        buf.set(pop.right().saturating_sub(1), y, Cell::with_attr(BOX_V, bc));
    }
    buf.set(pop.x, pop.y, Cell::with_attr(BOX_TL, bc));
    buf.set(
        pop.right().saturating_sub(1),
        pop.y,
        Cell::with_attr(BOX_TR, bc),
    );
    buf.set(
        pop.x,
        pop.bottom().saturating_sub(1),
        Cell::with_attr(BOX_BL, bc),
    );
    buf.set(
        pop.right().saturating_sub(1),
        pop.bottom().saturating_sub(1),
        Cell::with_attr(BOX_BR, bc),
    );
    // Cabecera: `«◄ MES AAAA ►»` en mayúsculas, centrada y en negrita.
    let inner = pop.w.saturating_sub(2);
    let title = format!(
        "{} {}",
        MONTHS[(cal.month - 1) as usize].to_uppercase(),
        cal.year
    );
    let head = format!("{HDR_PREV_YEAR}{HDR_PREV_MONTH} {title} {HDR_NEXT_MONTH}{HDR_NEXT_YEAR}");
    let head = fit_text(&head, inner);
    draw_text(
        buf,
        pop.x
            .saturating_add(1)
            .saturating_add(inner.saturating_sub(visible_len(&head)) / 2),
        pop.y.saturating_add(1),
        &head,
        Attr::bold(cal.foreground_color, cal.background_color),
    );
    // Semana + días (celda de 3: cursor Clipper en el seleccionado).
    let week = fit_text(WEEK_HEAD, inner);
    draw_text(
        buf,
        pop.x
            .saturating_add(1)
            .saturating_add(inner.saturating_sub(visible_len(&week)) / 2),
        pop.y.saturating_add(2),
        &week,
        Attr::bold(cal.header_fg, cal.background_color),
    );
    let first = weekday(cal.year, cal.month, 1);
    let ndays = days_in_month(cal.year, cal.month);
    let (ty, tm, td) = today_ymd();
    for day in 1..=ndays {
        let (x, y) = day_xy(pop, first, day);
        let sel = day == cal.day;
        let is_today = cal.year == ty && cal.month == tm && day == td;
        // El cursor manda; si no, hoy va en negrita roja.
        let (fg, bg) = if sel {
            (cal.selected_fg, cal.selected_bg)
        } else if is_today {
            (cal.today_fg, cal.background_color)
        } else {
            (cal.foreground_color, cal.background_color)
        };
        let a = Attr::new(fg, bg);
        let a = if !sel && is_today {
            Attr::bold(fg, bg)
        } else {
            a
        };
        buf.fill_rect(Rect::new(x, y, 3, 1), Cell::new(' ', fg, bg));
        draw_text(buf, x, y, &format!("{day:2}"), a);
    }
}

/// Celda (x, y) del día dentro del popup (celdas de 3).
fn day_xy(pop: Rect, first: u8, day: u8) -> (u16, u16) {
    let pos = first as usize + (day - 1) as usize;
    (
        pop.x.saturating_add(1).saturating_add((pos % 7) as u16 * 3),
        pop.y.saturating_add(3).saturating_add((pos / 7) as u16),
    )
}

/// Navegación pura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalNav {
    Stay,
    Opened,
    Move(i32, u8, u8),
    Accepted(i32, u8, u8),
    Cancelled,
}

/// Cerrado: `Enter`/`Espacio` abre. Abierto: flechas días,
/// `PgUp`/`PgDn` meses, `Home`/`End` extremos, `Enter` acepta y
/// cierra, `Esc` restaura y cierra. Sin modificadores (años con
/// `calendar_key_mod`).
pub fn calendar_key(cal: &mut CalendarPicker, code: KeyCode) -> CalNav {
    calendar_key_mod(cal, code, KeyModifiers::empty())
}

/// Como `calendar_key`, más `Shift`/`Ctrl`+`PgUp`/`PgDn` = años.
pub fn calendar_key_mod(cal: &mut CalendarPicker, code: KeyCode, mods: KeyModifiers) -> CalNav {
    use CalNav::*;
    if !cal.is_open {
        return match code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                cal.open();
                Opened
            }
            _ => Stay,
        };
    }
    let years = mods.intersects(KeyModifiers::SHIFT | KeyModifiers::CONTROL);
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
            if years {
                cal.add_years(-1);
            } else {
                cal.add_months(-1);
            }
            Move(cal.year, cal.month, cal.day)
        }
        KeyCode::PageDown => {
            if years {
                cal.add_years(1);
            } else {
                cal.add_months(1);
            }
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
        KeyCode::Enter => {
            cal.accept();
            Accepted(cal.year, cal.month, cal.day)
        }
        KeyCode::Esc => {
            cal.cancel();
            Cancelled
        }
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
    fn closed_opens_with_enter_or_space() {
        let mut c = CalendarPicker::new(2026, 9, 15);
        assert!(!c.is_open);
        assert_eq!(calendar_key(&mut c, KeyCode::Enter), CalNav::Opened);
        assert!(c.is_open);
        let mut c2 = CalendarPicker::new(2026, 9, 15);
        assert_eq!(calendar_key(&mut c2, KeyCode::Char(' ')), CalNav::Opened);
        // Cerrado, lo demás no hace nada.
        let mut c3 = CalendarPicker::new(2026, 9, 15);
        assert_eq!(calendar_key(&mut c3, KeyCode::Right), CalNav::Stay);
        assert_eq!((c3.year, c3.month, c3.day), (2026, 9, 15));
    }

    #[test]
    fn open_navigates_months_and_years() {
        let mut c = CalendarPicker::new(2026, 9, 30);
        calendar_key(&mut c, KeyCode::Enter); // abre
        assert_eq!(
            calendar_key(&mut c, KeyCode::Right),
            CalNav::Move(2026, 10, 1)
        );
        assert_eq!(
            calendar_key(&mut c, KeyCode::Left),
            CalNav::Move(2026, 9, 30)
        );
        assert_eq!(
            calendar_key(&mut c, KeyCode::PageDown),
            CalNav::Move(2026, 10, 30)
        );
        // Shift+PgDn = año (Ctrl es la alternativa configurable).
        assert_eq!(
            calendar_key_mod(&mut c, KeyCode::PageDown, KeyModifiers::SHIFT),
            CalNav::Move(2027, 10, 30)
        );
        assert_eq!(
            calendar_key_mod(&mut c, KeyCode::PageUp, KeyModifiers::CONTROL),
            CalNav::Move(2026, 10, 30)
        );
        // Febrero recorta al día válido.
        let mut f = CalendarPicker::new(2026, 1, 31);
        calendar_key(&mut f, KeyCode::Enter);
        calendar_key(&mut f, KeyCode::PageDown);
        assert_eq!((f.month, f.day), (2, 28));
    }

    #[test]
    fn enter_accepts_and_esc_restores() {
        let mut c = CalendarPicker::new(2026, 9, 15);
        calendar_key(&mut c, KeyCode::Enter);
        calendar_key(&mut c, KeyCode::Right);
        calendar_key(&mut c, KeyCode::Right);
        assert_eq!(
            calendar_key(&mut c, KeyCode::Enter),
            CalNav::Accepted(2026, 9, 17)
        );
        assert!(!c.is_open);
        assert_eq!((c.year, c.month, c.day), (2026, 9, 17));
        // Esc restaura lo confirmado al abrir.
        let mut d = CalendarPicker::new(2026, 9, 15);
        calendar_key(&mut d, KeyCode::Enter);
        calendar_key(&mut d, KeyCode::Right);
        assert_eq!(calendar_key(&mut d, KeyCode::Esc), CalNav::Cancelled);
        assert!(!d.is_open);
        assert_eq!((d.year, d.month, d.day), (2026, 9, 15));
    }

    #[test]
    fn today_is_bold_red_and_selection_wins() {
        let t = Theme::clipper();
        let (ty, tm, td) = today_ymd();
        // Selección distinta de hoy para ver ambos resaltados.
        let sel = if td > 1 { td - 1 } else { td + 1 };
        let mut c = CalendarPicker::new(ty, tm, sel);
        calendar_key(&mut c, KeyCode::Enter);
        let mut b = Buffer::blank(40, 16, t.desktop);
        let r = Rect::new(2, 1, 21, 1);
        calendar_draw(&mut b, r, &c);
        let pop = calendar_popup_rect(r, Rect::new(0, 0, 40, 16));
        let first = weekday(ty, tm, 1);
        // Hoy: tinta roja + negrita (sin ser el cursor).
        let (tx, ty_) = day_xy(pop, first, td);
        let today_cell = b.get(tx, ty_).unwrap();
        assert_eq!(today_cell.fg, Color::Red);
        assert!(today_cell.bold);
        // El cursor Clipper manda sobre el rojo de hoy.
        let (sx, sy) = day_xy(pop, first, sel);
        assert_eq!(b.get(sx, sy).unwrap().bg, c.selected_bg);
        // Y si hoy ES la selección, el cursor gana.
        let mut c2 = CalendarPicker::new(ty, tm, td);
        calendar_key(&mut c2, KeyCode::Enter);
        calendar_draw(&mut b, r, &c2);
        assert_eq!(b.get(tx, ty_).unwrap().bg, c2.selected_bg);
    }

    #[test]
    fn datemask_formats_and_sizes_field() {
        let mut c = CalendarPicker::new(2026, 9, 8);
        // Estándar por defecto.
        assert_eq!(c.formatted(), "08/09/2026");
        assert_eq!(calendar_field_width(&c), 21);
        // ISO con literal intacto.
        c.datemask = Some("YYYY-MM-DD".into());
        assert_eq!(c.formatted(), "2026-09-08");
        assert_eq!(calendar_field_width(&c), 21);
        // Sin relleno + año corto.
        c.datemask = Some("M/D/YY".into());
        assert_eq!(c.formatted(), "9/8/26");
        assert_eq!(calendar_field_width(&c), 6 + 11);
        // Texto con minúsculas no se toca (tokens en mayúsculas).
        c.datemask = Some("Fecha: DD.MM.YYYY".into());
        assert_eq!(c.formatted(), "Fecha: 08.09.2026");
        // Helper público directo.
        assert_eq!(apply_datemask("DD/MM/YYYY", 2026, 9, 8), "08/09/2026");
        // El campo dibuja la máscara custom.
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 6, t.desktop);
        c.datemask = Some("YYYY-MM-DD".into());
        calendar_draw(&mut b, Rect::new(2, 1, calendar_field_width(&c), 1), &c);
        assert_eq!(b.get(5, 1).unwrap().ch, '2');
        assert_eq!(b.get(8, 1).unwrap().ch, '6');
        assert_eq!(b.get(9, 1).unwrap().ch, '-');
        assert_eq!(b.get(11, 1).unwrap().ch, '9');
    }

    #[test]
    fn collapsed_field_draws_mask_and_icon() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 6, t.desktop);
        let c = CalendarPicker::new(2026, 9, 8);
        calendar_draw(&mut b, Rect::new(2, 1, 21, 1), &c);
        // ` [ 08/09/2026 ] [ <icon> ]`: corchete, fecha e icono NF.
        assert_eq!(b.get(2, 1).unwrap().ch, ' ');
        assert_eq!(b.get(3, 1).unwrap().ch, '[');
        assert_eq!(b.get(5, 1).unwrap().ch, '0');
        assert_eq!(b.get(8, 1).unwrap().ch, '0');
        assert_eq!(b.get(9, 1).unwrap().ch, '9');
        assert_eq!(b.get(20, 1).unwrap().ch, '\u{f073}');
        assert_eq!(b.get(22, 1).unwrap().ch, ']');
    }

    #[test]
    fn open_popup_draws_double_box_and_selection() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(40, 16, t.desktop);
        let mut c = CalendarPicker::new(2026, 9, 8);
        calendar_key(&mut c, KeyCode::Enter);
        calendar_draw(&mut b, Rect::new(2, 1, 21, 1), &c);
        // Popup 23×10 bajo el campo: esquinas CP437 doble línea.
        let pop = calendar_popup_rect(Rect::new(2, 1, 21, 1), Rect::new(0, 0, 40, 16));
        assert_eq!(pop, Rect::new(2, 2, 23, 10));
        assert_eq!(b.get(2, 2).unwrap().ch, '\u{2554}'); // ╔
        assert_eq!(b.get(24, 2).unwrap().ch, '\u{2557}'); // ╗
        assert_eq!(b.get(2, 11).unwrap().ch, '\u{255a}'); // ╚
        assert_eq!(b.get(24, 11).unwrap().ch, '\u{255d}'); // ╝
                                                           // Cabecera con mes en mayúsculas + navegación.
        assert_eq!(b.get(3, 3).unwrap().ch, '\u{ab}'); // «
        assert_eq!(b.get(6, 3).unwrap().ch, 'S'); // SEPTIEMBRE
                                                  // Semana centrada + día 1 (martes, col 1): x = 3+3 = 6, y = 5.
        assert_eq!(b.get(3, 4).unwrap().ch, 'L');
        assert_eq!(b.get(7, 5).unwrap().ch, '1');
        // Día 8 misma columna, cursor Clipper (fondo navy).
        assert_eq!(b.get(6, 6).unwrap().bg, c.selected_bg);
        // Sombra translúcida a la derecha (offset 2,1): celda atenuada.
        assert!(b.get(25, 3).unwrap().dim);
    }

    #[test]
    fn popup_opens_above_when_no_room_below() {
        let screen = Rect::new(0, 0, 80, 25);
        let pop = calendar_popup_rect(Rect::new(2, 20, 21, 1), screen);
        assert_eq!((pop.y, pop.h), (10, 10)); // 20-10 encima del campo
        let pop2 = calendar_popup_rect(Rect::new(2, 1, 21, 1), screen);
        assert_eq!((pop2.y, pop2.h), (2, 10)); // debajo: y = 1+1
    }

    #[test]
    fn tiny_rects_do_not_panic() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(10, 4, t.desktop);
        let mut c = CalendarPicker::new(2026, 9, 8);
        calendar_draw(&mut b, Rect::new(0, 0, 0, 0), &c);
        calendar_key(&mut c, KeyCode::Enter);
        calendar_draw(&mut b, Rect::new(1, 1, 8, 2), &c); // popup no cabe: solo campo
        assert_eq!(b.get(1, 1).unwrap().ch, ' ');
    }
}
