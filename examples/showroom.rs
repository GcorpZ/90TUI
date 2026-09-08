//! Ejemplo `showroom`: todas las capacidades a la vista (estilo PCTools).
//!
//! Escritorio con menubar, botonera de unidades, panel de árbol, panel de
//! archivos con tabla + scrollbars, diálogo central con radios + casillas
//! + OK/Cancel, línea de stats y barra F. Todo navegable con teclado.
//!
//! Foco con Tab: árbol → archivos → radios → casillas → botones.
//! Flechas mueven, Espacio alterna/elige, Enter acepta, Esc sale.
//!
//! ```sh
//! cargo run --example showroom
//! ```

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use tui90::{
    button, check_key, draw_text, enter_screen, fkey_bar, leave_screen, list_key, menubar_draw,
    radio_key, status_bar, table_draw, table_key, top_bar, vscrollbar, window, Attr, Backend,
    Buffer, Cell, CheckItem, CheckNav, Color, CrosstermBackend, FKeyDef, MenuDef, RadioNav, Rect,
    Screen, TableDef, TableState, Theme, WindowOpts,
};

const TREE: [&str; 14] = [
    "C:",
    "├─ cfg",
    "├─ dos",
    "├─ drv",
    "│  └─ video",
    "├─ data",
    "├─ system",
    "├─ tools",
    "├─ inbox",
    "├─ backup",
    "├─ temp",
    "├─ docs",
    "└─ log",
    "   readme",
];

const RADIO_A: [&str; 3] = ["&Full Encryption", "&Quick Encryption", "&No Encryption"];
const RADIO_B: [&str; 3] = ["&No Delete", "&Quick Delete", "&DOD Delete"];

const FOCUS_NAMES: [&str; 6] = [
    "árbol", "archivos", "radios A", "radios B", "casillas", "botones",
];

struct Show {
    screen: Screen,
    menus: Vec<MenuDef>,
    tree_sel: usize,
    tree_top: usize,
    files: TableDef,
    files_state: TableState,
    radio_a: usize,
    radio_b: usize,
    checks: Vec<CheckItem>,
    check_focus: usize,
    btn_sel: usize,
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
    let dialog = Rect::centered_in(58, 17, bounds);
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
            radio_a: 1,
            radio_b: 1,
            checks: vec![
                CheckItem::new("&Compression", true),
                CheckItem::new("&One Key", true),
                CheckItem::new("&Expert Mode", false),
            ],
            check_focus: 0,
            btn_sel: 0,
            focus: 0,
            message: "Tab cambia de panel · Flechas mueven · Espacio alterna · Esc sale"
                .to_string(),
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
        let msg = self.message.clone();
        let tree_sel = self.tree_sel;
        let tree_top = self.tree_top;
        let files_state = self.files_state;
        let radio_a = self.radio_a;
        let radio_b = self.radio_b;
        let checks = self.checks.clone();
        let check_focus = self.check_focus;
        let btn_sel = self.btn_sel;
        let focus = self.focus;
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

        // Fila de unidades.
        draw_text(
            buf,
            2,
            2,
            "ID = DEMO",
            Attr::new(Color::Black, Color::DarkGrey),
        );
        let mut dx = 14u16;
        for drv in ["A", "C", "D"] {
            dx += button(buf, dx, 2, drv, t) + 1;
        }

