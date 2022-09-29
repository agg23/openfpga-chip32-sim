use std::{
    fs::File,
    io::{self, Read},
};

use crate::mem::Memory;

pub struct CPU {
    pub pc: u16,
    pub sp: u16,
    pub work_regs: [u32; 16],
    pub carry: bool,
    pub zero: bool,

    pub ram: Memory,
}

enum DataSize {
    Byte,
    Word,
    Long,
}

impl CPU {
    pub fn step(&mut self) {
        let inst_word = self.pc_word();
        let [inst_prefix_byte, inst_suffix_byte] = inst_word.to_be_bytes();

        match inst_prefix_byte {
            0x0 => {}                                                                           // NOP
            0x02 => self.basic_load_inst(inst_suffix_byte, false, DataSize::Byte), // ld.b Rx,(nnnn)
            0x03 => self.basic_load_inst(inst_suffix_byte, true, DataSize::Byte),  // ld.b (nnnn),Rx
            0x04 => self.basic_load_inst(inst_suffix_byte, false, DataSize::Word), // ld.w Rx,(nnnn)
            0x05 => self.basic_load_inst(inst_suffix_byte, true, DataSize::Word),  // ld.w (nnnn),Rx
            0x06 => self.basic_load_inst(inst_suffix_byte, false, DataSize::Long), // ld.l Rx,(nnnn)
            0x07 => self.basic_load_inst(inst_suffix_byte, true, DataSize::Long),  // ld.l (nnnn),Rx
            0x08 => self.basic_alu(inst_suffix_byte, false, |_, immediate| (immediate, false)), // ld Rx,#16
            0x09 => self.basic_alu(inst_suffix_byte, false, |reg, immediate| {
                // and Rx,#16
                (reg & immediate, false)
            }),
            0x0A => self.basic_alu(inst_suffix_byte, false, |reg, immediate| {
                // or Rx,#16
                (reg | immediate, false)
            }),
            0x0B => self.basic_alu(inst_suffix_byte, false, |reg, immediate| {
                // xor Rx,#16
                (reg ^ immediate, false)
            }),
            0x0C => self.basic_alu(inst_suffix_byte, true, |reg, immediate| {
                // add Rx,#16
                reg.overflowing_add(immediate)
            }),
            0x0D => self.basic_alu(inst_suffix_byte, true, |reg, immediate| {
                // sub Rx,#16
                reg.overflowing_sub(immediate)
            }),
            0x0E => {
                // cmp Rx,#16
                let reg = inst_suffix_byte;
                let immediate = self.pc_word();

                let (value, carry) = self.get_reg(reg).overflowing_sub(immediate as u32);

                // Don't set value
                self.set_carry(carry);
                self.set_zero(value);
            }
            0x0F => {
                // bit Rx,#16
                let reg = inst_suffix_byte;
                let immediate = self.pc_word();

                let value = self.get_reg(reg) & (immediate as u32);

                // Don't set value
                self.set_zero(value);
            }
            0x10 => todo!("RSET"),
            0x11 => todo!("CRC"),
            0x18 => {
                // ld Rx,#32
                let reg = inst_suffix_byte;
                let immediate = self.pc_long();

                self.set_reg(reg, immediate as u32);
                self.set_zero(immediate as u32);
            }
            _ => {
                // Do nothing
            }
        }
    }

    ///
    /// A load/store instruction. Can be `b`, `w`, or `l`
    ///
    fn basic_load_inst(&mut self, inst_suffix_byte: u8, write: bool, size: DataSize) {
        let reg = inst_suffix_byte;
        let address = self.pc_word();

        if write {
            let value = self.get_reg(reg);

            match size {
                DataSize::Byte => self.ram.mem_write_byte(address, value.to_le_bytes()[0]),
                DataSize::Word => {
                    let bytes = value.to_le_bytes();

                    self.ram
                        .mem_write_word(address, (bytes[1] as u16) << 8 | (bytes[0] as u16))
                }
                DataSize::Long => self.ram.mem_write_long(address, value),
            }
        } else {
            let value = match size {
                DataSize::Byte => self.ram.mem_read_byte(address).into(),
                DataSize::Word => self.ram.mem_read_word(address).into(),
                DataSize::Long => self.ram.mem_read_long(address),
            };

            self.set_reg(reg, value);
            self.set_zero(value);
        }
    }

    fn basic_alu<T: Fn(u32, u32) -> (u32, bool)>(
        &mut self,
        inst_suffix_byte: u8,
        set_carry: bool,
        operation: T,
    ) {
        let reg = inst_suffix_byte;
        let immediate = self.pc_word();

        let (value, carry) = operation(self.get_reg(reg), immediate as u32);

        self.set_reg(reg, value);
        if set_carry {
            self.set_carry(carry);
        }
        self.set_zero(value);
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

    fn pc_long(&mut self) -> u32 {
        let value = self.ram.mem_read_long(self.pc);

        self.pc += 4;

        value
    }

    fn set_zero(&mut self, value: u32) {
        self.zero = value == 0;
    }

    fn set_carry(&mut self, carry: bool) {
        self.carry = carry;
    }

    fn set_reg(&mut self, reg: u8, value: u32) {
        self.work_regs[reg as usize] = value;
    }

    fn get_reg(&self, reg: u8) -> u32 {
        self.work_regs[reg as usize]
    }

    // Loading

    pub fn load_file(path_str: &str) -> Result<Self, io::Error> {
        let mut file = File::open(path_str)?;

        let mut buffer = Vec::<u8>::new();

        file.read_to_end(&mut buffer)?;

        Ok(CPU {
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
