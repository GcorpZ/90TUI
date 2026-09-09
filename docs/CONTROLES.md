# G90TUI — Catálogo de controles y elementos

> **Esta es la lista completa de lo que la librería pone a disposición
> del programador.** Todo es `pub` desde la raíz (`use g90tui::...`).
> Detalle fino de cada firma: `cargo doc --open` (rustdoc sale de los
> comentarios del código). Demos copiables en `examples/`.

Convenciones globales: `&` marca hotkey amarilla (`"&Respaldo"`),
colores siempre vía `Theme` (nunca literales), todo dibuja sobre
`Buffer`/`Screen` (apilable con `savescreen`), toda navegación es pura
(`*_key`, testeable sin terminal). Fuente recomendada con `─ │ ├ └ ─
◄ ▲ ▼ … ○ ● ☐ ☑ ■ ►` (DejaVu, Cascadia, Consolas).
Parámetros globales de estilo (`core::WidgetStyle`, colores `Color` 16 ANSI,
no RGB): `foreground_color` / `background_color` / `border_color` /
`has_shadow` — presentes en `ButtonOpts`, `WindowOpts` (`shadow` +
`border_color`), `CheckStyle`, `FKeyStyle` (`has_shadow`), `Dropdown`,
`InputField`, `ProgressBar`, `ListBox`, `StatusBar`, `FileDialog`,
`TextArea`, `Hyperlink`, `TuiChart`, `TabControl`, `GridTable`,
`CalendarPicker` y `MsgBox` (vía `Buttons` + botones de 1 fila).

## prim — primitivas de dibujo (`g90tui::prim`)

| Función | Parámetros | Devuelve | Efecto |
|---|---|---|---|
| `window(buf, rect, opts, theme)` | `&mut Buffer`, `Rect`, `&WindowOpts`, `Theme` | — | marco `BorderStyle::{None, Single, Double, Bevel3D}` (presets con `Double`) + título centrado incrustado + sombra cromática + caja `[■]` (`\u{25a0}`) en `x+1,x+2,x+3` opt |
| `WindowOpts::modal/form/dialog(título, theme)` | `&str`, `Theme` | `WindowOpts` | presets gris/negro/menta con `Single` (`.controls`, `.shadow_style`, `.border_color` tiñe glifos, `.shadow`=`has_shadow`, `border_style` elegible) |
| `shadow(buf, rect, theme)` | buffer, rect, tema | — | sombra cromática (offset 2,1): conserva glifo, oscurece fg/bg con `darken_color` + `dim` ANSI |
| `shadow_solid / shadow_stipple / shadow_styled / shadow_offset` | + `dx,dy` / `ShadowStyle` | — | variantes (botones = sólida) |
| `Cell.dim` / `Attr::faint` | flag | — | `\x1b[2m` real en el backend |
| `button(buf, x, y, label, theme)` | coords, texto | `u16` ancho | botón teal 1 fila, padding 2 por lado (`  TEXTO  `, mín. 10); sombra media celda con fondo REAL detectado: lateral `▄` + inferior `▀`, tinta negra |
| `button_draw(..., pressed)` | + `bool` | `u16` ancho | hundido (+1,+1, sin sombra) si `pressed` |
| `button_ex / button_draw_ex / button_draw_opts` | + `ButtonOpts` | `u16` ancho | estilo global (`foreground/background/border_color`, `has_shadow`) |
| `button_width(label)` | `&str` | `u16` | texto+4 con mínimo 10 |
| `BUTTON_MIN_WIDTH` | const `= 10` | — | ancho mínimo de botón |
| `draw_hot_label(buf, x, y, s, base, hot)` | texto con `&` | `u16` ancho | etiqueta con hotkey |
| `draw_text / fit_text / visible_len / parse_hotkey / hot_key_of / base_on` | — | — | utilidades de texto y `…` |
| `hsep(buf, y, x0, x1, fg, bg)` | coords, colores | — | separador fino `─` |
| `top_bar / status_bar(buf, izq, der, theme)` | textos | — | barras navy |
| `vscrollbar(buf, col, total, top, visible, theme)` | `Rect` col, números | — | `▲ ░ █ ▼` proporcional |
| `folder(buf, x, y, open, attr, accent, glyphs)` | `FolderGlyphs::{nerd, ascii}` | `u16`=1 | U+F07B/U+F07C o ASCII |
| `drive(buf, x, y, letra, attr, accent)` | `char` | `u16`=4 | `[C:]` |
| `win_close / win_min(buf, x, y, ...)` | — | `u16`=3 | `[■]` / `[-]` de título |
| `icons20::{drive_badge, folder_badge, file_badge, fkey_badge}` (`IconMode::{RetroCp437, NerdFont}`) | `DriveType::{Floppy35, Floppy525, HardDisk, CdRom}`, activo; `bg` del llamante | `u16`=1–7 | carcasas `[≡]/[■-]/[○]` o NF `U+F0A0/F1C0`, `[+]/[-]` sin parches, `*`/`≡` + aire (nombre en x+2), `N acción` Norton sin `F` |
| `icons20::{win_close, win_zoom, win_min}` / `win_resize_grip` | `attr`, `accent` | `u16`=3 / 1 | `[■]` `[▲]` `[▼]` / `◢` |

