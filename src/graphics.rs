use std::io::Write;

use crate::Result;
use crossterm::{cursor, style, terminal, ExecutableCommand, QueueableCommand};

pub struct Graphics<W: Write> {
    // 64 * 32 display
    pub gfx: [[u8; 64]; 32],
    out: W,
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
            gfx: [[0; 64]; 32],
            out,
        })
    }

    pub fn clear(&mut self) -> Result<()> {
        for mut row in self.gfx {
            row.fill(0);
        }
        self.draw()
    }

    pub fn draw(&mut self) -> Result<()> {
        for y in 0..32 {
            for x in 0..64 {
                let pixel = if self.gfx[y][x] == 1 { '*' } else { ' ' };
                self.out
                    .queue(cursor::MoveTo(x as u16 + 1, y as u16 + 1))?
                    .queue(style::Print(pixel))?;
            }
        }
        self.out.queue(cursor::Hide)?.flush()?;
        Ok(())
    }
}
