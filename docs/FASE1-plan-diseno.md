# G90TUI — FASE 1: Plan de Diseño

> Rama docs: `g90tui-docs` | Objetivo: mejor alternativa a Ratatui para exigentes, con el alma de la gTUI de gestión de los 90.

## 1. Visión

Ratatui es excelente pero "se siente TUI": box-drawing por todos lados, alto contraste, hay que hacer malabares para un menú desplegable decente.
**G90TUI = gTUI artesanal de los 90, reencarnada en Rust seguro:**
llamas `menu_bar()`, `popup()`, `form()`, `browse()` y la pantalla queda como la gestión Clipper de principios de los 90: pastel, sombreada, responsive, con pila de pantallas y hotkeys amarillas automáticas.

Lema: `Una función = un control completo en pantalla.`

## 2. Objetivos medibles

* O1 Facilidad: menú con sub-opciones en <10 líneas (ejemplo §7).
* O2 Fidelidad a la técnica Clipper: `Theme::clipper()` reproduce el look de referencia con delta visual <5% (inspección contra capturas).
* O3 Responsive: 80x25 mínimo, hasta 200x60, sin solapamientos rotos. Layout por `Rect` + anclas.
* O4 Rendimiento: diff-render 60fps en terminal típica, sin flicker (doble buffer).
* O5 Portabilidad: Windows (conhost/Windows Terminal), macOS (Terminal/iTerm), Linux (xterm/kitty). Solo `crossterm` como backend.
* O6 Seguridad Rust: sin `unsafe`, sin acceso real a `0xB800`, 100% tests en lógica de buffer.

No-objetivos v1: mouse pixel-perfect DOS, sonido PC-speaker, DBF/NTX real, fuentes VGA reales.

## 3. Decisiones de arquitectura (estilo 90s, ejecución moderna)

```
┌ App (loop eventos, foco, F-keys) ─────────┐
│ Controles alto nivel (Menu, Form, Browse, │  <- UNA LLAMADA
│  Dialog, Progress, Launcher)              │
│ Primitivas estilo Clipper (Window,        │  <- relleno+sombra
│  Button, Bar, Separator, HotLabel)        │
│ Núcleo (Cell, Buffer, Rect, Theme,        │  <- VRAM virtual
│  ScreenStack, DiffRenderer, Backend)      │
└── crossterm (ANSI, teclado, resize) ──────┘
```

* Núcleo no sabe de widgets. Widgets no tocan crossterm directo, solo `Buffer`.
* `ScreenStack`: `savescreen(rect) -> id`, `restscreen(id)` — clon exacto semántica Clipper.
* `Theme` centraliza paleta pastel, no colores hardcodeados.
* `Rect` + `Anchor` + `Layout::center/popup/split` = responsive sin cálculos manuales.

## 4. División en 4 partes de construcción (FASE 3-6)

**Parte 1 — Núcleo VRAM virtual (FASE 3):**
`src/core/{cell.rs, color.rs, rect.rs, buffer.rs, theme.rs, screen.rs, backend.rs}`
+ `lib.rs` re-exports. Tests: buffer diff, rect clip, stack push/pop.

**Parte 2 — Primitivas gTUI estilo Clipper (FASE 4):**
`src/prim/{window.rs, shadow.rs, button.rs, label.rs, separator.rs, title.rs}`
Ventana plana + sombra dura, botón teal, hotlabel amarilla, separador blanco, titlebar teal/navy.

**Parte 3 — Controles de una llamada (FASE 5):**
`src/widgets/{menubar.rs, popup.rs, list.rs, form.rs, table.rs, dialog.rs, progress.rs, statusbar.rs}`
API: `popup_menu(buf, items)`, `dual_progress()`, `confirm_si_no()`, `input_field()` estilo `@...GET`.

**Parte 4 — App shell + eventos + responsive (FASE 6):**
`src/app/{app.rs, events.rs, focus.rs, layout.rs, fkeys.rs}`, `examples/clipper_shell.rs`, `examples/ordenar_indices.rs` (recrea layout de captura de referencia).
Loop: `poll → update → render_diff → present`. Resize re-layout automático.

## 5. Catálogo API v1 (firmas objetivo, pueden variar poco)

```rust
// Shell
g90tui::app::App::new(Theme::clipper())?.run(|scr| demo(scr))?;

// Una llamada = control responsive
menu_bar(scr, &["Archivos","Transacciones","Reportes","Varios"], 3)?;
popup_menu(scr, anchor, &["Estado del &sistema","Resumen &gerencial","Cierre &mensual"])?;
confirm(scr, "¿ Está conforme ?", &["Si","No"])?; // botones teal + sombra
input_form(scr, "DEPARTAMENTOS", &[Field::text("Código"), Field::text("Descripción")])?;
browse_table(scr, headers, rows)?; // estilo tabla de inventario de la época
dual_progress(scr, "ORDENAR INDICES", &Progress{file:"APROD.DAT", pct:12, cur:60, total:546})?;
status_bar(scr, "G90TUI Demo v0.0.1", "Alt-F1: Ayuda")?;
fkey_bar(scr, &[("F2","Grabar"),("Esc","Salir")])?;
```

`&` marca hotkey amarilla. Sin `&` = primera letra mayúscula automática.

## 6. Backend y compatibilidad

* `crossterm 0.28` (raw mode, alt screen, key events, resize). Windows/macOS/Linux probado en FASE 7.
* Sin `ratatui` dependency (somos alternativa, no wrapper).
* MSRV 1.75, edition 2021. `unicode-width` para anchos.
* Feature flags: `default=[]`, `serde` opt para serializar temas.

## 7. Ejemplo canónico (la promesa)

```rust
use g90tui as tui;
fn main() -> anyhow::Result<()> {
    let mut app = tui::App::new(tui::Theme::clipper())?;
    app.top_bar("EMPRESA DEMO C.A.", "Jueves 26 de Diciembre de 2013")?;
    app.status("G90TUI Demo v0.0.1", "Alt-F1: Ayuda")?;
    let menus = vec![
        ("Archivos", vec!["Proveedores","Departamentos","Depósitos"]),
        ("Varios", vec!["Estado del sistema","Respaldo de datos","Ordenar índices","Finalizar"]),
    ];
    // 1 llamada dibuja menubar + popup + sombra + hotkeys + responsive:
    app.menu_bar(&menus)?;
    app.fkey_bar(&[("Esc","Salir")])?;
    app.run()
}
```

Esto recrea el shell clásico de gestión sin malabares.

## 8. Riesgos y mitigaciones

* Unicode vs CP437: usar aproximaciones (`◄▲▼`) + tests de ancho.
* Terminales con pocos colores: fallback a 16 colores ANSI, seguir legible.
* Resize agresivo: `Rect::clamp`, nunca panic, recorta con `...`.
* Expectativa "pixel-perfect VGA": documentar que es evocación, no emulación de hardware.

## 9. Criterios de aceptación por fase

* F3: `cargo test` buffer/stack verde, example `blank` sin flicker.
* F4: example `windows` muestra sombras idénticas a captura (foto vs screenshot).
* F5: `popup_menu` + `confirm` + `dual_progress` recrean ORDENAR INDICES.
* F6: `clipper_shell` navegable solo con teclado, resize sin romper, F-keys operativas.

---
Siguiente: `FASE2-partes-libreria.md` (explicación pedagógica).