## widgets — controles de una llamada (`g90tui::widgets`)

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
| `progress_draw(buf, screen, info, theme)` | `ProgressInfo{título, espacio, tiempo, bars}` | `BarInfo::new(etiqueta, archivo, pct, cur, total)`; fila 2 = `ProgressBar` real; `bar_fill_width` (compat) |
| `form_draw(buf, rect, título, state, theme)` | `FormState{fields, focus}` | `form_key` → `Moved/Accept/Cancel`; `Field::{text, number, yesno}` |
| `table_draw(buf, rect, def, state, footer, theme)` | `TableDef{headers, rows}` | `table_key(state, n, visible, code)`; `TableState{row, top}` |
| `checkbox_draw(buf, x, y, item, focused, style)` | `CheckItem{label, checked}` | `check_key` → `Move/Toggled`; Espacio/letra alterna |
| `radio_draw(buf, x, y, label, sel, focus, style)` | — | `radio_key` → `Select`; flechas eligen directo |
| `CheckStyle::new(attrs, GlyphSet::{modern, ascii})` | colores + glifos | integrales `☐/☑` `○/◉` o ASCII con marcos (+`border_color`, `has_shadow` fila) |
| `dropdown_draw(buf, rect, dd, theme)` | `Dropdown{options, selected_index, is_open, label, max_height, bg/fg/active_bg, border_color, has_shadow}` | cerrado: selección + `▼`; abierto: overlay + scroll + `dropdown_key` (`↑↓` circular, `Enter` acepta, letra filtra) |
| `input_draw(buf, rect, field, focused)` | `InputField{value, max_len, mask, cursor, fg/bg/active_bg, border_color, has_shadow}` | `input_key` → inserta/borra/mueve; máscara solo visual |
| `progressbar_draw(buf, rect, bar)` | `ProgressBar{pct 0-100, fg/bg, border_color, has_shadow, show_pct}` | octavos `SUB_BLOCKS` (8x) + frontera sin pisar por `NN%` + corchetes o caja simple |
| `listbox_draw(buf, rect, lb)` | `ListBox{items, selected, top, fg/bg/highlight, border_color, has_shadow}` | scrollbar `▲`/`▼` + `█` proporcional; `listbox_key` (flechas/PgUp/PgDn/letra) |
| `fkey_bar(buf, y, keys, theme)` | `&[(&str,&str)]` | 1 fila clásica |
| `fkey_bar_styled(buf, y, keys, FKeyStyle)` | `FKeyStyle{key_fg/bg, label_fg/bg, has_shadow}` | a) cantidad b) etiquetas d/e) colores |
| `fkey_bar_stacked(buf, y, keys, style)` | 2 filas | F sobre el número, alineados |
| `fkey_bar_compact(buf, y, keys, style)` | 1 fila | `F¹Help F²Qview F¹⁰Menu` (F + superíndice) |
| `statusbar_draw(buf, y, bar)` | `StatusBar` + `add_column(start_col, max_len, Alignment)` | penúltima fila (h-2) por secciones; `set_text(i)` dinámico; última fila (h-1) solo F-keys; prohibido duplicar barras |
| `win_close(buf, x, y, ...)` | — | `u16`=3 | `[■]` (`\u{25a0}`) parte del borde superior, no flotante |
| `filedialog_draw(buf, screen, dlg, theme)` | `FileDialog::new(&ruta)` (`std::fs`, árbol + `ListBox`) | `filedialog_key` → `Accepted(PathBuf)` / `Cancelled`; Enter entra/elige, Bksp sube |
| `textarea_draw(buf, rect, area, focused)` | `TextArea{lines, cursor, max_chars, fg/bg/active_bg, border_color, has_shadow}` | `textarea_key` → Enter parte línea, flechas/PgUp/PgDn navegan, scroll vertical |
| `hyperlink_draw(buf, x, y, link)` | `Hyperlink{text, url}` | texto negrita coloreado; `osc8_sequence()` = `\x1b]8;;URL\x1b\\TEXTO\x1b]8;;\x1b\\` clickeable |
| `tuichart_draw(buf, rect, chart)` | `TuiChart{kind: Bars3D/Line/Pie, series: [(label, value)]}` | barras `█`+`▒` 3D · líneas braille `⠃⠇⠽` · tarta sectorial + leyenda `%` |
| `tab_draw(buf, rect, tabs)` | `TabControl{tabs:[{label, fg, bg}], active, position: Top/Left}` | `tab_viewport(rect)` = área útil de página; `tab_key` (`←→`/`↑↓` circular); `active_scope()` nombra el ámbito de foco |
| `grid_draw(buf, rect, grid)` | `GridTable{headers, rows, widths (0=auto)}` | anchos auto/fijos, fila resaltada, scroll vertical; `handle_event(EventCtx, key)` nativo (↑↓/PgUp/PgDn) |
| `FocusManager::{add_scope, set_active, cycle_next, is_focusable}` | ámbitos por página (nombres = etiquetas de tab) | `Tab` global solo circula el ámbito activo; páginas ocultas fuera del teclado |
| `msgbox_draw(buf, screen, título, msg, buttons, sel, theme)` | `Buttons::{Ok, OkCancel, YesNo}` | modal centrado + wrap + botones 1 fila con `▀`; `msgbox_key` → `Accept(i)`/`Cancel` |
| `calendar_draw(buf, rect, cal)` | `CalendarPicker::new/current()` (matemática civil propia) + `is_open` + `datemask` opt (`YYYY/YY/MM/M/DD/D`, std `DD/MM/YYYY`) + `today_fg` | campo 1 fila ` [ fecha ] [ 󰃭 ]` (`\u{f073}`, ancho `calendar_field_width(&cal)`); abierto: popup 23×10 (`calendar_popup_rect`, debajo o encima) con caja simple `┌─┐│└┘`, cabecera `«◄ MES AAAA ►»`, semana `Lu..Do`, cursor Clipper + **hoy en negrita roja**; `calendar_key` (`Enter`/Espacio abre) y `calendar_key_mod` (flechas días, `PgUp/PgDn` mes, `Shift/Ctrl`+`PgUp/PgDn` año, `Enter` acepta, `Esc` restaura) |
| `popup_size(items)` | — | `(w, h)` para layout manual |

## app — shell (`g90tui::app`)

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

## Núcleo (`g90tui::core`, para avanzados)

`Buffer` (`text/fill/hline/blit/snapshot/diff`), `Rect`
(`intersect/clamp_in/centered_in/popup_at`), `Screen`
(`frame/savescreen/restscreen/present_ops/resize`), `Theme::{clipper, turbo}`
+ attrs derivados, `Cell/Color/Attr`, `Backend` + `CrosstermBackend` /
`TestBackend`, `enter_screen/leave_screen`.
