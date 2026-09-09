//! Backend: la "tarjeta VGA" moderna (terminal ANSI vía crossterm).
//!
//! Los widgets nunca tocan la terminal: dibujan en `Buffer` y el backend
//! solo presenta el `diff`. `TestBackend` permite probar sin TTY.

use std::io::{self, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    style::{Attribute, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand,
};

use super::buffer::DrawOp;

/// Destino de pintado.
pub trait Backend {
    fn size(&self) -> io::Result<(u16, u16)>;
    fn present(&mut self, ops: &[DrawOp]) -> io::Result<()>;
}

/// Backend real sobre crossterm.
pub struct CrosstermBackend<W: Write> {
    out: W,
}

impl<W: Write> CrosstermBackend<W> {
    pub fn new(out: W) -> Self {
        Self { out }
    }

    pub fn present_ops(&mut self, ops: &[DrawOp]) -> io::Result<()> {
        self.present(ops)
    }
}

impl<W: Write> Backend for CrosstermBackend<W> {
    fn size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    fn present(&mut self, ops: &[DrawOp]) -> io::Result<()> {
        for op in ops {
            self.out.queue(MoveTo(op.x, op.y))?;
            self.out
                .queue(SetForegroundColor(op.cell.fg.to_crossterm()))?;
            self.out
                .queue(SetBackgroundColor(op.cell.bg.to_crossterm()))?;
            if op.cell.bold {
                self.out.queue(SetAttribute(Attribute::Bold))?;
            }
            if op.cell.dim {
                // Atenuación ANSI (\x1b[2m): sombras fantasma.
                self.out.queue(SetAttribute(Attribute::Dim))?;
            }
            self.out.queue(Print(op.cell.ch))?;
            if op.cell.bold || op.cell.dim {
                // Restablece para no "contagiar" la siguiente celda.
                // La próxima op reprograma fg/bg de todos modos.
                self.out.queue(SetAttribute(Attribute::Reset))?;
            }
        }
        self.out.flush()
    }
}

/// Entra a pantalla alternativa + raw mode + oculta cursor.
/// Es el "modo gráfico" de G90TUI. Llamar una vez al arrancar.
pub fn enter_screen() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?.execute(Hide)?;
    Ok(())
}

/// Sale restaurando la terminal (como el `Restscreen` final).
pub fn leave_screen() -> io::Result<()> {
    io::stdout().execute(Show)?.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

/// Backend de pruebas: tamaño fijo, graba lo presentado. Sin TTY.
#[derive(Debug, Default)]
pub struct TestBackend {
    pub w: u16,
    pub h: u16,
    pub seen: Vec<DrawOp>,
}

impl TestBackend {
    pub fn new(w: u16, h: u16) -> Self {
        Self {
            w,
            h,
            seen: Vec::new(),
        }
    }

    /// Consume lo grabado desde la última llamada.
    pub fn take(&mut self) -> Vec<DrawOp> {
        std::mem::take(&mut self.seen)
    }
}

impl Backend for TestBackend {
    fn size(&self) -> io::Result<(u16, u16)> {
        Ok((self.w, self.h))
    }

    fn present(&mut self, ops: &[DrawOp]) -> io::Result<()> {
        self.seen.extend_from_slice(ops);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::buffer::Buffer;
    use super::super::color::Color;
    use super::*;

    #[test]
    fn test_backend_records_ops() {
        let mut be = TestBackend::new(80, 25);
        assert_eq!(be.size().unwrap(), (80, 25));
        let back = Buffer::blank(80, 25, Color::Black);
        let front = Buffer::blank(80, 25, Color::Cyan);
        let ops = back.diff(&front);
        be.present(&ops).unwrap();
        assert_eq!(be.take().len(), 80 * 25);
        assert!(be.take().is_empty());
    }
}
