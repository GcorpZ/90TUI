//! App shell (Fase 6): eventos, foco, layout, botonera y `App`.
//!
//! El loop concreto vive en la app usuaria (ver `examples/clipper_shell.rs`):
//! `poll → matches por foco → pintar → present`. Aquí están las piezas.

#[allow(clippy::module_inception)]
pub mod app;
pub mod events;
pub mod fkeys;
pub mod focus;
pub mod layout;

pub use app::{menu_title_x, test_app, App, ShellContent};
pub use events::{map_key, poll_event, to_crossterm, AppEvent, AppKey};
pub use fkeys::{draw_fkeys, match_fkey, FKeyDef};
pub use focus::{Focus, Layer};
pub use layout::{centered, desktop, popup, DesktopLayout};
