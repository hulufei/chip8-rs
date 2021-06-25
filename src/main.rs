use chip::Chip;
use std::path::PathBuf;
use structopt::StructOpt;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

mod chip;
mod graphics;
mod keyboard;

#[derive(Debug, StructOpt)]
#[structopt(name = "c8", about = "A chip-8 emulator")]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    rom: PathBuf,
    #[structopt(short)]
    debug: bool,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut chip = Chip::new(opt.debug);
    chip.load(opt.rom)?;
    chip.run()
}
