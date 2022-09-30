use std::{
    fs::File,
    io::{self, Read},
    ops::{Shl, Shr},
};

use crate::{
    mem::Memory,
    util::{
        bitwise::BitIndex,
        num::{LowerLong, LowerWord},
    },
};

pub struct CPU {
    pub pc: u16,
    /// The stack pointer
    ///
    /// When None, there are no values on the stack
    pub sp: usize,
    pub work_regs: [u32; 16],
    pub carry: bool,
    pub zero: bool,

    pub ram: Memory,
    // TODO: It is unclear if this should live in memory or separately, and unclear how large it should be
    pub stack: [u32; 32],

    pub halt: HaltState,

    pub logs: Vec<String>,
}

pub enum HaltState {
    Running,
    Success,
    Failure,
}

enum DataSize {
    Byte,
    Word,
    Long,
}

impl CPU {
    pub fn step(&mut self) {
        if match self.halt {
            HaltState::Running => false,
            _ => true,
        } {
            // Not running, do nothing
            return;
        }

        let inst_word = self.pc_word();
        let [inst_prefix_byte, inst_suffix_byte] = inst_word.to_be_bytes();

        let inst_prefix_upper_nibble = (inst_prefix_byte >> 4) & 0xF;

        let alu_32_immed_bit = (inst_prefix_byte & 0x10) != 0;
        let alu_reg_bit = (inst_prefix_byte & 0x20) != 0;
        let load_immed_bit = (inst_prefix_byte & 0x30) == 0;

        match inst_prefix_byte {
            0x0 => {} // NOP
            0x02 | 0x32 => {
                // ld.b Rx,(nnnn) | ld.b Rx,(Ry)
                self.load_mem_inst(inst_suffix_byte, false, load_immed_bit, DataSize::Byte)
            }
            0x03 | 0x33 => {
                // ld.b (nnnn),Rx | ld.b (Ry),Rx
                self.load_mem_inst(inst_suffix_byte, true, load_immed_bit, DataSize::Byte)
            }
            0x04 | 0x34 => {
                // ld.w Rx,(nnnn) | ld.w Rx,(Ry)
                self.load_mem_inst(inst_suffix_byte, false, load_immed_bit, DataSize::Word)
            }
            0x05 | 0x35 => {
                // ld.w (nnnn),Rx | ld.w (Ry),Rx
                self.load_mem_inst(inst_suffix_byte, true, load_immed_bit, DataSize::Word)
            }
            0x06 | 0x36 => {
                // ld.l Rx,(nnnn) | ld.l Rx,(Ry)
                self.load_mem_inst(inst_suffix_byte, false, load_immed_bit, DataSize::Long)
            }
            0x07 | 0x37 => {
                // ld.l (nnnn),Rx | ld.l (Ry),Rx
                self.load_mem_inst(inst_suffix_byte, true, load_immed_bit, DataSize::Long)
            }
            0x08 | 0x18 | 0x28 => {
                // ld Rx,#16/32 | ld Rx,Ry
                self.alu_immediate_or_reg_inst(
                    inst_suffix_byte,
                    false,
                    alu_reg_bit,
                    alu_32_immed_bit,
                    true,
                    |_, immediate| (immediate, false),
                )
            }
            0x09 | 0x19 | 0x29 => {
                // and Rx,#16/32 | and Rx,Ry
                self.alu_immediate_or_reg_inst(
                    inst_suffix_byte,
                    false,
                    alu_reg_bit,
                    alu_32_immed_bit,
                    true,
                    |reg, immediate| (reg & immediate, false),
                )
            }
            0x0A | 0x1A | 0x2A => {
                // or Rx,#16/32 | or Rx,Ry
                self.alu_immediate_or_reg_inst(
                    inst_suffix_byte,
                    false,
                    alu_reg_bit,
                    alu_32_immed_bit,
                    true,
                    |reg, immediate| (reg | immediate, false),
                )
            }
            0x0B | 0x1B | 0x2B => {
                // xor Rx,#16/32 | xor Rx,Ry
                self.alu_immediate_or_reg_inst(
                    inst_suffix_byte,
                    false,
                    alu_reg_bit,
                    alu_32_immed_bit,
                    true,
                    |reg, immediate| (reg ^ immediate, false),
                )
            }
            0x0C | 0x1C | 0x2C => {
                // add Rx,#16/32 | add Rx,Ry
                self.alu_immediate_or_reg_inst(
                    inst_suffix_byte,
                    true,
                    alu_reg_bit,
                    alu_32_immed_bit,
                    true,
                    |reg, immediate| reg.overflowing_add(immediate),
                )
            }
            0x0D | 0x1D | 0x2D => {
                // sub Rx,#16/32 | sub Rx,Ry
                self.alu_immediate_or_reg_inst(
                    inst_suffix_byte,
                    true,
                    alu_reg_bit,
                    alu_32_immed_bit,
                    true,
                    |reg, immediate| reg.overflowing_sub(immediate),
                )
            }
            0x0E | 0x1E | 0x2E => {
                // cmp Rx,#16/32 | cmp Rx,Ry
                self.alu_immediate_or_reg_inst(
                    inst_suffix_byte,
                    false,
                    alu_reg_bit,
                    alu_32_immed_bit,
                    false,
                    |reg, immediate| reg.overflowing_sub(immediate),
                )
            }
            0x0F | 0x1F | 0x2F => {
                // bit Rx,#16/32 | bit Rx,Ry
                self.alu_immediate_or_reg_inst(
                    inst_suffix_byte,
                    false,
                    alu_reg_bit,
                    alu_32_immed_bit,
                    false,
                    |reg, immediate| (reg & immediate, false),
                )
            }
            0x10 => todo!("RSET"),
            0x11 => todo!("CRC"),
            0x20 => {
                // asl Rx,Ry
                self.alu_double_value_inst(inst_suffix_byte, true, false, |reg_x, reg_y| {
                    // u64 is used to capture if there is carry
                    let long_long = (reg_x as u64).shl(reg_y);
                    (long_long.to_lower_long(), long_long > u32::MAX as u64)
                })
            }
            0x21 => {
                // lsr Rx,Ry
                self.alu_double_value_inst(inst_suffix_byte, true, false, |reg_x, reg_y| {
                    let long = reg_x.shr(reg_y);

                    // Carry is bit at position reg_y - 1
                    let index = reg_x.bit_at_index(reg_y - 1);
                    (long, index)
                })
            }
            0x22 => {
                // rol Rx,Ry
                self.alu_double_value_inst(inst_suffix_byte, true, false, |reg_x, reg_y| {
                    let long = reg_x.shl(reg_y);

                    // Carry is bit at 32 - reg_y
                    let index = reg_x.bit_at_index(32 - reg_y);
                    (long, index)
                })
            }
            0x23 => {
                // ror Rx,Ry
                self.alu_double_value_inst(inst_suffix_byte, true, false, |reg_x, reg_y| {
                    let long = reg_x.shr(reg_y);

                    // Carry is bit at reg_y - 1
                    let index = reg_x.bit_at_index(reg_y - 1);
                    (long, index)
                })
            }
            0x24 => {
                // asl Rx,#
                self.alu_double_value_inst(inst_suffix_byte, true, true, |reg, immediate| {
                    // For some reason the immediate is locked to be >= 1
                    let immediate = immediate + 1;

                    let long_long = (reg as u64).shl(immediate);
                    (long_long.to_lower_long(), long_long > u32::MAX as u64)
                })
            }
            0x25 => {
                // lsr Rx,#
                self.alu_double_value_inst(inst_suffix_byte, true, true, |reg, immediate| {
                    // For some reason the immediate is locked to be >= 1
                    let immediate = immediate + 1;
                    let long = reg.shr(immediate);

                    // Carry is bit at position immediate - 1
                    let index = reg.bit_at_index(immediate - 1);
                    (long, index)
                })
            }
            0x26 => {
                // rol Rx,#
                self.alu_double_value_inst(inst_suffix_byte, true, true, |reg, immediate| {
                    // For some reason the immediate is locked to be >= 1
                    let immediate = immediate + 1;
                    let long = reg.shl(immediate);

                    // Carry is bit at 32 - immediate
                    let index = reg.bit_at_index(32 - immediate);
                    (long, index)
                })
            }
            0x27 => {
                // ror Rx,#
                self.alu_double_value_inst(inst_suffix_byte, true, true, |reg, immediate| {
                    // For some reason the immediate is locked to be >= 1
                    let immediate = immediate + 1;
                    let long = reg.shr(immediate);

                    // Carry is bit at immediate - 1
                    let index = reg.bit_at_index(immediate - 1);
                    (long, index)
                })
            }
            0x38 => {
                // mul Rx,Ry
                self.alu_double_value_inst(inst_suffix_byte, false, false, |reg_x, reg_y| {
                    reg_x.overflowing_mul(reg_y)
                })
            }
            0x39..=0x3D => todo!("{inst_prefix_byte}"),
            0x3E => {
                // div Rx,Ry
                let reg_x_index = inst_suffix_byte & 0xF;
                let reg_y_index = (inst_suffix_byte >> 4) & 0xF;

                let reg_x = self.get_reg(reg_x_index);
                let reg_y = self.get_reg(reg_y_index);

                if reg_y == 0 {
                    // Divide by 0
                    self.jump_to_error();
                } else {
                    let quotient = reg_x / reg_y;
                    let remainder = reg_x % reg_y;

                    self.set_reg(reg_x_index, quotient);
                    self.set_reg(reg_y_index, remainder);

                    self.set_zero(quotient);
                    self.set_carry(remainder == 0);
                }
            }
            0x40 => {
                // printf Rx
                let reg_x_index = inst_suffix_byte & 0xF;
                let address = self.get_reg(reg_x_index).to_lower_word();

                let mut string_bytes = Vec::new();

                let mut count = 1;
                let mut byte = self.ram.mem_read_byte(address);
                // Max at 255 chars, and stop at nullchar
                while count < 256 && byte != 0 {
                    string_bytes.push(byte);

                    byte = self.ram.mem_read_byte(address + count);
                    count += 1;
                }

                let string = String::from_utf8(string_bytes);

                let string = string.map_or(
                    format!("Error: Could not parse printed string at {address:#X}"),
                    |s| s,
                );

                self.logs.push(string);
            }
            0x41 => {
                // hex.* Rx | dec.* Rx
                let reg_x_index = inst_suffix_byte & 0xF;
                let identifier = (inst_suffix_byte >> 4) & 0xF;

                let reg_x = self.get_reg(reg_x_index);

                let string = match identifier {
                    // hex
                    0 => {
                        let byte = reg_x.to_le_bytes()[0];
                        format!("{byte:02X}")
                    }
                    1 => {
                        let word = reg_x.to_lower_word();
                        format!("{word:04X}")
                    }
                    2 => {
                        format!("{reg_x:08X}")
                    }
                    // dec
                    3 => {
                        let byte = reg_x.to_le_bytes()[0];
                        format!("{byte:02}")
                    }
                    4 => {
                        let word = reg_x.to_lower_word();
                        format!("{word:04}")
                    }
                    5 => {
                        format!("{reg_x:08}")
                    }
                    _ => panic!("Unexpected identifier {identifier} in 0x41"),
                };

                let string = if identifier < 3 {
                    format!("Hex: 0x{string}")
                } else {
                    format!("Dec: {string}")
                };

                self.logs.push(string);
            }
            0x42 => {
                // ret *
                let identifier = inst_suffix_byte & 0xF;

                self.return_inst(|zero, carry| match identifier {
                    0 => true,   // ret
                    1 => !zero,  // ret NZ
                    2 => zero,   // ret Z
                    3 => !carry, // ret NC
                    4 => carry,  // ret C
                    _ => panic!("Unexpected identifier {identifier} in 0x42"),
                })
            }
            0x43 => {
                // push Rx
                let reg_x_index = inst_suffix_byte & 0xF;

                self.stack[self.sp] = self.get_reg(reg_x_index);

                self.sp += 1;
            }
            0x44 => {
                // pop Rx
                // SP must be >= 1
                if self.sp == 0 {
                    // Error
                    return self.jump_to_error();
                }

                let reg_x_index = inst_suffix_byte & 0xF;

                self.sp -= 1;

                self.set_reg(reg_x_index, self.stack[self.sp]);
            }
            // TODO: 0x45 ERR
            0x46 => {
                // exit
                let identifier = inst_suffix_byte & 0xF;

                match identifier {
                    0 => HaltState::Success,
                    1 => HaltState::Failure,
                    _ => panic!("Unknown identifier {identifier} for 0x46"),
                };
            }
            0x47 => {
                // clc/sec
                let identifier = inst_suffix_byte & 0xF;

                match identifier {
                    0 => self.set_carry(false),
                    1 => self.set_carry(true),
                    _ => panic!("Unknown identifier {identifier} for 0x47"),
                };
            }
            _ => {
                match inst_prefix_upper_nibble {
                    0x6..=0xA => {
                        self.jump_inst(inst_prefix_byte, inst_suffix_byte, |zero, carry| {
                            match inst_prefix_upper_nibble {
                                0x6 => true,   // jp n
                                0x7 => !zero,  // jp nz
                                0x8 => zero,   // jp z
                                0x9 => !carry, // jp nc
                                0xA => carry,  // jp c
                                _ => unreachable!(),
                            }
                        })
                    }
                    _ => {
                        // Do nothing
                        todo!("Unimplemented {inst_prefix_byte:#X}")
                    }
                }
            }
        }
    }

