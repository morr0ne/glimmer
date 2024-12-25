use anyhow::{bail, Result};
use std::{fs, ops::Range, path::Path};

pub struct Cpu {
    pc: u32,
    inter: InterConnect,
}

impl Cpu {
    pub fn new(inter: InterConnect) -> Self {
        Self {
            pc: 0xbfc00000, // Start of the bios
            inter,
        }
    }

    pub fn run_next_instruction(&mut self) {
        let pc = self.pc;

        let instruction = self.load32(pc);

        self.pc = self.pc.wrapping_add(4);

        self.decode_and_execute(instruction);
    }

    pub fn load32(&self, addr: u32) -> u32 {
        self.inter.load32(addr)
    }

    fn decode_and_execute(&mut self, instruction: u32) {
        panic!("Unhandled instruction {instruction:#x}")
    }
}

pub struct Bios {
    data: Vec<u8>,
}

impl Bios {
    const SIZE: u32 = 512 * 1024;
    const RANGE: Range<u32> = 0xbfc00000..Self::SIZE + 0xbfc00000;

    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let data = fs::read(path)?;

        if data.len() != Self::SIZE as usize {
            bail!("Corrupted bios")
        }

        Ok(Self { data })
    }

    pub fn load32(&self, offset: u32) -> u32 {
        u32::from_le_bytes(
            self.data[offset as usize..4]
                .try_into()
                .expect("Failed to read offset bytes"),
        )
    }
}

pub struct InterConnect {
    bios: Bios,
}

impl InterConnect {
    pub fn new(bios: Bios) -> Self {
        Self { bios }
    }

    pub fn load32(&self, addr: u32) -> u32 {
        if Bios::RANGE.contains(&addr) {
            return self.bios.load32(addr - Bios::RANGE.start);
        }

        panic!("unhandled fetch32 at addres {addr:#x}");
    }
}

fn main() -> Result<()> {
    let bios = Bios::new("bios/SCPH1001.BIN")?;
    let inter = InterConnect::new(bios);
    let mut cpu = Cpu::new(inter);

    loop {
        cpu.run_next_instruction();
    }

    Ok(())
}
