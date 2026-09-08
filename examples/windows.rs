//! Ejemplo `windows`: modal gris + botonera teal + barras (aceptación Fase 4).
//!
//! Composición estática estilo gestión de los 90 sobre el núcleo + prim:
//! shell, menubar, ventana modal con lista y botones, todo con sombra dura.
//!
//! ```sh
//! cargo run --example windows
//! ```

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use tui90::{
    button, draw_text, enter_screen, leave_screen, status_bar, top_bar, window, Attr, Backend,
    Buffer, Cell, Color, CrosstermBackend, Rect, Screen, Theme, WindowOpts,
};

const ITEMS: [&str; 6] = [
    "Proveedores",
    "Clientes",
    "Inventario",
    "Varios",
    "Compras",
    "Facturas",
];

fn paint(s: &mut Screen) {
    let t = s.theme;
    let bounds = s.bounds();
    let buf: &mut Buffer = s.frame();

    // Shell: escritorio + trabajo + barras.
    buf.fill_rect(bounds, Cell::blank(t.desktop));
    let work =
        Rect::new(1, 2, bounds.w.saturating_sub(2), bounds.h.saturating_sub(4)).clamp_in(bounds);
    buf.fill_rect(work, Cell::blank(t.work));
    top_bar(
        buf,
        "EMPRESA DEMO C.A.",
        "Jueves 26 de Diciembre de 2013",
        t,
    );
    status_bar(buf, "TUI90 Demo v0.0.1", "Alt-F1: Ayuda", t);

    // Menubar teal fila 1 con un item activo.
    if bounds.h > 2 {
        buf.fill_rect(
            Rect::new(1, 1, work.w, 1),
            Cell::new(' ', t.status_fg, t.teal),
        );
        let menus = ["Archivos", "Transacciones", "Reportes", "Varios"];
        let mut cx = 4u16;
        for (i, m) in menus.iter().enumerate() {
            let a = t.menu_attr(i == 3);
            let w = draw_text(buf, cx, 1, &format!(" {m} "), a);
            cx = cx.saturating_add(w).saturating_add(2);
        }
    }

    // Modal gris centrado.
    let r = Rect::centered_in(46, 15, bounds);
    if r.is_empty() {
        return;
    }
    window(buf, r, &WindowOpts::modal("ORDENAR INDICES", t), t);

    // Lista: primer item seleccionado (barra azul + texto blanco).
    for (i, it) in ITEMS.iter().enumerate() {
        let y = r.y + 2 + i as u16;
        if y >= r.y + r.h - 2 {
            break;
        }
        if i == 0 {
            buf.fill_rect(
                Rect::new(r.x + 2, y, r.w.saturating_sub(4), 1),
                Cell::new(' ', t.popup_text, t.popup),
            );
            draw_text(
                buf,
                r.x + (r.w - it.len() as u16) / 2,
                y,
                it,
                t.popup_sel_attr(),
            );
        } else {
            draw_text(
                buf,
                r.x + (r.w - it.len() as u16) / 2,
                y,
                it,
                Attr::new(Color::Black, t.window_bg),
            );
        }
    }

    // Botonera inferior del modal.
    let by = r.y + r.h - 2;
    let mut bx = r.x + 3;
    for label in ["Ordenar", "Todos", "↑", "↓"] {
        bx = bx
            .saturating_add(button(buf, bx, by, label, t))
            .saturating_add(2);
        if bx + 6 >= r.x + r.w {
            break;
        }
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
