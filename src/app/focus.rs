//! Pila de foco: quién recibe las teclas cuando hay capas apiladas.
//!
//! Menubar → popup → diálogo → formulario: la cima manda. Al cerrar una
//! capa se hace `pop` y el foco vuelve solo a la anterior.

/// Capa de UI que puede tener el foco.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Desktop,
    MenuBar,
    Popup,
    Dialog,
    Form,
    Table,
}

/// Pila LIFO de capas (la cima = foco actual).
#[derive(Clone, Debug, Default)]
pub struct Focus {
    stack: Vec<Layer>,
}

impl Focus {
    pub fn new() -> Self {
        Self {
            stack: vec![Layer::Desktop],
        }
    }

    pub fn top(&self) -> Layer {
        self.stack.last().copied().unwrap_or(Layer::Desktop)
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    pub fn push(&mut self, layer: Layer) {
        self.stack.push(layer);
    }

    /// Saca la cima. Nunca deja la pila vacía (vuelve a Desktop).
    pub fn pop(&mut self) -> Layer {
        if self.stack.len() > 1 {
            self.stack.pop().unwrap_or(Layer::Desktop)
        } else {
            Layer::Desktop
        }
    }

    /// ¿Hay algún modal abierto sobre el escritorio?
    pub fn is_modal(&self) -> bool {
        !matches!(self.top(), Layer::Desktop | Layer::MenuBar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_restores_previous() {
        let mut f = Focus::new();
        assert_eq!(f.top(), Layer::Desktop);
        f.push(Layer::MenuBar);
        f.push(Layer::Popup);
        assert!(f.is_modal());
        assert_eq!(f.pop(), Layer::Popup);
        assert_eq!(f.top(), Layer::MenuBar);
        assert!(!f.is_modal());
        f.pop();
        f.pop(); // no baja de Desktop
        assert_eq!(f.top(), Layer::Desktop);
        assert_eq!(f.depth(), 1);
    }
}
