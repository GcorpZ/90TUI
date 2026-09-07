# 90TUI — FASE 0: Análisis visual y técnicas (SAINT vs Turbo Vision)

> Rama: `90tui-docs` | Fecha: 2026-09-07 | Rol: Ingeniero de Sistemas / Software graduado 90s
> Fuentes visuales: `90tui_ref/*.png|jpg` (SAINT 7.51) + `90tui_ref/NDD/*.png` (Norton Desktop / PC Tools 9)

Pude VER las imágenes. No hizo falta alternativa. Lo que sigue es peritaje directo píxel a píxel.

## 1. Veredicto en una línea

* **NDD / PC Tools = Turbo Vision puro:** alto contraste, marcos con box-drawing (`╔═╗║╚`), escritorio gris con patrón, árbol con `├─└─`, listas azul eléctrico con texto blanco.
* **SAINT 7.51 = escuela CA-Clipper 5.x + libs C/ASM a medida (Funky/Tools III style):** sin box-drawing, bloques sólidos de color pastel, sombra dura negra offset, hotkeys amarillas, tipografía gruesa custom. Se siente "artesanal / refrescante" porque evita el blanco-sobre-azul quemante.

Tu intuición es correcta: mismo hardware (VGA texto 80x25, VRAM `0xB800:0000`), distinta filosofía de dibujo.

## 2. Inventario SAINT observado

### 2.1 Shell base (SAIN_Admin2.png)
* Borde exterior cian brillante sólido (no hay borde dibujado, es relleno).
* Área de trabajo blanca sólida interior.
* Barra superior azul oscuro `#0000AA` aprox: `FERREAGRO SION ... Jueves 26 de Diciembre de 2013` en blanco.
* Segunda barra teal oscuro `Archivos Transacciones Reportes Varios`. Activo = fondo negro + texto blanco (`Varios` / `Reportes`).
* Icono `[-]` gris arriba-izquierda (botón ventana, no es ASCII, es glifo redefinido).
* Barra inferior azul oscuro: `SAINT Versión 7.51 Serial: ... Alt-F1: Ayuda`.
* Logo gris abajo-izq `SAINT Módulo Administrativo` + botón teal `Esc Salir`, ambos con sombra negra sólida desplazada +1 fila / +2 cols.

### 2.2 Menú desplegable (SAINT_Admin1.jpg / SAIN_Admin3.png)
* Popup azul medio sólido, SIN marco. Separadores = línea blanca fina de 1px entre grupos:
  `Estado del sistema / Resumen gerencial / Cierre mensual / Consulta... | Respaldo / Recuperar / Ordenar | Cambio fecha | Finalizar`.
* Hotkey = primera letra en amarillo brillante, resto blanco. Ej: `**R**espaldo`, `**F**inalizar`.
* Selección = barra gris claro con texto negro (NO invertido azul). En captura: `Ordenar índices`, `Recuperar datos`.
* Sombra = rectángulo negro sólido detrás, offset (dx=+2, dy=+1). Sin difuminado, sin `▒▓`. Es `SaveScreen()` + blit + relleno negro. Rapidísimo en 386.

### 2.3 Ventana modal gris (SAIN_Admin4.png — ORDENAR INDICES)
* Ventana gris medio centrada, título teal `ORDENAR INDICES` centrado, sin bordes redondeados.
* Lista centrada, selección = barra azul oscuro + texto blanco (`Proveedores`).
* Botonera inferior teal: `◄ Ordenar | F2 Todos | ↑ | ↓`. Cada botón = bloque teal + texto blanco + sombra negra.
* Lo de atrás queda congelado (pila de pantallas). Clásico `Savescreen()/Restscreen()` de Clipper.

### 2.4 Progreso doble (SAIN_Admin5.png)
* Dialog verde menta/pistacho sólido. Dos barras: track gris + fill azul oscuro, texto `%` y `60/546`, `115/1170`, `APROD.DAT`.
* Cabecera `Espacio en C: 999848KB Tiempo en uso: 00:00:00`. Esto es polling de disco + timer DOS.
* Encomillado: apilamiento de 3 niveles (base + ORDENAR + PROGRESO), cada uno con su sombra. Prueba de que el stack de pantallas funciona.

### 2.4b StarMenú III launcher (SAIN_Admin6.png)
* Escritorio teal plano. Popups gris claro con texto negro, selección cian. Botonera F inferior gris con relieve 3D mínimo (`F2 Mantenimiento | F3 Teléfonos | F4 Libro`). Reloj `Hora: 3:34:36 Pm`.

### 2.5 Formularios negros (SAIN_Admin7-11.png)
* Modo "captura": fondo negro, labels gris claro, valores blanco brillante, campo activo = fondo gris.
* Cabecera tabla = fondo gris + texto negro: `Referencia | Descripción | Cantidad | UND | Costo | Total Costo`.
* Footer btns: `F1 Información | F3 Depósito | Esc Totalizar | F2 Grabar | Si/No/↑`.
* Diálogo `¿ Está conforme ? [Si][No][↑]` — botones mini teal. Minimalismo total.

