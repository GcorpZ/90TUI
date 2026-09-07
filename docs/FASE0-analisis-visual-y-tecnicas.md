# 90TUI — FASE 0: Análisis visual y técnicas (escuela Clipper vs Turbo Vision)

> Rama: `90tui-docs` | Fecha: 2026-09-07 | Rol: Ingeniero de Sistemas / Software graduado 90s
> Fuentes visuales: set de capturas de apps de gestión DOS de principios de los 90 (`90tui_ref/`) + set Norton/PC Tools (`90tui_ref/NDD/`)

> **Nota legal y de origen:** 90TUI es un proyecto independiente que nace de recordar la *tecnología* y las *técnicas* de la época —acceso directo a VRAM, doble buffering, pila `Savescreen()/Restscreen()`, paletas de 16 colores, fuentes VGA programables— popularizadas por el ecosistema **CA-Clipper a principios de los 90** (el compilador traía pantallas apilables y menús de texto; librerías de terceros en C/ASM añadieron ventanas, sombras duras y glifos propios). No existe afiliación ni relación con ningún producto comercial de la época; los nombres de terceros citados como referencia histórica pertenecen a sus respectivos dueños. En este documento los datos concretos de las capturas (razones sociales, seriales, versiones) se describen en genérico a propósito.

Pude VER las imágenes. No hizo falta alternativa. Lo que sigue es peritaje directo píxel a píxel.

## 1. Veredicto en una línea

* **NDD / PC Tools = Turbo Vision puro:** alto contraste, marcos con box-drawing (`╔═╗║╚`), escritorio gris con patrón, árbol con `├─└─`, listas azul eléctrico con texto blanco.
* **Gestión Clipper = escuela CA-Clipper 5.x + libs C/ASM a medida:** sin box-drawing, bloques sólidos de color pastel, sombra dura negra offset, hotkeys amarillas, tipografía gruesa custom. Se siente "artesanal / refrescante" porque evita el blanco-sobre-azul quemante.

Tu intuición es correcta: mismo hardware (VGA texto 80x25, VRAM `0xB800:0000`), distinta filosofía de dibujo.

## 2. Inventario observado (set de gestión, técnica Clipper)

### 2.1 Shell base (captura: shell principal)
* Borde exterior cian brillante sólido (no hay borde dibujado, es relleno).
* Área de trabajo blanca sólida interior.
* Barra superior azul oscuro `#0000AA` aprox: razón social de ejemplo + fecha del sistema, en blanco.
* Segunda barra teal oscuro `Archivos Transacciones Reportes Varios`. Activo = fondo negro + texto blanco.
* Icono `[-]` gris arriba-izquierda (botón ventana, no es ASCII, es glifo redefinido).
* Barra inferior azul oscuro: nombre + versión demo + serial de ejemplo + `Alt-F1: Ayuda`.
* Tarjeta gris abajo-izq con el nombre del módulo + botón teal `Esc Salir`, ambos con sombra negra sólida desplazada +1 fila / +2 cols.

### 2.2 Menú desplegable (capturas: menú principal abierto)
* Popup azul medio sólido, SIN marco. Separadores = línea blanca fina de 1px entre grupos:
  `Estado del sistema / Resumen gerencial / Cierre mensual / Consulta... | Respaldo / Recuperar / Ordenar | Cambio fecha | Finalizar`.
* Hotkey = primera letra en amarillo brillante, resto blanco. Ej: `**R**espaldo`, `**F**inalizar`.
* Selección = barra gris claro con texto negro (NO invertido azul). En captura: `Ordenar índices`, `Recuperar datos`.
* Sombra = rectángulo negro sólido detrás, offset (dx=+2, dy=+1). Sin difuminado, sin `▒▓`. Es `SaveScreen()` + blit + relleno negro. Rapidísimo en 386.

### 2.3 Ventana modal gris (captura: diálogo de reorganización de índices)
* Ventana gris medio centrada, título teal centrado, sin bordes redondeados.
* Lista centrada, selección = barra azul oscuro + texto blanco.
* Botonera inferior teal: `◄ Ordenar | F2 Todos | ↑ | ↓`. Cada botón = bloque teal + texto blanco + sombra negra.
* Lo de atrás queda congelado (pila de pantallas). Clásico `Savescreen()/Restscreen()` de Clipper.

### 2.4 Progreso doble (captura: proceso con barras de avance)
* Dialog verde menta/pistacho sólido. Dos barras: track gris + fill azul oscuro, texto `%` y contadores `60/546`, `115/1170` sobre un `.DAT` de ejemplo.
* Cabecera con espacio en disco + tiempo en uso. Esto es polling de disco + timer DOS.
* Encomillado: apilamiento de 3 niveles (base + diálogo + progreso), cada uno con su sombra. Prueba de que el stack de pantallas funciona.

### 2.4b Lanzador de programas (captura: menú inicial)
* Escritorio teal plano. Popups gris claro con texto negro, selección cian. Botonera F inferior gris con relieve 3D mínimo (`F2 Mantenimiento | F3 Teléfonos | F4 Libro`). Reloj en barra inferior.

