use graphics::Graphics;
use rand::random;
use std::io::{stdout, Read, Stdout};
use std::{fs::File, path::Path};

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

mod graphics;

struct Chip {
    memory: [u8; 4096],
    // Registers
    v: [u8; 16],
    vi: u16,
    // The program counter, store the currently executing address
    pc: u16,
    // The stack pointer, used to point to the topmost level of the stack
    sp: u8,
    stack: [u16; 16],
    // 64 * 32 display
    // gfx: [[u8; 64]; 32],
    gfx: Graphics<Stdout>,
    // Key(0-F) pressed status
    key: [bool; 16],
    delay_timer: u8,
    sound_timer: u8,
}

impl Chip {
    fn new() -> Self {
        let fontset = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        let mut memory = [0; 4096];

        for i in 0..80 {
            memory[i] = fontset[i];
        }

        let gfx = Graphics::new(stdout()).expect("Initialize graphics successfully");

        Chip {
            memory,
            v: [0; 16],
            vi: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            gfx,
            key: [false; 16],
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let mut f = File::open(path)?;
        f.read(&mut self.memory[512..])?;
        Ok(())
    }

    fn fetch_opcode(&mut self) -> u16 {
        let hi_bits = self.memory[self.pc as usize];
        let lo_bits = self.memory[self.pc as usize + 1];
        self.pc += 2;
        (hi_bits as u16) << 8 | lo_bits as u16
    }

    fn run(&mut self) -> Result<()> {
        loop {
            self.exec_cycle()?;
        }
    }

    // Emulate one cycle
    fn exec_cycle(&mut self) -> Result<()> {
        let opcode = self.fetch_opcode();

        // Decode and execute
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let n = (opcode & 0x000F) as u8;
        let nn = (opcode & 0x00FF) as u8; // low
        let nnn = opcode & 0x0FFF;

        match opcode {
            0x00E0 => {
                self.gfx.clear()?;
            }
            0x00EE => {
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1;
            }
            _ => match opcode & 0xF000 {
                0x1000 => {
                    // Jump to location nnn
                    self.pc = nnn;
                }
                0x2000 => {
                    // Call subroutine at nnn
                    self.sp += 1;
                    self.stack[self.sp as usize] = self.pc;
                    self.pc = nnn;
                }
                0x3000 => {
                    // Skip next instruction if Vx == nn
                    if self.v[x as usize] == nn {
                        self.pc += 2;
                    }
                }
                0x4000 => {
                    // Skip next instruction if Vx != nn
                    if self.v[x as usize] != nn {
                        self.pc += 2;
                    }
                }
                0x5000 => {
                    // Skip next instruction if Vx == Vy
                    if self.v[x as usize] == self.v[y as usize] {
                        self.pc += 2;
                    }
                }
                0x6000 => {
                    self.v[x as usize] = nn;
                }
                0x7000 => {
                    self.v[x as usize] += nn;
                }
                0x8000 => match opcode & 0x000F {
                    0 => {
                        self.v[x as usize] = self.v[y as usize];
                    }
                    1 => {
                        self.v[x as usize] |= self.v[y as usize];
                    }
                    2 => {
                        self.v[x as usize] &= self.v[y as usize];
                    }
                    3 => {
                        self.v[x as usize] ^= self.v[y as usize];
                    }
                    4 => {
                        let result = self.v[x as usize] as u16 + self.v[y as usize] as u16;
                        // VF
                        self.v[0xF] = if result > 255 { 1 } else { 0 };
                        // Keep the lower bits
                        self.v[x as usize] = result as u8;
                    }
                    5 => {
                        let vx = self.v[x as usize];
                        let vy = self.v[y as usize];
                        self.v[0xF] = if vx > vy { 1 } else { 0 };
                        // Consider as borrow from VF
                        // let vx = 0x0100 | self.v[x as usize] as u16;
                        // let vy = self.v[y as usize] as u16;
                        // self.v[x as usize] = (vx - vy) as u8;
                        self.v[x as usize] = vx.wrapping_sub(vy);
                    }
                    7 => {
                        let vx = self.v[x as usize];
                        let vy = self.v[y as usize];
                        self.v[0xF] = if vx < vy { 1 } else { 0 };
                        self.v[x as usize] = vy.wrapping_sub(vx);
                    }
                    6 => {
                        let vx = self.v[x as usize];
                        self.v[0xF] = if vx & 1 == 1 { 1 } else { 0 };
                        self.v[x as usize] = vx >> 1;
                    }
                    0xE => {
                        let vx = self.v[x as usize];
                        self.v[0xF] = if vx & 0x10 == 1 { 1 } else { 0 };
                        self.v[x as usize] = vx << 1;
                    }
                    _ => Err(format!("Unknown instruction {:x}", opcode))?,
                },
                0x9000 if opcode & 1 == 0 => {
                    if self.v[x as usize] != self.v[y as usize] {
                        self.pc += 2;
                    }
                }
                0xA000 => {
                    self.vi = nnn;
                }
                0xB000 => {
                    self.pc = nnn + self.v[0] as u16;
                }
                0xC000 => {
                    let rnd_byte = random::<u8>();
                    self.v[x as usize] = rnd_byte & nn;
                }
                0xD000 => {
                    // Read n bytes from memory(sprites = 8 * n pixel), starting at vi
                    let vi = self.vi as usize;
                    let sprites = &self.memory[vi..(vi + n as usize)];
                    let x = self.v[x as usize] % 64;
                    let y = self.v[y as usize] % 32;

                    for (r, byte) in sprites.iter().enumerate() {
                        let y = y as usize + r;
                        // Out of vertical edge
                        if y > 32 {
                            break;
                        }

                        for c in 0..8 {
                            let x = x as usize + c;
                            if x > 64 {
                                // Out of horizontal edge
                                break;
                            }
                            let sprite_bit = (byte >> (7 - c)) & 1;
                            let screen_bit = self.gfx.pixels[y][x];
                            let pixel = sprite_bit ^ screen_bit;

                            self.gfx.pixels[y][x] = pixel;

                            // Erased screen (on -> off)
                            if screen_bit == 1 && pixel == 0 {
                                self.v[0xF] = 1;
                            } else {
                                self.v[0xF] = 0;
                            }
                        }
                    }
                    self.gfx.draw()?;
                }
                0xE000 if opcode & 0x00FF == 0x009E => {
                    let vx = self.v[x as usize] as usize;
                    if self.key[vx] {
                        self.pc += 2;
                    }
                }
                0xE000 if opcode & 0x00FF == 0x00A1 => {
                    let vx = self.v[x as usize] as usize;
                    if !self.key[vx] {
                        self.pc += 2;
                    }
                }
                0xF000 => match opcode & 0x00FF {
                    0x07 => {
                        self.v[x as usize] = self.delay_timer;
                    }
                    0x0A => {
                        //  All execution stops until a key is pressed, then the value of that key is stored in Vx.
                        if let Some((k, _)) = self.key.iter().enumerate().find(|(_, &v)| v) {
                            self.v[x as usize] = k as u8;
                        } else {
                            self.pc -= 2;
                        }
                    }
                    0x15 => {
                        self.delay_timer = self.v[x as usize];
                    }
                    0x18 => {
                        self.sound_timer = self.v[x as usize];
                    }
                    0x1E => {
                        self.vi += self.v[x as usize] as u16;
                    }
                    0x29 => {
                        self.vi = self.v[x as usize] as u16;
                    }
                    0x33 => {
                        let vx = self.v[x as usize];
                        let vi = self.vi as usize;
                        self.memory[vi] = vx / 100;
                        self.memory[vi + 1] = (vx % 100) / 10;
                        self.memory[vi + 2] = (vx % 100) % 10;
                    }
                    0x55 => {
                        for i in 0..=x as usize {
                            let vi = self.vi as usize;
                            self.memory[vi + i] = self.v[i];
                        }
                    }
                    0x65 => {
                        for i in 0..=x as usize {
                            let vi = self.vi as usize;
                            self.v[i] = self.memory[vi + i];
                        }
                    }
                    _ => Err(format!("Unknown instruction {:x}", opcode))?,
                },
                _ => Err(format!("Unknown instruction {:x}", opcode))?,
            },
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            println!("beep!");
            self.sound_timer -= 1;
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let mut chip = Chip::new();
    chip.load("rom/IBM Logo.ch8")?;
    chip.run()
    // let _ = Graphics::new(std::io::stdout())?;
    // Ok(())
}
