//! Iconografía nivel PC Tools 9 / Norton Desktop (`icons20`).
//!
//! Hermano mayor de `icons` (que queda intacto): mismo contrato (todo
//! dibuja sobre `Buffer`, devuelve el ancho, recorte silencioso), pero
//! con **modo dual** de renderizado:
//! * `RetroCp437`: cero dependencias de fuente — carcasas y bloques
//!   CP437 / geométricos presentes hasta en Consolas.
//! * `NerdFont`: glifos extendidos de alta fidelidad (JetBrainsMono NF).
//!
//! Fondos de contexto: insignias de unidad sobre `theme.desktop`,
//! árbol/archivos sobre `theme.window_bg` (panel gris). La unidad activa
//! invierte a fondo amarillo. Las cajas `win_*` viven también aquí con
//! nombres propios para no chocar con `icons` (úsalas por su ruta
//! completa: `g90tui::prim::icons20::win_close`).
//!
//! Solo tipos de `crate::core`: `Buffer`, `Cell`, `Color`, `Attr`, `Theme`.

use crate::core::{Attr, Buffer, Cell, Color, Theme};

/// Modo de renderizado de los iconos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconMode {
    /// Cero dependencias: CP437 / geométrico universal.
    RetroCp437,
    /// Glifos extendidos (JetBrainsMono Nerd Font).
    NerdFont,
}

/// Tipo de unidad para `drive_badge`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriveType {
    Floppy35,
    Floppy525,
    HardDisk,
    CdRom,
}

/// Insignia de unidad con carcasa según hardware (`active` = fondo
/// amarillo invertido). Devuelve el ancho dibujado (retro 6–7, NF 4).
#[allow(clippy::too_many_arguments)] // firma del contrato icons20: contexto completo en una llamada
pub fn drive_badge(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    letter: char,
    drive_type: DriveType,
    mode: IconMode,
    active: bool,
    theme: Theme,
) -> u16 {
    let up = letter.to_ascii_uppercase();
    // Base: fondo escritorio; activa: amarillo invertido.
    let (fg, bg) = if active {
        (Color::Black, Color::Yellow)
    } else {
        (Color::Yellow, theme.desktop)
    };
    let base = Attr::new(fg, bg);
    match mode {
        IconMode::NerdFont => {
            let g = match drive_type {
                DriveType::Floppy35 | DriveType::Floppy525 => '\u{f0a0}',
                DriveType::HardDisk => '\u{f02ca}',
                DriveType::CdRom => '\u{f111}',
            };
            buf.set(x, y, Cell::with_attr(g, Attr::new(Color::Yellow, bg)));
            buf.set(x.saturating_add(1), y, Cell::new(' ', fg, bg));
            buf.set(x.saturating_add(2), y, Cell::with_attr(up, base));
            buf.set(x.saturating_add(3), y, Cell::with_attr(':', base));
            4
        }
        IconMode::RetroCp437 => {
            // Carcasa física + letra (`A:`), LED verde en el disco duro.
            let (mid, w) = match drive_type {
                DriveType::Floppy35 => ('\u{2261}', 6),  // [≡]
                DriveType::Floppy525 => ('\u{2550}', 6), // [═]
                DriveType::HardDisk => ('\u{25a0}', 7),  // [■-]
                DriveType::CdRom => ('\u{25cb}', 6),     // [○]
            };
            let chassis = Attr::new(if active { fg } else { Color::Grey }, bg);
            buf.set(x, y, Cell::with_attr('[', chassis));
            buf.set(x.saturating_add(1), y, Cell::with_attr(mid, chassis));
            if drive_type == DriveType::HardDisk {
                // LED verde junto al cuerpo (visible también en activo).
                buf.set(
                    x.saturating_add(2),
                    y,
                    Cell::with_attr('-', Attr::new(Color::Green, bg)),
                );
                buf.set(x.saturating_add(3), y, Cell::with_attr(']', chassis));
                buf.set(x.saturating_add(4), y, Cell::new(' ', fg, bg));
                buf.set(x.saturating_add(5), y, Cell::with_attr(up, base));
                buf.set(x.saturating_add(6), y, Cell::with_attr(':', base));
            } else {
                buf.set(x.saturating_add(2), y, Cell::with_attr(']', chassis));
                buf.set(x.saturating_add(3), y, Cell::new(' ', fg, bg));
                buf.set(x.saturating_add(4), y, Cell::with_attr(up, base));
                buf.set(x.saturating_add(5), y, Cell::with_attr(':', base));
            }
            w
        }
    }
}

