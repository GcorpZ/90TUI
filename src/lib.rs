//! 90TUI (`tui90`) — gTUI artesanal estilo SAINT 7.51 / CA-Clipper para Rust.
//!
//! Licencia dual: MIT OR Apache-2.0. Ver `LICENSE-MIT` y `LICENSE-APACHE`.
//!
//! Estado: esqueleto de documentación (Fase 0-2). La construcción por partes
//! vive en la rama `90tui-built` a partir de la Fase 3:
//! Parte 1 `core`, Parte 2 `prim`, Parte 3 `widgets`, Parte 4 `app`.

/// Versión del esqueleto documental.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton_version_is_set() {
        assert!(!VERSION.is_empty());
    }
}