    ///
    /// A load/store instruction. Can be `b`, `w`, or `l`
    ///
    /// `second_value_is_immed` considers the Y value to be an immediate word, otherwise it's a register
    fn load_mem_inst(
        &mut self,
        inst_suffix_byte: u8,
        write_mem: bool,
        second_value_is_immed: bool,
        size: DataSize,
    ) {
        let reg_x_index = inst_suffix_byte & 0xF;
        let reg_y_index = (inst_suffix_byte >> 4) & 0xF;

        let reg_x = self.get_reg(reg_x_index);

        let address = if second_value_is_immed {
            self.pc_word()
        } else {
            self.get_reg(reg_y_index).to_lower_word()
        };

        if write_mem {
            match size {
                DataSize::Byte => self.ram.mem_write_byte(address, reg_x.to_le_bytes()[0]),
                DataSize::Word => self.ram.mem_write_word(address, reg_x.to_lower_word()),
                DataSize::Long => self.ram.mem_write_long(address, reg_x),
            }
        } else {
            let value = match size {
                DataSize::Byte => self.ram.mem_read_byte(address).into(),
                DataSize::Word => self.ram.mem_read_word(address).into(),
                DataSize::Long => self.ram.mem_read_long(address),
            };

            self.set_reg(reg_x_index, value);
            self.set_zero(value);
        }
    }