### 2.5 Formularios negros (capturas: captura de datos e inventario)
* Modo "captura": fondo negro, labels gris claro, valores blanco brillante, campo activo = fondo gris.
* Cabecera tabla = fondo gris + texto negro: `Referencia | Descripción | Cantidad | UND | Costo | Total Costo`.
* Footer btns: `F1 Información | F3 Depósito | Esc Totalizar | F2 Grabar | Si/No/↑`.
* Diálogo `¿ Está conforme ? [Si][No][↑]` — botones mini teal. Minimalismo total.

## 3. Inventario NDD / PC Tools observado

* Set NDD, pantalla inicial (Norton Desktop): desktop gris sólido, ventana con título blanco + menú `File Disk View...`, lista archivos azul eléctrico, popup ayuda cian con botones `Yes/No` con sombra. Scrollbars laterales con flechas.
* Set NDD, administrador de archivos (PC Tools 9): doble panel, árbol izq con glifos carpeta + líneas `├`, tabla der con iconitos + tamaños + fechas. Barra F inferior `F1Help F2Quiew...F9Select F10Menu`.
* Set NDD, ayuda y antivirus: verde Norton en ayuda, marcos simples blancos, selección negra, botones grises con sombra `▒`.
* Patrón común: **todo enmarcado** con `─│┌┐└┘═║╔╗`, alto contraste blanco/amarillo sobre azul/verde, gris moteado de fondo.

## 4. Tabla comparativa técnica

| Aspecto | Turbo Vision (NDD/PCT) | Gestión Clipper artesanal |
|---|---|---|
| Marco ventana | Box-drawing doble/simple | Sin marco, bloque sólido + sombra dura |
| Fondo escritorio | Gris con patrón `░` | Cian/teal/blanco plano pastel |
| Popup | Blanco/cian con borde | Azul medio sólido sin borde |
| Selección | Negro sobre verde / blanco sobre azul | Gris claro + texto negro (menús), azul+blanco (listas) |
| Hotkey | Rojo/blanco | Amarillo sobre azul/negro, sistemático |
| Botón | `[ Go To ]` gris con sombra `▓` | Bloque teal ` Esc Salir ` + sombra negra sólida |
| Sombra | Caracteres `▒▓` / atributo gris | Rect negro sólido dx=2 dy=1 |
| Fuente | CP437 stock 8x16 | VGA RAM redefinida: flechas, `Esc`, botones redondeados, logo |
| Sensación | Técnica, alto contraste, cansa | Pastel, aireada, bajo contraste, "refrescante" |
| Stack pantallas | `TView` / `TDialog` objetos | `Savescreen()/Restscreen()` procedural + enlazador modo protegido / XMS |

## 5. Cómo lo hacían en hardware (y cómo lo emulamos en Rust moderno)

Original DOS (ambos):
1. Modo texto 80x25, 2 bytes/celda en `0xB8000` (char + attr).
2. Doble buffering en RAM convencional: dibujar todo en buffer, `memcpy` de una vez (evita flicker y es más rápido que INT 10h).
3. Sombras: leer celda, conservar char, forzar attr `0x00/0x08`.
4. Fuentes: cargar 512 glifos en RAM VGA para iconos/mouse/flechas.
5. Memoria: enlazadores en modo protegido, overlays + XMS para tablas masivas.

Emulación 90TUI (terminal moderna, sin acceso a VRAM):
1. `BackBuffer = Vec<Cell{ch, fg, bg, bold}>` 80x25 escalable a cualquier tamaño (responsive).
2. `Savescreen = push(buffer.clone_rect)` / `Restscreen = pop()` — pila, igual que Clipper.
3. Render por diff: solo celdas cambiadas → `crossterm::queue!` + ANSI. Tan instantáneo como el `memcpy` a `0xB800`.
4. Sombras: `fill_rect(x+2,y+1,black)` sin caracteres de sombreado, igual que la escuela Clipper.
5. Fuentes/iconos: no podemos tocar VGA, usamos Unicode aprox (`◄ ▲ ▼ ● Esc`) + tema de color que simula la fuente gruesa. Opcional: `font` feature con dibujo de logo en celdas.
6. Paleta pastel de referencia como `Theme::clipper()`: cian escritorio, blanco trabajo, navy barras, teal menús/botones, azul popup, amarillo hotkeys, gris selección, negro sombra, menta diálogos, negro formularios.

## 6. Reglas de oro extraídas para la librería

1. **Prohibido alto contraste por defecto.** El tema pastel estilo Clipper es el default, no un skin.
2. **Ventana = relleno + sombra dura.** Nada de `╔═╗` salvo `Theme::turbo` opt-in para nostalgia.
3. **Un control = una llamada.** `menu_bar(&[..])`, `popup_menu()`, `form()`, `browse_table()`, `dual_progress()` dibujan responsive solos.
4. **Todo apilable.** Cualquier diálogo guarda/restaura. Anidamiento infinito (probado: 3 niveles en captura).
5. **Hotkeys amarillas automáticas.** Derivar de `&Letra` o primera mayúscula, pintar amarillo, `Alt+letra` dispara.
6. **Botonera F siempre visible.** `F1..F10` + `Esc` como en la época (PC Tools y gestión Clipper), no esconder acciones.

---
Siguiente: `FASE1-plan-diseno.md`.
