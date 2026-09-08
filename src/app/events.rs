//! Eventos estables de la app (mapeo fino sobre crossterm).
//!
//! Los widgets y ejemplos programan contra `AppEvent`/`AppKey`, no contra
//! crossterm directo: si mañana cambiamos de backend, solo se toca aquí.

use std::io;
use std::time::Duration;

/// Tecla lógica (estable entre versiones de crossterm).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppKey {
    Char(char),
    Enter,
    Esc,
    Tab,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
    Unknown,
}

/// Evento de la app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppEvent {
    Key { key: AppKey, alt: bool, ctrl: bool },
    Resize(u16, u16),
}

impl AppEvent {
    /// Constructor rápido para tests y demos (`key('a')`, sin modificadores).
    pub fn key(key: AppKey) -> Self {
        Self::Key {
            key,
            alt: false,
            ctrl: false,
        }
    }

    pub fn is_esc(self) -> bool {
        matches!(
            self,
            Self::Key {
                key: AppKey::Esc,
                ..
            }
        )
    }
}

/// Traduce un `KeyEvent` de crossterm a `AppEvent` (puro, testeable).
pub fn map_key(
    code: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
) -> AppEvent {
    use crossterm::event::{KeyCode as C, KeyModifiers as M};
    let alt = modifiers.contains(M::ALT);
    let ctrl = modifiers.contains(M::CONTROL)
        || modifiers.contains(M::SUPER)
        || modifiers.contains(M::HYPER);
    let key = match code {
        C::Char(c) => AppKey::Char(c),
        C::Enter => AppKey::Enter,
        C::Esc => AppKey::Esc,
        C::Tab | C::BackTab => AppKey::Tab,
        C::Backspace => AppKey::Backspace,
        C::Delete => AppKey::Delete,
        C::Up => AppKey::Up,
        C::Down => AppKey::Down,
        C::Left => AppKey::Left,
        C::Right => AppKey::Right,
        C::Home => AppKey::Home,
        C::End => AppKey::End,
        C::PageUp => AppKey::PageUp,
        C::PageDown => AppKey::PageDown,
        C::F(n) => AppKey::F(n),
        _ => AppKey::Unknown,
    };
    AppEvent::Key { key, alt, ctrl }
}

/// Convierte `AppKey` de vuelta a `crossterm::KeyCode` para reutilizar la
/// navegación pura de los widgets (`popup_key`, `form_key`, ...).
pub fn to_crossterm(key: AppKey) -> crossterm::event::KeyCode {
    use crossterm::event::KeyCode as C;
    match key {
        AppKey::Char(c) => C::Char(c),
        AppKey::Enter => C::Enter,
        AppKey::Esc => C::Esc,
        AppKey::Tab => C::Tab,
        AppKey::Backspace => C::Backspace,
        AppKey::Delete => C::Delete,
        AppKey::Up => C::Up,
        AppKey::Down => C::Down,
        AppKey::Left => C::Left,
        AppKey::Right => C::Right,
        AppKey::Home => C::Home,
        AppKey::End => C::End,
        AppKey::PageUp => C::PageUp,
        AppKey::PageDown => C::PageDown,
        AppKey::F(n) => C::F(n),
        AppKey::Unknown => C::Null,
    }
}

/// Versión con kind: ignora `Release` y procesa `Press`/`Repeat`.
/// Sin este filtro, en Windows cada toque genera Press + Release y todo
/// se mueve de 2 en 2 (menús que saltan una opción). La repetición por
/// tecla mantenida (`Repeat`) sí se atiende para scroll fluido.
pub fn map_key_event(
    code: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
    kind: crossterm::event::KeyEventKind,
) -> Option<AppEvent> {
    if kind == crossterm::event::KeyEventKind::Release {
        return None;
    }
    Some(map_key(code, modifiers))
}

/// Lee un evento con timeout. `Ok(None)` = sin novedad (el loop puede
/// animar/reloj). Ignora mouse, foco y `Release` (solo teclado + resize).
pub fn poll_event(timeout: Duration) -> io::Result<Option<AppEvent>> {
    use crossterm::event::Event;
    if !crossterm::event::poll(timeout)? {
        return Ok(None);
    }
    match crossterm::event::read()? {
        Event::Key(k) => Ok(map_key_event(k.code, k.modifiers, k.kind)),
        Event::Resize(w, h) => Ok(Some(AppEvent::Resize(w, h))),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode as C, KeyModifiers as M};

    #[test]
    fn maps_letters_and_modifiers() {
        assert_eq!(
            map_key(C::Char('a'), M::empty()),
            AppEvent::key(AppKey::Char('a'))
        );
        assert_eq!(
            map_key(C::Char('x'), M::ALT),
            AppEvent::Key {
                key: AppKey::Char('x'),
                alt: true,
                ctrl: false
            }
        );
        assert_eq!(map_key(C::F(2), M::empty()), AppEvent::key(AppKey::F(2)));
        assert!(map_key(C::Esc, M::empty()).is_esc());
        assert_eq!(map_key(C::BackTab, M::SHIFT), AppEvent::key(AppKey::Tab));
    }

    #[test]
    fn release_is_ignored_repeat_passes() {
        use crossterm::event::KeyEventKind as K;
        let press = map_key_event(C::Down, M::empty(), K::Press);
        assert_eq!(press, Some(AppEvent::key(AppKey::Down)));
        let repeat = map_key_event(C::Down, M::empty(), K::Repeat);
        assert_eq!(repeat, Some(AppEvent::key(AppKey::Down)));
        assert_eq!(map_key_event(C::Down, M::empty(), K::Release), None);
        assert_eq!(map_key_event(C::Esc, M::empty(), K::Release), None);
    }

    #[test]
    fn roundtrip_to_crossterm() {
        assert_eq!(to_crossterm(AppKey::Enter), C::Enter);
        assert_eq!(to_crossterm(AppKey::F(10)), C::F(10));
        assert_eq!(to_crossterm(AppKey::Char('q')), C::Char('q'));
    }
}
