//! Diálogos de confirmación: ventana menta + pregunta + botones teal.
//!
//! El clásico `¿ Está conforme ? [Si][No]`. Hotkeys por letra (S/N).

use crossterm::event::KeyCode;

use crate::core::{Buffer, Rect, Theme};
use crate::prim::{
    button_ex, button_width, draw_text, hot_key_of, visible_len, window, ButtonOpts, WindowOpts,
};

/// Calcula rect del diálogo + rects de cada botón (para futuro mouse).
/// `buttons` admite `&` (ej `"&Si"`, `"&No"`).
pub fn confirm_layout(question: &str, buttons: &[String], screen: Rect) -> (Rect, Vec<Rect>) {
    let qlen = visible_len(question);
    let btns_w: u16 = buttons.iter().map(|s| button_width(s)).sum::<u16>()
        + 2u16.saturating_mul(buttons.len().saturating_sub(1) as u16);
    let w = (qlen.max(btns_w) + 6).clamp(20, screen.w.saturating_sub(2).max(20));
    let h = 7u16.min(screen.h.max(1));
    let r = Rect::centered_in(w, h, screen);
    // Fila de botones centrada.
    let mut rects = Vec::with_capacity(buttons.len());
    let mut bx = r.x.saturating_add(r.w.saturating_sub(btns_w) / 2);
    let by = r.y.saturating_add(r.h).saturating_sub(2);
    for b in buttons {
        let bw = button_width(b);
        rects.push(Rect::new(bx, by, bw, 1));
        bx = bx.saturating_add(bw).saturating_add(2);
    }
    (r, rects)
}

/// Dibuja pregunta + botones con `selected` en foco luminoso (el botón
/// encendido comunica el foco, sin manchas ni marcas externas).
pub fn confirm_draw(
    buf: &mut Buffer,
    screen: Rect,
    title: &str,
    question: &str,
    buttons: &[String],
    selected: usize,
    theme: Theme,
) {
    let (r, _) = confirm_layout(question, buttons, screen);
    if r.is_empty() {
        return;
    }
    window(buf, r, &WindowOpts::dialog(title, theme), theme);
    let qx =
        r.x.saturating_add(r.w.saturating_sub(visible_len(question)) / 2);
    draw_text(buf, qx, r.y + 2, question, theme.dialog_attr());
    // Botones: el seleccionado lleva `focused = true`.
    let (_, rects) = confirm_layout(question, buttons, screen);
    for (i, (b, br)) in buttons.iter().zip(rects.iter()).enumerate() {
        button_ex(
            buf,
            br.x,
            br.y,
            b,
            theme,
            ButtonOpts::default().focused(i == selected.min(buttons.len().saturating_sub(1))),
        );
    }
}

/// Resultado de tecla en el diálogo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogKey {
    Stay,
    Move(usize),
    Accept(usize),
    Cancel,
}

/// ←→/Tab mueven, Enter acepta, Esc cancela, letra = hotkey directa.
pub fn confirm_key(buttons: &[String], selected: usize, code: KeyCode) -> DialogKey {
    use DialogKey::*;
    let n = buttons.len();
    if n == 0 {
        return Cancel;
    }
    let sel = selected.min(n - 1);
    match code {
        KeyCode::Left | KeyCode::Up => Move((sel + n - 1) % n),
        KeyCode::Right | KeyCode::Down | KeyCode::Tab => Move((sel + 1) % n),
        KeyCode::Home => Move(0),
        KeyCode::End => Move(n - 1),
        KeyCode::Enter => Accept(sel),
        KeyCode::Esc => Cancel,
        KeyCode::Char(c) => {
            let lc = c.to_ascii_lowercase();
            for (i, b) in buttons.iter().enumerate() {
                if hot_key_of(b).map(|h| h.to_ascii_lowercase()) == Some(lc) {
                    return Accept(i);
                }
            }
            Stay
        }
        _ => Stay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn btns() -> Vec<String> {
        vec!["&Si".to_string(), "&No".to_string()]
    }

    #[test]
    fn layout_centers_and_fits() {
        let screen = Rect::new(0, 0, 80, 25);
        let (r, rects) = confirm_layout("¿ Está conforme ?", &btns(), screen);
        // Centrado real (división entera): x = (80 - w) / 2.
        assert_eq!(r.x, (80 - r.w) / 2);
        assert!(r.w >= 17 + 6); // pregunta + aire
        assert_eq!(rects.len(), 2);
        assert!(rects[1].x > rects[0].x);
    }

    #[test]
    fn keys_accept_cancel_move() {
        let b = btns();
        assert_eq!(confirm_key(&b, 0, KeyCode::Right), DialogKey::Move(1));
        assert_eq!(confirm_key(&b, 1, KeyCode::Right), DialogKey::Move(0));
        assert_eq!(confirm_key(&b, 0, KeyCode::Enter), DialogKey::Accept(0));
        assert_eq!(confirm_key(&b, 1, KeyCode::Esc), DialogKey::Cancel);
        assert_eq!(confirm_key(&b, 0, KeyCode::Char('n')), DialogKey::Accept(1));
        assert_eq!(confirm_key(&b, 0, KeyCode::Char('s')), DialogKey::Accept(0));
    }

    #[test]
    fn draws_question_and_buttons() {
        let t = Theme::clipper();
        let mut buf = Buffer::blank(80, 25, t.desktop);
        let screen = Rect::new(0, 0, 80, 25);
        confirm_draw(
            &mut buf,
            screen,
            "AVISO",
            "¿ Está conforme ?",
            &btns(),
            0,
            t,
        );
        // Diálogo menta presente y pregunta centrada visible.
        let (r, _) = confirm_layout("¿ Está conforme ?", &btns(), screen);
        assert_eq!(buf.get(r.x + 2, r.y + 2).unwrap().bg, t.dialog);
        let qx = r.x + (r.w - 17) / 2; // "¿ Está conforme ?" = 17 celdas
        assert_eq!(buf.get(qx, r.y + 2).unwrap().ch, '¿');
    }
}
