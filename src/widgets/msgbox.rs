//! Diálogo modal rápido (`MsgBox`).
//!
//! Alerta centrada automáticamente: mensaje + configuración de botones
//! (`Buttons::Ok`, `Buttons::OkCancel`, `Buttons::YesNo`). Reutiliza
//! obligatoriamente el botón perfecto de 1 fila con sombra inferior de
//! media celda (`\u{2580}`) vía `prim::button`.

use crossterm::event::KeyCode;

use crate::core::{Buffer, Cell, Rect, Theme};
use crate::prim::{button, button_width, draw_text, visible_len, window, WindowOpts};

/// Configuración de botones del modal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Buttons {
    #[default]
    Ok,
    OkCancel,
    YesNo,
}

impl Buttons {
    /// Etiquetas en español, consistentes con el resto de la librería.
    pub fn labels(&self) -> Vec<String> {
        match self {
            Buttons::Ok => vec!["Aceptar".to_string()],
            Buttons::OkCancel => vec!["Aceptar".to_string(), "Cancelar".to_string()],
            Buttons::YesNo => vec!["Sí".to_string(), "No".to_string()],
        }
    }
}

/// Parte el mensaje en líneas que caben en `w`.
pub fn msgbox_wrap(message: &str, w: u16) -> Vec<String> {
    let mut lines = Vec::new();
    for part in message.split('\n') {
        let mut cur = String::new();
        for word in part.split(' ') {
            if cur.is_empty() {
                cur.push_str(word);
            } else if visible_len(&format!("{cur} {word}")) <= w {
                cur.push(' ');
                cur.push_str(word);
            } else {
                lines.push(cur);
                cur = word.to_string();
            }
        }
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Rect del modal centrado según el mensaje y los botones.
pub fn msgbox_layout(message: &str, buttons: &Buttons, screen: Rect) -> (Rect, Vec<Rect>) {
    let labels = buttons.labels();
    let btns_w: u16 = labels.iter().map(|s| button_width(s)).sum::<u16>()
        + 2u16.saturating_mul(labels.len().saturating_sub(1) as u16);
    let inner = screen.w.saturating_sub(8).clamp(20, 60);
    let wrapped = msgbox_wrap(message, inner.saturating_sub(4));
    let text_w = wrapped.iter().map(|s| visible_len(s)).max().unwrap_or(0);
    let w = (text_w.max(btns_w) + 6).clamp(24, screen.w.saturating_sub(2).max(24));
    let h = (wrapped.len() as u16 + 5).min(screen.h.max(1));
    let r = Rect::centered_in(w, h, screen);
    let mut rects = Vec::with_capacity(labels.len());
    let mut bx = r.x.saturating_add(r.w.saturating_sub(btns_w) / 2);
    let by = r.y.saturating_add(r.h).saturating_sub(2);
    for b in &labels {
        let bw = button_width(b);
        rects.push(Rect::new(bx, by, bw, 1));
        bx = bx.saturating_add(bw).saturating_add(2);
    }
    (r, rects)
}

/// Dibuja el modal (no toca la pila; el llamante hace `savescreen`).
pub fn msgbox_draw(
    buf: &mut Buffer,
    screen: Rect,
    title: &str,
    message: &str,
    buttons: &Buttons,
    selected: usize,
    theme: Theme,
) {
    let labels = buttons.labels();
    let (r, rects) = msgbox_layout(message, buttons, screen);
    if r.is_empty() {
        return;
    }
    window(buf, r, &WindowOpts::dialog(title, theme), theme);
    let wrapped = msgbox_wrap(message, r.w.saturating_sub(4));
    for (i, line) in wrapped.iter().enumerate() {
        let y = r.y.saturating_add(2).saturating_add(i as u16);
        if y >= r.bottom().saturating_sub(1) {
            break;
        }
        let qx =
            r.x.saturating_add(r.w.saturating_sub(visible_len(line)) / 2);
        draw_text(buf, qx, y, line, theme.dialog_attr());
    }
    // Botones de 1 fila con sombra `▀` (el foco: fondo negro en 1ª celda).
    for (i, (b, br)) in labels.iter().zip(rects.iter()).enumerate() {
        button(buf, br.x, br.y, b, theme);
        if i == selected.min(labels.len().saturating_sub(1)) {
            buf.fill_rect(
                Rect::new(br.x, br.y, 1, 1),
                Cell::new(' ', crate::core::Color::White, crate::core::Color::Black),
            );
        }
    }
}

/// Resultado de tecla en el modal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsgBoxKey {
    Stay,
    Move(usize),
    Accept(usize),
    Cancel,
}

/// `←→` mueven, `Enter` acepta, `Esc` cancela.
pub fn msgbox_key(buttons: &Buttons, selected: usize, code: KeyCode) -> MsgBoxKey {
    let labels = buttons.labels();
    match crate::widgets::dialog::confirm_key(&labels, selected, code) {
        crate::widgets::dialog::DialogKey::Stay => MsgBoxKey::Stay,
        crate::widgets::dialog::DialogKey::Move(i) => MsgBoxKey::Move(i),
        crate::widgets::dialog::DialogKey::Accept(i) => MsgBoxKey::Accept(i),
        crate::widgets::dialog::DialogKey::Cancel => MsgBoxKey::Cancel,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_per_variant() {
        assert_eq!(Buttons::Ok.labels(), vec!["Aceptar".to_string()]);
        assert_eq!(Buttons::OkCancel.labels().len(), 2);
        assert_eq!(
            Buttons::YesNo.labels(),
            vec!["Sí".to_string(), "No".to_string()]
        );
    }

    #[test]
    fn layout_centers_with_wrapped_text() {
        let screen = Rect::new(0, 0, 80, 25);
        let (r, rects) = msgbox_layout("¿Abrir explorador de archivos?", &Buttons::YesNo, screen);
        assert_eq!(r.x, (80 - r.w) / 2);
        assert_eq!(rects.len(), 2);
        assert!(rects[1].x > rects[0].x);
        assert!(r.w >= 24);
    }

    #[test]
    fn keys_accept_cancel_move() {
        let b = Buttons::YesNo;
        assert_eq!(msgbox_key(&b, 0, KeyCode::Right), MsgBoxKey::Move(1));
        assert_eq!(msgbox_key(&b, 1, KeyCode::Right), MsgBoxKey::Move(0));
        assert_eq!(msgbox_key(&b, 0, KeyCode::Enter), MsgBoxKey::Accept(0));
        assert_eq!(msgbox_key(&b, 0, KeyCode::Esc), MsgBoxKey::Cancel);
    }

    #[test]
    fn draws_message_and_one_row_buttons() {
        let t = Theme::clipper();
        let mut b = Buffer::blank(80, 25, t.desktop);
        let screen = Rect::new(0, 0, 80, 25);
        msgbox_draw(&mut b, screen, "AVISO", "Hola", &Buttons::Ok, 0, t);
        let (r, _) = msgbox_layout("Hola", &Buttons::Ok, screen);
        // Ventana menta + mensaje centrado + botón teal de 1 fila.
        assert_eq!(b.get(r.x + 2, r.y + 2).unwrap().bg, t.dialog);
        let mut saw_button = false;
        let mut saw_shadow = false;
        for y in r.y..r.bottom() {
            for x in r.x..r.right() {
                let c = b.get(x, y).unwrap();
                if c.bg == t.button_bg {
                    saw_button = true;
                }
                if c.ch == '\u{2580}' {
                    saw_shadow = true;
                }
            }
        }
        assert!(saw_button && saw_shadow);
    }

    #[test]
    fn long_message_wraps_inside() {
        let lines = msgbox_wrap(
            "Esta es una frase deliberadamente larga para forzar el ajuste",
            20,
        );
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| visible_len(l) <= 20));
    }
}
