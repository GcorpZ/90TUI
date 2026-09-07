# 90TUI — FASE 2: Partes de la librería (guía pedagógica)

> Para alguien que nunca construyó un framework TUI. De lo general a lo particular, con analogía DOS.

Piensa en la librería como un PC de los 90 en miniatura:
disco = tu código, RAM = `Buffer`, tarjeta VGA = `Backend`, DOS = `App`, programas `.EXE` = `Widgets`.

## 1. ¿Qué es una librería Rust? (crate)

* Un crate-lib es una caja de herramientas: expones funciones/structs vía `lib.rs`, otros la usan con `cargo add 90tui`.
* Estructura:
```
90TUI/
  Cargo.toml          # nombre, versión, deps (crossterm)
  src/lib.rs          # fachada: `pub mod core, prim, widgets, app;`
  src/core/*.rs       # piezas pequeñas (cada archivo = una responsabilidad)
  src/prim/*.rs       # dibujos básicos
  src/widgets/*.rs    # controles completos
  src/app/*.rs        # loop y eventos
  examples/*.rs       # demos copiables
  tests/*.rs          # pruebas
  docs/*.md           # lo que estás leyendo
```
* Regla: un archivo = una idea (ej `rect.rs` solo sabe de rectángulos). `lib.rs` re-exporta para que el usuario haga `use tui90::popup_menu;` sin perderse.

## 2. Mapa de partes (qué hace cada una y por qué existe)

### A) NÚCLEO — la VRAM virtual
| Archivo | Rol DOS | Qué contiene | Parámetros clave |
|---|---|---|---|
| `core/cell.rs` | 2 bytes en `0xB800` | `struct Cell{ch:char, fg:Color, bg:Color}` | char + colores |
| `core/color.rs` | Atributo VGA | `enum Color` + `Theme` mapping a ANSI | fg, bg, bold |
| `core/rect.rs` | Coordenadas pantalla | `struct Rect{x,y,w,h}` + `clip, center, shrink` | x,y,w,h |
| `core/buffer.rs` | RAM espejo | `Vec<Cell>` + `set, fill, text, hline, blit, diff` | — |
| `core/theme.rs` | Paleta SAINT | `Saint751{desktop_cyan, navy, teal, popup_blue, hot_yellow...}` + `Turbo` opt | preset |
| `core/screen.rs` | `Savescreen()` | `stack: Vec<Snapshot>` + `save(rect)/restore(id)` | rect |
| `core/backend.rs` | INT 10h / VRAM | `trait Backend{size, present(diff)}` + `CrosstermBackend` | — |

Sin esto no hay nada: todo dibujo escribe en `Buffer`, nunca directo a terminal (igual que doble buffering DOS).

### B) PRIMITIVAS — el pincel SAINT
| Archivo | Dibuja | Función ejemplo |
|---|---|---|
| `prim/window.rs` | Bloque plano + título + sombra dura | `window(buf, rect, title, theme)` |
| `prim/shadow.rs` | Rect negro offset | `shadow(buf, rect, dx=2, dy=1)` |
| `prim/button.rs` | Botón teal ` Esc Salir ` | `button(buf, x,y,label,focused)` |
| `prim/label.rs` | Texto con hotkey amarilla | `hot_label(buf, x,y,"&Respaldo")` |
| `prim/separator.rs` | Línea blanca separadora de grupos | `hsep(buf, y, x0,x1)` |
| `prim/title.rs` | Barras sup/inf navy + teal | `top_bar(), status_bar()` |

Diferencia vs Turbo: aquí NO se usan `╔═╗`. Son `fill_rect()` sólidos. Por eso se ve suave.

### C) WIDGETS — controles de UNA llamada (tu objetivo principal)
Cada widget = compone primitivas + maneja su propio `save/restore` + devuelve elección.

