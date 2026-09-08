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

pub mod app;
pub mod core;
pub mod prim;
pub mod widgets;

// Re-exports para `use tui90::Buffer;` directo.
pub use app::{
    centered as app_centered, desktop as app_desktop, draw_fkeys, map_key, map_key_event,
    match_fkey, menu_title_x, poll_event, popup as app_popup, test_app, to_crossterm, App,
    AppEvent, AppKey, DesktopLayout, FKeyDef, Focus, Layer, ShellContent,
};
pub use core::{
    enter_screen, leave_screen, Attr, Backend, Buffer, Cell, Color, CrosstermBackend, DrawOp, Rect,
    Screen, ScreenStack, Snapshot, TestBackend, Theme,
};
pub use prim::{
    button, button_draw, button_width, draw_hot_label, draw_text, drive, fit_text, folder,
    hot_key_of, hsep, parse_hotkey, shadow, shadow_offset, shadow_solid, shadow_stipple,
    shadow_styled, status_bar, top_bar, visible_len, vscrollbar, win_close, win_min, window,
    HotAttrs, ShadowStyle, WindowOpts,
};
pub use widgets::{
    bar_fill_width, check_key, check_width, checkbox_draw, confirm_draw, confirm_key,
    confirm_layout, fkey_bar, fkey_bar_stacked, fkey_bar_styled, form_draw, form_key, list_draw,
    list_key, menubar_draw, menubar_key, popup_draw, popup_key, popup_layout, popup_size,
    progress_draw, radio_draw, radio_key, table_draw, table_key, BarInfo, CheckItem, CheckNav,
    CheckStyle, DialogKey, FKeyStyle, Field, FieldKind, FormKey, FormState, GlyphSet, MenuBarKey,
    MenuDef, MenuItem, PopupItem, PopupKey, ProgressInfo, RadioNav, TableDef, TableState,
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
