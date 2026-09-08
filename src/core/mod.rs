//! Núcleo 90TUI: VRAM virtual (doble buffer + pila + tema + backend).
//!
//! Capas superiores (`prim`, `widgets`, `app` en Fases 4-6) solo usan
//! `Buffer`, `Rect`, `Screen`, `Theme` y el trait `Backend`.

pub mod backend;
pub mod buffer;
pub mod cell;
pub mod color;
pub mod rect;
pub mod screen;
pub mod style;
pub mod theme;

pub use backend::{enter_screen, leave_screen, Backend, CrosstermBackend, TestBackend};
pub use buffer::{Buffer, DrawOp, Snapshot};
pub use cell::Cell;
pub use color::{Attr, Color};
pub use rect::Rect;
pub use screen::{Screen, ScreenStack};
pub use style::WidgetStyle;
pub use theme::Theme;
