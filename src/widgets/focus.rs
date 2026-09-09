//! Gestión de foco por ámbitos (`FocusManager`) + trait de eventos.
//!
//! Arquitectura de framework, no de demo: la inteligencia vive aquí.
//!
//! * `FocusManager`: el `TabControl` es el contenedor primario. Cada página
//!   declara su ámbito (scope) con los widgets que contiene; el ciclo de
//!   `Tab` solo visita widgets del ámbito ACTIVO. Los widgets de páginas
//!   ocultas quedan deshabilitados para el teclado y omitidos del ciclo.
//!   Convención: el nombre del ámbito coincide con la etiqueta de la
//!   pestaña (`TabControl::active_scope`), así el demo solo declara.
//! * `HandleEvent`: punto de entrada orientado a objetos para eventos de
//!   teclado (`*_key` sigue existiendo como función pura testeable).
//!   El contexto (`EventCtx`) porta lo que el widget necesita (filas
//!   visibles para scroll, etc.).

use std::collections::HashMap;

use crossterm::event::KeyCode;

/// Contexto de un evento de teclado para `HandleEvent`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventCtx {
    /// Filas visibles (para widgets con scroll: listas, tablas, memos).
    pub visible: usize,
}

impl EventCtx {
    pub fn new(visible: usize) -> Self {
        Self { visible }
    }
}

/// Punto de entrada OO de eventos. Los widgets implementan este trait
/// delegando en su función pura (`grid_key`, `listbox_key`, …).
pub trait HandleEvent {
    /// Resultado de navegación propio del widget.
    type Out;
    fn handle_event(&mut self, ctx: &EventCtx, code: KeyCode) -> Self::Out;
}

/// Gestor de foco por ámbitos. `Tab` avanza solo dentro del activo.
#[derive(Clone, Debug, Default)]
pub struct FocusManager {
    /// Ámbito → paradas en orden visual.
    scopes: HashMap<String, Vec<usize>>,
    active: String,
    /// Índice dentro del ámbito activo.
    pos: usize,
}

impl FocusManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Declara un ámbito (`"General"`, `"Inventario"`, …) con sus paradas.
    pub fn add_scope(&mut self, name: &str, stops: &[usize]) {
        self.scopes.insert(name.to_string(), stops.to_vec());
        if self.active.is_empty() {
            self.active = name.to_string();
        }
    }

    /// Activa un ámbito (típico: `tabs.active_scope()`). Recoloca el
    /// cursor: si la parada actual no pertenece al ámbito, salta a la
    /// primera (nunca se queda en un widget invisible).
    pub fn set_active(&mut self, name: &str) {
        let cur = self.current(); // ANTES de cambiar: parada del ámbito previo
        self.active = name.to_string();
        if let Some(list) = self.scopes.get(&self.active) {
            self.pos = cur
                .and_then(|c| list.iter().position(|&s| s == c))
                .unwrap_or(0)
                .min(list.len().saturating_sub(1));
        } else {
            self.pos = 0;
        }
    }

    pub fn active(&self) -> &str {
        &self.active
    }

    fn list(&self) -> &[usize] {
        self.scopes
            .get(&self.active)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Parada actual (si el ámbito no está vacío).
    pub fn current(&self) -> Option<usize> {
        self.list().get(self.pos).copied()
    }

    /// ¿Pertenece `stop` al ámbito activo? (guarda de enrutado).
    pub fn is_focusable(&self, stop: usize) -> bool {
        self.list().contains(&stop)
    }

    /// Avanza (`Tab`): solo dentro del ámbito activo, circular.
    /// Si la parada actual es de otro ámbito, salta a la primera.
    pub fn cycle_next(&mut self) -> Option<usize> {
        let list = self.list().to_vec();
        if list.is_empty() {
            return None;
        }
        let cur = self.current();
        let i = cur
            .and_then(|c| list.iter().position(|&s| s == c))
            .map(|p| (p + 1) % list.len())
            .unwrap_or(0);
        self.pos = i;
        Some(list[i])
    }

    /// Retrocede (Shift+Tab futuro): circular dentro del activo.
    pub fn cycle_prev(&mut self) -> Option<usize> {
        let list = self.list().to_vec();
        if list.is_empty() {
            return None;
        }
        let cur = self.current();
        let i = cur
            .and_then(|c| list.iter().position(|&s| s == c))
            .map(|p| (p + list.len() - 1) % list.len())
            .unwrap_or(0);
        self.pos = i;
        Some(list[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fm() -> FocusManager {
        let mut f = FocusManager::new();
        f.add_scope("General", &[0, 1, 2, 3]);
        f.add_scope("Inventario", &[0, 1, 3]);
        f
    }

    #[test]
    fn cycles_only_inside_active_scope() {
        let mut f = fm();
        f.set_active("General");
        assert_eq!(f.cycle_next(), Some(1)); // pos 0 → 1
        f.set_active("Inventario");
        // La parada 1 sí pertenece; sigue desde ahí.
        assert_eq!(f.cycle_next(), Some(3));
        // Circular dentro del ámbito: 3 → 0 (salta el 2 oculto).
        assert_eq!(f.cycle_next(), Some(0));
        assert!(!f.is_focusable(2));
        assert!(f.is_focusable(3));
    }

    #[test]
    fn snaps_to_first_when_current_is_hidden() {
        let mut f = fm();
        f.set_active("General");
        f.cycle_next(); // 1
        f.cycle_next(); // 2
        assert_eq!(f.current(), Some(2));
        f.set_active("Inventario"); // el 2 no existe aquí → primera
        assert_eq!(f.current(), Some(0));
    }

    #[test]
    fn empty_or_unknown_scope_is_quiet() {
        let mut f = FocusManager::new();
        assert_eq!(f.cycle_next(), None);
        f.add_scope("A", &[]);
        f.set_active("A");
        assert_eq!(f.cycle_next(), None);
        f.set_active("Inexistente");
        assert_eq!(f.current(), None);
    }
}
