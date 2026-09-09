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
    Screen, ScreenStack, Snapshot, TestBackend, Theme, WidgetStyle,
};
pub use prim::{
    button, button_draw, button_draw_ex, button_draw_opts, button_ex, button_width, draw_hot_label,
    draw_text, drive, fit_text, folder, hot_key_of, hsep, parse_hotkey, shadow, shadow_offset,
    shadow_solid, shadow_stipple, shadow_styled, status_bar, top_bar, visible_len, vscrollbar,
    win_close, win_min, window, ButtonOpts, FolderGlyphs, HotAttrs, ShadowStyle, WindowOpts,
    BUTTON_MIN_WIDTH,
};
pub use widgets::{
    bar_fill_width, bar_heights, calendar_draw, calendar_key, check_key, check_width,
    check_width_styled, checkbox_draw, civil_from_days, confirm_draw, confirm_key, confirm_layout,
    days_from_civil, days_in_month, dropdown_draw, dropdown_key, dropdown_overlay_rect,
    dropdown_size, filedialog_draw, filedialog_key, filedialog_layout, fkey_bar, fkey_bar_compact,
    fkey_bar_stacked, fkey_bar_styled, form_draw, form_key, grid_column_widths, grid_draw,
    grid_key, grid_visible, hyperlink_draw, input_draw, input_key, is_leap, line_dots, list_draw,
    list_key, listbox_draw, listbox_key, listbox_visible, menubar_draw, menubar_key, msgbox_draw,
    msgbox_key, msgbox_layout, msgbox_wrap, pie_sectors, popup_draw, popup_key, popup_layout,
    popup_size, progress_draw, progress_fill, progressbar_draw, radio_draw, radio_key,
    statusbar_draw, tab_draw, tab_key, tab_viewport, table_draw, table_key, textarea_draw,
    textarea_key, textarea_visible, tuichart_draw, weekday, Alignment, BarInfo, Buttons, CalNav,
    CalendarPicker, ChartKind, ChartPoint, CheckItem, CheckNav, CheckStyle, DialogKey, Dropdown,
    DropdownKey, FKeyStyle, Field, FieldKind, FileDialog, FileDialogKey, FormKey, FormState,
    GlyphSet, GridNav, GridTable, Hyperlink, InputField, InputKey, ListBox, ListNav, MenuBarKey,
    MenuDef, MenuItem, MsgBoxKey, PopupItem, PopupKey, ProgressBar, ProgressInfo, RadioNav,
    StatusBar, StatusColumn, Tab, TabControl, TabNav, TabPosition, TableDef, TableState, TextArea,
    TextAreaKey, TuiChart,
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
