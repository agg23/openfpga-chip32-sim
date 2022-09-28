use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use crate::mem::Memory;

pub struct State {
    pub pc: u16,
    pub work_regs: [u32; 16],
    pub carry: bool,
    pub zero: bool,

    pub ram: Memory,
}

impl State {
    fn step(mut self) {
        let current_inst_prefix = self.pc_byte();

        self.pc += 1;

        match current_inst_prefix {
            0x0 => {} // NOP
            0x02 => {
                // LD.b Rx,(nnnn)
                let reg = self.pc_byte();
                let address = self.pc_word();
                let value = self.ram.mem_read_word(address);

                self.set_reg(reg, value.into());
                self.set_zero(value.into());
            }
            _ => {
                // Do nothing
            }
        }
    }

    fn pc_byte(&mut self) -> u8 {
        let value = self.ram.mem_read_byte(self.pc);

        self.pc += 1;

        value
    }

    fn pc_word(&mut self) -> u16 {
        let value = self.ram.mem_read_word(self.pc);

        self.pc += 2;

        value
    }

    fn set_zero(&mut self, value: u32) {
        self.zero = value == 0;
    }

    fn set_reg(&mut self, reg: u8, value: u32) {
        self.work_regs[reg as usize] = value;
    }

    // Loading

    pub fn load_file(path_str: String) -> Result<Self, io::Error> {
        let mut file = File::open(&path_str)?;

        let mut buffer = Vec::<u8>::new();

        file.read_to_end(&mut buffer)?;

        Ok(State {
            pc: 0x2,
            work_regs: [0; 16],
            carry: false,
            zero: false,
            ram: Memory::from_bytes(buffer),
        })
    }
}
