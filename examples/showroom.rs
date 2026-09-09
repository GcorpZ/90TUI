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
    enter_screen, filedialog_draw, filedialog_key, fkey_bar_compact, folder, grid_draw, grid_key,
    input_draw, input_key, leave_screen, list_key, listbox_draw, listbox_key, menubar_draw,
    msgbox_draw, msgbox_key, progressbar_draw, radio_key, statusbar_draw, tab_draw, tab_key,
    table_draw, table_key, top_bar, tuichart_draw, vscrollbar, window, Alignment, Attr, Backend,
    Buffer, Buttons, Cell, ChartKind, ChartPoint, CheckItem, CheckNav, CheckStyle, Color,
    CrosstermBackend, Dropdown, DropdownKey, FKeyDef, FKeyStyle, FileDialog, FileDialogKey,
    FolderGlyphs, GlyphSet, GridTable, HotAttrs, InputField, InputKey, ListBox, MenuDef, MsgBoxKey,
    RadioNav, Rect, Screen, StatusBar, TabControl, TabPosition, TableDef, TableState, Theme,
    TuiChart, WindowOpts,
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

const FOCUS_NAMES: [&str; 12] = [
    "árbol",
    "archivos",
    "dropdown",
    "nombre",
    "clave",
    "radios A",
    "radios B",
    "casillas",
    "lista",
    "progreso",
    "botones",
    "pestañas",
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
    /// Vista de gráficos (F4) en vez del diálogo de controles.
    charts_view: bool,
    /// Modal de archivos abierto (captura todo el teclado).
    file_dialog: Option<FileDialog>,
    /// Confirmación para Abrir (MsgBox YesNo).
    msgbox_sel: Option<usize>,
    /// Pestañas del diálogo central + inventario de la 2ª página.
    tabs: TabControl,
    grid: GridTable,
    /// `StatusBar` avanzada (columnas dinámicas del bucle principal).
    status: StatusBar,
    status_w: u16,
    col_msg: usize,
    col_focus: usize,
    col_info: usize,
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

/// Vista de gráficos (F4): ventana con barras 3D + líneas braille + tarta.
fn paint_charts(buf: &mut Buffer, area: Rect, t: Theme) {
    if area.is_empty() {
        return;
    }
    let mut wo = WindowOpts::dialog("Gráficos 90TUI (F4 vuelve)", t);
    wo.controls = true;
    window(buf, area, &wo, t);
    let cw = area.w.saturating_sub(4) / 3; // 3 columnas
    let ch = area.h.saturating_sub(4);
    let bars = TuiChart {
        accent_color: t.teal,
        has_shadow: true,
        ..TuiChart::new(
            ChartKind::Bars3D,
            "Ventas",
            vec![
                ChartPoint::new("Ene", 12.0),
                ChartPoint::new("Feb", 30.0),
                ChartPoint::new("Mar", 20.0),
                ChartPoint::new("Abr", 25.0),
            ],
        )
    };
    let line = TuiChart {
        accent_color: Color::Yellow,
        has_shadow: true,
        ..TuiChart::new(
            ChartKind::Line,
            "Tendencia",
            vec![
                ChartPoint::new("1", 2.0),
                ChartPoint::new("2", 8.0),
                ChartPoint::new("3", 5.0),
                ChartPoint::new("4", 9.0),
                ChartPoint::new("5", 6.0),
            ],
        )
    };
    let pie = TuiChart {
        has_shadow: true,
        ..TuiChart::new(
            ChartKind::Pie,
            "Cuota",
            vec![
                ChartPoint::new("A", 50.0),
                ChartPoint::new("B", 30.0),
                ChartPoint::new("C", 20.0),
            ],
        )
    };
    tuichart_draw(buf, Rect::new(area.x + 2, area.y + 2, cw, ch), &bars);
    tuichart_draw(
        buf,
        Rect::new(area.x + 2 + cw + 1, area.y + 2, cw, ch),
        &line,
    );
    tuichart_draw(
        buf,
        Rect::new(area.x + 2 + (cw + 1) * 2, area.y + 2, cw, ch),
        &pie,
    );
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
            charts_view: false,
            file_dialog: None,
            msgbox_sel: None,
            tabs: TabControl::new(
                &["General", "Inventario"],
                TabPosition::Top,
                Theme::clipper(),
            ),
            grid: GridTable::new(
                &["Ref", "Descripción", "Cant.", "Costo"],
                vec![
                    vec![
                        "A001".into(),
                        "Tornillo 5mm".into(),
                        "500".into(),
                        "0.10".into(),
                    ],
                    vec![
                        "A002".into(),
                        "Tuerca 5mm".into(),
                        "480".into(),
                        "0.08".into(),
                    ],
                    vec![
                        "B010".into(),
                        "Arandela plana".into(),
                        "900".into(),
                        "0.03".into(),
                    ],
                    vec![
                        "B011".into(),
                        "Arandela presión".into(),
                        "320".into(),
                        "0.05".into(),
                    ],
                    vec![
                        "C100".into(),
                        "Cable 2m".into(),
                        "150".into(),
                        "1.20".into(),
                    ],
                    vec!["C101".into(), "Cable 5m".into(), "80".into(), "2.40".into()],
                    vec![
                        "D020".into(),
                        "Fusible 10A".into(),
                        "210".into(),
                        "0.45".into(),
                    ],
                    vec![
                        "D021".into(),
                        "Fusible 16A".into(),
                        "190".into(),
                        "0.48".into(),
                    ],
                    vec![
                        "E300".into(),
                        "Interruptor".into(),
                        "95".into(),
                        "3.10".into(),
                    ],
                    vec![
                        "E301".into(),
                        "Tomacorriente".into(),
                        "120".into(),
                        "2.75".into(),
                    ],
                    vec![
                        "F400".into(),
                        "Cinta aisladora".into(),
                        "300".into(),
                        "0.90".into(),
                    ],
                    vec![
                        "F401".into(),
                        "Bornera 12p".into(),
                        "140".into(),
                        "1.60".into(),
                    ],
                    vec!["G500".into(), "Relé 12V".into(), "60".into(), "4.20".into()],
                    vec![
                        "G501".into(),
                        "Diodo LED rojo".into(),
                        "1000".into(),
                        "0.06".into(),
                    ],
                ],
            ),
            status: StatusBar::new(Color::White, Color::Navy),
            status_w: 0,
            col_msg: 0,
            col_focus: 0,
            col_info: 0,
            fkeys: vec![
                FKeyDef::new("F1", "Help"),
                FKeyDef::new("F2", "Qview"),
                FKeyDef::new("F3", "Exit"),
                FKeyDef::new("F4", "Graphs"),
                FKeyDef::new("F5", "Copy"),
                FKeyDef::new("F9", "Select"),
                FKeyDef::new("F10", "Menu"),
            ],
        }
    }

    /// Columnas de la StatusBar según el ancho (se recrean al redimensionar).
    fn ensure_status_cols(&mut self, w: u16) {
        if self.status_w == w && !self.status.columns.is_empty() {
            return;
        }
        self.status.columns.clear();
        self.col_msg = self
            .status
            .add_column(1, w.saturating_sub(34).max(10), Alignment::Left);
        self.col_focus = self
            .status
            .add_column(w.saturating_sub(32), 14, Alignment::Center);
        self.col_info = self
            .status
            .add_column(w.saturating_sub(16), 15, Alignment::Right);
        self.status_w = w;
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
        let charts_view = self.charts_view;
        // StatusBar dinámica (columnas del bucle principal).
        self.ensure_status_cols(bounds.w);
        let (cm, cf, ci) = (self.col_msg, self.col_focus, self.col_info);
        self.status.set_text(cm, &msg);
        self.status.set_text(
            cf,
            &format!("[{}]", FOCUS_NAMES[focus.min(FOCUS_NAMES.len() - 1)]),
        );
        // Columna derecha fija: atajo de ayuda (nunca se trunca: 13 <= 15).
        self.status.set_text(ci, "Alt-F1: Ayuda");
        let status = self.status.clone();
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

        // Dos filas fijas al final, sin duplicados:
        // h-2 = StatusBar por columnas, h-1 = teclas de función.
        if bounds.h >= 3 {
            let fstyle = FKeyStyle::highlight(Color::Yellow, Color::White, Color::DarkGrey);
            fkey_bar_compact(buf, bounds.h - 1, &fkey_refs, fstyle);
            statusbar_draw(buf, bounds.h - 2, &status);
        }

        if charts_view {
            paint_charts(buf, lay.dialog, t);
        } else if !lay.dialog.is_empty() {
            // Diálogo central: todos los controles (dropdown se pinta al final,
            // para que su overlay quede encima).
            let d = lay.dialog;
            let mut wo = WindowOpts::dialog("Showroom 90TUI", t);
            wo.controls = true;
            window(buf, d, &wo, t);
            let base = t.dialog_attr();
            let hot = Attr::bold(Color::Red, t.dialog);
            let cstyle = CheckStyle::new(HotAttrs { base, hot }, GlyphSet::modern());
            // Tira de pestañas sobre la zona de controles (página 0 = General,
            // página 1 = Inventario). En modo Left el contenido se corre.
            let tab_box = Rect::new(d.x + 2, d.y + 1, d.w.saturating_sub(4), 14);
            let vp = tab_draw(buf, tab_box, &self.tabs);
            let shift = if self.tabs.position == TabPosition::Left {
                self.tabs.strip_width()
            } else {
                0
            };
            let lx = d.x + 3 + shift; // columna izquierda
            let rx = d.x + 36 + shift; // columna derecha
            let label_attr = Attr::new(Color::Black, t.dialog);
            let dd_rect = Rect::new(lx.saturating_add(8), d.y + 2, 20, 1);
            if focus == 11 {
                let (mx, my) = if self.tabs.position == TabPosition::Left {
                    (tab_box.x, tab_box.y + 1 + self.tabs.active as u16)
                } else {
                    (tab_box.x, tab_box.y)
                };
                draw_text(buf, mx, my, "►", Attr::bold(Color::Red, t.dialog));
            }

            if self.tabs.active == 1 {
                // Página Inventario: GridTable con scroll en el viewport.
                let gr = Rect::new(vp.x, vp.y, vp.w, vp.h.min(12));
                grid_draw(buf, gr, &self.grid);
                draw_text(
                    buf,
                    vp.x,
                    vp.y + vp.h.min(12),
                    "↑↓ mueve · PgUp/PgDn página · Tab cambia",
                    Attr::new(Color::Black, t.dialog),
                );
                if focus == 8 {
                    draw_text(
                        buf,
                        vp.x.saturating_sub(1),
                        vp.y + (self.grid.selected.saturating_sub(self.grid.top)) as u16,
                        "►",
                        Attr::bold(Color::Red, t.dialog),
                    );
                }
            } else {
                // Página General: todos los controles (dropdown se pinta al final,
                // para que su overlay quede encima).

                // Dropdown + inputs (izquierda, filas 2/4/5).
                draw_text(buf, lx, d.y + 2, "Depto:", label_attr);
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
                let lb_w = 25.min(d.right().saturating_sub(rx).saturating_sub(1).max(10));
                let lb_rect = Rect::new(rx, d.y + 2, lb_w, 8);
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
            } // fin página General

            // Botones (padding 2, centrados): OK · Cancel · Abrir… (MsgBox).
            let bw_ok = button_width("OK");
            let bw_cancel = button_width("Cancel");
            let bw_open = button_width("Abrir…");
            let total = bw_ok + 2 + bw_cancel + 2 + bw_open;
            let bx0 = d.x + (d.w.saturating_sub(total)) / 2;
            let bx1 = bx0 + bw_ok + 2;
            let bx2 = bx1 + bw_cancel + 2;
            button_draw(buf, bx0, d.y + 16, "OK", t, flash == Some(0));
            button_draw(buf, bx1, d.y + 16, "Cancel", t, flash == Some(1));
            button_draw(buf, bx2, d.y + 16, "Abrir…", t, flash == Some(2));
            if focus == 10 {
                let mx = [bx0, bx1, bx2][btn_sel.min(2)] - 1;
                draw_text(buf, mx, d.y + 16, "►", Attr::bold(Color::Red, t.dialog));
            }
            // Etiqueta de foco actual (ancho en celdas, con saturación).
            let fl = format!("[{}]", FOCUS_NAMES[focus]);
            let fw: u16 = fl.chars().count() as u16;
            let fx = d.x.saturating_add(d.w.saturating_sub(fw).saturating_sub(2));
            draw_text(buf, fx, d.y + d.h - 1, &fl, base);

            // Dropdown al final: línea + overlay encima (solo página General).
            if self.tabs.active == 0 {
                dropdown_draw(buf, dd_rect, &self.dropdown, t);
            }
        }

        // MsgBox de confirmación, encima del diálogo.
        if let Some(sel) = self.msgbox_sel {
            msgbox_draw(
                buf,
                bounds,
                "Confirmar",
                "¿Abrir explorador de archivos?",
                &Buttons::YesNo,
                sel,
                t,
            );
        }

        // Modal de archivos, encima de absolutamente todo.
        if let Some(dlg) = &self.file_dialog {
            filedialog_draw(buf, bounds, dlg, t);
        }
    }

    /// Devuelve `false` para salir.
    fn key(&mut self, code: KeyCode) -> bool {
        // Modal de archivos: captura todo (Esc cancela, Enter acepta).
        if self.file_dialog.is_some() {
            if code == KeyCode::Esc {
                self.file_dialog = None;
                self.message = "Explorador cancelado.".to_string();
                return true;
            }
            if let Some(dlg) = self.file_dialog.as_mut() {
                match filedialog_key(dlg, code) {
                    FileDialogKey::Accepted(p) => {
                        self.message = format!("Elegido: {}.", p.display());
                        self.file_dialog = None;
                    }
                    FileDialogKey::Cancelled => {
                        self.message = "Explorador cancelado.".to_string();
                        self.file_dialog = None;
                    }
                    _ => {}
                }
            }
            return true;
        }
        // MsgBox de confirmación: captura todo (Esc cancela).
        if self.msgbox_sel.is_some() {
            if code == KeyCode::Esc {
                self.msgbox_sel = None;
                self.message = "Confirmación cancelada.".to_string();
                return true;
            }
            if let Some(sel) = self.msgbox_sel {
                match msgbox_key(&Buttons::YesNo, sel, code) {
                    MsgBoxKey::Stay => {}
                    MsgBoxKey::Move(i) => self.msgbox_sel = Some(i),
                    MsgBoxKey::Accept(0) => {
                        self.msgbox_sel = None;
                        let cwd = std::env::current_dir()
                            .unwrap_or_else(|_| std::path::PathBuf::from("."));
                        self.file_dialog = Some(FileDialog::new(&cwd));
                        self.message = "Explorador abierto.".to_string();
                    }
                    MsgBoxKey::Accept(_) => {
                        self.msgbox_sel = None;
                        self.message = "Confirmación: No.".to_string();
                    }
                    MsgBoxKey::Cancel => {
                        self.msgbox_sel = None;
                        self.message = "Confirmación cancelada.".to_string();
                    }
                }
            }
            return true;
        }
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
        // F4 alterna la vista de gráficos.
        if matches!(code, KeyCode::F(4)) {
            self.charts_view = !self.charts_view;
            self.message = if self.charts_view {
                "Vista de gráficos (F4 vuelve).".to_string()
            } else {
                "Vista de controles.".to_string()
            };
            return true;
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
                if self.tabs.active == 1 {
                    // Página Inventario: la lista mueve el GridTable.
                    let vis = 10usize;
                    match grid_key(&mut self.grid, vis, code) {
                        tui90::GridNav::Move(i) => {
                            if let Some(row) = self.grid.rows.get(i) {
                                self.message = format!("Ítem: {} {}.", row[0], row[1]);
                            }
                        }
                        tui90::GridNav::Stay => {}
                    }
                } else {
                    let vis = 8usize; // alto del listbox en el diálogo
                    match listbox_key(&mut self.listbox, vis, code) {
                        tui90::ListNav::Move(i) => {
                            self.message = format!("Archivo: {}.", self.listbox.items[i]);
                        }
                        tui90::ListNav::Stay => {}
                    }
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
            10 => match code {
                KeyCode::Left | KeyCode::Right => {
                    self.btn_sel = (self.btn_sel + 1) % 3;
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    // Clic animado: el loop pinta hundido, espera y suelta.
                    self.flash = Some(self.btn_sel);
                    match self.btn_sel {
                        0 => {
                            self.message = "OK: ajustes aplicados (demo).".to_string();
                        }
                        1 => {
                            self.message = "Cancelado (demo).".to_string();
                        }
                        _ => {
                            // Abrir… dispara el MsgBox; Sí abre el explorador.
                            self.msgbox_sel = Some(0);
                            self.message = "Confirmar apertura.".to_string();
                        }
                    }
                }
                _ => {}
            },
            // Pestañas del diálogo: flechas cambian de página, `t` rota Top/Left.
            11 => {
                if matches!(code, KeyCode::Char('t' | 'T')) {
                    self.tabs.position = match self.tabs.position {
                        TabPosition::Top => TabPosition::Left,
                        TabPosition::Left => TabPosition::Top,
                    };
                    self.message = format!("Pestañas: {:?}.", self.tabs.position);
                } else {
                    match tab_key(&mut self.tabs, code) {
                        tui90::TabNav::Move(i) => {
                            self.message = format!("Página: {}.", self.tabs.tabs[i].label);
                        }
                        tui90::TabNav::Stay => {}
                    }
                }
            }
            _ => {} // focus siempre < 12; defensivo
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