/// Badge de tecla de función para la status bar (sin superíndices):
/// `F{n}` en bloque mecánico (negro sobre amarillo) + acción en
/// blanco sobre navy. Devuelve el ancho dibujado.
pub fn fkey_badge(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    key_num: u8,
    action: &str,
    theme: Theme,
) -> u16 {
    let key = format!("F{key_num}");
    let key_a = Attr::bold(Color::Black, Color::Yellow);
    let act_a = Attr::new(Color::White, theme.navy);
    let mut cx = x;
    for ch in key.chars() {
        buf.set(cx, y, Cell::with_attr(ch, key_a));
        cx = cx.saturating_add(1);
    }
    buf.set(cx, y, Cell::new(' ', Color::White, theme.navy));
    cx = cx.saturating_add(1);
    for ch in action.chars() {
        buf.set(cx, y, Cell::with_attr(ch, act_a));
        cx = cx.saturating_add(1);
    }
    cx.saturating_sub(x)
}

/// Caja de cierre `[■]` (`\u{25a0}`). Ancho fijo 3.
pub fn win_close(buf: &mut Buffer, x: u16, y: u16, attr: Attr, accent: Color) -> u16 {
    buf.set(x, y, Cell::with_attr('[', attr));
    buf.set(
        x.saturating_add(1),
        y,
        Cell::with_attr('\u{25a0}', Attr { fg: accent, ..attr }),
    );
    buf.set(x.saturating_add(2), y, Cell::with_attr(']', attr));
    3
}

/// Caja de zoom `[▲]` (`\u{25b2}`). Ancho fijo 3.
pub fn win_zoom(buf: &mut Buffer, x: u16, y: u16, attr: Attr, accent: Color) -> u16 {
    buf.set(x, y, Cell::with_attr('[', attr));
    buf.set(
        x.saturating_add(1),
        y,
        Cell::with_attr('\u{25b2}', Attr { fg: accent, ..attr }),
    );
    buf.set(x.saturating_add(2), y, Cell::with_attr(']', attr));
    3
}

/// Caja de minimizar `[▼]` (`\u{25bc}`). Ancho fijo 3.
pub fn win_min(buf: &mut Buffer, x: u16, y: u16, attr: Attr, accent: Color) -> u16 {
    buf.set(x, y, Cell::with_attr('[', attr));
    buf.set(
        x.saturating_add(1),
        y,
        Cell::with_attr('\u{25bc}', Attr { fg: accent, ..attr }),
    );
    buf.set(x.saturating_add(2), y, Cell::with_attr(']', attr));
    3
}

/// Grip de redimensión en la esquina inferior derecha (`◢`, 1 celda).
pub fn win_resize_grip(buf: &mut Buffer, x: u16, y: u16, attr: Attr) -> u16 {
    buf.set(x, y, Cell::with_attr('\u{25e2}', attr));
    1
}

/// Carpeta del árbol: retro `[+]`/`[-]` (3 celdas) o NF
/// `\u{f07b}`/`\u{f07c}` (1 celda), en amarillo sobre `window_bg`.
pub fn folder_badge(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    open: bool,
    mode: IconMode,
    theme: Theme,
) -> u16 {
    let a = Attr::new(Color::Yellow, theme.window_bg);
    match mode {
        IconMode::NerdFont => {
            let g = if open { '\u{f07c}' } else { '\u{f07b}' };
            buf.set(x, y, Cell::with_attr(g, a));
            1
        }
        IconMode::RetroCp437 => {
            let mid = if open { '-' } else { '+' };
            buf.set(x, y, Cell::with_attr('[', a));
            buf.set(x.saturating_add(1), y, Cell::with_attr(mid, a));
            buf.set(x.saturating_add(2), y, Cell::with_attr(']', a));
            3
        }
    }
}

