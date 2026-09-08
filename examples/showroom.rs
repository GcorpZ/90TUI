//! Ejemplo `showroom`: todas las capacidades a la vista (estilo PCTools).
//!
//! Escritorio con menubar, iconos de unidad, panel de árbol con iconos de
//! carpeta, panel de archivos con tabla + scrollbars y diálogo central con
//! radios `○/◉` + casillas `☐/☑` + dropdown + inputs (texto/clave) +
//! listbox con scrollbar + progressbar + botones OK/Cancel (ancho mínimo 10,
//! sombra CUA, clic animado), F-bar compacta (`F¹Help…`) y status.
//!
//! Foco con Tab: árbol → archivos → dropdown → nombre → clave → radios A →
//! radios B → casillas → lista → progreso → botones. Flechas mueven, Espacio
//! alterna/elige, Enter acepta, Esc sale. En progreso: `←→` ajusta ±5.
//!
//! ```sh
//! cargo run --example showroom
//! ```

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use tui90::{
    button_draw, button_width, check_key, draw_text, drive, dropdown_draw, dropdown_key,
    enter_screen, fkey_bar_compact, folder, input_draw, input_key, leave_screen, list_key,
    listbox_draw, listbox_key, menubar_draw, progressbar_draw, radio_key, status_bar, table_draw,
    table_key, top_bar, vscrollbar, window, Attr, Backend, Buffer, Cell, CheckItem, CheckNav,
    CheckStyle, Color, CrosstermBackend, Dropdown, DropdownKey, FKeyDef, FKeyStyle, FolderGlyphs,
    GlyphSet, HotAttrs, InputField, InputKey, ListBox, MenuDef, RadioNav, Rect, Screen, TableDef,
    TableState, Theme, WindowOpts,
};

/// (prefijo de rama, nombre, abierta?) — el icono lo pinta `folder()`.
const TREE: [(&str, &str, bool); 14] = [
    ("", "C:", true),
    ("├─ ", "cfg", false),
    ("├─ ", "dos", false),
    ("├─ ", "drv", true),
    ("│  └─ ", "video", false),
    ("├─ ", "data", false),
    ("├─ ", "system", false),
    ("├─ ", "tools", false),
    ("├─ ", "inbox", false),
    ("├─ ", "backup", false),
    ("├─ ", "temp", false),
    ("├─ ", "docs", false),
    ("└─ ", "log", false),
    ("   ", "readme", false),
];

const RADIO_A: [&str; 3] = ["&Full Encryption", "&Quick Encryption", "&No Encryption"];
const RADIO_B: [&str; 3] = ["&No Delete", "&Quick Delete", "&DOD Delete"];

const FOCUS_NAMES: [&str; 11] = [
    "árbol", "archivos", "dropdown", "nombre", "clave", "radios A", "radios B", "casillas",
    "lista", "progreso", "botones",
];

struct Show {
    screen: Screen,
    menus: Vec<MenuDef>,
    tree_sel: usize,
    tree_top: usize,
    files: TableDef,
    files_state: TableState,
    dropdown: Dropdown,
    input_name: InputField,
    input_pass: InputField,
    radio_a: usize,
    radio_b: usize,
    checks: Vec<CheckItem>,
    check_focus: usize,
    listbox: ListBox,
    progress_pct: u8,
    btn_sel: usize,
    /// Botón con clic en curso (se pinta hundido un instante).
    flash: Option<usize>,
    focus: usize,
    message: String,
    fkeys: Vec<FKeyDef>,
}

struct Layout {
    tree_panel: Rect,
    file_panel: Rect,
    dialog: Rect,
}

fn layout(bounds: Rect) -> Layout {
    let tree_panel = Rect::new(1, 3, 26, 19)
        .intersect(bounds)
        .unwrap_or(Rect::new(0, 0, 0, 0));
    let file_panel = Rect::new(27, 3, 52, 19)
        .intersect(bounds)
        .unwrap_or(Rect::new(0, 0, 0, 0));
    let dialog = Rect::centered_in(64, 21, bounds);
    Layout {
        tree_panel,
        file_panel,
        dialog,
    }
}

