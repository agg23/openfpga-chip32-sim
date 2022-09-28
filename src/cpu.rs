use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use crate::mem::Memory;

pub struct State {
    pub pc: u16,
    pub sp: u16,
    pub work_regs: [u32; 16],
    pub carry: bool,
    pub zero: bool,

    pub ram: Memory,
}

impl State {
    pub fn step(&mut self) {
        let inst_word = self.pc_word();
        let [inst_prefix_byte, inst_suffix_byte] = inst_word.to_be_bytes();

        match inst_prefix_byte {
            0x0 => {} // NOP
            0x02 => {
                // LD.b Rx,(nnnn)
                let reg = inst_suffix_byte - 1;
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
            // TODO: Is this right?
            sp: 0,
            work_regs: [0; 16],
            carry: false,
            zero: false,
            ram: Memory::from_bytes(buffer),
        })
    }
}
