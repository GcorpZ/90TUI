//! Ejemplo `ordenar_indices`: modal + lista + progreso doble (layout de
//! referencia, aceptación Fase 5/6). Composición estática; cualquier tecla
//! sale. Resize repinta.
//!
//! ```sh
//! cargo run --example ordenar_indices
//! ```

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};

use tui90::{
    draw_text, enter_screen, leave_screen, list_draw, progress_draw, status_bar, top_bar, window,
    Backend, BarInfo, Buffer, CrosstermBackend, ProgressInfo, Rect, Screen, Theme, WindowOpts,
};

const ITEMS: [&str; 8] = [
    "Proveedores",
    "Clientes",
    "Inventario",
    "Varios",
    "Cuentas por cobrar",
    "Facturas",
    "Compras",
    "Reportes",
];

fn paint(s: &mut Screen) {
    let t = s.theme;
    let bounds = s.bounds();
    let buf: &mut Buffer = s.frame();
    buf.fill_rect(bounds, tui90::Cell::blank(t.desktop));
    top_bar(
        buf,
        "EMPRESA DEMO C.A.",
        "Jueves 26 de Diciembre de 2013",
        t,
    );
    status_bar(buf, "TUI90 Demo v0.0.1", "Alt-F1: Ayuda", t);

    // Modal gris con lista (selección azul en "Proveedores").
    let modal = Rect::centered_in(48, 16, bounds);
    window(buf, modal, &WindowOpts::modal("ORDENAR INDICES", t), t);
    let list_rect = Rect::new(modal.x + 2, modal.y + 2, modal.w - 4, 8);
    let items: Vec<String> = ITEMS.iter().map(|s| s.to_string()).collect();
    list_draw(buf, list_rect, &items, 0, true, t);
    draw_text(
        buf,
        modal.x + 3,
        modal.y + modal.h - 2,
        "F2 Todos   ↑↓ elegir   Enter ordenar",
        tui90::Attr::new(tui90::Color::Black, t.window_bg),
    );

    // Progreso doble encima (tercer nivel apilado).
    progress_draw(
        buf,
        bounds,
        &ProgressInfo::new(
            "ORDENAR INDICES",
            "999848KB",
            "00:00:00",
            vec![
                BarInfo::new("Inventario", "APROD.DAT", 12, 60, 546),
                BarInfo::new("Total registros", "TOTAL", 10, 115, 1170),
            ],
        ),
        t,
    );
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
                Event::Key(k)
                    if matches!(k.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) =>
                {
                    break
                }
                Event::Key(_) => {}
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