    ///
    /// ALU load/logic with immediate or register second argument
    ///
    /// `bit32_immed` only applies if second argument is immediate
    fn alu_immediate_or_reg_inst<T: Fn(u32, u32) -> (u32, bool)>(
        &mut self,
        inst_suffix_byte: u8,
        set_carry: bool,
        is_register: bool,
        bit32_immed: bool,
        save_output: bool,
        operation: T,
    ) {
        let (value, reg_x_index) = if is_register {
            // Register
            let reg_y_index = (inst_suffix_byte >> 4) & 0xF;
            (self.get_reg(reg_y_index), inst_suffix_byte & 0xF)
        } else {
            // Immediate
            if bit32_immed {
                (self.pc_long(), inst_suffix_byte)
            } else {
                (self.pc_word() as u32, inst_suffix_byte)
            }
        };

        let (value, carry) = operation(self.get_reg(reg_x_index), value);

        if save_output {
            self.set_reg(reg_x_index, value);
        }
        if set_carry {
            self.set_carry(carry);
        }
        self.set_zero(value);
    }

    ///
    /// An ALU instruction with two packed values into the instruction word (**XY)
    ///
    /// `second_value_is_immed` considers the Y value to be an immediate, otherwise it's a register
    fn alu_double_value_inst<T: Fn(u32, u32) -> (u32, bool)>(
        &mut self,
        inst_suffix_byte: u8,
        set_carry: bool,
        second_value_is_immed: bool,
        operation: T,
    ) {
        let reg_x_index = inst_suffix_byte & 0xF;
        let reg_y_index = (inst_suffix_byte >> 4) & 0xF;

        let reg_x = self.get_reg(reg_x_index);
        let value_y = if second_value_is_immed {
            reg_y_index as u32
        } else {
            self.get_reg(reg_y_index)
        };

        let (value, carry) = operation(reg_x, value_y);

        self.set_reg(reg_x_index, value);
        if set_carry {
            self.set_carry(carry);
        }
        self.set_zero(value);
    }