## 3. Inventario NDD / PC Tools observado

* `01_ND_Start.png` (Norton Desktop): desktop gris sólido, ventana con título blanco + menú `File Disk View...`, lista archivos azul eléctrico (`CTMOUSE SUB-DIR...`), popup ayuda cian con botones `Yes/No` con sombra. Scrollbars laterales con flechas.
* `pctool_Delux_9_GTUI.png` (PC Tools 9): doble panel `ID=DOS` + `C:\PCTOOLS9\*.*`, árbol izq con glifos carpeta + líneas `├`, tabla der con iconitos + tamaños + fechas. Barra F inferior `F1Help F2Quiew...F9Select F10Menu`.
* `optionsmenuNDD.png` / `12_Norton_Antivirus.png`: verde Norton ayuda, marcos simples blancos, selección negra, botones grises con sombra `▒`.
* Patrón común: **todo enmarcado** con `─│┌┐└┘═║╔╗`, alto contraste blanco/amarillo sobre azul/verde, gris moteado de fondo.

## 4. Tabla comparativa técnica

| Aspecto | Turbo Vision (NDD/PCT) | SAINT / Clipper artesanal |
|---|---|---|
| Marco ventana | Box-drawing doble/simple | Sin marco, bloque sólido + sombra dura |
| Fondo escritorio | Gris con patrón `░` | Cian/teal/blanco plano pastel |
| Popup | Blanco/cian con borde | Azul medio sólido sin borde |
| Selección | Negro sobre verde / blanco sobre azul | Gris claro + texto negro (menús), azul+blanco (listas) |
| Hotkey | Rojo/blanco | Amarillo sobre azul/negro, sistemático |
| Botón | `[ Go To ]` gris con sombra `▓` | Bloque teal ` Esc Salir ` + sombra negra sólida |
| Sombra | Caracteres `▒▓` / atributo gris | Rect negro sólido dx=2 dy=1 |
| Fuente | CP437 stock 8x16 | VGA RAM redefinida: flechas, `Esc`, `Si/No` redondeados, logo |
| Sensación | Técnica, alto contraste, cansa | Pastel, aireada, bajo contraste, "refrescante" |
| Stack pantallas | `TView` / `TDialog` objetos | `Savescreen()/Restscreen()` procedural + `Blinker` modo protegido / XMS |

## 5. Cómo lo hacían en hardware (y cómo lo emulamos en Rust moderno)

Original DOS (ambos):
1. Modo texto 80x25, 2 bytes/celda en `0xB8000` (char + attr).
2. Doble buffering en RAM convencional: dibujar todo en buffer, `memcpy` de una vez (evita flicker y es más rápido que INT 10h).
3. Sombras: leer celda, conservar char, forzar attr `0x00/0x08`.
4. Fuentes: cargar 512 glifos en RAM VGA para iconos/mouse/flechas.
5. Memoria: enlazadores `Blinker/RTLink`, overlays + XMS para DBF masivos.

Emulación 90TUI (terminal moderna, sin acceso a VRAM):
1. `BackBuffer = Vec<Cell{ch, fg, bg, bold}>` 80x25 escalable a cualquier tamaño (responsive).
2. `Savescreen = push(buffer.clone_rect)` / `Restscreen = pop()` — pila, igual que Clipper.
3. Render por diff: solo celdas cambiadas → `crossterm::queue!` + ANSI. Tan instantáneo como el `memcpy` a `0xB800`.
4. Sombras: `fill_rect(x+2,y+1,black)` sin caracteres de sombreado, igual que SAINT.
5. Fuentes/iconos: no podemos tocar VGA, usamos Unicode aprox (`◄ ▲ ▼ ● Esc`) + tema de color que simula la fuente gruesa. Opcional: `font` feature con `figlet`-like para logo.
6. Paleta SAINT exacta como `Theme::Saint751`: `desktop_cyan, work_white, navy_top, teal_menu, popup_blue, hot_yellow, sel_grey, btn_teal, shadow_black, mint_dialog, form_black`.

## 6. Reglas de oro extraídas para la librería

1. **Prohibido alto contraste por defecto.** Tema SAINT pastel es el default, no un skin.
2. **Ventana = relleno + sombra dura.** Nada de `╔═╗` salvo `Theme::Turbo` opt-in para nostalgia.
3. **Un control = una llamada.** `menu_bar(&[..])`, `popup_menu()`, `form()`, `browse_table()`, `dual_progress()` dibujan responsive solos.
4. **Todo apilable.** Cualquier diálogo guarda/restaura. Anidamiento infinito (probado: 3 niveles en captura).
5. **Hotkeys amarillas automáticas.** Derivar de `&Letra` o primera mayúscula, pintar amarillo, `Alt+letra` dispara.
6. **Botonera F siempre visible.** `F1..F10` + `Esc` como en PCTOOLS/SAINT, no esconder acciones.

---
Siguiente: `FASE1-plan-diseno.md`.
