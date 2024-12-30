use anyhow::{bail, Result};
use std::{fs, ops::Range, path::Path};

pub struct Cpu {
    pc: u32,
    registers: [u32; 32],
    inter: InterConnect,
}

impl Cpu {
    pub fn new(inter: InterConnect) -> Self {
        let registers = const {
            let mut registers = [0u32; 32];
            registers[29] = 0x801FFFF0;
            registers
        };

        Self {
            pc: 0xbfc00000, // Start of the bios
            registers,
            inter,
        }
    }

    fn regs(&self, index: u32) -> u32 {
        self.registers[index as usize]
    }

    fn set_regs(&mut self, index: u32, value: u32) {
        self.registers[index as usize] = value;
        self.registers[0] = 0;
    }

    pub fn run_next_instruction(&mut self) {
        let pc = self.pc;

        let instruction = self.load32(pc);

        self.pc = self.pc.wrapping_add(4);

        self.decode_and_execute(Instruction(instruction));
    }

    pub fn load32(&self, addr: u32) -> u32 {
        self.inter.load32(addr)
    }

    fn decode_and_execute(&mut self, instruction: Instruction) {
        match instruction.function() {
            0b001111 => self.op_lui(instruction),
            _ => {
                panic!("Unhandled instruction {:#x}", instruction.0)
            }
        }
    }

    fn op_lui(&mut self, instruction: Instruction) {
        let i = instruction.imm();
        let t = instruction.t();

        let v = i << 16;

        self.set_regs(t, v);
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
        let start = offset as usize;
        let end = start + 4;

        if end > self.data.len() {
            panic!(
                "Attempted to read beyond BIOS memory bounds: offset {:#x}",
                offset
            );
        }

        u32::from_le_bytes(
            self.data[start..end]
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

struct Instruction(u32);

impl Instruction {
    #[inline]
    pub const fn function(&self) -> u32 {
        self.0 >> 26
    }

    #[inline]
    pub const fn t(&self) -> u32 {
        (self.0 >> 16) & 0x1f
    }

    #[inline]
    pub const fn imm(&self) -> u32 {
        self.0 & 0xffff
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