    ///
    /// A return instruction. The return is performed if the conditional is true
    ///
    fn return_inst<T: Fn(bool, bool) -> bool>(&mut self, conditional: T) {
        if conditional(self.zero, self.carry) {
            // Should return
            // SP must be >= 1
            if self.sp == 0 {
                // Error
                return self.jump_to_error();
            }

            self.sp -= 1;

            let pointed = self.stack[self.sp].to_lower_word() & 0x1FFF;

            self.pc = pointed;
        }
    }

    fn jump_inst<T: Fn(bool, bool) -> bool>(
        &mut self,
        inst_prefix_byte: u8,
        inst_suffix_byte: u8,
        conditional: T,
    ) {
        if conditional(self.zero, self.carry) {
            // Should jump
            let inst_prefix_byte = inst_prefix_byte & 0xF;
            let highest_nibble = ((inst_prefix_byte as u16) << 4) & 0xF00;

            let address = highest_nibble | (inst_suffix_byte as u16);
            self.pc = address * 2;
        }
    }

    // Util

    fn jump_to_error(&mut self) {
        self.pc = 0;
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
            sp: 0,
            work_regs: [0; 16],
            carry: false,
            zero: false,
            ram: Memory::from_bytes(buffer),
            stack: [0; 32],
            halt: HaltState::Running,
            logs: Vec::new(),
        })
    }
}
