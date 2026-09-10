//! Botonera de función como modelo: `F2 Grabar`, `Esc Salir`.
//!
//! Une dibujo (`widgets::fkey_bar`) con disparo (`match_fkey`): la app
//! declara las teclas una vez y el mismo modelo pinta y responde.
//! `FKeyBar` es la variante declarativa con cupo: `cant_funcs` reserva
//! los slots y `funcs_loads` recibe las acciones en orden secuencial.

use crate::core::{Buffer, Theme};
use crate::prim::fkey_badge;
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

/// Un slot cargado: número de función + acción (p.ej. `1` + `"Help"`).
/// El número es explícito para permitir huecos (`F9`, `F10`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FKeySlot {
    pub num: u8,
    pub action: String,
}

/// Error de carga de `FKeyBar`: más funciones que teclas declaradas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FKeyError {
    /// `cant_funcs` = cupo declarado, `tried` = total que se intentó cargar.
    Overflow { cant_funcs: usize, tried: usize },
}

impl std::fmt::Display for FKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Overflow { cant_funcs, tried } => write!(
                f,
                "FKeyBar: {tried} funciones no caben en {cant_funcs} teclas declaradas"
            ),
        }
    }
}

impl std::error::Error for FKeyError {}

/// Botonera declarativa con cupo: `cant_funcs` fija cuántas teclas F
/// declara el programador y `funcs_loads` recibe las acciones en orden
/// secuencial vía `load`/`load_many`. Cargar de más devuelve
/// `FKeyError::Overflow` (p.ej. 4 funciones con 3 declaradas).
/// Pinta con `icons20::fkey_badge` (dos tonos sobre navy).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FKeyBar {
    pub cant_funcs: usize,
    pub funcs_loads: Vec<FKeySlot>,
}

impl FKeyBar {
    /// Reserva el cupo (`cant_funcs` teclas, sin acciones todavía).
    pub fn new(cant_funcs: usize) -> Self {
        Self {
            cant_funcs,
            funcs_loads: Vec::new(),
        }
    }

    /// Agrega una acción al siguiente slot. Falla si el cupo está lleno.
    pub fn load(&mut self, num: u8, action: &str) -> Result<(), FKeyError> {
        if self.funcs_loads.len() >= self.cant_funcs {
            return Err(FKeyError::Overflow {
                cant_funcs: self.cant_funcs,
                tried: self.funcs_loads.len() + 1,
            });
        }
        self.funcs_loads.push(FKeySlot {
            num,
            action: action.to_string(),
        });
        Ok(())
    }

    /// Agrega varias en orden; si no caben, no carga ninguna (atómico).
    pub fn load_many(&mut self, slots: &[(u8, &str)]) -> Result<(), FKeyError> {
        if self.funcs_loads.len() + slots.len() > self.cant_funcs {
            return Err(FKeyError::Overflow {
                cant_funcs: self.cant_funcs,
                tried: self.funcs_loads.len() + slots.len(),
            });
        }
        for (num, action) in slots {
            self.funcs_loads.push(FKeySlot {
                num: *num,
                action: (*action).to_string(),
            });
        }
        Ok(())
    }

    /// Acciones cargadas hasta ahora.
    pub fn len(&self) -> usize {
        self.funcs_loads.len()
    }

    /// Sin acciones cargadas.
    pub fn is_empty(&self) -> bool {
        self.funcs_loads.is_empty()
    }

    /// Cupo completo (el próximo `load` fallaría).
    pub fn is_full(&self) -> bool {
        self.funcs_loads.len() >= self.cant_funcs
    }

    /// Pinta los slots en orden desde x=1 en la fila `y`.
    /// Devuelve las celdas exactas ocupadas (suma de los badges).
    pub fn draw(&self, buf: &mut Buffer, y: u16, theme: Theme) -> u16 {
        let mut cx = 1u16;
        let mut w = 0u16;
        for slot in &self.funcs_loads {
            let bw = fkey_badge(buf, cx, y, slot.num, &slot.action, theme);
            cx = cx.saturating_add(bw);
            w = w.saturating_add(bw);
            if cx >= buf.width() {
                break;
            }
        }
        w
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

    #[test]
    fn bar_loads_in_order_and_draws_two_tone() {
        use crate::core::Color;
        let t = Theme::clipper();
        let mut bar = FKeyBar::new(3);
        assert!(bar.is_empty());
        assert!(!bar.is_full());
        bar.load(1, "Help").unwrap();
        bar.load_many(&[(2, "Qview"), (3, "Exit")]).unwrap();
        assert_eq!(bar.len(), 3);
        assert!(bar.is_full());
        // Orden secuencial preservado.
        assert_eq!(bar.funcs_loads[0].action, "Help");
        assert_eq!(bar.funcs_loads[2].num, 3);
        let mut buf = Buffer::blank(60, 25, t.desktop);
        // F¹Help=8 + F²Qview=9 + F³Exit=8.
        assert_eq!(bar.draw(&mut buf, 24, t), 8 + 9 + 8);
        assert_eq!(buf.get(1, 24).unwrap().ch, 'F');
        assert_eq!(buf.get(1, 24).unwrap().fg, Color::Yellow);
        assert_eq!(buf.get(2, 24).unwrap().fg, Color::DarkYellow);
    }

    #[test]
    fn bar_overflow_is_an_error() {
        let mut bar = FKeyBar::new(3);
        bar.load(1, "Help").unwrap();
        bar.load(2, "Qview").unwrap();
        bar.load(3, "Exit").unwrap();
        // La 4ª no cabe: error con cupo e intento.
        assert_eq!(
            bar.load(4, "Graphs"),
            Err(FKeyError::Overflow {
                cant_funcs: 3,
                tried: 4
            })
        );
        assert_eq!(bar.len(), 3); // nada se agregó
                                  // `load_many` es atómico: si no caben, no carga ninguna.
        let mut bar2 = FKeyBar::new(2);
        assert_eq!(
            bar2.load_many(&[(1, "A"), (2, "B"), (3, "C")]),
            Err(FKeyError::Overflow {
                cant_funcs: 2,
                tried: 3
            })
        );
        assert!(bar2.is_empty());
        // El display no paniquea y menciona el cupo.
        assert!(format!(
            "{}",
            FKeyError::Overflow {
                cant_funcs: 3,
                tried: 4
            }
        )
        .contains('3'));
    }
}