impl Show {
    fn new(w: u16, h: u16) -> Self {
        let rows: Vec<Vec<String>> = vec![
            vec!["binstall".into(), "exe".into(), "406544".into()],
            vec!["cmdfix".into(), "exe".into(), "315502".into()],
            vec!["config".into(), "cpz".into(), "8152".into()],
            vec!["cpav".into(), "exe".into(), "599504".into()],
            vec!["cpshelp".into(), "ovl".into(), "29868".into()],
            vec!["desktop".into(), "exe".into(), "518615".into()],
            vec!["diskfix".into(), "exe".into(), "10573".into()],
            vec!["filechk".into(), "exe".into(), "196429".into()],
            vec!["filefix".into(), "exe".into(), "11460".into()],
            vec!["netmsg".into(), "exe".into(), "320000".into()],
            vec!["park".into(), "com".into(), "887499".into()],
            vec!["swapsh".into(), "com".into(), "1449018".into()],
            vec!["unformt".into(), "exe".into(), "1375441".into()],
            vec!["wpatch".into(), "com".into(), "40654".into()],
        ];
        Self {
            screen: Screen::new(w, h, Theme::clipper()),
            menus: vec![
                MenuDef::new("File", &[]),
                MenuDef::new("Disk", &[]),
                MenuDef::new("Tools", &[]),
                MenuDef::new("Windows", &[]),
                MenuDef::new("Tree", &[]),
                MenuDef::new("Help", &[]),
            ],
            tree_sel: 0,
            tree_top: 0,
            files: TableDef::new(&[("Nombre", 12), ("Ext", 5), ("Tamaño", 9)], rows),
            files_state: TableState::new(),
            dropdown: Dropdown::new("Depto", &["Ventas", "Compras", "Gerencia", "Soporte"]),
            input_name: InputField::new(18),
            input_pass: {
                let mut p = InputField::new(12);
                p.mask = Some('*');
                p
            },
            radio_a: 1,
            radio_b: 1,
            checks: vec![
                CheckItem::new("&Compression", true),
                CheckItem::new("&One Key", true),
                CheckItem::new("&Expert Mode", false),
            ],
            check_focus: 0,
            listbox: ListBox::new(&[
                "CONFIG.SYS",
                "AUTOEXEC.BAT",
                "COMMAND.COM",
                "README.TXT",
                "DATA.DBF",
                "INDEX.NTX",
                "BACKUP.ZIP",
                "LOG.TXT",
            ]),
            progress_pct: 42,
            btn_sel: 0,
            flash: None,
            focus: 0,
            message: "487,464,960 Bytes Free".to_string(),
            fkeys: vec![
                FKeyDef::new("F1", "Help"),
                FKeyDef::new("F2", "Qview"),
                FKeyDef::new("F3", "Exit"),
                FKeyDef::new("F5", "Copy"),
                FKeyDef::new("F9", "Select"),
                FKeyDef::new("F10", "Menu"),
            ],
        }
    }

    fn tree_vis(&self, lay: &Layout) -> usize {
        lay.tree_panel.h.saturating_sub(2) as usize
    }

    fn files_vis(&self, lay: &Layout) -> usize {
        lay.file_panel.h.saturating_sub(3) as usize
    }

