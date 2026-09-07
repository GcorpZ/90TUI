//! Temas de color: SAINT pastel por defecto, Turbo opt-in.
//!
//! La queja original contra Ratatui/Turbo Vision era el alto contraste
//! que "hace doler a la vista". El tema por defecto reproduce la
//! paleta refrescante de SAINT 7.51 (cian/teal/azul medio/gris/amarillo
//! hotkey sobre fondos claros). Nada de colores hardcodeados en los
//! widgets: todo sale de aquí.

use super::color::{Attr, Color};

/// Paleta completa de la librería.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub desktop: Color,
    pub work: Color,
    pub work_text: Color,
    pub navy: Color,
    pub teal: Color,
    pub popup: Color,
    pub popup_text: Color,
    pub hot: Color,
    pub select_bg: Color,
    pub select_fg: Color,
    pub button_bg: Color,
    pub button_fg: Color,
    pub shadow: Color,
    pub dialog: Color,
    pub dialog_text: Color,
    pub form_bg: Color,
    pub form_text: Color,
    pub field_bg: Color,
    pub status_bg: Color,
    pub status_fg: Color,
}

impl Theme {
    /// SAINT Administrativo 7.51 (ver FASE0): escritorio cian, trabajo
    /// blanco, barras navy, menús teal, popups azul medio, hotkeys
    /// amarillas, selección gris+negro, botones teal, diálogos menta,
    /// formularios negros, sombras negras duras.
    pub fn saint751() -> Self {
        Self {
            desktop: Color::Cyan,
            work: Color::White,
            work_text: Color::Black,
            navy: Color::Navy,
            teal: Color::Teal,
            popup: Color::Blue,
            popup_text: Color::White,
            hot: Color::Yellow,
            select_bg: Color::Grey,
            select_fg: Color::Black,
            button_bg: Color::Teal,
            button_fg: Color::White,
            shadow: Color::Black,
            dialog: Color::Mint,
            dialog_text: Color::Black,
            form_bg: Color::Black,
            form_text: Color::White,
            field_bg: Color::Grey,
            status_bg: Color::Navy,
            status_fg: Color::White,
        }
    }

    /// Nostalgia Turbo Vision (NDD/PCTOOLS): alto contraste clásico.
    /// Opt-in explícito; nunca el default.
    pub fn turbo() -> Self {
        Self {
            desktop: Color::DarkGrey,
            work: Color::Blue,
            work_text: Color::White,
            navy: Color::Navy,
            teal: Color::Teal,
            popup: Color::Grey,
            popup_text: Color::Black,
            hot: Color::Red,
            select_bg: Color::Black,
            select_fg: Color::White,
            button_bg: Color::Grey,
            button_fg: Color::Black,
            shadow: Color::Black,
            dialog: Color::Cyan,
            dialog_text: Color::Black,
            form_bg: Color::Blue,
            form_text: Color::White,
            field_bg: Color::Cyan,
            status_bg: Color::Navy,
            status_fg: Color::White,
        }
    }

    // --- Atributos derivados (los widgets usan estos, no los colores) ---

    pub fn desktop_attr(self) -> Attr {
        Attr::new(self.desktop, self.desktop)
    }
    pub fn work_attr(self) -> Attr {
        Attr::new(self.work_text, self.work)
    }
    pub fn top_attr(self) -> Attr {
        Attr::bold(self.status_fg, self.navy)
    }
    pub fn status_attr(self) -> Attr {
        Attr::bold(self.status_fg, self.status_bg)
    }
    pub fn menu_attr(self, active: bool) -> Attr {
        if active {
            Attr::bold(Color::White, Color::Black)
        } else {
            Attr::bold(Color::White, self.teal)
        }
    }
    pub fn popup_attr(self) -> Attr {
        Attr::new(self.popup_text, self.popup)
    }
    pub fn popup_sel_attr(self) -> Attr {
        Attr::bold(self.select_fg, self.select_bg)
    }
    pub fn hot_attr(self) -> Attr {
        Attr::bold(self.hot, self.popup)
    }
    pub fn button_attr(self) -> Attr {
        Attr::bold(self.button_fg, self.button_bg)
    }
    pub fn dialog_attr(self) -> Attr {
        Attr::new(self.dialog_text, self.dialog)
    }
    pub fn form_attr(self) -> Attr {
        Attr::new(self.form_text, self.form_bg)
    }
    pub fn field_attr(self) -> Attr {
        Attr::bold(Color::Black, self.field_bg)
    }
    pub fn shadow_cell(self) -> super::cell::Cell {
        super::cell::Cell::new(' ', self.shadow, self.shadow)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::saint751()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saint_default_is_pastel_not_high_contrast() {
        let s = Theme::saint751();
        // Escritorio claro + trabajo claro: la seña SAINT.
        assert_eq!(s.desktop, Color::Cyan);
        assert_eq!(s.work, Color::White);
        // Selección gris legible, no invertido quemante.
        assert_eq!((s.select_bg, s.select_fg), (Color::Grey, Color::Black));
    }

    #[test]
    fn turbo_differs_from_saint() {
        assert_ne!(Theme::saint751(), Theme::turbo());
    }

    #[test]
    fn menu_active_is_black_bar() {
        let t = Theme::saint751();
        let a = t.menu_attr(true);
        assert_eq!((a.fg, a.bg), (Color::White, Color::Black));
    }
}
