//! Controles de una llamada (Fase 5): menús, listas, diálogos,
//! progresos, formularios, tablas y botonera de función.
//!
//! Todo dibuja sobre `Buffer`/`Screen` con la pila `savescreen` para
//! apilar. La navegación es pura (`*_key`) para probarse sin TTY;
//! el loop de eventos llega en Fase 6 (`app`).

pub mod chart;
pub mod checkradio;
pub mod dialog;
pub mod dropdown;
pub mod filedialog;
pub mod form;
pub mod hyperlink;
pub mod input;
pub mod list;
pub mod listbox;
pub mod menubar;
pub mod popup;
pub mod progress;
pub mod progressbar;
pub mod statusbar;
pub mod table;
pub mod textarea;

pub use chart::{
    bar_heights, line_dots, pie_sectors, tuichart_draw, ChartKind, ChartPoint, TuiChart,
};
pub use checkradio::{
    check_key, check_width, check_width_styled, checkbox_draw, radio_draw, radio_key, CheckItem,
    CheckNav, CheckStyle, GlyphSet, RadioNav,
};
pub use dialog::{confirm_draw, confirm_key, confirm_layout, DialogKey};
pub use dropdown::{
    dropdown_draw, dropdown_key, dropdown_overlay_rect, dropdown_size, Dropdown, DropdownKey,
};
pub use filedialog::{
    filedialog_draw, filedialog_key, filedialog_layout, FileDialog, FileDialogKey,
};
pub use form::{form_draw, form_key, Field, FieldKind, FormKey, FormState};
pub use hyperlink::{hyperlink_draw, Hyperlink};
pub use input::{input_draw, input_key, InputField, InputKey};
pub use list::{list_draw, list_key};
pub use listbox::{listbox_draw, listbox_key, listbox_visible, ListBox, ListNav};
pub use menubar::{menubar_draw, menubar_key, MenuBarKey, MenuDef, MenuItem};
pub use popup::{popup_draw, popup_key, popup_layout, popup_size, PopupItem, PopupKey};
pub use progress::{bar_fill_width, progress_draw, BarInfo, ProgressInfo};
pub use progressbar::{progress_fill, progressbar_draw, ProgressBar};
pub use statusbar::{
    fkey_bar, fkey_bar_compact, fkey_bar_stacked, fkey_bar_styled, statusbar_draw, Alignment,
    FKeyStyle, StatusBar, StatusColumn,
};
pub use table::{table_draw, table_key, TableDef, TableState};
pub use textarea::{textarea_draw, textarea_key, textarea_visible, TextArea, TextAreaKey};