    fn paint(&mut self) {
        let t = self.screen.theme;
        let bounds = self.screen.bounds();
        let lay = layout(bounds);
        // Copias para el borrow checker.
        let tree_sel = self.tree_sel;
        let tree_top = self.tree_top;
        let files_state = self.files_state;
        let radio_a = self.radio_a;
        let radio_b = self.radio_b;
        let checks = self.checks.clone();
        let check_focus = self.check_focus;
        let btn_sel = self.btn_sel;
        let flash = self.flash;
        let focus = self.focus;
        let msg = self.message.clone();
        let active_menu = 3usize;
        let tree_vis = lay.tree_panel.h.saturating_sub(2).max(1) as usize;
        let files_vis = lay.file_panel.h.saturating_sub(3).max(1) as usize;
        let fkey_refs: Vec<(&str, &str)> = self
            .fkeys
            .iter()
            .map(|f| (f.key.as_str(), f.label.as_str()))
            .collect();

        let buf: &mut Buffer = self.screen.frame();
        buf.fill_rect(bounds, Cell::blank(Color::DarkGrey));
        top_bar(buf, "TUI90 Showroom", "12:00", t);
        menubar_draw(
            buf,
            Rect::new(0, 1, bounds.w, 1),
            &self.menus,
            active_menu,
            t,
        );

        // Fila de unidades con iconos `[A:]`.
        let drv_attr = Attr::new(Color::Black, Color::DarkGrey);
        draw_text(buf, 2, 2, "ID = DEMO", drv_attr);
        let mut dx = 14u16;
        for drv in ['A', 'C', 'D'] {
            dx += drive(buf, dx, 2, drv, drv_attr, Color::Yellow) + 1;
        }

        // Panel árbol con iconos de carpeta.
        if !lay.tree_panel.is_empty() {
            let p = lay.tree_panel;
            let mut wo = WindowOpts::modal("ID = DEMO", t);
            wo.controls = true;
            window(buf, p, &wo, t);
            let vis = tree_vis;
            for i in 0..vis {
                let y = p.y + 1 + i as u16;
                let Some(&(pre, name, open)) = TREE.get(tree_top + i) else {
                    break;
                };
                let selected = tree_top + i == tree_sel;
                if selected {
                    buf.fill_rect(
                        Rect::new(p.x + 1, y, p.w.saturating_sub(3), 1),
                        Cell::new(' ', t.popup_text, t.popup),
                    );
                }
                let row_attr = if selected {
                    t.list_sel_attr()
                } else {
                    Attr::new(Color::Black, t.window_bg)
                };
                let pre_attr = if selected {
                    t.list_sel_attr()
                } else {
                    Attr::new(Color::DarkGrey, t.window_bg)
                };
                let mut cx = p.x + 2;
                cx += draw_text(buf, cx, y, pre, pre_attr);
                cx += folder(
                    buf,
                    cx,
                    y,
                    open,
                    row_attr,
                    Color::Yellow,
                    FolderGlyphs::nerd(),
                );
                draw_text(buf, cx + 1, y, name, row_attr);
            }
            vscrollbar(
                buf,
                Rect::new(p.x + p.w - 2, p.y + 1, 1, p.h.saturating_sub(2)),
                TREE.len(),
                tree_top,
                vis,
                t,
            );
            if focus == 0 {
                draw_text(
                    buf,
                    p.x + 1,
                    p.y + p.h - 1,
                    "►",
                    Attr::bold(Color::Red, t.window_bg),
                );
            }
        }

        // Panel archivos (tabla + scrollbar + footer).
        if !lay.file_panel.is_empty() {
            let p = lay.file_panel;
            let mut wo = WindowOpts::modal("C:\\DEMO\\*.*", t);
            wo.controls = true;
            window(buf, p, &wo, t);
            let area = Rect::new(
                p.x + 1,
                p.y + 1,
                p.w.saturating_sub(3),
                p.h.saturating_sub(2),
            );
            table_draw(
                buf,
                area,
                &self.files,
                &files_state,
                "56 Listed = 3,482,374 Bytes",
                t,
            );
            vscrollbar(
                buf,
                Rect::new(p.x + p.w - 2, p.y + 1, 1, p.h.saturating_sub(2)),
                self.files.rows.len(),
                files_state.top,
                files_vis,
                t,
            );
            if focus == 1 {
                draw_text(
                    buf,
                    p.x + 1,
                    p.y + p.h - 1,
                    "►",
                    Attr::bold(Color::Red, t.window_bg),
                );
            }
        }

        // F-bar compacta (¹Help ²Qview…) + fila de pista + status.
        if bounds.h >= 4 {
            let fstyle = FKeyStyle::highlight(Color::Yellow, Color::White, Color::DarkGrey);
            fkey_bar_compact(buf, bounds.h - 3, &fkey_refs, fstyle);
            draw_text(
                buf,
                1,
                bounds.h - 2,
                "Tab panel · Espacio alterna · Enter acepta",
                Attr::new(Color::Black, Color::DarkGrey),
            );
            status_bar(buf, &msg, "Alt-F1: Ayuda", t);
        }

        // Diálogo central: todos los controles (dropdown se pinta al final,
        // para que su overlay quede encima).
        if !lay.dialog.is_empty() {
            let d = lay.dialog;
            let mut wo = WindowOpts::dialog("Showroom 90TUI", t);
            wo.controls = true;
            window(buf, d, &wo, t);
            let base = t.dialog_attr();
            let hot = Attr::bold(Color::Red, t.dialog);
            let cstyle = CheckStyle::new(HotAttrs { base, hot }, GlyphSet::modern());
            let lx = d.x + 3; // columna izquierda
            let rx = d.x + 36; // columna derecha
            let label_attr = Attr::new(Color::Black, t.dialog);

            // Dropdown + inputs (izquierda, filas 2/4/5).
            draw_text(buf, lx, d.y + 2, "Depto:", label_attr);
            let dd_rect = Rect::new(lx + 8, d.y + 2, 20, 1);
            draw_text(buf, lx, d.y + 4, "Nombre:", label_attr);
            input_draw(
                buf,
                Rect::new(lx + 8, d.y + 4, 20, 1),
                &self.input_name,
                focus == 3,
            );
            draw_text(buf, lx, d.y + 5, "Clave:", label_attr);
            input_draw(
                buf,
                Rect::new(lx + 8, d.y + 5, 20, 1),
                &self.input_pass,
                focus == 4,
            );
            if focus == 2 {
                draw_text(buf, lx - 1, d.y + 2, "►", Attr::bold(Color::Red, t.dialog));
            }
            if focus == 3 {
                draw_text(buf, lx - 1, d.y + 4, "►", Attr::bold(Color::Red, t.dialog));
            }
            if focus == 4 {
                draw_text(buf, lx - 1, d.y + 5, "►", Attr::bold(Color::Red, t.dialog));
            }

            // Radios A (izq) y lista con scrollbar (der).
            for (i, label) in RADIO_A.iter().enumerate() {
                tui90::radio_draw(
                    buf,
                    lx,
                    d.y + 7 + i as u16,
                    label,
                    radio_a == i,
                    focus == 5,
                    cstyle,
                );
            }
            if focus == 5 {
                draw_text(
                    buf,
                    lx - 1,
                    d.y + 7 + radio_a as u16,
                    "►",
                    Attr::bold(Color::Red, t.dialog),
                );
            }
            draw_text(buf, rx, d.y + 1, "Archivos:", label_attr);
            let lb_rect = Rect::new(rx, d.y + 2, 25, 8);
            listbox_draw(buf, lb_rect, &self.listbox);
            if focus == 8 {
                let lrow = (self.listbox.selected.saturating_sub(self.listbox.top)) as u16;
                draw_text(
                    buf,
                    rx - 1,
                    d.y + 2 + lrow,
                    "►",
                    Attr::bold(Color::Red, t.dialog),
                );
            }

            // Radios B (der, bajo la lista) y casillas (izq).
            for (i, label) in RADIO_B.iter().enumerate() {
                tui90::radio_draw(
                    buf,
                    rx,
                    d.y + 11 + i as u16,
                    label,
                    radio_b == i,
                    focus == 6,
                    cstyle,
                );
            }
            if focus == 6 {
                draw_text(
                    buf,
                    rx - 1,
                    d.y + 11 + radio_b as u16,
                    "►",
                    Attr::bold(Color::Red, t.dialog),
                );
            }
            for (i, c) in checks.iter().enumerate() {
                tui90::checkbox_draw(
                    buf,
                    lx,
                    d.y + 11 + i as u16,
                    c,
                    focus == 7 && check_focus == i,
                    cstyle,
                );
            }
            if focus == 7 {
                draw_text(
                    buf,
                    lx - 1,
                    d.y + 11 + check_focus as u16,
                    "►",
                    Attr::bold(Color::Red, t.dialog),
                );
            }

            // Progreso (izq, fila 15) con etiqueta de % dentro.
            draw_text(buf, lx, d.y + 14, "Copia:", label_attr);
            let mut pbar = tui90::ProgressBar::new(self.progress_pct);
            pbar.foreground_color = t.teal;
            progressbar_draw(buf, Rect::new(lx + 8, d.y + 14, 22, 1), &pbar);
            if focus == 9 {
                draw_text(buf, lx - 1, d.y + 14, "►", Attr::bold(Color::Red, t.dialog));
            }

            // Botones (ancho mínimo 10, centrados).
            let bw_ok = button_width("OK");
            let bw_cancel = button_width("Cancel");
            let total = bw_ok + 4 + bw_cancel;
            let bx0 = d.x + (d.w.saturating_sub(total)) / 2;
            button_draw(buf, bx0, d.y + 16, "OK", t, flash == Some(0));
            button_draw(
                buf,
                bx0 + bw_ok + 4,
                d.y + 16,
                "Cancel",
                t,
                flash == Some(1),
            );
            if focus == 10 {
                let mx = if btn_sel == 0 {
                    bx0 - 1
                } else {
                    bx0 + bw_ok + 4 - 1
                };
                draw_text(buf, mx, d.y + 16, "►", Attr::bold(Color::Red, t.dialog));
            }
            // Etiqueta de foco actual (ancho en celdas, con saturación).
            let fl = format!("[{}]", FOCUS_NAMES[focus]);
            let fw: u16 = fl.chars().count() as u16;
            let fx = d.x.saturating_add(d.w.saturating_sub(fw).saturating_sub(2));
            draw_text(buf, fx, d.y + d.h - 1, &fl, base);

            // Dropdown al final: línea + overlay encima de todo.
            dropdown_draw(buf, dd_rect, &self.dropdown, t);
        }
    }

