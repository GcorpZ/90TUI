//! Explorador de archivos modal (`FileDialog`).
//!
//! Ventana emergente con `std::fs`: árbol de directorios a la izquierda
//! (glifos `prim::folder`) y `ListBox` de archivos a la derecha.
//! `Tab` cambia de panel, `Enter` en un archivo acepta y cierra el modal
//! retornando su `PathBuf`, `Enter` en un directorio entra en él,
//! `Backspace` sube al padre, `Esc` cancela.
//!
//! Expone los 4 parámetros globales (`foreground_color`,
//! `background_color`, `border_color`, `has_shadow`).

use std::path::{Path, PathBuf};

use crossterm::event::KeyCode;

use crate::core::{Attr, Buffer, Cell, Color, Rect, Theme};
use crate::prim::{button, draw_text, fit_text, folder, FolderGlyphs};
use crate::widgets::listbox::{listbox_draw, listbox_key, ListBox, ListNav};

/// Explorador modal. Crear con `FileDialog::new(&ruta)`.
#[derive(Clone, Debug)]
pub struct FileDialog {
    pub title: String,
    pub current: PathBuf,
    /// Nombres de subdirectorios (sin `..`, que siempre es la fila 0).
    pub dirs: Vec<String>,
    pub dir_sel: usize,
    pub dir_top: usize,
    /// Panel derecho: archivos (reutiliza `ListBox`).
    pub file_list: ListBox,
    /// 0 = árbol, 1 = archivos.
    pub panel: usize,
    pub error: Option<String>,
    pub foreground_color: Color,
    pub background_color: Color,
    pub border_color: Option<Color>,
    pub has_shadow: bool,
}

impl FileDialog {
    pub fn new(path: &Path) -> Self {
        let mut d = Self {
            title: "Abrir archivo".to_string(),
            current: path.to_path_buf(),
            dirs: Vec::new(),
            dir_sel: 0,
            dir_top: 0,
            file_list: ListBox::new(&[]),
            panel: 1,
            error: None,
            foreground_color: Color::Black,
            background_color: Color::White,
            border_color: None,
            has_shadow: true,
        };
        d.file_list.foreground_color = Color::Black;
        d.file_list.background_color = Color::White;
        d.refresh();
        d
    }

    /// Relee el directorio actual (errores → `error`, listas vacías).
    pub fn refresh(&mut self) {
        self.dirs.clear();
        let mut files: Vec<String> = Vec::new();
        match std::fs::read_dir(&self.current) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name.starts_with('.') {
                        continue;
                    }
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        self.dirs.push(name);
                    } else {
                        files.push(name);
                    }
                }
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
        self.dirs.sort();
        files.sort();
        self.file_list = ListBox::new(&files.iter().map(String::as_str).collect::<Vec<_>>());
        self.file_list.foreground_color = self.foreground_color;
        self.file_list.background_color = self.background_color;
        self.dir_sel = 0;
        self.dir_top = 0;
    }

    /// Filas del árbol: `..` + subdirectorios.
    pub fn tree_rows(&self) -> usize {
        self.dirs.len() + 1
    }

    fn dir_name(&self, row: usize) -> Option<&str> {
        if row == 0 {
            Some("..")
        } else {
            self.dirs.get(row - 1).map(String::as_str)
        }
    }

    fn enter_row(&mut self, row: usize) {
        if row == 0 {
            self.go_parent();
            return;
        }
        if let Some(name) = self.dirs.get(row - 1).cloned() {
            self.current.push(name);
            self.refresh();
        }
    }

    fn go_parent(&mut self) {
        if let Some(parent) = self.current.parent() {
            self.current = parent.to_path_buf();
            self.refresh();
        }
    }

    /// Ruta del archivo seleccionado (si hay).
    pub fn selected_path(&self) -> Option<PathBuf> {
        self.file_list
            .items
            .get(self.file_list.selected)
            .map(|name| self.current.join(name))
    }
}

/// Rect del modal centrado (60x17 o lo que quepa).
pub fn filedialog_layout(screen: Rect) -> Rect {
    Rect::centered_in(60.min(screen.w), 17.min(screen.h.max(1)), screen)
}

/// Resultado de tecla en el diálogo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileDialogKey {
    Stay,
    Moved,
    Accepted(PathBuf),
    Cancelled,
}

