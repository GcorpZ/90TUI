//! `App`: el shell mínimo (pantalla + backend + foco + bandera de salida).
//!
//! Genérica sobre `Backend` para probarse sin TTY (`TestBackend`).
//! El estado de cada widget lo guarda la app usuaria (ver ejemplos);
//! aquí vive solo lo compartido: pintar el shell y presentar diffs.

use std::io::{self, Stdout};

use crate::core::{Backend, CrosstermBackend, Screen, TestBackend, Theme};
use crate::prim::{status_bar, top_bar};
use crate::widgets::{fkey_bar, menubar_draw, MenuDef};

use super::events::{poll_event, AppEvent};
use super::fkeys::FKeyDef;
use super::focus::Focus;
use super::layout::{self, DesktopLayout};

/// Columna x donde empieza el título `i` (igual que `menubar_draw`: x=2,
/// ` titulo ` + 2 de aire). Sirve para anclar popups.
pub fn menu_title_x(menus: &[MenuDef], i: usize) -> u16 {
    let mut cx = 2u16;
    for m in menus.iter().take(i) {
        cx = cx
            .saturating_add(m.title.len() as u16 + 2)
            .saturating_add(2);
    }
    cx
}

/// Shell de aplicación.
pub struct App<B: Backend = CrosstermBackend<Stdout>> {
    pub screen: Screen,
    pub backend: B,
    pub focus: Focus,
    running: bool,
}

impl App<CrosstermBackend<Stdout>> {
    /// Crea contra la terminal real (tamaño actual).
    pub fn new(theme: Theme) -> io::Result<Self> {
        let (w, h) = crossterm::terminal::size().unwrap_or((80, 25));
        Ok(Self {
            screen: Screen::new(w, h, theme),
            backend: CrosstermBackend::new(io::stdout()),
            focus: Focus::new(),
            running: true,
        })
    }
}

/// Contenido del shell (una struct para no tener 8 parámetros sueltos).
pub struct ShellContent<'a> {
    pub company: &'a str,
    pub date: &'a str,
    pub status: &'a str,
    pub help: &'a str,
    pub menus: &'a [MenuDef],
    pub active: usize,
    pub fkeys: &'a [FKeyDef],
}

impl<B: Backend> App<B> {
    /// Crea con backend y tamaño explícitos (tests y embebidos).
    pub fn with_backend(backend: B, w: u16, h: u16, theme: Theme) -> Self {
        Self {
            screen: Screen::new(w, h, theme),
            backend,
            focus: Focus::new(),
            running: true,
        }
    }

    pub fn layout(&self) -> DesktopLayout {
        layout::desktop(self.screen.bounds())
    }

    pub fn should_quit(&self) -> bool {
        !self.running
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Pinta shell completo: barras + menubar + botonera F.
    pub fn paint_shell(&mut self, c: &ShellContent) {
        let t = self.screen.theme;
        let d = layout::desktop(self.screen.bounds());
        let bounds = self.screen.bounds();
        let buf = self.screen.frame();
        buf.fill_rect(bounds, crate::core::Cell::blank(t.desktop));
        buf.fill_rect(d.work, crate::core::Cell::blank(t.work));
        top_bar(buf, c.company, c.date, t);
        menubar_draw(buf, d.menu, c.menus, c.active, t);
        let refs: Vec<(&str, &str)> = c
            .fkeys
            .iter()
            .map(|f| (f.key.as_str(), f.label.as_str()))
            .collect();
        fkey_bar(buf, d.fkeys.y, &refs, t);
        status_bar(buf, c.status, c.help, t);
    }

    /// Envía el diff al backend.
    pub fn present(&mut self) -> io::Result<()> {
        let ops = self.screen.present_ops();
        self.backend.present(&ops)
    }

    /// Un evento con timeout (None = silencio).
    pub fn poll(&self, timeout: std::time::Duration) -> io::Result<Option<AppEvent>> {
        let _ = self;
        poll_event(timeout)
    }

    /// Resize con re-layout perezoso (la app repinta tras llamar).
    pub fn resize(&mut self, w: u16, h: u16) {
        self.screen.resize(w, h);
    }
}

/// Atajo de prueba: App sobre `TestBackend` de 80x25.
pub fn test_app() -> App<TestBackend> {
    App::with_backend(TestBackend::new(80, 25), 80, 25, Theme::clipper())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::MenuDef;

    #[test]
    fn title_x_matches_draw_walk() {
        let m = vec![MenuDef::new("Archivos", &[]), MenuDef::new("Varios", &[])];
        assert_eq!(menu_title_x(&m, 0), 2);
        // " Archivos " = 10 + 2 de aire.
        assert_eq!(menu_title_x(&m, 1), 2 + 10 + 2);
    }

    #[test]
    fn shell_paints_and_presents() {
        let mut app = test_app();
        let menus = vec![
            MenuDef::new("Archivos", &["X"]),
            MenuDef::new("Varios", &["Y"]),
        ];
        let fkeys = vec![FKeyDef::new("Esc", "Salir")];
        app.paint_shell(&ShellContent {
            company: "EMPRESA",
            date: "FECHA",
            status: "v0.0.1",
            help: "Ayuda",
            menus: &menus,
            active: 1,
            fkeys: &fkeys,
        });
        app.present().unwrap();
        let ops = app.backend.take();
        assert!(!ops.is_empty());
        // Segunda presentación sin cambios: silencio.
        app.present().unwrap();
        assert!(app.backend.take().is_empty());
        assert!(!app.should_quit());
        app.quit();
        assert!(app.should_quit());
    }

    #[test]
    fn resize_keeps_app_alive() {
        let mut app = test_app();
        app.resize(40, 10);
        assert_eq!(app.screen.size(), (40, 10));
        let d = app.layout();
        assert_eq!(d.status.bottom(), 10);
    }
}
