//! Ejemplo `showroom`: todas las capacidades a la vista (estilo PCTools).
//!
//! Escritorio con menubar, iconos de unidad, panel de árbol con iconos de
//! carpeta, panel de archivos con tabla + scrollbars y diálogo central con
//! radios `○/◉` + casillas `☐/☑` + dropdown + inputs (texto/clave) +
//! fecha (calendario popup) +
//! listbox con scrollbar + progressbar + botones OK/Cancel (ancho mínimo 10,
//! sombra CUA, clic animado), F-bar compacta (`F¹Help…`) y status.
//!
//! Foco con Tab: árbol → archivos → dropdown → nombre → clave → fecha →
//! radios A → radios B → casillas → lista → progreso → botones. Flechas
//! mueven, Espacio alterna/elige, Enter acepta, Esc sale. La copia avanza
//! sola en ciclo 0-100 (octavos visibles); en progreso `←→` ajusta ±5 a
//! mano. En fecha: `PgUp/PgDn` mes, `Shift`+`PgUp/PgDn` año.
//!
//! ```sh
//! cargo run --example showroom
//! ```

use std::io::{self, stdout};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use g90tui::{
    button_draw, button_width, calendar_draw, calendar_field_width, calendar_key_mod, check_key,
    draw_text, drive_badge, dropdown_draw, dropdown_key, enter_screen, file_badge, filedialog_draw,
    filedialog_key, fkey_badge, folder_badge, grid_draw, input_draw, input_key, leave_screen,
    list_key, listbox_draw, listbox_key, menubar_draw, msgbox_draw, msgbox_key, progressbar_draw,
    radio_key, statusbar_draw, tab_draw, tab_key, table_draw, table_key, top_bar, tuichart_draw,
    vscrollbar, window, Alignment, Attr, Backend, Buffer, Buttons, CalNav, CalendarPicker, Cell,
    ChartKind, ChartPoint, CheckItem, CheckNav, CheckStyle, Color, CrosstermBackend, DriveType,
    Dropdown, DropdownKey, EventCtx, FKeyDef, FileDialog, FileDialogKey, FocusManager, GlyphSet,
    GridTable, HandleEvent, HotAttrs, IconMode, InputField, InputKey, ListBox, MenuDef, MsgBoxKey,
    RadioNav, Rect, Screen, StatusBar, TabControl, TabPosition, TableDef, TableState, Theme,
    TuiChart, WindowOpts,
};

/// (prefijo de rama, nombre, abierta?) — el icono lo pinta `folder_badge()`.
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

