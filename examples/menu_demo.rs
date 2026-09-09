//! Ejemplo `menu_demo`: menubar + popup navegables con teclado (Fase 5).
//!
//! La promesa central de G90TUI: un menú con sub-opciones en pocas líneas,
//! dibujado responsive y con hotkeys amarillas automáticas.
//!
//! Teclas: ←→ menús · ↓/Enter abrir · ↑↓ moverse · letra = hotkey ·
//! Enter elegir · Esc cerrar/salir.
//!
//! ```sh
//! cargo run --example menu_demo
//! ```

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use g90tui::{
    draw_text, enter_screen, fkey_bar, leave_screen, menubar_draw, menubar_key, popup_draw,
    popup_key, popup_layout, status_bar, top_bar, Attr, Backend, Buffer, Cell, Color,
    CrosstermBackend, MenuBarKey, MenuDef, PopupItem, PopupKey, Rect, Screen, Theme,
};

struct Demo {
    screen: Screen,
    menus: Vec<MenuDef>,
    active: usize,
    open: Option<usize>,
    sel: Vec<usize>,
    message: String,
}

impl Demo {
    fn new(w: u16, h: u16) -> Self {
        let menus = vec![
            MenuDef::new(
                "Archivos",
                &["Proveedores", "Clientes", "Inventario", "Varios"],
            ),
            MenuDef::new(
                "Varios",
                &[
                    "&Estado del sistema",
                    "&Resumen gerencial",
                    "&Cierre mensual",
                ],
            ),
            MenuDef::new(
                "Sistema",
                &[
                    "&Respaldo de datos",
                    "Rec&uperar datos",
                    "&Ordenar índices",
                    "&Finalizar",
                ],
            ),
        ];
        let sel = vec![0; menus.len()];
        Self {
            screen: Screen::new(w, h, Theme::clipper()),
            menus,
            active: 0,
            open: None,
            sel,
            message: "↓ abre el menú · letra salta por hotkey · Esc sale".to_string(),
        }
    }

    /// Columna x donde empieza el título i (igual que `menubar_draw`).
    fn title_x(&self, i: usize) -> u16 {
        let mut cx = 2u16;
        for m in self.menus.iter().take(i) {
            cx = cx
                .saturating_add(m.title.len() as u16 + 2)
                .saturating_add(2);
        }
        cx
    }

    fn popup_items(&self, m: usize) -> Vec<PopupItem> {
        self.menus[m]
            .items
            .iter()
            .map(|i| PopupItem::item(&i.label))
            .collect()
    }

    fn paint(&mut self) {
        let t = self.screen.theme;
        let bounds = self.screen.bounds();
        let msg = self.message.clone();
        // Precalcular el popup con préstamos inmutables cortos, antes del
        // préstamo mutable del frame (regla del borrow checker).
        let open: Option<(Rect, Vec<PopupItem>, usize)> = self.open.map(|m| {
            let items = self.popup_items(m);
            let r = popup_layout(self.title_x(m).saturating_add(1), 1, &items, bounds);
            let sel = self.sel[m];
            (r, items, sel)
        });
        let buf: &mut Buffer = self.screen.frame();
        buf.fill_rect(bounds, Cell::blank(t.desktop));
        let work = Rect::new(1, 2, bounds.w.saturating_sub(2), bounds.h.saturating_sub(4))
            .clamp_in(bounds);
        buf.fill_rect(work, Cell::blank(t.work));
        top_bar(
            buf,
            "EMPRESA DEMO C.A.",
            "Jueves 26 de Diciembre de 2013",
            t,
        );
        status_bar(buf, "G90TUI Demo v0.0.1", "Alt-F1: Ayuda", t);
        if bounds.h > 3 {
            fkey_bar(
                buf,
                bounds.h.saturating_sub(2),
                &[("F2", "Grabar"), ("Esc", "Salir")],
                t,
            );
        }

        menubar_draw(buf, Rect::new(1, 1, work.w, 1), &self.menus, self.active, t);
        draw_text(buf, 3, 4, &msg, Attr::new(Color::Black, t.work));

        if let Some((r, items, sel)) = &open {
            popup_draw(buf, *r, items, *sel, t);
        }
    }

    /// Devuelve `false` para salir del loop.
    fn key(&mut self, code: KeyCode) -> bool {
        if let Some(m) = self.open {
            let items = self.popup_items(m);
            match popup_key(&items, self.sel[m], code) {
                PopupKey::Move(i) => self.sel[m] = i,
                PopupKey::Choose(i) => {
                    self.message = format!(
                        "Elegido: {} > {}",
                        self.menus[m].title,
                        items[i].label.replace('&', "")
                    );
                    self.open = None;
                }
                PopupKey::Dismiss => self.open = None,
                PopupKey::Stay => {
                    // ←→ cambia de menú abierto; Esc cierra (ya cubierto).
                    match code {
                        KeyCode::Left => {
                            self.active = (self.active + self.menus.len() - 1) % self.menus.len();
                            self.open = Some(self.active);
                        }
                        KeyCode::Right => {
                            self.active = (self.active + 1) % self.menus.len();
                            self.open = Some(self.active);
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') => self.open = None,
                        _ => {}
                    }
                }
            }
            return true;
        }
        match menubar_key(&self.menus, self.active, code) {
            MenuBarKey::Move(i) => self.active = i,
            MenuBarKey::Open(i) => {
                self.active = i;
                self.open = Some(i);
            }
            MenuBarKey::Dismiss => return false,
            MenuBarKey::Stay => {
                if matches!(code, KeyCode::Char('q') | KeyCode::Char('Q')) {
                    return false;
                }
            }
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
    let mut demo = Demo::new(w, h);
    let mut be = CrosstermBackend::new(stdout());
    demo.paint();
    be.present(&demo.screen.present_ops())?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(k) => {
                    // Filtra Release: sin esto el menú salta de 2 en 2.
                    if k.kind == KeyEventKind::Release {
                        continue;
                    }
                    if !demo.key(k.code) {
                        break;
                    }
                    demo.paint();
                    be.present(&demo.screen.present_ops())?;
                }
                Event::Resize(w, h) => {
                    demo.screen.resize(w, h);
                    demo.paint();
                    be.present(&demo.screen.present_ops())?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
