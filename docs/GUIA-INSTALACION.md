# 90TUI — Guía de instalación y primera corrida (Windows / macOS / Linux)

> Rama: `90tui-help` | Vale para v0.0.1 en adelante.
> Tiempo estimado: 10–15 min con internet. Solo necesitas un compilador
> de C enlazador (lo trae tu SO o las Build Tools) + Rust vía `rustup`.

## 0. Requisitos mínimos

| Qué | Mínimo | Recomendado |
|---|---|---|
| Rust | 1.75 (MSRV del crate) | stable actual (probado con **1.97.0**) |
| Terminal | cualquiera con ANSI + UTF-8 | ver §5 por SO |
| Tamaño | 80x25 celdas | 100x30 o más |
| Colores | 16 ANSI | 256 colores (el tema usa 16) |
| Disco/red | ~500 MB para toolchain + índice crates.io | — |

Verificado por el equipo: **Windows 10/11 + Rust 1.97.0** (`cargo test`: 68 lib + 5 core + 5 app + 3 prim + 4 widgets, todo verde).

## 1. Instalar Rust

### Windows 10/11 (PowerShell)

```powershell
# 1) rustup (elige el .exe de tu arquitectura si no usas winget)
winget install -e --id Rustlang.Rustup

# 2) Toolchain MSVC (compilador C que pide crossterm en Windows).
#    Opción A (recomendada): Visual Studio Build Tools con "Desktop C++".
winget install -e --id Microsoft.VisualStudio.2022.BuildTools
#    Opción B: solo lo justo vía rustup + winapi ya incluido en deps
#    (si `cargo build` pide link.exe, instala la opción A).

# 3) Cierra y reabre la terminal, luego:
rustup default stable-x86_64-pc-windows-msvc
rustc --version   # debe decir 1.75 o más
cargo --version
```

> Usa **Windows Terminal** (Store) o al menos `conhost` actualizado.
> CMD y PowerShell sirven; Git Bash también. Evita consolas sin UTF-8.

### macOS (Apple Silicon o Intel)

```sh
# 1) Herramientas de compilación de Apple (una sola vez)
xcode-select --install

# 2) Rust vía rustup (recomendado frente a brew para controlar versiones)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Alternativa con Homebrew (vale, pero rustup manda en versiones):
# brew install rustup && rustup-init

rustc --version
cargo --version
```

> Terminal: **Terminal.app** sirve, **iTerm2 / WezTerm / Ghostty** mejor
> (paleta de 16 colores configurable, ver §5).

### Linux (Debian/Ubuntu · Fedora · Arch)

```sh
# Debian/Ubuntu
sudo apt update && sudo apt install -y build-essential curl pkg-config git

# Fedora
sudo dnf install -y gcc curl pkgconf-pkg-config git

# Arch
sudo pacman -S --needed base-devel curl git

# Rust vía rustup en todos (evita el paquete del distro, suele ir viejo)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

rustc --version
cargo --version
```

> Terminal: GNOME Terminal, Konsole, **kitty / Alacritty / WezTerm** van
> perfecto. `xterm` clásico también (fija `TERM=xterm-256color`).

## 2. Clonar 90TUI y elegir rama

```sh
git clone https://github.com/GcorpZ/90TUI.git
cd 90TUI
git branch -a   # verás main + 90tui-docs / 90tui-built / 90tui-help
```

| Rama | Contenido | Tú quieres… |
|---|---|---|
| `main` | estable publicado | usar la lib |
| `90tui-docs` | Fases 0–2 + licencias | leer el diseño |
| `90tui-built` | **código Fases 3–6** | compilar y probar |
| `90tui-help` | esta guía + README (Fases 7–8) | instalar sin pelear |

```sh
git checkout 90tui-built   # para compilar la librería
```

## 3. Compilar y probar (igual en los 3 SO)

```sh
cargo build            # compila la lib (debug)
cargo test             # 85 pruebas: lib + core/app/prim/widgets
cargo clippy --all-targets   # cero warnings (política del proyecto)
cargo fmt --check      # formato canónico
```

Salida esperada de `cargo test` (resumen):

```
test result: ok. 68 passed  (lib: core+prim+widgets+app)
test result: ok. 5 passed   (tests/core.rs)
test result: ok. 5 passed   (tests/app.rs)
test result: ok. 3 passed   (tests/prim.rs)
test result: ok. 4 passed   (tests/widgets.rs)
```

Si algo falla, ve directo a §6 (troubleshooting).

## 4. Correr los ejemplos (tu primera gTUI)

> Los ejemplos toman la terminal en **pantalla alternativa + modo raw**
> y la devuelven intacta al salir. Si algo se congela: `Esc` o `q`.
> En el peor caso cierra la pestaña: la terminal no queda dañada.

| Comando | Qué verás | Salir |
|---|---|---|
| `cargo run --example blank` | shell base: cian + blanco + barras navy | `Esc` / `q` |
| `cargo run --example windows` | modal gris + lista + botonera teal + sombras | `Esc` / `q` |
| `cargo run --example menu_demo` | menubar + popups navegables + hotkeys | `Esc` (cerrar) luego `Esc` (salir) |
| `cargo run --example clipper_shell` | shell completo: menús → popup → confirm, F2, resize | `Esc` por capas / `Finalizar > Si` |
| `cargo run --example ordenar_indices` | modal + lista + progreso doble apilados | cualquier tecla (`Esc`/`Enter`/`q`) |

