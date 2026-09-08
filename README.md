# 90TUI — la gTUI artesanal de los 90, reencarnada en Rust

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Windows · macOS · Linux](https://img.shields.io/badge/platform-win%20%7C%20mac%20%7C%20linux-lightgrey.svg)](docs/GUIA-INSTALACION.md)
[![Tests 85 passing](https://img.shields.io/badge/tests-85%20passing-brightgreen.svg)](https://github.com/GcorpZ/90TUI)

> **Una alternativa a Ratatui con alma de gestión DOS de principios de los 90:**
> bloques sólidos pastel en vez de box-drawing, sombra dura en vez de `▒▓`,
> hotkeys amarillas automáticas — y cada control se dibuja con **una sola llamada**.

```
 EMPRESA DEMO C.A.                              Jueves 26 de Diciembre de 2013
 ┌ Archivos  Transacciones [Varios] ─────────────────────────────────────────┐
 │                                                                            │
 │   ┌──────────── Respaldo de datos ──────────────┐                          │
 │   │  Estado del sistema                        │██                        │
 │   │  Resumen gerencial                         │██  ¿ Finalizar la demo ?  │
 │   │ ──────────────────────────────────────     │██  ┌──────┐  ┌──────┐    │
 │   │  Respaldo de datos                         │██  │  Si  │  │  No  │    │
 │   │  Ordenar índices                           │██  └──────┘  └──────┘    │
 │   └────────────────────────────────────────────┘██                        │
 │ F2 Grabar   Esc Salir                                                     │
 TUI90 Demo v0.0.1                                          Alt-F1: Ayuda
```

*(Boceto ASCII del look: menubar teal con activo negro, popup azul con
hotkeys, selección gris, botones teal con sombra. Corre los ejemplos y
míralo en colores de verdad.)*

## Por qué existe

Ratatui es excelente, pero "se siente TUI": marcos `╔═╗` por todos lados,
alto contraste que cansa, y hay que hacer malabares para un menú
desplegable decente. La escuela **CA-Clipper de principios de los 90**
demostró otra vía con el mismo hardware (VGA texto, `0xB800`): pintar
bloques de color pastel, apilar pantallas con `Savescreen()/Restscreen()`
y rematar con sombras duras. 90TUI trae esa filosofía a Rust seguro —
**técnicas, no productos** (ver nota legal abajo).

```rust
use tui90::{App, MenuDef, Theme};

fn main() -> std::io::Result<()> {
    tui90::enter_screen()?;
    let mut app = App::new(Theme::clipper())?;   // pastel por defecto
    let menus = vec![MenuDef::new("Varios", &["&Respaldo", "&Finalizar"])];
    // ... menubar + popup + teclas con un puñado de llamadas ...
    app.present()?;
    tui90::leave_screen()
}
```

## Características (v0.0.1, Fases 3–6)

* **Núcleo VRAM virtual** (`core`): `Cell`/`Buffer` con diff-render sin
  flicker, `Rect` con clamp responsive, pila `savescreen`/`restscreen`
  LIFO, temas `clipper()` pastel y `turbo()` nostalgia, backend crossterm.
* **Primitivas** (`prim`): ventana plana + sombra dura, botón teal,
  hotlabels `&R`espaldo, separadores finos, barras navy. Cero box-drawing.
* **Controles de una llamada** (`widgets`): menubar, popup azul,
  lista con selección azul, confirm `¿Está conforme? [Si][No]`,
  progreso de doble barra, formulario `@...GET`, tabla browse, botonera F.
* **App shell** (`app`): eventos propios (`AppKey`/`AppEvent`), pila de
  foco por capas, layout que sobrevive de 200x60 a 1x1, F-keys declaradas
  una vez (pintan y disparan).
* **Calidad**: 85 tests, clippy sin warnings, `cargo fmt`, MSRV 1.75,
  sin `unsafe`.

## Empieza en 5 minutos

```sh
git clone https://github.com/GcorpZ/90TUI.git && cd 90TUI
git checkout 90tui-built
cargo test        # todo verde
cargo run --example clipper_shell   # ← empieza aquí (Esc sale)
```

Guía completa por SO (Windows/macOS/Linux + troubleshooting):
**[`docs/GUIA-INSTALACION.md`](docs/GUIA-INSTALACION.md)**.

Usarla en tu proyecto (aún no está en crates.io):

```toml
[dependencies]
tui90 = { git = "https://github.com/GcorpZ/90TUI.git", branch = "90tui-built" }
```

## Tour de ejemplos

| Ejemplo | Muestra | Salir |
|---|---|---|
| `blank` | shell base sin flicker | `Esc`/`q` |
| `windows` | modal gris + lista + botonera + sombras | `Esc`/`q` |
| `menu_demo` | menubar + popups navegables + hotkeys | `Esc` |
| `clipper_shell` ⭐ | app completa: menús → popup → confirm, F2, resize | `Esc` / `Finalizar > Si` |
| `ordenar_indices` | modal + lista + progreso doble apilados | cualquier tecla |

## Documentación

| Documento | Qué es |
|---|---|
| `docs/FASE0-analisis-visual-y-tecnicas.md` | peritaje píxel a píxel: escuela Clipper vs Turbo Vision |
| `docs/FASE1-plan-diseno.md` | visión, API objetivo y división en 4 partes |
| `docs/FASE2-partes-libreria.md` | guía pedagógica: cómo está armado el crate |
| `docs/GUIA-INSTALACION.md` | instalación Win/Mac/Linux + troubleshooting |

## Ramas

| Rama | Contenido |
|---|---|
| `main` | estable |
| `90tui-docs` | Fases 0–2 + licencias duales |
| `90tui-built` | código Fases 3–6 |
| `90tui-help` | guía de instalación + este README (Fases 7–8) |

## Hoja de ruta

* [ ] Publicar `tui90 0.1.0` en crates.io (fijar API mínima)
* [ ] Cursor visible y edición completa en formularios
* [ ] Mouse (click en botones y menús)
* [ ] Temas serializables (`serde`) + editor de paletas
* [ ] Más widgets: tabs, checkbox, radio, spinner, ayuda F1

## Nota legal

Proyecto independiente inspirado en *técnicas* públicas de la era DOS
(VRAM de texto, doble buffering, pantallas apilables, paletas de 16
colores), no en ningún producto concreto. Sin afiliación con terceros;
los nombres históricos citados como referencia técnica pertenecen a sus
respectivos dueños.

## Licencia

Dual **MIT OR Apache-2.0** — elige la que prefieras.
Ver [`LICENSE-MIT`](LICENSE-MIT) y [`LICENSE-APACHE`](LICENSE-APACHE).

Hecho con nostalgia y `cargo test` en verde por **GcorpZ**.
