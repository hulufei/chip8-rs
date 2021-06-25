use crate::Result;
use crossterm::{cursor, queue, style, terminal, ExecutableCommand, QueueableCommand};
use std::io::Write;

pub struct Graphics<W: Write> {
    // 64 * 32 display
    pub pixels: [[u8; 64]; 32],
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
            pixels: [[0; 64]; 32],
            out,
        })
    }

    pub fn clear(&mut self) -> Result<()> {
        for mut row in self.pixels {
            row.fill(0);
        }
        self.draw()
    }

    pub fn draw(&mut self) -> Result<()> {
        for y in 0..32 {
            for x in 0..64 {
                let pixel = if self.pixels[y][x] == 1 { '*' } else { ' ' };
                self.out
                    .queue(cursor::MoveTo(x as u16 + 1, y as u16 + 1))?
                    .queue(style::Print(pixel))?;
            }
        }
        self.out.flush()?;
        Ok(())
    }

    pub fn draw_debugger(&mut self) -> Result<()> {
        queue!(
            self.out,
            cursor::MoveTo(0, 35),
            style::Print("Press n to run next instruction"),
            cursor::MoveToNextLine(2)
        )?;
        self.out.flush()?;
        Ok(())
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
