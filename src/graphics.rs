use crate::Result;
use crossterm::{
    cursor::{self, MoveTo},
    style,
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::io::Write;

pub struct Graphics<W: Write> {
    // 64 * 32 display
    pub pixels: [[u8; 64]; 32],
    out: W,
    debugger_layout: DebuggerLayout,
}

impl<W: Write> Graphics<W> {
    pub fn new(mut out: W) -> Result<Self> {
        // Draw a screen
        out.execute(terminal::Clear(terminal::ClearType::All))?;
        out.queue(cursor::MoveTo(0, 0))?
            .queue(style::Print("⥨".repeat(66)))?;
        for _ in 0..32 {
            out.queue(cursor::MoveToNextLine(1))?
                .queue(style::Print('⥮'))?
                .queue(cursor::MoveToColumn(66))?
                .queue(style::Print('⥮'))?;
        }
        out.queue(cursor::MoveToNextLine(1))?
            .queue(style::Print("⥨".repeat(66)))?
            .flush()?;

        Ok(Self {
            pixels: [[0; 64]; 32],
            debugger_layout: DebuggerLayout::new((0, 35)),
            out,
        })
    }

    pub fn clear(&mut self) -> std::io::Result<()> {
        for mut row in self.pixels {
            row.fill(0);
        }
        self.draw()
    }

    pub fn draw(&mut self) -> std::io::Result<()> {
        for y in 0..32 {
            for x in 0..64 {
                let pixel = if self.pixels[y][x] == 1 { '*' } else { ' ' };
                self.out
                    .queue(cursor::MoveTo(x as u16 + 1, y as u16 + 1))?
                    .queue(style::Print(pixel))?;
            }
        }
        self.out.flush()
    }

    fn cursor_move_to(pos: CursorPos) -> MoveTo {
        cursor::MoveTo(pos.0, pos.1)
    }

    pub fn log_op(&mut self, op: &str) -> std::io::Result<()> {
        self.out
            .queue(Self::cursor_move_to(self.debugger_layout.op))?
            .queue(terminal::Clear(ClearType::UntilNewLine))?
            .queue(style::Print(op))?
            // Move cursor to the end, so that exit program will keep the whole logs
            .queue(cursor::MoveDown(17))?
            .flush()
    }

    pub fn log_values(&mut self, registers: [u8; 16], pc: u16, vi: u16) -> std::io::Result<()> {
        for (i, v) in registers.iter().enumerate() {
            self.out
                .queue(Self::cursor_move_to(self.debugger_layout.registers[i]))?
                .queue(style::Print(format!("V{:<2}: {:#04X}", i, v)))?;
        }
        self.out
            .queue(Self::cursor_move_to(self.debugger_layout.pc))?
            .queue(style::Print(format!("PC: {:#06X}", pc)))?
            .queue(Self::cursor_move_to(self.debugger_layout.vi))?
            .queue(style::Print(format!(" I: {:#06X}", vi)))?
            // Move cursor to the end, so that exit program will keep the whole logs
            .queue(cursor::MoveDown(14))?
            .flush()
    }

    pub fn draw_debugger(&mut self) -> std::io::Result<()> {
        self.out
            .queue(Self::cursor_move_to(self.debugger_layout.start))?
            .queue(terminal::Clear(ClearType::FromCursorDown))?
            .queue(style::Print("Debugger"))?
            .flush()
    }
}

type CursorPos = (u16, u16);

struct DebuggerLayout {
    start: CursorPos,
    registers: [CursorPos; 16],
    pc: CursorPos,
    vi: CursorPos,
    op: CursorPos,
}

impl DebuggerLayout {
    fn new(start: CursorPos) -> Self {
        let (start_x, start_y) = start;
        let op = (start_x, start_y + 2);
        let register_start_y = op.1 + 2;

        let mut registers = [(0, 0); 16];
        let pos = (0..16)
            .map(|row| (start_x, register_start_y + row))
            .collect::<Vec<_>>();
        registers.copy_from_slice(&pos);

        Self {
            start,
            op,
            registers,
            // V0: 0xFF(5 space) PC: 0xFFFF
            pc: (start_x + 12, register_start_y),
            // V1: 0xFF(5 space) I: 0xFFFF
            vi: (start_x + 12, register_start_y + 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufWriter;

    #[test]
    fn it_initialize_screen() -> Result<()> {
        let mut buffer = Vec::new();
        let out = BufWriter::new(&mut buffer);
        let _ = Graphics::new(out)?;
        insta::assert_snapshot!(String::from_utf8(buffer)?, @"[2J[1;1H⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥮[66G⥮[1E⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨⥨");
        Ok(())
    }
}
