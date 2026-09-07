# 90TUI

gTUI artesanal de los 90 al estilo de la técnica **CA-Clipper** para Rust.
Una alternativa pastel y de **una-llamada-por-control** a Ratatui.

> Estado: Fase 0–2 (documentación) en rama `90tui-docs`.
> Construcción (Fase 3–6) en rama `90tui-built`. Manuales (Fase 7–8) en `90tui-help`.

> **Origen y legal:** proyecto independiente inspirado en las *técnicas* públicas de la era DOS (VRAM de texto, doble buffering, pantallas apilables, paletas de 16 colores), no en ningún producto concreto. Sin afiliación con terceros; los nombres históricos citados solo como referencia técnica pertenecen a sus dueños.

## Documentos de diseño

* `docs/FASE0-analisis-visual-y-tecnicas.md` — escuela Clipper vs Turbo Vision
* `docs/FASE1-plan-diseno.md` — plan de diseño y división en 4 partes
* `docs/FASE2-partes-libreria.md` — guía pedagógica de las partes del crate

## Uso futuro

```toml
[dependencies]
tui90 = "0.0.1"
```

```rust
// Próximamente (Fase 3+):
// use tui90::App;
```

## Licencia

Dual: **MIT OR Apache-2.0**. Ver `LICENSE-MIT` y `LICENSE-APACHE`.
