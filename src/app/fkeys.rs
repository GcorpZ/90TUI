//! Botonera de función como modelo: `F2 Grabar`, `Esc Salir`.
//!
//! Une dibujo (`widgets::fkey_bar`) con disparo (`match_fkey`): la app
//! declara las teclas una vez y el mismo modelo pinta y responde.

use crate::core::{Buffer, Theme};
use crate::widgets::fkey_bar;

use super::events::AppKey;

/// Una tecla de la botonera.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FKeyDef {
    pub key: String,
    pub label: String,
}

impl FKeyDef {
    pub fn new(key: &str, label: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
        }
    }
}

/// Dibuja la botonera en la fila `y` (vía `widgets::fkey_bar`).
pub fn draw_fkeys(buf: &mut Buffer, y: u16, defs: &[FKeyDef], theme: Theme) -> u16 {
    let refs: Vec<(&str, &str)> = defs
        .iter()
        .map(|d| (d.key.as_str(), d.label.as_str()))
        .collect();
    fkey_bar(buf, y, &refs, theme)
}

/// ¿Qué botón dispara esta tecla? `F(n)` ↔ `"Fn"`, `Esc` ↔ `"Esc"`.
/// Devuelve el índice en `defs`.
pub fn match_fkey(defs: &[FKeyDef], key: AppKey) -> Option<usize> {
    match key {
        AppKey::F(n) => {
            let want = format!("F{n}");
            defs.iter().position(|d| d.key.eq_ignore_ascii_case(&want))
        }
        AppKey::Esc => defs.iter().position(|d| d.key.eq_ignore_ascii_case("Esc")),
        AppKey::Enter => defs
            .iter()
            .position(|d| d.key.eq_ignore_ascii_case("Enter")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defs() -> Vec<FKeyDef> {
        vec![FKeyDef::new("F2", "Grabar"), FKeyDef::new("Esc", "Salir")]
    }

    #[test]
    fn matches_f_and_esc_case_insensitive() {
        let d = defs();
        assert_eq!(match_fkey(&d, AppKey::F(2)), Some(0));
        assert_eq!(match_fkey(&d, AppKey::F(9)), None);
        assert_eq!(match_fkey(&d, AppKey::Esc), Some(1));
        assert_eq!(match_fkey(&d, AppKey::Enter), None);
        assert_eq!(match_fkey(&d, AppKey::Char('x')), None);
    }

    #[test]
    fn draws_row() {
        let t = Theme::clipper();
        let mut buf = Buffer::blank(60, 25, t.desktop);
        draw_fkeys(&mut buf, 23, &defs(), t);
        assert_eq!(buf.get(2, 23).unwrap().ch, 'F');
        assert_eq!(buf.get(2, 23).unwrap().bg, t.button_bg);
    }
}
