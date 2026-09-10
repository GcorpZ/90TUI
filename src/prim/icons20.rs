//! Iconografía nivel PC Tools 9 / Norton Desktop (`icons20`).
//!
//! Hermano mayor de `icons` (que queda intacto): mismo contrato (todo
//! dibuja sobre `Buffer`, devuelve el ancho, recorte silencioso), pero
//! con **modo dual** de renderizado:
//! * `RetroCp437`: cero dependencias de fuente — carcasas y bloques
//!   CP437 / geométricos presentes hasta en Consolas.
//! * `NerdFont`: glifos extendidos de alta fidelidad (JetBrainsMono NF).
//!
//! Fondos de contexto: insignias de unidad sobre `theme.desktop`;
//! árbol/archivos reciben el `bg` del llamante (selección o cuerpo) para
//! no generar parches. La unidad activa invierte a fondo amarillo. Las cajas `win_*` viven también aquí con
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

/// Insignia de unidad fiel a PC Tools 9 (`active` = selección amarilla
/// con texto negro, uniforme en todo el badge). Dos estéticas coherentes:
/// retro = carcasa de hardware para todas (`[≡]`/`[═]` floppy,
/// `[▬-]` disco con LED verde, `[○]` CD, letra en blanco);
/// NerdFont = glifo de 1 celda + espacio + letra + `:` para todas
/// (`\u{f0a0}` floppy, `\u{f4bc}` disco, `\u{f111}` óptico).
/// Devuelve el ancho dibujado (retro 6–7, NF 4).
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
    // Base: letra blanca sobre escritorio; activa: negro sobre amarillo.
    let (fg, bg) = if active {
        (Color::Black, Color::Yellow)
    } else {
        (Color::White, theme.desktop)
    };
    let base = Attr::new(fg, bg);
    if mode == IconMode::NerdFont {
        let g = match drive_type {
            DriveType::Floppy35 | DriveType::Floppy525 => '\u{f0a0}',
            DriveType::HardDisk => '\u{f4bc}',
            DriveType::CdRom => '\u{f111}',
        };
        buf.set(x, y, Cell::with_attr(g, Attr::new(Color::Yellow, bg)));
        buf.set(x.saturating_add(1), y, Cell::new(' ', fg, bg));
        buf.set(x.saturating_add(2), y, Cell::with_attr(up, base));
        buf.set(x.saturating_add(3), y, Cell::with_attr(':', base));
        return 4;
    }
    // Carcasa física + letra (`A:`), LED verde en el disco duro.
    let (mid, body, w) = match drive_type {
        DriveType::Floppy35 => ('\u{2261}', Color::Grey, 6), // [≡]
        DriveType::Floppy525 => ('\u{2550}', Color::Grey, 6), // [═]
        DriveType::HardDisk => ('\u{25ac}', Color::White, 7), // [▬-]
        DriveType::CdRom => ('\u{25cb}', Color::Grey, 6),    // [○]
    };
    let chassis = Attr::new(if active { fg } else { body }, bg);
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