    /// Devuelve `false` para salir.
    fn key(&mut self, code: KeyCode) -> bool {
        if code == KeyCode::Esc {
            // Si el dropdown está abierto, Esc primero lo cancela.
            if self.dropdown.is_open {
                dropdown_key(&mut self.dropdown, code);
                return true;
            }
            return false;
        }
        // F10 también sale (como en la referencia).
        if matches!(code, KeyCode::F(10)) {
            return false;
        }
        let lay = layout(self.screen.bounds());
        if code == KeyCode::Tab {
            self.focus = (self.focus + 1) % FOCUS_NAMES.len();
            self.message = format!("Foco: panel {}.", FOCUS_NAMES[self.focus]);
            return true;
        }
        // Dropdown abierto: captura todo (menos Tab/Esc ya tratados).
        if self.dropdown.is_open {
            match dropdown_key(&mut self.dropdown, code) {
                DropdownKey::Accepted(i) => {
                    self.message = format!("Depto: {}.", self.dropdown.options[i]);
                }
                DropdownKey::Cancelled => {
                    self.message = "Dropdown cancelado.".to_string();
                }
                _ => {}
            }
            return true;
        }
        match self.focus {
            0 => {
                let vis = self.tree_vis(&lay).max(1) as u16;
                self.tree_sel = list_key(self.tree_sel, TREE.len(), code, vis);
                let top = self.tree_top;
                if self.tree_sel < top {
                    self.tree_top = self.tree_sel;
                } else if self.tree_sel >= top + vis as usize {
                    self.tree_top = self.tree_sel + 1 - vis as usize;
                }
            }
            1 => {
                let vis = self.files_vis(&lay).max(1);
                table_key(&mut self.files_state, self.files.rows.len(), vis, code);
                if matches!(code, KeyCode::Enter) {
                    if let Some(row) = self.files.rows.get(self.files_state.row) {
                        self.message = format!("Elegido: {}.{}", row[0], row[1]);
                    }
                }
            }
            2 => match dropdown_key(&mut self.dropdown, code) {
                DropdownKey::Accepted(i) => {
                    self.message = format!("Depto: {}.", self.dropdown.options[i]);
                }
                DropdownKey::Opened => {
                    self.message = "Dropdown abierto (↑↓ Enter Esc).".to_string();
                }
                _ => {}
            },
            3 => {
                if input_key(&mut self.input_name, code) == InputKey::Changed {
                    self.message = format!("Nombre: {}.", self.input_name.value);
                }
            }
            4 => {
                if input_key(&mut self.input_pass, code) == InputKey::Changed {
                    self.message =
                        format!("Clave: {} chars.", self.input_pass.value.chars().count());
                }
            }
            5 => {
                if let RadioNav::Select(i) = radio_key(RADIO_A.len(), self.radio_a, code) {
                    self.radio_a = i;
                }
            }
            6 => {
                if let RadioNav::Select(i) = radio_key(RADIO_B.len(), self.radio_b, code) {
                    self.radio_b = i;
                }
            }
            7 => match check_key(&self.checks, self.check_focus, code) {
                CheckNav::Move(i) => self.check_focus = i,
                CheckNav::Toggled(i) => {
                    if let Some(c) = self.checks.get_mut(i) {
                        c.checked = !c.checked;
                        self.message = format!(
                            "{}: {}.",
                            c.label.replace('&', ""),
                            if c.checked { "sí" } else { "no" }
                        );
                    }
                }
                CheckNav::Stay => {}
            },
            8 => {
                let vis = 8usize; // alto del listbox en el diálogo
                match listbox_key(&mut self.listbox, vis, code) {
                    tui90::ListNav::Move(i) => {
                        self.message = format!("Archivo: {}.", self.listbox.items[i]);
                    }
                    tui90::ListNav::Stay => {}
                }
            }
            9 => match code {
                KeyCode::Left | KeyCode::Down => {
                    self.progress_pct = self.progress_pct.saturating_sub(5);
                }
                KeyCode::Right | KeyCode::Up => {
                    self.progress_pct = self.progress_pct.saturating_add(5).min(100);
                }
                KeyCode::Home => self.progress_pct = 0,
                KeyCode::End => self.progress_pct = 100,
                _ => {}
            },
            _ => match code {
                KeyCode::Left | KeyCode::Right => {
                    self.btn_sel = (self.btn_sel + 1) % 2;
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    // Clic animado: el loop pinta hundido, espera y suelta.
                    self.flash = Some(self.btn_sel);
                    self.message = if self.btn_sel == 0 {
                        "OK: ajustes aplicados (demo).".to_string()
                    } else {
                        "Cancelado (demo).".to_string()
                    };
                }
                _ => {}
            },
        }
        true
    }
}

fn main() -> io::Result<()> {
    enter_screen()?;
    let res = run();
    leave_screen()?;
    res
}

fn run() -> io::Result<()> {
    let (w, h) = crossterm::terminal::size().unwrap_or((80, 25));
    let mut show = Show::new(w, h);
    let mut be = CrosstermBackend::new(stdout());
    show.paint();
    be.present(&show.screen.present_ops())?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(k) => {
                    // Filtra Release: sin esto todo salta de 2 en 2.
                    if k.kind == KeyEventKind::Release {
                        continue;
                    }
                    if !show.key(k.code) {
                        break;
                    }
                    show.paint();
                    be.present(&show.screen.present_ops())?;
                    // Frame del clic: hundido 120ms y de vuelta.
                    if show.flash.take().is_some() {
                        std::thread::sleep(Duration::from_millis(120));
                        show.paint();
                        be.present(&show.screen.present_ops())?;
                    }
                }
                Event::Resize(w, h) => {
                    show.screen.resize(w, h);
                    show.paint();
                    be.present(&show.screen.present_ops())?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
