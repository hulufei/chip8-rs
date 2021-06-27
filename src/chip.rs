use crate::graphics::Graphics;
use crate::keyboard::Keyboard;
use crate::Result;
use rand::random;
use std::io::{stdout, Read, Stdout};
use std::time::Instant;
use std::{fs::File, path::Path};

pub struct Chip {
    memory: [u8; 4096],
    // Registers
    v: [u8; 16],
    vi: u16,
    // The program counter, store the currently executing address
    pc: u16,
    // The stack pointer, used to point to the topmost level of the stack
    sp: u8,
    stack: [u16; 16],
    gfx: Graphics<Stdout>,
    keyboard: Keyboard,
    delay_timer: u8,
    sound_timer: u8,
    debug: bool,
    fps: u32,
}

impl Chip {
    pub fn new(fps: u32, debug: bool) -> Self {
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

        let mut gfx = Graphics::new(stdout()).expect("Initialize graphics successfully");
        let keyboard = Keyboard::new().expect("Initialize keyboard successfully");

        if debug {
            gfx.draw_debugger()
                .expect("Initialize debugger successfully");
        }

        Chip {
            debug,
            memory,
            v: [0; 16],
            vi: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            gfx,
            keyboard,
            delay_timer: 0,
            sound_timer: 0,
            fps,
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
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

    pub fn run(&mut self) -> Result<()> {
        'frame: loop {
            let start = Instant::now();
            let mut op_count = 0;
            loop {
                let time_frame = Instant::now().duration_since(start).as_millis();

                if op_count == self.fps && time_frame >= 1000 {
                    continue 'frame;
                }

                if op_count < self.fps {
                    if self.debug {
                        self.gfx.log_op("NEXT OP: Press n to fetch")?;
                        // Log previous result, press next to fetch next opcode
                        self.gfx.log_values(self.v, self.pc, self.vi)?;
                        Keyboard::block_until_press_next();
                    }
                    // Fetch opcode and execute
                    self.exec_cycle()?;
                    if self.debug {
                        // Log next opcode, press next to log result
                        Keyboard::block_until_press_next();
                    }
                    op_count += 1;
                }
            }
        }
    }

    fn log_op(&mut self, opcode: u16, msg: &str) -> std::io::Result<()> {
        if self.debug {
            self.gfx.log_op(&format!(
                "NEXT OP: {:#06X} {}, Press n to execute",
                opcode, msg
            ))?
        }
        Ok(())
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
                self.log_op(opcode, "CLS")?;
                self.gfx.clear()?;
            }
            0x00EE => {
                self.log_op(opcode, "RET")?;
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1;
            }
            _ => match opcode & 0xF000 {
                0x1000 => {
                    self.log_op(opcode, &format!("JP {:#06X}", nnn))?;
                    self.pc = nnn;
                }
                0x2000 => {
                    self.log_op(opcode, &format!("CALL {:#06X}", nnn))?;
                    self.sp += 1;
                    self.stack[self.sp as usize] = self.pc;
                    self.pc = nnn;
                }
                0x3000 => {
                    self.log_op(opcode, &format!("SE V{} {:#04X}", x, nn))?;
                    if self.v[x as usize] == nn {
                        self.pc += 2;
                    }
                }
                0x4000 => {
                    self.log_op(opcode, &format!("SNE V{} {:#04X}", x, nn))?;
                    if self.v[x as usize] != nn {
                        self.pc += 2;
                    }
                }
                0x5000 => {
                    self.log_op(opcode, &format!("SE V{} V{}", x, y))?;
                    if self.v[x as usize] == self.v[y as usize] {
                        self.pc += 2;
                    }
                }
                0x6000 => {
                    self.log_op(opcode, &format!("LD V{} {:#04X}", x, nn))?;
                    self.v[x as usize] = nn;
                }
                0x7000 => {
                    self.log_op(opcode, &format!("ADD V{} {:#04X}", x, nn))?;
                    let result = self.v[x as usize] as u16 + nn as u16;
                    self.v[x as usize] = result as u8;
                }
                0x8000 => match opcode & 0x000F {
                    0 => {
                        self.log_op(opcode, &format!("LD V{} V{}", x, y))?;
                        self.v[x as usize] = self.v[y as usize];
                    }
                    1 => {
                        self.log_op(opcode, &format!("OR V{} V{}", x, y))?;
                        self.v[x as usize] |= self.v[y as usize];
                    }
                    2 => {
                        self.log_op(opcode, &format!("AND V{} V{}", x, y))?;
                        self.v[x as usize] &= self.v[y as usize];
                    }
                    3 => {
                        self.log_op(opcode, &format!("XOR V{} V{}", x, y))?;
                        self.v[x as usize] ^= self.v[y as usize];
                    }
                    4 => {
                        self.log_op(opcode, &format!("ADD V{} V{}", x, y))?;
                        let result = self.v[x as usize] as u16 + self.v[y as usize] as u16;
                        // VF
                        self.v[0xF] = if result > 255 { 1 } else { 0 };
                        // Keep the lower bits
                        self.v[x as usize] = result as u8;
                    }
                    5 => {
                        self.log_op(opcode, &format!("SUB V{} V{}", x, y))?;
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
                        self.log_op(opcode, &format!("SUBN V{} V{}", x, y))?;
                        let vx = self.v[x as usize];
                        let vy = self.v[y as usize];
                        self.v[0xF] = if vx < vy { 1 } else { 0 };
                        self.v[x as usize] = vy.wrapping_sub(vx);
                    }
                    6 => {
                        self.log_op(opcode, &format!("SHR V{} {{, V{}}}", x, y))?;
                        let vx = self.v[x as usize];
                        self.v[0xF] = if vx & 1 == 1 { 1 } else { 0 };
                        self.v[x as usize] = vx >> 1;
                    }
                    0xE => {
                        self.log_op(opcode, &format!("SHL V{} {{, V{}}}", x, y))?;
                        let vx = self.v[x as usize];
                        self.v[0xF] = if vx & 0x10 == 1 { 1 } else { 0 };
                        self.v[x as usize] = vx << 1;
                    }
                    _ => Err(format!("Unknown instruction {:#06X}", opcode))?,
                },
                0x9000 if opcode & 1 == 0 => {
                    self.log_op(opcode, &format!("SNE V{} V{}", x, y))?;
                    if self.v[x as usize] != self.v[y as usize] {
                        self.pc += 2;
                    }
                }
                0xA000 => {
                    self.log_op(opcode, &format!("LD I {:#06X}", nnn))?;
                    self.vi = nnn;
                }
                0xB000 => {
                    self.log_op(opcode, &format!("JP V0 {:#06X}", nnn))?;
                    self.pc = nnn + self.v[0] as u16;
                }
                0xC000 => {
                    self.log_op(opcode, &format!("RND V{} {:#04X}", x, nn))?;
                    let rnd_byte = random::<u8>();
                    self.v[x as usize] = rnd_byte & nn;
                }
                0xD000 => {
                    self.log_op(opcode, &format!("DRW V{} V{} {:#04X}", x, y, n))?;
                    // Read n bytes from memory(sprites = 8 * n pixel), starting at vi
                    let vi = self.vi as usize;
                    let sprites = &self.memory[vi..(vi + n as usize)];
                    let x = self.v[x as usize] % 64;
                    let y = self.v[y as usize] % 32;

                    for (r, byte) in sprites.iter().enumerate() {
                        let y = y as usize + r;
                        // Out of vertical edge
                        if y >= 32 {
                            break;
                        }

                        for c in 0..8 {
                            let x = x as usize + c;
                            if x >= 64 {
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
                    self.log_op(opcode, &format!("SKP V{}", x))?;
                    let vx = self.v[x as usize] as usize;
                    if self.keyboard.get(vx) {
                        self.pc += 2;
                    }
                }
                0xE000 if opcode & 0x00FF == 0x00A1 => {
                    self.log_op(opcode, &format!("SKNP V{}", x))?;
                    let vx = self.v[x as usize] as usize;
                    if !self.keyboard.get(vx) {
                        self.pc += 2;
                    }
                }
                0xF000 => match opcode & 0x00FF {
                    0x07 => {
                        self.log_op(opcode, &format!("LD V{} DT", x))?;
                        self.v[x as usize] = self.delay_timer;
                    }
                    0x0A => {
                        self.log_op(opcode, &format!("LD V{} K", x))?;
                        //  All execution stops until a key is pressed, then the value of that key is stored in Vx.
                        if let Some(k) = self.keyboard.find_pressed_key() {
                            self.v[x as usize] = k;
                        } else {
                            self.pc -= 2;
                        }
                    }
                    0x15 => {
                        self.log_op(opcode, &format!("LD DT V{}", x))?;
                        self.delay_timer = self.v[x as usize];
                    }
                    0x18 => {
                        self.log_op(opcode, &format!("LD ST V{}", x))?;
                        self.sound_timer = self.v[x as usize];
                    }
                    0x1E => {
                        self.log_op(opcode, &format!("ADD I V{}", x))?;
                        self.vi += self.v[x as usize] as u16;
                    }
                    0x29 => {
                        self.log_op(opcode, &format!("LD F V{}", x))?;
                        self.vi = self.v[x as usize] as u16;
                    }
                    0x33 => {
                        self.log_op(opcode, &format!("LD B V{}", x))?;
                        let vx = self.v[x as usize];
                        let vi = self.vi as usize;
                        self.memory[vi] = vx / 100;
                        self.memory[vi + 1] = (vx % 100) / 10;
                        self.memory[vi + 2] = (vx % 100) % 10;
                    }
                    0x55 => {
                        self.log_op(opcode, &format!("LD [I] V{}", x))?;
                        for i in 0..=x as usize {
                            let vi = self.vi as usize;
                            self.memory[vi + i] = self.v[i];
                        }
                    }
                    0x65 => {
                        self.log_op(opcode, &format!("LD V{} [I]", x))?;
                        for i in 0..=x as usize {
                            let vi = self.vi as usize;
                            self.v[i] = self.memory[vi + i];
                        }
                    }
                    _ => Err(format!("Unknown instruction {:#06X}", opcode))?,
                },
                _ => Err(format!("Unknown instruction {:#06X}", opcode))?,
            },
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            println!("beep!");
            self.sound_timer -= 1;
        }

        self.keyboard.poll();

        Ok(())
    }
}
