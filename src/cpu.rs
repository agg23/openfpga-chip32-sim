use std::{
    fs::File,
    io::{self, Read},
    ops::Shl,
};

use crate::{
    mem::Memory,
    util::num::{LowerLong, LowerWord},
};

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

        let alu_32_bit = (inst_prefix_byte & 0x10) != 0;

        match inst_prefix_byte {
            0x0 => {}                                                              // NOP
            0x02 => self.basic_load_inst(inst_suffix_byte, false, DataSize::Byte), // ld.b Rx,(nnnn)
            0x03 => self.basic_load_inst(inst_suffix_byte, true, DataSize::Byte),  // ld.b (nnnn),Rx
            0x04 => self.basic_load_inst(inst_suffix_byte, false, DataSize::Word), // ld.w Rx,(nnnn)
            0x05 => self.basic_load_inst(inst_suffix_byte, true, DataSize::Word),  // ld.w (nnnn),Rx
            0x06 => self.basic_load_inst(inst_suffix_byte, false, DataSize::Long), // ld.l Rx,(nnnn)
            0x07 => self.basic_load_inst(inst_suffix_byte, true, DataSize::Long),  // ld.l (nnnn),Rx
            0x08 | 0x18 => {
                self.alu_immediate(inst_suffix_byte, false, alu_32_bit, |_, immediate| {
                    // ld Rx,#16
                    (immediate, false)
                })
            }
            0x09 | 0x19 => {
                self.alu_immediate(inst_suffix_byte, false, alu_32_bit, |reg, immediate| {
                    // and Rx,#16
                    (reg & immediate, false)
                })
            }
            0x0A | 0x1A => {
                self.alu_immediate(inst_suffix_byte, false, alu_32_bit, |reg, immediate| {
                    // or Rx,#16
                    (reg | immediate, false)
                })
            }
            0x0B | 0x1B => {
                self.alu_immediate(inst_suffix_byte, false, alu_32_bit, |reg, immediate| {
                    // xor Rx,#16
                    (reg ^ immediate, false)
                })
            }
            0x0C | 0x1C => {
                self.alu_immediate(inst_suffix_byte, true, alu_32_bit, |reg, immediate| {
                    // add Rx,#16
                    reg.overflowing_add(immediate)
                })
            }
            0x0D | 0x1D => {
                self.alu_immediate(inst_suffix_byte, true, alu_32_bit, |reg, immediate| {
                    // sub Rx,#16
                    reg.overflowing_sub(immediate)
                })
            }
            0x0E | 0x1E => {
                // cmp Rx,#16
                let reg = inst_suffix_byte;
                let immediate = if alu_32_bit {
                    self.pc_long()
                } else {
                    self.pc_word() as u32
                };

                let (value, _) = self.get_reg(reg).overflowing_sub(immediate);

                // Don't set value
                self.set_zero(value);
            }
            0x0F | 0x1F => {
                // bit Rx,#16
                let reg = inst_suffix_byte;
                let immediate = if alu_32_bit {
                    self.pc_long()
                } else {
                    self.pc_word() as u32
                };

                let value = self.get_reg(reg) & (immediate);

                // Don't set value
                self.set_zero(value);
            }
            0x10 => todo!("RSET"),
            0x11 => todo!("CRC"),
            0x20 => {
                // asl Rx,Ry
                self.alu_double_reg(inst_suffix_byte, true, |reg_x, reg_y| {
                    let long_long = (reg_x as u64).shl(reg_y);
                    (long_long.to_lower_long(), long_long > u32::MAX as u64)
                })
            }
            0x21 => {
                // lsr Rx,Ry
                self.alu_double_reg(inst_suffix_byte, true, |reg_x, reg_y| {
                    let (long_long, _) = (reg_x as u64).overflowing_shr(reg_y);
                    // Get bit at position reg_y - 1
                    (long_long.to_lower_long(), (reg_x & (1 << (reg_y - 1)) != 0))
                })
            }
            _ => {
                // Do nothing
                todo!("Unimplemented {inst_prefix_byte:#X}")
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
                DataSize::Word => self.ram.mem_write_word(address, value.to_lower_word()),
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

    fn alu_immediate<T: Fn(u32, u32) -> (u32, bool)>(
        &mut self,
        inst_suffix_byte: u8,
        set_carry: bool,
        bit32: bool,
        operation: T,
    ) {
        let reg = inst_suffix_byte;
        let immediate = if bit32 {
            self.pc_long()
        } else {
            self.pc_word() as u32
        };

        let (value, carry) = operation(self.get_reg(reg), immediate);

        self.set_reg(reg, value);
        if set_carry {
            self.set_carry(carry);
        }
        self.set_zero(value);
    }

    fn alu_double_reg<T: Fn(u32, u32) -> (u32, bool)>(
        &mut self,
        inst_suffix_byte: u8,
        set_carry: bool,
        operation: T,
    ) {
        let reg_x = inst_suffix_byte & 0xF;
        let reg_y = (inst_suffix_byte >> 4) & 0xF;

        let (value, carry) = operation(self.get_reg(reg_x), self.get_reg(reg_y));

        self.set_reg(reg_x, value);
        if set_carry {
            self.set_carry(carry);
        }
        self.set_zero(value);
    }

    // fn pc_byte(&mut self) -> u8 {
    //     let value = self.ram.mem_read_byte(self.pc);

    //     self.pc += 1;

    //     value
    // }

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
