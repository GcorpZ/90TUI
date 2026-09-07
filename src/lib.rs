//! 90TUI (`tui90`) — gTUI artesanal de los 90 al estilo de la técnica CA-Clipper para Rust.
//!
//! Licencia dual: MIT OR Apache-2.0. Ver `LICENSE-MIT` y `LICENSE-APACHE`.
//!
//! # Capas
//! * [`core`] — VRAM virtual: celdas, rects, back-buffer con diff,
//!   pila `Savescreen`/`Restscreen`, temas Clipper/Turbo y backend crossterm.
//! * `prim` _(Fase 4)_ — ventanas planas con sombra dura, botones, hotlabels.
//! * `widgets` _(Fase 5)_ — controles de una llamada (menús, forms, tablas).
//! * `app` _(Fase 6)_ — loop de eventos, foco y layout responsive.

pub mod core;
pub mod prim;

// Re-exports para `use tui90::Buffer;` directo.
pub use core::{
    enter_screen, leave_screen, Attr, Backend, Buffer, Cell, Color, CrosstermBackend, DrawOp, Rect,
    Screen, ScreenStack, Snapshot, TestBackend, Theme,
};
pub use prim::{
    button, button_width, draw_hot_label, draw_text, fit_text, hot_key_of, hsep, parse_hotkey,
    shadow, shadow_offset, status_bar, top_bar, visible_len, window, WindowOpts,
};

/// Versión del crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton_version_is_set() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn core_is_reexported() {
        let t = Theme::clipper();
        let s = Screen::new(80, 25, t);
        assert_eq!(s.size(), (80, 25));
    }
}
