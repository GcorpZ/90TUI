//! Ejemplo `blank`: shell base estilo Clipper sin flicker (aceptación Fase 3).
//!
//! Pinta escritorio cian + área blanca + barras navy, presenta por diff
//! y sale con `Esc` o `q`. Redimensionar repinta todo (resize = full frame).
//!
//! ```sh
//! cargo run --example blank
//! ```

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use g90tui::{enter_screen, leave_screen, Backend, Cell, CrosstermBackend, Rect, Screen, Theme};

fn paint(s: &mut Screen) {
    let t = s.theme;
    let bounds = s.bounds();
    s.frame().fill_rect(bounds, Cell::blank(t.desktop));
    // Área de trabajo blanca con inset de 2x3 (layout clásico de gestión).
    let work = Rect::new(2, 3, bounds.w.saturating_sub(4), bounds.h.saturating_sub(6));
    let work = work.clamp_in(bounds);
    s.frame().fill_rect(work, Cell::blank(t.work));
    // Barras superior e inferior navy.
    s.frame()
        .fill_rect(Rect::new(0, 0, bounds.w, 1), Cell::blank(t.navy));
    s.frame().fill_rect(
        Rect::new(0, bounds.h.saturating_sub(1), bounds.w, 1),
        Cell::blank(t.navy),
    );
    s.frame()
        .text_bold(2, 0, "EMPRESA DEMO C.A.", t.status_fg, t.navy);
    s.frame().text(
        2,
        bounds.h.saturating_sub(1),
        "G90TUI Demo v0.0.1",
        t.status_fg,
        t.navy,
    );
    let help = "Alt-F1: Ayuda   Esc/q: Salir";
    if help.len() as u16 + 2 < bounds.w {
        s.frame().text(
            bounds.w - help.len() as u16 - 2,
            bounds.h.saturating_sub(1),
            help,
            t.status_fg,
            t.navy,
        );
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
    let mut screen = Screen::new(w, h, Theme::clipper());
    let mut be = CrosstermBackend::new(stdout());
    paint(&mut screen);
    be.present(&screen.present_ops())?;

    loop {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(k) => {
                    // Filtra Release: sin esto cada toque cuenta doble.
                    if k.kind == KeyEventKind::Release {
                        continue;
                    }
                    match k.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        _ => {}
                    }
                }
                Event::Resize(w, h) => {
                    screen.resize(w, h);
                    paint(&mut screen);
                    be.present(&screen.present_ops())?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