        // Panel árbol.
        if !lay.tree_panel.is_empty() {
            let p = lay.tree_panel;
            window(buf, p, &WindowOpts::modal("ID = DEMO", t), t);
            let vis = tree_vis;
            for i in 0..vis {
                let y = p.y + 1 + i as u16;
                let Some(row) = TREE.get(tree_top + i) else {
                    break;
                };
                if tree_top + i == tree_sel {
                    buf.fill_rect(
                        Rect::new(p.x + 1, y, p.w.saturating_sub(3), 1),
                        Cell::new(' ', t.popup_text, t.popup),
                    );
                    draw_text(buf, p.x + 2, y, row, t.list_sel_attr());
                } else {
                    draw_text(buf, p.x + 2, y, row, Attr::new(Color::Black, t.window_bg));
                }
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
            window(buf, p, &WindowOpts::modal("C:\\DEMO\\*.*", t), t);
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

        // Stats + F-bar + status.
        if bounds.h >= 3 {
            let sy = bounds.h - 3;
            draw_text(buf, 1, sy, &msg, Attr::bold(Color::Yellow, Color::DarkGrey));
            fkey_bar(buf, bounds.h - 2, &fkey_refs, t);
            status_bar(buf, "TUI90 v0.0.1", "Alt-F1: Ayuda", t);
        }

        // Diálogo central con radios + casillas + botones.
        if !lay.dialog.is_empty() {
            let d = lay.dialog;
            window(buf, d, &WindowOpts::dialog("Secure Settings", t), t);
            let base = t.dialog_attr();
            let hot = Attr::bold(Color::Red, t.dialog);
            let hattrs = tui90::HotAttrs { base, hot };
            // Radios A (izq) y B (der).
            for (i, label) in RADIO_A.iter().enumerate() {
                tui90::radio_draw(
                    buf,
                    d.x + 3,
                    d.y + 2 + i as u16,
                    label,
                    radio_a == i,
                    focus == 2,
                    hattrs,
                );
            }
            for (i, label) in RADIO_B.iter().enumerate() {
                tui90::radio_draw(
                    buf,
                    d.x + 30,
                    d.y + 2 + i as u16,
                    label,
                    radio_b == i,
                    focus == 3,
                    hattrs,
                );
            }
            // Casillas.
            for (i, c) in checks.iter().enumerate() {
                let (cx, cy) = if i < 2 {
                    (d.x + 3, d.y + 6 + i as u16)
                } else {
                    (d.x + 30, d.y + 6)
                };
                tui90::checkbox_draw(buf, cx, cy, c, focus == 4 && check_focus == i, hattrs);
            }
            // Botones.
            let bw = button(buf, d.x + 12, d.y + 11, "OK", t);
            button(buf, d.x + 12 + bw + 4, d.y + 11, "Cancel", t);
            if focus == 5 {
                let mx = if btn_sel == 0 {
                    d.x + 11
                } else {
                    d.x + 11 + bw + 4
                };
                draw_text(buf, mx, d.y + 11, "►", Attr::bold(Color::Red, t.dialog));
            }
            // Etiqueta de foco actual (ancho en celdas, con saturación).
            let fl = format!("[{}]", FOCUS_NAMES[focus]);
            let fw: u16 = fl.chars().count() as u16;
            let fx = d.x.saturating_add(d.w.saturating_sub(fw).saturating_sub(2));
            draw_text(buf, fx, d.y + d.h - 1, &fl, base);
        }
    }

    /// Devuelve `false` para salir.
    fn key(&mut self, code: KeyCode) -> bool {
        if code == KeyCode::Esc {
            return false;
        }
        // F10 también sale (como en la referencia).
        if matches!(code, KeyCode::F(10)) {
            return false;
        }
        let lay = layout(self.screen.bounds());
        if code == KeyCode::Tab {
            self.focus = (self.focus + 1) % 6;
            self.message = format!("Foco: panel {}.", FOCUS_NAMES[self.focus]);
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
            2 => {
                if let RadioNav::Select(i) = radio_key(RADIO_A.len(), self.radio_a, code) {
                    self.radio_a = i;
                }
            }
            3 => {
                if let RadioNav::Select(i) = radio_key(RADIO_B.len(), self.radio_b, code) {
                    self.radio_b = i;
                }
            }
            4 => match check_key(&self.checks, self.check_focus, code) {
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
            _ => match code {
                KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                    self.btn_sel = (self.btn_sel + 1) % 2;
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
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
