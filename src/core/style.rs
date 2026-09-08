//! Estilo global de controles: los 4 parámetros de tematización.
//!
//! Toda estructura de configuración de widgets expone (directa o vía
//! `Option`, donde `None` = "usa el `Theme`"):
//! * `foreground_color` — texto / interior del icono.
//! * `background_color` — fondo del widget.
//! * `border_color` — bordes del componente (si aplica).
//! * `has_shadow` — activa la proyección de sombra.
//!
//! Nota de tipos: la especificación pedía `(u8, u8, u8)`, pero el motor
//! es de 16 colores ANSI (`Color`, como la VRAM de texto de la época);
//! se usa `Color` con los mismos nombres de campo.

use super::color::Color;

/// Los 4 parámetros globales de estilo de un control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidgetStyle {
    pub foreground_color: Color,
    pub background_color: Color,
    pub border_color: Color,
    pub has_shadow: bool,
}

impl Default for WidgetStyle {
    fn default() -> Self {
        Self {
            foreground_color: Color::White,
            background_color: Color::Blue,
            border_color: Color::Black,
            has_shadow: true,
        }
    }
}

impl WidgetStyle {
    /// Estilo que deriva del tema Clipper (popup azul, sombra sí).
    pub fn clipper_popup() -> Self {
        Self {
            foreground_color: Color::White,
            background_color: Color::Blue,
            border_color: Color::Black,
            has_shadow: true,
        }
    }

    /// Estilo botón teal clásico.
    pub fn clipper_button() -> Self {
        Self {
            foreground_color: Color::White,
            background_color: Color::Teal,
            border_color: Color::Black,
            has_shadow: true,
        }
    }

    /// Sin sombra ni borde visible (controles de texto).
    pub fn flat(foreground: Color, background: Color) -> Self {
        Self {
            foreground_color: foreground,
            background_color: background,
            border_color: background,
            has_shadow: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_carry_shadow() {
        let s = WidgetStyle::default();
        assert!(s.has_shadow);
        assert!(!WidgetStyle::flat(Color::Black, Color::White).has_shadow);
    }
}
