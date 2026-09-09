//! Ejemplo `clipper_shell`: shell navegable completo (aceptación Fase 6).
//!
//! Solo teclado: menubar → popup → confirm, F-keys operativas, resize
//! sin romperse. El loop (`poll → foco → pintar → present`) es el patrón
//! que la Fase 6 propone para apps reales.
//!
//! Teclas: ←→↑↓ · letra = hotkey · Enter elegir · F2 grabar (demo) ·
//! Esc cerrar/salir.
//!
//! ```sh
//! cargo run --example clipper_shell
//! ```

use std::io;
use std::time::Duration;

use g90tui::{
    confirm_draw, confirm_key, draw_text, enter_screen, leave_screen, match_fkey, menu_title_x,
    menubar_key, popup_draw, popup_key, popup_layout, to_crossterm, App, AppEvent, AppKey, Attr,
    Color, DialogKey, FKeyDef, Layer, MenuBarKey, MenuDef, PopupItem, PopupKey,
};

struct Shell {
    app: App,
    menus: Vec<MenuDef>,
    active: usize,
    open: Option<usize>,
    sel: Vec<usize>,
    confirm: Option<(String, Vec<String>, usize)>,
    message: String,
    fkeys: Vec<FKeyDef>,
}

impl Shell {
    fn new(mut app: App) -> Self {
        let menus = vec![
            MenuDef::new(
                "Archivos",
                &["Proveedores", "Clientes", "Inventario", "Varios"],
            ),
            MenuDef::new("Transacciones", &["Compras", "Facturas", "Cargos"]),
            MenuDef::new(
                "Varios",
                &[
                    "&Estado del sistema",
                    "&Respaldo de datos",
                    "&Ordenar índices",
                    "&Finalizar",
                ],
            ),
        ];
        let sel = vec![0; menus.len()];
        let fkeys = vec![FKeyDef::new("F2", "Grabar"), FKeyDef::new("Esc", "Salir")];
        app.focus.push(Layer::MenuBar);
        Self {
            app,
            menus,
            active: 0,
            open: None,
            sel,
            confirm: None,
            message: "↓ abre el menú · F2 graba (demo) · Esc sale".to_string(),
            fkeys,
        }
    }

    fn items(&self, m: usize) -> Vec<PopupItem> {
        self.menus[m]
            .items
            .iter()
            .map(|i| PopupItem::item(&i.label))
            .collect()
    }

    fn paint(&mut self) {
        let t = self.app.screen.theme;
        self.app.paint_shell(&g90tui::ShellContent {
            company: "EMPRESA DEMO C.A.",
            date: "Jueves 26 de Diciembre de 2013",
            status: "G90TUI Demo v0.0.1",
            help: "Alt-F1: Ayuda",
            menus: &self.menus,
            active: self.active,
            fkeys: &self.fkeys,
        });
        let bounds = self.app.screen.bounds();
        let msg = self.message.clone();
        let open = self.open.map(|m| {
            let items = self.items(m);
            let r = popup_layout(
                menu_title_x(&self.menus, m).saturating_add(1),
                1,
                &items,
                bounds,
            );
            (r, items, self.sel[m])
        });
        let dlg = self.confirm.clone();
        let buf = self.app.screen.frame();
        draw_text(buf, 3, 4, &msg, Attr::new(Color::Black, t.work));
        if let Some((r, items, sel)) = &open {
            popup_draw(buf, *r, items, *sel, t);
        }
        if let Some((q, btns, sel)) = &dlg {
            confirm_draw(buf, bounds, "CONFIRMAR", q, btns, *sel, t);
        }
    }

    fn on_chosen(&mut self, menu: usize, item: usize) {
        let label = self.menus[menu].items[item].label.replace('&', "");
        match label.as_str() {
            "Finalizar" => {
                self.open = None;
                self.app.focus.pop();
                self.confirm = Some((
                    "¿ Finalizar la demo ?".to_string(),
                    vec!["&Si".to_string(), "&No".to_string()],
                    0,
                ));
                self.app.focus.push(Layer::Dialog);
            }
            _ => {
                self.message = format!("Elegido: {} > {label}", self.menus[menu].title);
                self.open = None;
                self.app.focus.pop();
            }
        }
    }

    /// Devuelve `false` para salir.
    fn key(&mut self, ev: AppEvent) -> bool {
        let AppEvent::Key { key, .. } = ev else {
            return true;
        };
        // F-keys globales (salvo en diálogo, donde Esc cancela).
        if self.confirm.is_none() {
            if let Some(i) = match_fkey(&self.fkeys, key) {
                if i == 0 {
                    self.message = "Grabado (demo).".to_string();
                    return true;
                }
            }
        }
        match self.app.focus.top() {
            Layer::Dialog => {
                let Some((_, btns, sel)) = self.confirm.clone() else {
                    return true;
                };
                match confirm_key(&btns, sel, to_crossterm(key)) {
                    DialogKey::Move(i) => {
                        if let Some((_, _, s)) = self.confirm.as_mut() {
                            *s = i;
                        }
                    }
                    DialogKey::Accept(0) => return false, // Si → salir
                    DialogKey::Accept(_) | DialogKey::Cancel => {
                        self.confirm = None;
                        self.app.focus.pop();
                        self.message = "Operación cancelada.".to_string();
                    }
                    DialogKey::Stay => {}
                }
                true
            }
            _ if self.open.is_some() => {
                let m = self.open.unwrap_or(0);
                let items = self.items(m);
                match popup_key(&items, self.sel[m], to_crossterm(key)) {
                    PopupKey::Move(i) => self.sel[m] = i,
                    PopupKey::Choose(i) => self.on_chosen(m, i),
                    PopupKey::Dismiss => {
                        self.open = None;
                        self.app.focus.pop();
                    }
                    PopupKey::Stay => match key {
                        AppKey::Left => {
                            self.active = (self.active + self.menus.len() - 1) % self.menus.len();
                            self.open = Some(self.active);
                        }
                        AppKey::Right => {
                            self.active = (self.active + 1) % self.menus.len();
                            self.open = Some(self.active);
                        }
                        _ => {}
                    },
                }
                true
            }
            _ => match menubar_key(&self.menus, self.active, to_crossterm(key)) {
                MenuBarKey::Move(i) => {
                    self.active = i;
                    true
                }
                MenuBarKey::Open(i) => {
                    self.active = i;
                    self.open = Some(i);
                    self.app.focus.push(Layer::Popup);
                    true
                }
                MenuBarKey::Dismiss => false,
                MenuBarKey::Stay => !matches!(key, AppKey::Char('q') | AppKey::Char('Q')),
            },
        }
    }
}

fn main() -> io::Result<()> {
    enter_screen()?;
    let res = run();
    leave_screen()?;
    if res.is_ok() {
        println!("Sesión terminada. ¡Gracias por probar G90TUI!");
    }
    res
}

fn run() -> io::Result<()> {
    let mut shell = Shell::new(App::new(g90tui::Theme::clipper())?);
    shell.paint();
    shell.app.present()?;

    loop {
        match shell.app.poll(Duration::from_millis(100))? {
            None => {}
            Some(AppEvent::Resize(w, h)) => {
                shell.app.resize(w, h);
                shell.paint();
                shell.app.present()?;
            }
            Some(ev) => {
                if !shell.key(ev) {
                    break;
                }
                shell.paint();
                shell.app.present()?;
            }
        }
    }
    Ok(())
}