const FOCUS_NAMES: [&str; 13] = [
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
    "fecha",
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
    cal: CalendarPicker,
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
    /// Ámbitos de foco declarados por página (el TabControl manda).
    fm: FocusManager,
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
    let mut wo = WindowOpts::dialog("Gráficos G90TUI (F4 vuelve)", t);
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
        let mut show = Self {
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
            cal: CalendarPicker::current(),
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
            fm: FocusManager::new(),
        };
        // Declaración de ámbitos (nombres = etiquetas de pestaña):
        // General ve todo; Inventario solo su grid + comunes.
        show.fm
            .add_scope("General", &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        show.fm.add_scope("Inventario", &[0, 1, 8, 9, 10, 11]);
        show.fm.set_active("General");
        show
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

        let buf: &mut Buffer = self.screen.frame();
        buf.fill_rect(bounds, Cell::blank(Color::DarkGrey));
        top_bar(buf, "G90TUI Showroom", "12:00", t);
        menubar_draw(
            buf,
            Rect::new(0, 1, bounds.w, 1),
            &self.menus,
            active_menu,
            t,
        );

        // Fila de unidades con insignias `icons20` (A: floppy, C: disco, D: CD).
        let drv_attr = Attr::new(Color::Black, Color::DarkGrey);
        draw_text(buf, 2, 2, "ID = DEMO", drv_attr);
        let mut dx = 14u16;
        for (drv, dt) in [
            ('A', DriveType::Floppy35),
            ('C', DriveType::HardDisk),
            ('D', DriveType::CdRom),
        ] {
            dx += drive_badge(buf, dx, 2, drv, dt, IconMode::NerdFont, drv == 'C', t) + 1;
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
                // El badge hereda el fondo REAL de la fila (azul popup en
                // selección, gris fuera): barra continua, icono visible.
                let dir_bg = if selected { t.popup } else { t.window_bg };
                cx += folder_badge(buf, cx, y, open, IconMode::NerdFont, Color::Yellow, dir_bg);
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
            // Iconos icons20 por extensión: 2 celdas (glifo + aire) y el
            // nombre alineado en x+2, todo sobre el fondo de la fila.
            for i in 0..files_vis {
                let Some(row) = self.files.rows.get(files_state.top + i) else {
                    break;
                };
                let y = area.y + 1 + i as u16;
                let full = format!(
                    "{}.{}",
                    row.first().map(String::as_str).unwrap_or(""),
                    row.get(1).map(String::as_str).unwrap_or("")
                );
                let selected = files_state.top + i == files_state.row;
                let row_bg = if selected { t.select_bg } else { t.window_bg };
                file_badge(buf, area.x, y, &full, IconMode::NerdFont, row_bg);
                let fg = if selected {
                    t.popup_sel_attr().fg
                } else {
                    t.form_attr().fg
                };
                if let Some(name) = row.first() {
                    for (k, ch) in name.chars().take(10).enumerate() {
                        let x = area.x.saturating_add(2).saturating_add(k as u16);
                        buf.set(x, y, Cell::new(ch, fg, row_bg));
                    }
                }
            }
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
        // h-2 = StatusBar por columnas, h-1 = badges de función icons20.
        if bounds.h >= 3 {
            let fy = bounds.h - 1;
            buf.fill_rect(
                Rect::new(0, fy, bounds.w, 1),
                Cell::new(' ', Color::White, t.navy),
            );
            let mut fx = 1u16;
            for f in &self.fkeys {
                let num: u8 = f.key.trim_start_matches('F').parse().unwrap_or(0);
                // El badge ya trae su aire (2 celdas): avance directo.
                fx = fx.saturating_add(fkey_badge(buf, fx, fy, num, &f.label, t));
                if fx >= bounds.w {
                    break;
                }
            }
            statusbar_draw(buf, bounds.h - 2, &status);
        }

        if charts_view {
            paint_charts(buf, lay.dialog, t);
        } else if !lay.dialog.is_empty() {
            // Diálogo central: todos los controles (dropdown se pinta al final,
            // para que su overlay quede encima).
            let d = lay.dialog;
            let mut wo = WindowOpts::dialog("Showroom G90TUI", t);
            wo.controls = true;
            window(buf, d, &wo, t);
            let base = t.dialog_attr();
            let hot = Attr::bold(Color::Red, t.dialog);
            let cstyle = CheckStyle::new(HotAttrs { base, hot }, GlyphSet::modern());
            // Tira de pestañas sobre la zona de controles (página 0 = General,
            // página 1 = Inventario). En modo Left el contenido se corre.
            let tab_box = Rect::new(d.x + 2, d.y + 1, d.w.saturating_sub(4), 14);
            let vp = tab_draw(buf, tab_box, &self.tabs);
            // Orígenes derivados del viewport (el offset de la tira lo
            // calcula la librería en `tab_viewport`, no el demo).
            let lx = vp.x.saturating_add(1);
            let rx = vp.x.saturating_add(vp.w / 2).saturating_add(1);
            let label_attr = Attr::new(Color::Black, t.dialog);
            let dd_rect = Rect::new(lx.saturating_add(8), d.y + 2, 20, 1);
            let cal_rect = Rect::new(
                lx.saturating_add(8),
                d.y + 6,
                calendar_field_width(&self.cal),
                1,
            );
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
                // Fecha con popup de calendario (fila 6, overlay al final).
                draw_text(buf, lx, d.y + 6, "Fecha:", label_attr);
                if focus == 12 {
                    draw_text(buf, lx - 1, d.y + 6, "►", Attr::bold(Color::Red, t.dialog));
                }

                // Radios A (izq) y lista con scrollbar (der).
                for (i, label) in RADIO_A.iter().enumerate() {
                    g90tui::radio_draw(
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
                let lb_w = 25.min(
                    vp.x.saturating_add(vp.w)
                        .saturating_sub(rx)
                        .saturating_sub(1)
                        .max(10),
                );
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
                    g90tui::radio_draw(
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
                    g90tui::checkbox_draw(
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
                let mut pbar = g90tui::ProgressBar::new(self.progress_pct);
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

            // Dropdown + calendario al final: línea + overlays encima
            // (solo página General).
            if self.tabs.active == 0 {
                dropdown_draw(buf, dd_rect, &self.dropdown, t);
                calendar_draw(buf, cal_rect, &self.cal);
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

    /// Avance automático de la demo: +2 cada tick, al 100% reinicia a 0.
    fn tick_progress(&mut self) {
        self.progress_pct = if self.progress_pct >= 100 {
            0
        } else {
            self.progress_pct.saturating_add(2).min(100)
        };
    }

    /// Devuelve `false` para salir.
    fn key(&mut self, code: KeyCode, mods: KeyModifiers) -> bool {
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
            // Si hay un popup abierto, Esc primero lo cancela.
            if self.dropdown.is_open {
                dropdown_key(&mut self.dropdown, code);
                return true;
            }
            if self.cal.is_open {
                calendar_key_mod(&mut self.cal, code, mods);
                self.message = "Fecha cancelada.".to_string();
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
        // Tab global: solo circula por el ámbito ACTIVO (el TabControl manda).
        if code == KeyCode::Tab {
            self.fm.set_active(self.tabs.active_scope());
            if let Some(n) = self.fm.cycle_next() {
                self.focus = n;
                self.message = format!("Foco: panel {}.", FOCUS_NAMES[self.focus]);
            }
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
        // Calendario abierto: captura todo (año con Shift/Ctrl+PgUp/PgDn).
        if self.cal.is_open {
            match calendar_key_mod(&mut self.cal, code, mods) {
                CalNav::Accepted(y, m, d) => {
                    self.message = format!("Fecha: {d:02}/{m:02}/{y:04}.");
                }
                CalNav::Cancelled => {
                    self.message = "Fecha cancelada.".to_string();
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
                    // Página Inventario: la lista mueve el GridTable vía su
                    // `handle_event` nativo (↑↓ alteran selected + scroll).
                    let vis = 10usize;
                    match self.grid.handle_event(&EventCtx::new(vis), code) {
                        g90tui::GridNav::Move(i) => {
                            if let Some(row) = self.grid.rows.get(i) {
                                self.message = format!("Ítem: {} {}.", row[0], row[1]);
                            }
                        }
                        g90tui::GridNav::Stay => {}
                    }
                } else {
                    let vis = 8usize; // alto del listbox en el diálogo
                    match listbox_key(&mut self.listbox, vis, code) {
                        g90tui::ListNav::Move(i) => {
                            self.message = format!("Archivo: {}.", self.listbox.items[i]);
                        }
                        g90tui::ListNav::Stay => {}
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
            // Fecha del calendario (página General).
            12 => match calendar_key_mod(&mut self.cal, code, mods) {
                CalNav::Accepted(y, m, d) => {
                    self.message = format!("Fecha: {d:02}/{m:02}/{y:04}.");
                }
                CalNav::Opened => {
                    self.message = "Fecha: elige día (Enter acepta, Esc cancela).".to_string();
                }
                _ => {}
            },
            // Pestañas del diálogo: flechas cambian de página, `t` rota Top/Left.
            // Al cambiar de página se re-activa su ámbito: el foco salta
            // solo a widgets visibles (nunca a la página oculta).
            11 => {
                if matches!(code, KeyCode::Char('t' | 'T')) {
                    self.tabs.position = match self.tabs.position {
                        TabPosition::Top => TabPosition::Left,
                        TabPosition::Left => TabPosition::Top,
                    };
                    self.message = format!("Pestañas: {:?}.", self.tabs.position);
                } else {
                    match tab_key(&mut self.tabs, code) {
                        g90tui::TabNav::Move(i) => {
                            self.fm.set_active(self.tabs.active_scope());
                            self.focus = self.fm.current().unwrap_or(self.focus);
                            self.message = format!("Página: {}.", self.tabs.tabs[i].label);
                        }
                        g90tui::TabNav::Stay => {}
                    }
                }
            }
            _ => {} // focus siempre < 13; defensivo
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
    // La barra de progreso avanza sola (+2 cada 150ms, ciclo 0-100) para
    // ver los octavos en movimiento; las flechas la ajustan a mano.
    let mut last_tick = Instant::now();

    loop {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(k) => {
                    // Filtra Release: sin esto todo salta de 2 en 2.
                    if k.kind == KeyEventKind::Release {
                        continue;
                    }
                    if !show.key(k.code, k.modifiers) {
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
        if last_tick.elapsed() >= Duration::from_millis(150) {
            last_tick = Instant::now();
            show.tick_progress();
            show.paint();
            be.present(&show.screen.present_ops())?;
        }
    }
    Ok(())
}