Prueba el resize: agranda/achica la ventana, todo se re-acomoda sin
romperse (layout con clamp, probado hasta 1x1).

## 5. Poner bonita la terminal (importa de verdad)

90TUI dibuja con Unicode y 16 colores ANSI. Si ves `?` o cajas:

1. **Fuente** con cobertura de `─ │ ◄ ▲ ▼ … ●`:
   * Windows: *Cascadia Code / Cascadia Mono*, Consolas.
   * macOS: *Menlo*, *SF Mono*, Hack Nerd Font.
   * Linux: *DejaVu Sans Mono*, *JetBrains Mono*, Hack.
2. **UTF-8**: Windows Terminal ya es UTF-8; en `conhost`/CMD ejecuta
   `chcp 65001` si los bordes salen raros. En Linux/macOS verifica
   `locale` con `*.UTF-8`.
3. **16 colores**: el tema `Theme::clipper()` usa la paleta clásica
   (negro, azul, cian, gris, amarillo…). Si tu terminal tiene un
   esquema "neón", el cian puede verse chillón: elige un esquema
   sobrio tipo *Solarized / OneHalf / Tango*.
4. **Tamaño**: mínimo 80x25; los diálogos se truncan con `…` si no caben.

## 6. Troubleshooting

### Windows

| Síntoma | Causa probable | Fix |
|---|---|---|
| `link.exe not found` al compilar | faltan Build Tools MSVC | instala VS Build Tools (opción A, §1) y reabre terminal |
| `error: no override and no default toolchain` | rustup sin default | `rustup default stable-x86_64-pc-windows-msvc` |
| Glifos `─◄…` como `?` | fuente/CMD sin UTF-8 | Windows Terminal + Cascadia; o `chcp 65001` |
| Colores "quemados" | esquema chillón | esquema Tango/Solarized en Windows Terminal |
| El antivirus retiene el `.exe` de ejemplo | heurística con binarios nuevos | permitir carpeta del proyecto / compilar con `--release` y reintentar |
| `cargo` no se encuentra tras instalar | PATH sin `%USERPROFILE%\.cargo\bin` | reabre terminal o agrégalo a mano |

### macOS

| Síntoma | Causa probable | Fix |
|---|---|---|
| `xcrun: command line tools missing` | sin CLT | `xcode-select --install` |
| Teclas F ocupadas por brillo/volumen | función multimedia | en iTerm2/WezTerm mapea Fn, o usa `fn+F2` |
| `Alt+letra` no llega (hotkeys) | Terminal.app come Option | iTerm2: *Prefs → Profiles → Keys → Left Option = Esc+*; WezTerm: `send_composed_key_when_alt_is_pressed` |
| Colores distintos a la guía | perfil con paleta custom | perfil base + 16 ANSI sin tocar |

### Linux

| Síntoma | Causa probable | Fix |
|---|---|---|
| `cc: command not found` | sin gcc | instala `build-essential` / grupo `base-devel` (§1) |
| `rustc` viejo del distro (<1.75) | paquete del sistema | desinstálalo y usa rustup (§1) |
| Caracteres raros por SSH | locale del servidor en `C` | `export LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8` o instala el locale |
| Dentro de `tmux/screen` se descuadran colores | TERM sin 256 colores | `export TERM=xterm-256color` / `tmux -2`, y `set -g default-terminal "xterm-256color"` |
| Parpadeo al redimensionar | terminal sin sync | normal en `xterm`; en kitty/Alacritty/WezTerm no ocurre (diff-render) |

### Los tres SO

* `cargo test` falla en red (timeouts a crates.io) → reintenta; si hay
  proxy corporativo configura `HTTP(S)_PROXY` o el `config.toml` de cargo.
* Primera compilación lenta (2–5 min): normal, compila `crossterm` y
  amigos. Las siguientes son segundos (caché en `target/`).
* `Address already in use` / terminal "rara" tras matar un ejemplo con
  Ctrl+C: ejecuta `reset` (Linux/macOS) o cierra la pestaña; el ejemplo
  restaura la terminal solo en salida limpia.

## 7. Usar 90TUI en tu proyecto

Aún no estamos en crates.io: depende por git (rama de construcción)
o por path (clon local):

```toml
# Por git (recomendado mientras sale la 0.1.0):
[dependencies]
tui90 = { git = "https://github.com/GcorpZ/90TUI.git", branch = "90tui-built" }

# Por path (desarrollo local):
# tui90 = { path = "../90TUI" }
```

```rust
use tui90::{App, Theme};

fn main() -> std::io::Result<()> {
    tui90::enter_screen()?;
    let mut app = App::new(Theme::clipper())?;
    // ... pinta con core/prim/widgets, presenta, lee eventos ...
    app.present()?;
    tui90::leave_screen()
}
```

Detalles de API: `docs/FASE1-plan-diseno.md` y `docs/FASE2-partes-libreria.md`
(rama `90tui-docs`), más los ejemplos como código copiable.

## 8. Actualizar y desinstalar

```sh
cd 90TUI
git pull
rustup update            # de vez en cuando (MSRV garantizado: 1.75+)
cargo clean              # si quieres recuperar disco (borra target/)
```

Para quitar Rust del todo: `rustup self uninstall`.

---
Siguiente: `README.md` definitivo (Fase 8) con tour visual y badges.
