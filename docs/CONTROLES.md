# 90TUI — Catálogo de controles y elementos

> **Esta es la lista completa de lo que la librería pone a disposición
> del programador.** Todo es `pub` desde la raíz (`use tui90::...`).
> Detalle fino de cada firma: `cargo doc --open` (rustdoc sale de los
> comentarios del código). Demos copiables en `examples/`.

Convenciones globales: `&` marca hotkey amarilla (`"&Respaldo"`),
colores siempre vía `Theme` (nunca literales), todo dibuja sobre
`Buffer`/`Screen` (apilable con `savescreen`), toda navegación es pura
(`*_key`, testeable sin terminal). Fuente recomendada con `─ │ ├ └ ─
◄ ▲ ▼ … ○ ● ☐ ☑ ■ ►` (DejaVu, Cascadia, Consolas).

## prim — primitivas de dibujo (`tui90::prim`)

| Función | Parámetros | Devuelve | Efecto |
|---|---|---|---|
| `window(buf, rect, opts, theme)` | `&mut Buffer`, `Rect`, `&WindowOpts`, `Theme` | — | bloque + título centrado + sombra + caja `[■]` opt |
| `WindowOpts::modal/form/dialog(título, theme)` | `&str`, `Theme` | `WindowOpts` | presets gris/negro/menta (`.controls`, `.shadow_style` ajustables) |
| `shadow(buf, rect, theme)` | buffer, rect, tema | — | sombra fantasma (offset 2,1) |
| `shadow_solid / shadow_stipple / shadow_styled / shadow_offset` | + `dx,dy` / `ShadowStyle` | — | variantes de sombra |
| `button(buf, x, y, label, theme)` | coords, texto | `u16` ancho | botón teal + sombra solo-abajo |
| `button_draw(..., pressed)` | + `bool` | `u16` ancho | hundido (+1,+1, sin sombra) si `pressed` |
| `button_width(label)` | `&str` | `u16` | ancho para layout |
| `draw_hot_label(buf, x, y, s, base, hot)` | texto con `&` | `u16` ancho | etiqueta con hotkey |
| `draw_text / fit_text / visible_len / parse_hotkey / hot_key_of / base_on` | — | — | utilidades de texto y `…` |
| `hsep(buf, y, x0, x1, fg, bg)` | coords, colores | — | separador fino `─` |
| `top_bar / status_bar(buf, izq, der, theme)` | textos | — | barras navy |
| `vscrollbar(buf, col, total, top, visible, theme)` | `Rect` col, números | — | `▲ ░ █ ▼` proporcional |
| `folder(buf, x, y, open, attr, accent)` | `bool` | `u16`=2 | `►■`/`▼■` |
| `drive(buf, x, y, letra, attr, accent)` | `char` | `u16`=4 | `[C:]` |
| `win_close / win_min(buf, x, y, ...)` | — | `u16`=3 | `[■]` / `[-]` de título |

## widgets — controles de una llamada (`tui90::widgets`)

| Función | Parámetros clave | Navegación (`*_key`) |
|---|---|---|
| `menubar_draw(buf, bar, menus, active, theme)` | `&[MenuDef]`, activo | `menubar_key` → `Move/Open/Dismiss`; `←→` envuelve, letra salta |
| `MenuDef::new(título, &[items])` / `MenuItem::new` | `&str` (+`&` hotkey) | — |
| `popup_layout(x, y, items, screen)` | ancla + pantalla | `Rect` clampado |
| `popup_draw(buf, rect, items, selected, theme)` | `&[PopupItem]` | azul, hotkeys, `---`→línea, selección gris |
| `PopupItem::{item, group, disabled}` | `&str` | `popup_key` → `Move/Choose/Dismiss`; salta seps |
| `list_draw(buf, rect, items, sel, centered, theme)` | `&[String]` | `list_key(sel, len, code, page)`; selección azul |
| `confirm_layout(pregunta, botones, screen)` | — | `(Rect, Vec<Rect>)` botones |
| `confirm_draw(buf, screen, título, pregunta, botones, sel, theme)` | `&[String]` | `confirm_key` → `Move/Accept/Cancel`; letra acepta |
| `progress_draw(buf, screen, info, theme)` | `ProgressInfo{título, espacio, tiempo, bars}` | `BarInfo::new(etiqueta, archivo, pct, cur, total)`; `bar_fill_width` |
| `form_draw(buf, rect, título, state, theme)` | `FormState{fields, focus}` | `form_key` → `Moved/Accept/Cancel`; `Field::{text, number, yesno}` |
| `table_draw(buf, rect, def, state, footer, theme)` | `TableDef{headers, rows}` | `table_key(state, n, visible, code)`; `TableState{row, top}` |
| `checkbox_draw(buf, x, y, item, focused, style)` | `CheckItem{label, checked}` | `check_key` → `Move/Toggled`; Espacio/letra alterna |
| `radio_draw(buf, x, y, label, sel, focus, style)` | — | `radio_key` → `Select`; flechas eligen directo |
| `CheckStyle::new(HotAttrs{base, hot}, GlyphSet::{modern, ascii})` | colores + glifos | `○ ●` / `☐ ☑` o ASCII |
| `fkey_bar(buf, y, keys, theme)` | `&[(&str,&str)]` | 1 fila clásica |
| `fkey_bar_styled(buf, y, keys, FKeyStyle)` | `FKeyStyle{key_fg/bg, label_fg/bg}` | a) cantidad b) etiquetas d/e) colores |
| `fkey_bar_stacked(buf, y, keys, style)` | 2 filas | F sobre el número, alineados |
| `popup_size(items)` | — | `(w, h)` para layout manual |

## app — shell (`tui90::app`)

| Elemento | Uso |
|---|---|
| `App::new(theme)` / `App::with_backend(be, w, h, theme)` | shell (real / testeable con `TestBackend`) |
| `app.paint_shell(&ShellContent{...})` | barras + menubar + F-bar de una vez |
| `app.present()` / `app.poll(timeout)` / `app.resize()` / `quit()` | diff, eventos, resize, salida |
| `menu_title_x(menus, i)` | anclar popups bajo el título |
| `AppEvent::{Key{key, alt, ctrl}, Resize}` + `map_key_event` (filtra `Release`) | eventos sin doble-salto |
| `Focus::{push, pop, top, is_modal}` + `Layer` | quién recibe teclas |
| `desktop(screen)` → `{top, menu, work, fkeys, status}` | layout responsive |
| `FKeyDef::new(tecla, etiqueta)` + `match_fkey(defs, key)` | c) qué hace cada función |

## Núcleo (`tui90::core`, para avanzados)

`Buffer` (`text/fill/hline/blit/snapshot/diff`), `Rect`
(`intersect/clamp_in/centered_in/popup_at`), `Screen`
(`frame/savescreen/restscreen/present_ops/resize`), `Theme::{clipper, turbo}`
+ attrs derivados, `Cell/Color/Attr`, `Backend` + `CrosstermBackend` /
`TestBackend`, `enter_screen/leave_screen`.