/// Navegación pura (el `std::fs` solo se toca al entrar/subir de dir).
pub fn filedialog_key(dlg: &mut FileDialog, code: KeyCode) -> FileDialogKey {
    use FileDialogKey::*;
    match code {
        KeyCode::Esc => return Cancelled,
        KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
            dlg.panel = (dlg.panel + 1) % 2;
            return Moved;
        }
        KeyCode::Backspace => {
            dlg.go_parent();
            return Moved;
        }
        _ => {}
    }
    if dlg.panel == 0 {
        let n = dlg.tree_rows();
        match code {
            KeyCode::Up => {
                dlg.dir_sel = (dlg.dir_sel + n - 1) % n;
                Moved
            }
            KeyCode::Down => {
                dlg.dir_sel = (dlg.dir_sel + 1) % n;
                Moved
            }
            KeyCode::Home => {
                dlg.dir_sel = 0;
                Moved
            }
            KeyCode::End => {
                dlg.dir_sel = n - 1;
                Moved
            }
            KeyCode::Enter => {
                dlg.enter_row(dlg.dir_sel);
                Moved
            }
            KeyCode::Char(c) => {
                let lc = c.to_ascii_lowercase();
                for k in 0..n {
                    let i = (dlg.dir_sel + 1 + k) % n;
                    if dlg
                        .dir_name(i)
                        .and_then(|s| s.chars().next())
                        .map(|f| f.to_ascii_lowercase() == lc)
                        .unwrap_or(false)
                    {
                        dlg.dir_sel = i;
                        return Moved;
                    }
                }
                Stay
            }
            _ => Stay,
        }
    } else {
        // Panel derecho: delega en el ListBox (8 filas visibles).
        match listbox_key(&mut dlg.file_list, 8, code) {
            ListNav::Move(_) => Moved,
            ListNav::Stay => {
                if code == KeyCode::Enter {
                    if let Some(p) = dlg.selected_path() {
                        return Accepted(p);
                    }
                }
                Stay
            }
        }
    }
}