/// Tecla de función estilo Norton / CUA 1993 para la status bar:
/// `F{n}` en amarillo brillante + negrita sobre navy, acción en blanco
/// sobre navy, + 2 espacios de aire entre comandos. Sin bloques: toda
/// la barra respira sobre el mismo navy.
/// Devuelve el ancho dibujado (incluido el aire).
pub fn fkey_badge(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    key_num: u8,
    action: &str,
    theme: Theme,
) -> u16 {
    let num_a = Attr::bold(Color::Yellow, theme.navy);
    let act_a = Attr::new(Color::White, theme.navy);
    let mut cx = x;
    for ch in format!("F{key_num} ").chars() {
        buf.set(cx, y, Cell::with_attr(ch, num_a));
        cx = cx.saturating_add(1);
    }
    for ch in format!("{action}  ").chars() {
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
/// `\u{f07b}`/`\u{f07c}` (1 celda), en `fg` sobre el `bg` que indique el
/// llamante (p.ej. `selected_bg` en la fila cursor, `window_bg` fuera).
/// Sin fondos hardcodeados: jamás genera parches.
pub fn folder_badge(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    open: bool,
    mode: IconMode,
    fg: Color,
    bg: Color,
) -> u16 {
    let a = Attr::new(fg, bg);
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

/// Icono de archivo según extensión sobre el `bg` del llamante, con
/// reserva estricta de 2 celdas (glifo + espacio obligatorio): la
/// columna del nombre arranca en `x + 2` y ninguna fuente lo pisa.
/// Indicadores sobrios de la época, iguales en ambos modos (sin glifos
/// ajenos): ejecutables (`exe/com/bat`) `*` verde, datos (`dbf/dat/ntx`)
/// `≡` cian, documentos (`txt/doc`) `·` gris, resto `·` gris. Devuelve 2.
pub fn file_badge(buf: &mut Buffer, x: u16, y: u16, name: &str, _mode: IconMode, bg: Color) -> u16 {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    let (g, fg) = match ext.as_str() {
        "exe" | "com" | "bat" => ('*', Color::Green),
        "dbf" | "dat" | "ntx" => ('\u{2261}', Color::Cyan),
        "txt" | "doc" => ('\u{b7}', Color::Grey),
        _ => ('\u{b7}', Color::Grey),
    };
    // Sobrio universal: el mismo glifo en ambos modos.
    buf.set(x, y, Cell::with_attr(g, Attr::new(fg, bg)));
    buf.set(x.saturating_add(1), y, Cell::new(' ', fg, bg));
    2
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
        assert_eq!(b.get(6, 1).unwrap().fg, Color::White);
        assert_eq!(b.get(7, 1).unwrap().ch, ':');
        // Disco duro `[▬-]` con LED verde y ancho 7.
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
        assert_eq!(b.get(3, 2).unwrap().ch, '\u{25ac}');
        assert_eq!(b.get(4, 2).unwrap().ch, '-');
        assert_eq!(b.get(4, 2).unwrap().fg, Color::Green);
        assert_eq!(b.get(7, 2).unwrap().ch, 'C');
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
    fn drive_nerdfont_uniform_badges() {
        let t = theme();
        let mut b = Buffer::blank(40, 4, t.desktop);
        // Todas en NF: glifo + espacio + letra + ':' (ancho 4).
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
        assert_eq!(b.get(2, 1).unwrap().ch, '\u{f4bc}');
        assert_eq!(b.get(3, 1).unwrap().ch, ' ');
        assert_eq!(b.get(4, 1).unwrap().ch, 'C');
        assert_eq!(b.get(4, 1).unwrap().fg, Color::White);
        let w2 = drive_badge(
            &mut b,
            8,
            1,
            'd',
            DriveType::CdRom,
            IconMode::NerdFont,
            false,
            t,
        );
        assert_eq!(w2, 4);
        assert_eq!(b.get(8, 1).unwrap().ch, '\u{f111}');
        let w3 = drive_badge(
            &mut b,
            14,
            1,
            'a',
            DriveType::Floppy35,
            IconMode::NerdFont,
            false,
            t,
        );
        assert_eq!(w3, 4);
        assert_eq!(b.get(14, 1).unwrap().ch, '\u{f0a0}');
        // Activa: badge completo uniforme en amarillo/negro.
        drive_badge(
            &mut b,
            20,
            1,
            'c',
            DriveType::HardDisk,
            IconMode::NerdFont,
            true,
            t,
        );
        assert_eq!(b.get(20, 1).unwrap().bg, Color::Yellow);
        assert_eq!(b.get(22, 1).unwrap().bg, Color::Yellow);
        assert_eq!(b.get(22, 1).unwrap().fg, Color::Black);
    }

    #[test]
    fn fkey_badge_is_norton_style() {
        let t = theme();
        let mut b = Buffer::blank(40, 4, t.desktop);
        // `F1 Help` + 2 de aire: 3 + 4 + 2.
        let w = fkey_badge(&mut b, 2, 1, 1, "Help", t);
        assert_eq!(w, 3 + 4 + 2);
        // Tecla amarilla brillante + negrita sobre navy, con prefijo F.
        assert_eq!(b.get(2, 1).unwrap().ch, 'F');
        assert_eq!(b.get(3, 1).unwrap().ch, '1');
        assert_eq!(b.get(2, 1).unwrap().bg, t.navy);
        assert_eq!(b.get(2, 1).unwrap().fg, Color::Yellow);
        assert!(b.get(2, 1).unwrap().bold);
        // Acción en blanco sobre navy.
        assert_eq!(b.get(5, 1).unwrap().ch, 'H');
        assert_eq!(b.get(5, 1).unwrap().bg, t.navy);
        assert_eq!(b.get(5, 1).unwrap().fg, Color::White);
        // Aire final también navy (barra limpia): `F1 Help··`.
        assert_eq!(b.get(8, 1).unwrap().ch, 'p');
        assert_eq!(b.get(9, 1).unwrap().ch, ' ');
        assert_eq!(b.get(10, 1).unwrap().ch, ' ');
        assert_eq!(b.get(10, 1).unwrap().bg, t.navy);
        // Ancho exacto: la celda 11 queda intacta (fondo escritorio).
        assert_eq!(b.get(11, 1).unwrap().bg, t.desktop);
        // Dos dígitos (`F10 Menu`): 4 + 4 + 2.
        let w2 = fkey_badge(&mut b, 14, 1, 10, "Menu", t);
        assert_eq!(w2, 4 + 4 + 2);
        assert_eq!(b.get(14, 1).unwrap().ch, 'F');
        assert_eq!(b.get(15, 1).unwrap().ch, '1');
        assert_eq!(b.get(16, 1).unwrap().ch, '0');
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
        // Retro: [+]/[-] de 3 celdas sobre el bg del llamante.
        assert_eq!(
            folder_badge(
                &mut b,
                2,
                1,
                false,
                IconMode::RetroCp437,
                Color::Yellow,
                t.window_bg
            ),
            3
        );
        assert_eq!(b.get(3, 1).unwrap().ch, '+');
        assert_eq!(b.get(2, 1).unwrap().bg, t.window_bg);
        assert_eq!(
            folder_badge(
                &mut b,
                2,
                2,
                true,
                IconMode::RetroCp437,
                Color::Yellow,
                t.select_bg
            ),
            3
        );
        assert_eq!(b.get(3, 2).unwrap().ch, '-');
        // En selección NO hay parches: todo el badge va en azul.
        assert_eq!(b.get(2, 2).unwrap().bg, t.select_bg);
        assert_eq!(b.get(4, 2).unwrap().bg, t.select_bg);
        // NF: glifos de 1 celda.
        assert_eq!(
            folder_badge(
                &mut b,
                2,
                3,
                false,
                IconMode::NerdFont,
                Color::Yellow,
                t.window_bg
            ),
            1
        );
        assert_eq!(b.get(2, 3).unwrap().ch, '\u{f07b}');
        assert_eq!(
            folder_badge(
                &mut b,
                2,
                4,
                true,
                IconMode::NerdFont,
                Color::Yellow,
                t.window_bg
            ),
            1
        );
        assert_eq!(b.get(2, 4).unwrap().ch, '\u{f07c}');
        // Archivos por extensión (retro): 2 celdas (glifo + aire).
        assert_eq!(
            file_badge(&mut b, 8, 1, "PARK.COM", IconMode::RetroCp437, t.window_bg),
            2
        );
        assert_eq!(b.get(8, 1).unwrap().ch, '*');
        assert_eq!(b.get(8, 1).unwrap().fg, Color::Green);
        assert_eq!(b.get(9, 1).unwrap().ch, ' ');
        assert_eq!(
            file_badge(&mut b, 8, 2, "APROD.DBF", IconMode::RetroCp437, t.window_bg),
            2
        );
        assert_eq!(b.get(8, 2).unwrap().ch, '\u{2261}');
        assert_eq!(b.get(8, 2).unwrap().fg, Color::Cyan);
        assert_eq!(
            file_badge(&mut b, 8, 3, "LEEME.TXT", IconMode::RetroCp437, t.window_bg),
            2
        );
        assert_eq!(b.get(8, 3).unwrap().ch, '\u{b7}');
        assert_eq!(b.get(8, 3).unwrap().fg, Color::Grey);
        assert_eq!(
            file_badge(&mut b, 8, 4, "RARO.XYZ", IconMode::RetroCp437, t.window_bg),
            2
        );
        assert_eq!(b.get(8, 4).unwrap().fg, Color::Grey);
        // Sobrio universal: el ejecutable también es `*` en NF.
        assert_eq!(
            file_badge(&mut b, 12, 1, "PARK.COM", IconMode::NerdFont, t.window_bg),
            2
        );
        assert_eq!(b.get(12, 1).unwrap().ch, '*');
        assert_eq!(b.get(12, 1).unwrap().fg, Color::Green);
        assert_eq!(b.get(13, 1).unwrap().ch, ' ');
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
        assert_eq!(fkey_badge(&mut b, 9, 0, 10, "Menu", t), 4 + 4 + 2);
        assert_eq!(
            folder_badge(
                &mut b,
                9,
                0,
                false,
                IconMode::RetroCp437,
                Color::Yellow,
                t.window_bg
            ),
            3
        );
        assert_eq!(
            file_badge(&mut b, 9, 1, "A.EXE", IconMode::RetroCp437, t.window_bg),
            2
        );
    }
}
