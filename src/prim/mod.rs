//! Primitivas gTUI estilo Clipper: bloques sólidos + sombra dura.
//!
//! Nada de box-drawing. Todo dibuja sobre `Buffer`; la pila de pantallas
//! y los eventos llegan en Fase 5 (`widgets`) y Fase 6 (`app`).

pub mod button;
pub mod icons;
pub mod label;
pub mod scrollbar;
pub mod separator;
pub mod shadow;
pub mod title;
pub mod window;

pub use button::{
    button, button_draw, button_draw_ex, button_draw_opts, button_ex, button_width, ButtonOpts,
    BUTTON_MIN_WIDTH,
};
pub use icons::{drive, folder, win_close, win_min, FolderGlyphs};
pub use label::{
    base_on, draw_hot_label, draw_text, fit_text, hot_key_of, parse_hotkey, visible_len, HotAttrs,
};
pub use scrollbar::vscrollbar;
pub use separator::hsep;
pub use shadow::{shadow, shadow_offset, shadow_solid, shadow_stipple, shadow_styled, ShadowStyle};
pub use title::{status_bar, top_bar};
pub use window::{window, WindowOpts};