/// Dibuja el modal (no toca la pila; el llamante hace `savescreen`).
pub fn filedialog_draw(buf: &mut Buffer, screen: Rect, dlg: &FileDialog, theme: Theme) {
    let r = filedialog_layout(screen);
    if r.is_empty() {
        return;
    }
    let mut opts = crate::prim::WindowOpts::modal(&dlg.title, theme);
    opts.controls = true;
    opts.shadow = dlg.has_shadow;
    opts.border_color = dlg.border_color;
    crate::prim::window(buf, r, &opts, theme);

    // Ruta actual.
    let path = dlg.current.to_string_lossy().into_owned();
    draw_text(
        buf,
        r.x.saturating_add(2),
        r.y.saturating_add(1),
        &fit_text(&path, r.w.saturating_sub(4)),
        Attr::bold(Color::Black, theme.window_bg),
    );

    // Panel árbol (izq): `..` + carpetas con icono.
    let tree = Rect::new(
        r.x.saturating_add(2),
        r.y.saturating_add(2),
        22,
        r.h.saturating_sub(6),
    );
    buf.fill_rect(
        tree,
        Cell::new(' ', dlg.foreground_color, dlg.background_color),
    );
    let vis = tree.h as usize;
    for i in 0..vis {
        let row = dlg.dir_top + i;
        if row >= dlg.tree_rows() {
            break;
        }
        let y = tree.y.saturating_add(i as u16);
        let sel = row == dlg.dir_sel && dlg.panel == 0;
        let (fg, bg) = if sel {
            (Color::White, theme.popup)
        } else {
            (dlg.foreground_color, dlg.background_color)
        };
        buf.fill_rect(Rect::new(tree.x, y, tree.w, 1), Cell::new(' ', fg, bg));
        let attr = Attr::new(fg, bg);
        let mut cx = tree.x.saturating_add(1);
        cx += folder(buf, cx, y, false, attr, Color::Yellow, FolderGlyphs::nerd());
        if let Some(name) = dlg.dir_name(row) {
            draw_text(
                buf,
                cx.saturating_add(1),
                y,
                &fit_text(name, tree.w.saturating_sub(4)),
                attr,
            );
        }
    }

    // Panel archivos (der): ListBox reutilizado.
    let list_rect = Rect::new(
        tree.right().saturating_add(1),
        tree.y,
        r.right()
            .saturating_sub(tree.right().saturating_add(1))
            .saturating_sub(2),
        tree.h,
    );
    let mut files = dlg.file_list.clone();
    files.foreground_color = dlg.foreground_color;
    files.background_color = dlg.background_color;
    listbox_draw(buf, list_rect, &files);

    // Error o ayuda + botones.
    let foot = if let Some(e) = &dlg.error {
        fit_text(e, r.w.saturating_sub(4))
    } else {
        "Tab panel · Enter abre/elige · Bksp sube · Esc cancela".to_string()
    };
    draw_text(
        buf,
        r.x.saturating_add(2),
        r.bottom().saturating_sub(3),
        &fit_text(&foot, r.w.saturating_sub(4)),
        Attr::new(Color::DarkGrey, theme.window_bg),
    );
    let by = r.bottom().saturating_sub(2);
    let wok = crate::prim::button_width("Aceptar");
    let wcancel = crate::prim::button_width("Cancelar");
    let gap = 2u16;
    let bx = r
        .right()
        .saturating_sub(2)
        .saturating_sub(wok)
        .saturating_sub(gap)
        .saturating_sub(wcancel);
    button(buf, bx, by, "Aceptar", theme);
    button(
        buf,
        bx.saturating_add(wok).saturating_add(gap),
        by,
        "Cancelar",
        theme,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sandbox(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("g90tui_fdlg_{}_{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(p.join("sub")).unwrap();
        std::fs::write(p.join("a.txt"), b"a").unwrap();
        std::fs::write(p.join("b.txt"), b"b").unwrap();
        p
    }

    #[test]
    fn lists_dirs_and_files_sorted() {
        let root = sandbox("list");
        let dlg = FileDialog::new(&root);
        assert_eq!(dlg.dirs, vec!["sub".to_string()]);
        assert_eq!(
            dlg.file_list.items,
            vec!["a.txt".to_string(), "b.txt".to_string()]
        );
        assert!(dlg.error.is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn enter_accepts_file_and_parent_works() {
        let root = sandbox("nav");
        let mut dlg = FileDialog::new(&root);
        dlg.panel = 1;
        // Acepta el primer archivo con Enter.
        match filedialog_key(&mut dlg, KeyCode::Enter) {
            FileDialogKey::Accepted(p) => assert_eq!(p, root.join("a.txt")),
            other => panic!("esperaba Accepted, fue {other:?}"),
        }
        // Entra al subdirectorio desde el árbol y vuelve.
        dlg.panel = 0;
        dlg.dir_sel = 1;
        filedialog_key(&mut dlg, KeyCode::Enter);
        assert_eq!(dlg.current, root.join("sub"));
        filedialog_key(&mut dlg, KeyCode::Backspace);
        assert_eq!(dlg.current, root);
        assert_eq!(
            filedialog_key(&mut dlg, KeyCode::Esc),
            FileDialogKey::Cancelled
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_dir_reports_error_without_panic() {
        let mut p = std::env::temp_dir();
        p.push(format!("g90tui_fdlg_noexiste_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        let dlg = FileDialog::new(&p);
        assert!(dlg.error.is_some());
        assert!(dlg.file_list.items.is_empty());
    }

    #[test]
    fn draws_tree_list_and_buttons() {
        let root = sandbox("draw");
        let dlg = FileDialog::new(&root);
        let t = Theme::clipper();
        let mut b = Buffer::blank(80, 25, t.desktop);
        filedialog_draw(&mut b, Rect::new(0, 0, 80, 25), &dlg, t);
        let r = filedialog_layout(Rect::new(0, 0, 80, 25));
        // Título + carpeta + archivo visibles.
        assert_eq!(b.get(r.x + 2, r.y).unwrap().bg, t.teal);
        let mut saw_folder = false;
        let mut saw_file = false;
        for y in 0..25u16 {
            for x in 0..80u16 {
                let ch = b.get(x, y).unwrap().ch;
                if ch == '\u{f07b}' {
                    saw_folder = true;
                }
                if ch == 'a' && b.get(x.saturating_add(1), y).map(|c| c.ch) == Some('.') {
                    saw_file = true;
                }
            }
        }
        assert!(saw_folder && saw_file);
        let _ = std::fs::remove_dir_all(&root);
    }
}