/// Icono de archivo según extensión (1 celda, sobre `window_bg`):
/// ejecutables (`exe/com/bat`) `*` verde, datos (`dbf/dat/ntx`) `≡`
/// cian, documentos (`txt/doc`) `≡` blanco, resto `·` gris.
/// En modo `NerdFont`, glifos dedicados con el mismo código de color.
pub fn file_badge(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    name: &str,
    mode: IconMode,
    theme: Theme,
) -> u16 {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    let kind = match ext.as_str() {
        "exe" | "com" | "bat" => 0,
        "dbf" | "dat" | "ntx" => 1,
        "txt" | "doc" => 2,
        _ => 3,
    };
    let (retro, nf, fg) = match kind {
        0 => ('*', '\u{f085}', Color::Green),
        1 => ('\u{2261}', '\u{f0ce}', Color::Cyan),
        2 => ('\u{2261}', '\u{f0f6}', Color::White),
        _ => ('\u{b7}', '\u{f016}', Color::Grey),
    };
    let g = match mode {
        IconMode::NerdFont => nf,
        IconMode::RetroCp437 => retro,
    };
    buf.set(x, y, Cell::with_attr(g, Attr::new(fg, theme.window_bg)));
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme() -> Theme {
        Theme::clipper()
    }

    #[test]
    fn drive_retro_draws_chassis_and_letter() {
        let t = theme();
        let mut b = Buffer::blank(30, 4, t.desktop);
        let w = drive_badge(
            &mut b,
            2,
            1,
            'a',
            DriveType::Floppy35,
            IconMode::RetroCp437,
            false,
            t,
        );
        assert_eq!(w, 6);
        assert_eq!(b.get(2, 1).unwrap().ch, '[');
        assert_eq!(b.get(3, 1).unwrap().ch, '\u{2261}');
        assert_eq!(b.get(6, 1).unwrap().ch, 'A');
        assert_eq!(b.get(7, 1).unwrap().ch, ':');
        // Disco duro con LED verde y ancho 7.
        let w2 = drive_badge(
            &mut b,
            2,
            2,
            'c',
            DriveType::HardDisk,
            IconMode::RetroCp437,
            false,
            t,
        );
        assert_eq!(w2, 7);
        assert_eq!(b.get(3, 2).unwrap().ch, '\u{25a0}');
        assert_eq!(b.get(4, 2).unwrap().ch, '-');
        assert_eq!(b.get(4, 2).unwrap().fg, Color::Green);
        // Activa: fondo amarillo invertido.
        drive_badge(
            &mut b,
            12,
            1,
            'd',
            DriveType::CdRom,
            IconMode::RetroCp437,
            true,
            t,
        );
        assert_eq!(b.get(12, 1).unwrap().bg, Color::Yellow);
        assert_eq!(b.get(16, 1).unwrap().ch, 'D');
    }

    #[test]
    fn drive_nerdfont_draws_glyphs() {
        let t = theme();
        let mut b = Buffer::blank(30, 4, t.desktop);
        let w = drive_badge(
            &mut b,
            2,
            1,
            'c',
            DriveType::HardDisk,
            IconMode::NerdFont,
            false,
            t,
        );
        assert_eq!(w, 4);
        assert_eq!(b.get(2, 1).unwrap().ch, '\u{f02ca}');
        assert_eq!(b.get(4, 1).unwrap().ch, 'C');
        let w2 = drive_badge(
            &mut b,
            8,
            1,
            'a',
            DriveType::Floppy35,
            IconMode::NerdFont,
            false,
            t,
        );
        assert_eq!(w2, 4);
        assert_eq!(b.get(8, 1).unwrap().ch, '\u{f0a0}');
    }

    #[test]
    fn fkey_badge_renders_key_block_and_action() {
        let t = theme();
        let mut b = Buffer::blank(30, 4, t.desktop);
        let w = fkey_badge(&mut b, 2, 1, 1, "Help", t);
        assert_eq!(w, 2 + 1 + 4);
        // Bloque mecánico: negro sobre amarillo + negrita.
        assert_eq!(b.get(2, 1).unwrap().ch, 'F');
        assert_eq!(b.get(2, 1).unwrap().bg, Color::Yellow);
        assert_eq!(b.get(2, 1).unwrap().fg, Color::Black);
        assert!(b.get(2, 1).unwrap().bold);
        // Acción en blanco sobre navy.
        assert_eq!(b.get(5, 1).unwrap().ch, 'H');
        assert_eq!(b.get(5, 1).unwrap().bg, t.navy);
        assert_eq!(b.get(5, 1).unwrap().fg, Color::White);
    }

    #[test]
    fn window_controls_have_fixed_width() {
        let t = theme();
        let mut b = Buffer::blank(30, 4, t.desktop);
        let a = Attr::new(Color::White, Color::Black);
        assert_eq!(win_close(&mut b, 1, 1, a, Color::Yellow), 3);
        assert_eq!(b.get(2, 1).unwrap().ch, '\u{25a0}');
        assert_eq!(win_zoom(&mut b, 5, 1, a, Color::Yellow), 3);
        assert_eq!(b.get(6, 1).unwrap().ch, '\u{25b2}');
        assert_eq!(win_min(&mut b, 9, 1, a, Color::Yellow), 3);
        assert_eq!(b.get(10, 1).unwrap().ch, '\u{25bc}');
        assert_eq!(win_resize_grip(&mut b, 13, 1, a), 1);
        assert_eq!(b.get(13, 1).unwrap().ch, '\u{25e2}');
    }

    #[test]
    fn folders_and_files_in_both_modes() {
        let t = theme();
        let mut b = Buffer::blank(30, 6, t.desktop);
        // Retro: [+]/[-] de 3 celdas.
        assert_eq!(
            folder_badge(&mut b, 2, 1, false, IconMode::RetroCp437, t),
            3
        );
        assert_eq!(b.get(3, 1).unwrap().ch, '+');
        assert_eq!(folder_badge(&mut b, 2, 2, true, IconMode::RetroCp437, t), 3);
        assert_eq!(b.get(3, 2).unwrap().ch, '-');
        // NF: glifos de 1 celda.
        assert_eq!(folder_badge(&mut b, 2, 3, false, IconMode::NerdFont, t), 1);
        assert_eq!(b.get(2, 3).unwrap().ch, '\u{f07b}');
        assert_eq!(folder_badge(&mut b, 2, 4, true, IconMode::NerdFont, t), 1);
        assert_eq!(b.get(2, 4).unwrap().ch, '\u{f07c}');
        // Archivos por extensión (retro).
        file_badge(&mut b, 8, 1, "PARK.COM", IconMode::RetroCp437, t);
        assert_eq!(b.get(8, 1).unwrap().ch, '*');
        assert_eq!(b.get(8, 1).unwrap().fg, Color::Green);
        file_badge(&mut b, 8, 2, "APROD.DBF", IconMode::RetroCp437, t);
        assert_eq!(b.get(8, 2).unwrap().ch, '\u{2261}');
        assert_eq!(b.get(8, 2).unwrap().fg, Color::Cyan);
        file_badge(&mut b, 8, 3, "LEEME.TXT", IconMode::RetroCp437, t);
        assert_eq!(b.get(8, 3).unwrap().fg, Color::White);
        file_badge(&mut b, 8, 4, "RARO.XYZ", IconMode::RetroCp437, t);
        assert_eq!(b.get(8, 4).unwrap().fg, Color::Grey);
    }

    #[test]
    fn icons_clip_at_buffer_edge() {
        let t = theme();
        let mut b = Buffer::blank(10, 2, t.desktop);
        // Fuera de rango: ancho igual, sin pánico (recorte silencioso).
        assert_eq!(
            drive_badge(
                &mut b,
                9,
                0,
                'c',
                DriveType::HardDisk,
                IconMode::RetroCp437,
                false,
                t
            ),
            7
        );
        assert_eq!(fkey_badge(&mut b, 9, 0, 10, "Menu", t), 3 + 1 + 4);
        assert_eq!(
            folder_badge(&mut b, 9, 0, false, IconMode::RetroCp437, t),
            3
        );
    }
}