* `widgets/menubar.rs`: `menu_bar(buf, menus: &[(&str, Vec<Item>)], active:int)` — dibuja barra teal, resalta activo en negro, abre popup. Params: títulos, hotkeys auto, ancho auto-responsive (centra o pega según `Rect`).
* `widgets/popup.rs`: `popup_menu(buf, anchor:Rect, items:&[MenuItem{label, hotkey, disabled}]) -> Option<usize>` — calcula w=max_label+4, h=n+2, clampa a pantalla, pinta sombra, separadores `---` como línea blanca, selección gris.
* `widgets/list.rs`: lista centrada con selección azul (ORDENAR INDICES). Params: `items, selected, on_key(↑↓Enter)`.
* `widgets/form.rs`: emula `@...GET`: `Field{name, kind:Text|Number|Bool, value, width, help}` + `form(title, fields) -> Values`. Incluye `¿Está conforme? [Si][No]`.
* `widgets/table.rs`: `browse(headers, rows, col_widths)` estilo CARGOS INVENTARIO, con header gris, cursor gris, footer `Linea: 1/0 Total:`.
* `widgets/dialog.rs`: `confirm(), info(), error()` — modal mint o gris según severidad.
* `widgets/progress.rs`: `dual_progress(info: {space, elapsed, file, pct1, cur1/total1, pct2...})` — dos barras gris+azul como captura.
* `widgets/statusbar.rs + fkeys.rs`: `status_bar(left,right)`, `fkey_bar(&[("F2","Grabar")])`.

Todos reciben `&mut Buffer` + struct de params, devuelven `EventResult`. Nada de builders eternos.

### D) APP — el DOS residente
* `app/app.rs`: `struct App{buf, front, stack, theme, running}` + `run(callback)` loop: `poll_event → dispatch → render_diff → present`.
* `app/events.rs`: `enum Event{Key(KeyCode,modifiers), Resize(w,h), Tick}` + `Key{Enter,Esc,Up,Down,F1..F10,Alt+c}`. Mapea crossterm → nuestro enum estable.
* `app/focus.rs`: quién recibe teclas (menubar → popup → form field → tabla). Pila de foco.
* `app/layout.rs`: `Layout::desktop() -> {top, work, status}`, `Layout::centered(w,h)`, `Layout::popup(anchor)` — responsive: si terminal <80x25, encoge y recorta con `…`.
* `app.rs` también expone `savescreen/restscreen` públicos para usuarios avanzados (como Clipper).

### E) HARDWARE (abstracción moderna)
No tocamos `0xB800`. `Backend` hace:
1. `enter_alt_screen + raw_mode`,
2. `get_size()`,
3. `present(diff: &[(x,y,Cell)])` → `queue!(MoveTo, SetFg/Bg, Print)`,
4. `leave` al salir (restaura terminal, como `Restscreen` final).

## 3. Flujo de un frame (ejemplo menú Varios)

1. `App::run` → `menu_bar()` pide `Layout::top()`.
2. `window()` hace `screen.save(debajo)` → `fill_rect(popup_blue)` → `shadow(negro)`.
3. Cada item → `hot_label()` (amarilla la letra tras `&`).
4. `present(diff)` solo envía celdas cambiadas.
5. Tecla `↓` → cambia `selected`, repinta solo 2 filas (gris ↔ azul).
6. `Enter` → `screen.restore()` (desaparece sin rastro) → retorna índice.
7. Si abre otro modal (ORDENAR → PROGRESO), se apilan 2 `save()`. Al cerrar, pops en orden. Igual que tus 3 niveles en captura.

## 4. Lista de controles v1 + parámetros (contrato)

```
TopBar{company, date} | MenuBar{menus:[{title, items:[{label, hot, action}]}]}
Popup{items, selected, group_seps} | ListBox{items, selected, centered:bool}
Button{label, key:F2/Esc, focused} | FKeyBar{keys:[(F2,Grabar)]}
Form{title, fields:[{name, value, len, type, hint}], ok_cancel} 
Table{headers, rows, widths, footer_total} | DualProgress{file, pct, cur/total x2}
Status{left, right} | Confirm{question, [Si,No]} | StarMenu{apps:[...]}
```

Todos aceptan `theme: &Theme` implícito desde `App`, no hay que pasar colores a mano.

## 5. Cómo leer el código cuando lo construyamos (FASE 3-6)

* Empieza por `examples/saint_shell.rs` (10 líneas) → salta a `widgets/menubar.rs` → baja a `prim/window.rs` → termina en `core/buffer.rs`. De arriba (fácil) a abajo (VRAM).
* Tests en `tests/saint_golden.rs`: buffers esperados vs reales (snapshot), garantizan que sombra/hotkey no se rompan.

Ya tienes el mapa. FASE 3 empieza el pico y pala: núcleo.
