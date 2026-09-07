//! Primitivas gTUI estilo Clipper: bloques sólidos + sombra dura.
//!
//! Nada de box-drawing. Todo dibuja sobre `Buffer`; la pila de pantallas
//! y los eventos llegan en Fase 5 (`widgets`) y Fase 6 (`app`).

pub mod button;
pub mod label;
pub mod separator;
pub mod shadow;
pub mod title;
pub mod window;

pub use button::{button, button_width};
pub use label::{
    base_on, draw_hot_label, draw_text, fit_text, hot_key_of, parse_hotkey, visible_len,
};
pub use separator::hsep;
pub use shadow::{shadow, shadow_offset};
pub use title::{status_bar, top_bar};
pub use window::{window, WindowOpts};
