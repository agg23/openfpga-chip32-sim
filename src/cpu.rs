use std::{
    fmt::Display,
    fs::File,
    io::{self, Read},
    ops::{Shl, Shr},
};

use crate::{
    apf::DataSlot,
    mem::Memory,
    util::{
        bitwise::BitIndex,
        num::{LowerLong, LowerWord},
    },
};

#[derive(Clone)]
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

    pub file_state: FileState,

    pub halt: HaltState,

    pub formatted_instruction: String,
    pub logs: Vec<String>,
}

#[derive(Clone)]
pub struct FileState {
    pub slots: Vec<DataSlot>,

    pub loaded: FileLoadedState,
}

#[derive(Clone)]
pub enum HaltState {
    Running,
    Success,
    Failure,
}

#[derive(Clone)]
pub enum FileLoadedState {
    None,
    Loaded {
        slot: u32,
        data: Vec<u8>,
        offset: usize,
    },
}

enum DataSize {
    Byte,
    Word,
    Long,
}

impl Display for DataSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DataSize::Byte => ".b",
            DataSize::Word => ".w",
            DataSize::Long => ".l",
        })
    }
}

enum InstructionKind {
    SingleReg {
        x: u8,
        size: Option<DataSize>,
    },
    DoubleReg {
        x: u8,
        y: u8,
        /// If Some, the direction a memory access flows
        ///
        /// If None, no memory access
        mem_direction_into_reg: Option<bool>,
        size: Option<DataSize>,
    },
    Immediate {
        x: u8,
        n: u32,
        size: Option<DataSize>,
    },
    RegMem {
        x: u8,
        address: u16,
        direction_into_reg: bool,
        size: DataSize,
    },
    Jump {
        immediate: Option<u16>,
        always: bool,
        zero: Option<bool>,
        carry: Option<bool>,
    },
    None,
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

        self.formatted_instruction = String::new();

        let inst_word = self.pc_word();
        let [inst_prefix_byte, inst_suffix_byte] = inst_word.to_be_bytes();

        let inst_prefix_upper_nibble = (inst_prefix_byte >> 4) & 0xF;
        let reg_x_index = inst_suffix_byte & 0xF;
        let reg_y_index = (inst_suffix_byte >> 4) & 0xF;

        let alu_32_immed_bit = (inst_prefix_byte & 0x10) != 0;
        let alu_reg_bit = (inst_prefix_byte & 0x20) != 0;
        let load_immed_bit = (inst_prefix_byte & 0x30) == 0;

        match inst_prefix_byte {
            0x0 => {
                // nop
                self.formatted_instruction = format!("nop")
            }
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
                    "ld",
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
                    "and",
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
                    "or",
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
                    "xor",
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
                    "add",
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
                    "sub",
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
                    "cmp",
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
                    "bit",
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
                self.alu_double_value_inst("asl", inst_suffix_byte, true, false, |reg_x, reg_y| {
                    // u64 is used to capture if there is carry
                    let long_long = (reg_x as u64).shl(reg_y);
                    (long_long.to_lower_long(), long_long > u32::MAX as u64)
                })
            }
            0x21 => {
                // lsr Rx,Ry
                self.alu_double_value_inst("lsr", inst_suffix_byte, true, false, |reg_x, reg_y| {
                    let long = reg_x.shr(reg_y);

                    // Carry is bit at position reg_y - 1
                    let index = reg_x.bit_at_index(reg_y - 1);
                    (long, index)
                })
            }
            0x22 => {
                // rol Rx,Ry
                self.alu_double_value_inst("rol", inst_suffix_byte, true, false, |reg_x, reg_y| {
                    let long = reg_x.shl(reg_y);

                    // Carry is bit at 32 - reg_y
                    let index = reg_x.bit_at_index(32 - reg_y);
                    (long, index)
                })
            }
            0x23 => {
                // ror Rx,Ry
                self.alu_double_value_inst("ror", inst_suffix_byte, true, false, |reg_x, reg_y| {
                    let long = reg_x.shr(reg_y);

                    // Carry is bit at reg_y - 1
                    let index = reg_x.bit_at_index(reg_y - 1);
                    (long, index)
                })
            }
            0x24 => {
                // asl Rx,#
                self.alu_double_value_inst("asl", inst_suffix_byte, true, true, |reg, immediate| {
                    // For some reason the immediate is locked to be >= 1
                    let immediate = immediate + 1;

                    let long_long = (reg as u64).shl(immediate);
                    (long_long.to_lower_long(), long_long > u32::MAX as u64)
                })
            }
            0x25 => {
                // lsr Rx,#
                self.alu_double_value_inst("lsr", inst_suffix_byte, true, true, |reg, immediate| {
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
                self.alu_double_value_inst("rol", inst_suffix_byte, true, true, |reg, immediate| {
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
                self.alu_double_value_inst("ror", inst_suffix_byte, true, true, |reg, immediate| {
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
                self.alu_double_value_inst("mul", inst_suffix_byte, false, false, |reg_x, reg_y| {
                    reg_x.overflowing_mul(reg_y)
                })
            }
            0x39..=0x3D => todo!("{inst_prefix_byte}"),
            0x3E => {
                // div Rx,Ry
                let reg_x = self.get_reg(reg_x_index);
                let reg_y = self.get_reg(reg_y_index);

                if reg_y == 0 {
                    // Divide by 0
                    self.logs.push(format!("Sim: Div by 0"));

                    self.jump_to_error();
                } else {
                    let quotient = reg_x / reg_y;
                    let remainder = reg_x % reg_y;

                    self.set_reg(reg_x_index, quotient);
                    self.set_reg(reg_y_index, remainder);

                    self.set_zero(quotient);
                    self.set_carry(remainder == 0);
                }

                self.set_instruction_string(
                    "div",
                    InstructionKind::DoubleReg {
                        x: reg_x_index,
                        y: reg_y_index,
                        mem_direction_into_reg: None,
                        size: None,
                    },
                )
            }
            0x40 => {
                // printf Rx
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

                self.set_instruction_string(
                    "printf",
                    InstructionKind::SingleReg {
                        x: reg_x_index,
                        size: None,
                    },
                );
            }
            0x41 => {
                // hex.* Rx | dec.* Rx
                let identifier = reg_y_index;

                let reg_x = self.get_reg(reg_x_index);

                let (string, size) = match identifier {
                    // hex
                    0 => {
                        let byte = reg_x.to_le_bytes()[0];
                        (format!("{byte:02X}"), DataSize::Byte)
                    }
                    1 => {
                        let word = reg_x.to_lower_word();
                        (format!("{word:04X}"), DataSize::Word)
                    }
                    2 => (format!("{reg_x:08X}"), DataSize::Long),
                    // dec
                    3 => {
                        let byte = reg_x.to_le_bytes()[0];
                        (format!("{byte:02}"), DataSize::Byte)
                    }
                    4 => {
                        let word = reg_x.to_lower_word();
                        (format!("{word:04}"), DataSize::Word)
                    }
                    5 => (format!("{reg_x:08}"), DataSize::Long),
                    _ => panic!("Unexpected identifier {identifier} in 0x41"),
                };

                let string = if identifier < 3 {
                    format!("Hex: 0x{string}")
                } else {
                    format!("Dec: {string}")
                };

                self.logs.push(string);

                self.set_instruction_string(
                    if identifier < 3 { "hex" } else { "dec" },
                    InstructionKind::SingleReg {
                        x: reg_x_index,
                        size: Some(size),
                    },
                );
            }
            0x42 => {
                // ret *
                let identifier = reg_x_index;

                self.return_inst(|zero, carry| match identifier {
                    0 => true,   // ret
                    1 => !zero,  // ret NZ
                    2 => zero,   // ret Z
                    3 => !carry, // ret NC
                    4 => carry,  // ret C
                    _ => panic!("Unexpected identifier {identifier} in 0x42"),
                });

                self.set_instruction_string(
                    "ret",
                    match identifier {
                        0 => InstructionKind::Jump {
                            immediate: None,
                            always: true,
                            zero: None,
                            carry: None,
                        },
                        1 => InstructionKind::Jump {
                            immediate: None,
                            always: false,
                            zero: Some(false),
                            carry: None,
                        },
                        2 => InstructionKind::Jump {
                            immediate: None,
                            always: false,
                            zero: Some(true),
                            carry: None,
                        },
                        3 => InstructionKind::Jump {
                            immediate: None,
                            always: false,
                            zero: None,
                            carry: Some(false),
                        },
                        4 => InstructionKind::Jump {
                            immediate: None,
                            always: false,
                            zero: None,
                            carry: Some(true),
                        },
                        _ => unreachable!(),
                    },
                )
            }
            0x43 => {
                // push Rx
                let reg_x_index = reg_x_index;

                self.stack[self.sp] = self.get_reg(reg_x_index);

                self.sp += 1;

                self.set_instruction_string(
                    "push",
                    InstructionKind::SingleReg {
                        x: reg_x_index,
                        size: None,
                    },
                )
            }
            0x44 => {
                // pop Rx
                self.set_instruction_string(
                    "pop",
                    InstructionKind::SingleReg {
                        x: reg_x_index,
                        size: None,
                    },
                );

                // SP must be >= 1
                if self.sp == 0 {
                    // Error
                    self.logs.push(format!("Sim: Stack underflow"));

                    return self.jump_to_error();
                }

                self.sp -= 1;

                self.set_reg(reg_x_index, self.stack[self.sp]);
            }
            // TODO: 0x45 ERR
            0x46 => {
                // exit
                let identifier = reg_x_index;

                self.halt = match identifier {
                    0 => HaltState::Success,
                    1 => HaltState::Failure,
                    _ => panic!("Unknown identifier {identifier} for 0x46"),
                };

                self.logs.push(format!("Sim: Halted with {identifier}"));

                self.set_instruction_string(
                    "exit",
                    InstructionKind::SingleReg {
                        x: identifier,
                        size: None,
                    },
                );
            }
            0x47 => {
                // clc/sec
                let identifier = reg_x_index;

                match identifier {
                    0 => self.set_carry(false),
                    1 => self.set_carry(true),
                    _ => panic!("Unknown identifier {identifier} for 0x47"),
                };

                self.set_instruction_string(
                    if identifier == 0 { "clc" } else { "sec" },
                    InstructionKind::None,
                );
            }
            0x56 => {
                // open Rx,Ry
                self.set_instruction_string(
                    "open",
                    InstructionKind::DoubleReg {
                        x: reg_x_index,
                        y: reg_y_index,
                        mem_direction_into_reg: None,
                        size: None,
                    },
                );

                if let FileLoadedState::Loaded { slot, .. } = self.file_state.loaded {
                    // File already open, error
                    self.logs
                        .push(format!("Sim: A file (slot {slot}) is already open"));

                    return self.jump_to_error();
                }

                let reg_x = self.get_reg(reg_x_index);

                if let Some(slot) = self.file_state.slots.iter().find(|s| s.id == reg_x) {
                    let file_content = file_to_buffer(&slot.filename);

                    if let Ok(data) = file_content {
                        // File successfully loaded
                        let len = data.len() as u32;

                        self.file_state.loaded = FileLoadedState::Loaded {
                            slot: reg_x,
                            data,
                            offset: 0,
                        };

                        // Set Ry to size
                        self.set_reg(reg_y_index, len);

                        self.zero = true;
                    } else {
                        // File could not be loaded, set error
                        self.zero = false;
                        self.set_reg(reg_y_index, 0);

                        self.logs.push(format!("Sim: File could not be loaded"));
                    }
                } else {
                    // No slot found, set error
                    self.zero = false;
                    self.set_reg(reg_y_index, 0);

                    self.logs.push(format!("Sim: Slot {reg_x} not found"));
                }
            }
            0x57 => {
                // close
                self.set_instruction_string("close", InstructionKind::None);

                if match self.file_state.loaded {
                    FileLoadedState::Loaded { .. } => false,
                    _ => true,
                } {
                    // No file loaded, throw error
                    self.logs
                        .push(format!("Sim: Attempted to close when no open file exists"));

                    return self.jump_to_error();
                }

                self.file_state.loaded = FileLoadedState::None;
            }
            0x58 => {
                // seek Rx
                self.set_instruction_string(
                    "seek",
                    InstructionKind::SingleReg {
                        x: reg_x_index,
                        size: None,
                    },
                );

                let reg_x = self.get_reg(reg_x_index) as usize;

                if let FileLoadedState::Loaded {
                    data,
                    ref mut offset,
                    ..
                } = &mut self.file_state.loaded
                {
                    if reg_x > data.len() {
                        // Attempted to seek past end of file
                        self.logs
                            .push(format!("Sim: Attempted to seek past end of file"));

                        self.zero = false;
                        return;
                    }

                    *offset = reg_x;
                    self.zero = true;
                } else {
                    // No open file, throw error
                    self.logs
                        .push(format!("Sim: Attempted to seek when no open file exists"));

                    return self.jump_to_error();
                }
            }
            0x59 => {
                // read Rx,Ry
                self.set_instruction_string(
                    "read",
                    InstructionKind::DoubleReg {
                        x: reg_x_index,
                        y: reg_y_index,
                        mem_direction_into_reg: None,
                        size: None,
                    },
                );

                let reg_x = self.get_reg(reg_x_index) as usize;
                let reg_y = self.get_reg(reg_y_index) as usize;

                if let FileLoadedState::Loaded {
                    data,
                    ref mut offset,
                    ..
                } = &mut self.file_state.loaded
                {
                    if reg_y > 4 * 1024 {
                        //  Can't load more than 4K at once
                        self.zero = false;

                        self.logs
                            .push(format!("Sim: Attempted to read more than 4K bytes"));
                        return;
                    } else if reg_y + *offset > data.len() {
                        // Can't load past end of file
                        self.zero = false;

                        self.logs
                            .push(format!("Sim: Attempted to read past end of file"));
                        return;
                    }

                    for i in 0..reg_y {
                        let byte = data[*offset + i];
                        self.ram.mem_write_byte((reg_x + i).to_lower_word(), byte);
                    }

                    self.zero = true;
                } else {
                    // No open file, throw error
                    self.logs
                        .push(format!("Sim: Attempted to read when no open file exists"));

                    return self.jump_to_error();
                }
            }
            _ => {
                match inst_prefix_upper_nibble {
                    0x6..=0xA => {
                        self.jump_inst(
                            inst_prefix_byte,
                            inst_suffix_byte,
                            inst_prefix_upper_nibble,
                            true,
                            |zero, carry| {
                                match inst_prefix_upper_nibble {
                                    0x6 => true,   // jp n
                                    0x7 => !zero,  // jp nz
                                    0x8 => zero,   // jp z
                                    0x9 => !carry, // jp nc
                                    0xA => carry,  // jp c
                                    _ => unreachable!(),
                                }
                            },
                        );
                    }
                    0xB..=0xF => {
                        self.call_inst(inst_prefix_byte, inst_suffix_byte, |zero, carry| {
                            match inst_prefix_upper_nibble {
                                0xB => true,   // call n
                                0xC => !zero,  // call nz,n
                                0xD => zero,   // call z,n
                                0xE => !carry, // call nz,n
                                0xF => carry,  // call z,n
                                _ => unreachable!(),
                            }
                        });

                        self.set_instruction_string(
                            "call",
                            match inst_prefix_upper_nibble {
                                0xB => InstructionKind::Jump {
                                    immediate: None,
                                    always: true,
                                    zero: None,
                                    carry: None,
                                },
                                0xC => InstructionKind::Jump {
                                    immediate: None,
                                    always: false,
                                    zero: Some(false),
                                    carry: None,
                                },
                                0xD => InstructionKind::Jump {
                                    immediate: None,
                                    always: false,
                                    zero: Some(true),
                                    carry: None,
                                },
                                0xE => InstructionKind::Jump {
                                    immediate: None,
                                    always: false,
                                    zero: None,
                                    carry: Some(false),
                                },
                                0xF => InstructionKind::Jump {
                                    immediate: None,
                                    always: false,
                                    zero: None,
                                    carry: Some(true),
                                },
                                _ => unreachable!(),
                            },
                        );
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

        self.set_instruction_string(
            "ld",
            if second_value_is_immed {
                InstructionKind::RegMem {
                    x: reg_x_index,
                    address,
                    direction_into_reg: !write_mem,
                    size,
                }
            } else {
                InstructionKind::DoubleReg {
                    x: reg_x_index,
                    y: reg_y_index,
                    mem_direction_into_reg: Some(!write_mem),
                    size: Some(size),
                }
            },
        )
    }

    ///
    /// ALU load/logic with immediate or register second argument
    ///
    /// `bit32_immed` only applies if second argument is immediate
    fn alu_immediate_or_reg_inst<T: Fn(u32, u32) -> (u32, bool)>(
        &mut self,
        inst_name: &str,
        inst_suffix_byte: u8,
        set_carry: bool,
        is_register: bool,
        bit32_immed: bool,
        save_output: bool,
        operation: T,
    ) {
        let reg_y_index = (inst_suffix_byte >> 4) & 0xF;

        let (input_value, reg_x_index) = if is_register {
            // Register
            (self.get_reg(reg_y_index), inst_suffix_byte & 0xF)
        } else {
            // Immediate
            if bit32_immed {
                (self.pc_long(), inst_suffix_byte)
            } else {
                (self.pc_word() as u32, inst_suffix_byte)
            }
        };

        let (value, carry) = operation(self.get_reg(reg_x_index), input_value);

        if save_output {
            self.set_reg(reg_x_index, value);
        }
        if set_carry {
            self.set_carry(carry);
        }
        self.set_zero(value);

        self.set_instruction_string(
            inst_name,
            if is_register {
                InstructionKind::DoubleReg {
                    x: reg_x_index,
                    y: reg_y_index,
                    mem_direction_into_reg: None,
                    size: None,
                }
            } else {
                InstructionKind::Immediate {
                    x: reg_x_index,
                    n: input_value,
                    size: None,
                }
            },
        )
    }

    ///
    /// An ALU instruction with two packed values into the instruction word (**XY)
    ///
    /// `second_value_is_immed` considers the Y value to be an immediate, otherwise it's a register
    fn alu_double_value_inst<T: Fn(u32, u32) -> (u32, bool)>(
        &mut self,
        inst_name: &str,
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

        self.set_instruction_string(
            inst_name,
            if second_value_is_immed {
                InstructionKind::Immediate {
                    x: reg_x_index,
                    n: reg_y_index as u32,
                    size: None,
                }
            } else {
                InstructionKind::DoubleReg {
                    x: reg_x_index,
                    y: reg_y_index,
                    mem_direction_into_reg: None,
                    size: None,
                }
            },
        )
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
                self.logs.push(format!("Sim: Stack underflow"));

                return self.jump_to_error();
            }

            self.sp -= 1;

            let pointed = self.stack[self.sp].to_lower_word() & 0x1FFF;

            self.pc = pointed;
        }
    }

    ///
    /// A jump instruction. The jump is performed if the conditional is true
    ///
    fn jump_inst<T: Fn(bool, bool) -> bool>(
        &mut self,
        inst_prefix_byte: u8,
        inst_suffix_byte: u8,
        inst_prefix_upper_nibble: u8,
        log_instruction: bool,
        conditional: T,
    ) {
        let inst_prefix_byte = inst_prefix_byte & 0xF;
        let highest_nibble = ((inst_prefix_byte as u16) << 4) & 0xF00;

        let address = highest_nibble | (inst_suffix_byte as u16);
        let address = address * 2;

        if conditional(self.zero, self.carry) {
            // Should jump
            self.pc = address;
        }

        if log_instruction {
            self.set_instruction_string(
                "jp",
                match inst_prefix_upper_nibble {
                    0x6 => InstructionKind::Jump {
                        immediate: Some(address),
                        always: true,
                        zero: None,
                        carry: None,
                    },
                    0x7 => InstructionKind::Jump {
                        immediate: Some(address),
                        always: false,
                        zero: Some(false),
                        carry: None,
                    },
                    0x8 => InstructionKind::Jump {
                        immediate: Some(address),
                        always: false,
                        zero: Some(true),
                        carry: None,
                    },
                    0x9 => InstructionKind::Jump {
                        immediate: Some(address),
                        always: false,
                        zero: None,
                        carry: Some(false),
                    },
                    0xA => InstructionKind::Jump {
                        immediate: Some(address),
                        always: false,
                        zero: None,
                        carry: Some(true),
                    },
                    _ => unreachable!(),
                },
            );
        }
    }

    ///
    /// A call instruction. The call is performed if the conditional is true
    ///
    fn call_inst<T: Fn(bool, bool) -> bool>(
        &mut self,
        inst_prefix_byte: u8,
        inst_suffix_byte: u8,
        conditional: T,
    ) {
        if conditional(self.zero, self.carry) {
            // Should return
            // SP must be < 31
            if self.sp >= 31 {
                // Error
                self.logs.push(format!("Sim: Stack overflow"));

                return self.jump_to_error();
            }

            self.stack[self.sp] = self.pc as u32;

            self.sp += 1;

            self.jump_inst(inst_prefix_byte, inst_suffix_byte, 0, false, |_, _| true);
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

    // Debugging

    fn set_instruction_string(&mut self, inst_name: &str, kind: InstructionKind) {
        // This is replicated from the beginning of step
        // let [inst_prefix_byte, inst_suffix_byte] = inst_word.to_be_bytes();

        // let inst_prefix_upper_nibble = (inst_prefix_byte >> 4) & 0xF;
        // let reg_x_index = inst_suffix_byte & 0xF;
        // let reg_y_index = (inst_suffix_byte >> 4) & 0xF;
        let string = match kind {
            InstructionKind::SingleReg { x, size } => {
                let size = format_size(size);

                format!("{inst_name}{size} R{x}")
            }
            InstructionKind::DoubleReg {
                x,
                y,
                mem_direction_into_reg,
                size,
            } => {
                let size = format_size(size);

                if let Some(mem_direction_into_reg) = mem_direction_into_reg {
                    // Memory write
                    if mem_direction_into_reg {
                        format!("{inst_name}{size} R{x},(R{y})")
                    } else {
                        format!("{inst_name}{size} (R{y}),R{x}")
                    }
                } else {
                    // Reg to reg
                    format!("{inst_name}{size} R{x},R{y}")
                }
            }
            InstructionKind::Immediate { x, n, size } => {
                let size = format_size(size);

                format!("{inst_name}{size} R{x},#{n:#X}")
            }
            InstructionKind::RegMem {
                x,
                address,
                direction_into_reg,
                size,
            } => {
                if direction_into_reg {
                    format!("{inst_name}{size} R{x},({address:#X})")
                } else {
                    format!("{inst_name}{size} ({address:#X}),R{x}")
                }
            }
            InstructionKind::Jump {
                immediate,
                always,
                zero,
                carry,
            } => {
                let modifier = if always {
                    ""
                } else if let Some(zero) = zero {
                    if zero {
                        "z "
                    } else {
                        "nz "
                    }
                } else if let Some(carry) = carry {
                    if carry {
                        "c "
                    } else {
                        "nc "
                    }
                } else {
                    unreachable!()
                };

                let immediate = if let Some(immediate) = immediate {
                    format!("#{immediate:#X}")
                } else {
                    "".into()
                };

                format!("{inst_name} {modifier}{immediate}").trim().into()
            }
            InstructionKind::None => inst_name.into(),
        };

        self.formatted_instruction = string;
    }

    // Loading

    pub fn load_file(path_str: &str, data_slots: Option<Vec<DataSlot>>) -> Result<Self, io::Error> {
        let buffer = file_to_buffer(path_str)?;

        let data_slots = if let Some(slots) = data_slots {
            slots
        } else {
            Vec::new()
        };

        Ok(CPU {
            pc: 0x2,
            sp: 0,
            work_regs: [0; 16],
            carry: false,
            zero: false,
            ram: Memory::from_bytes(buffer),
            stack: [0; 32],
            file_state: FileState {
                slots: data_slots,
                loaded: FileLoadedState::None,
            },
            halt: HaltState::Running,
            formatted_instruction: String::new(),
            logs: Vec::new(),
        })
    }
}

fn file_to_buffer(path_str: &str) -> Result<Vec<u8>, io::Error> {
    let mut file = File::open(path_str)?;

    let mut buffer = Vec::<u8>::new();

    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn format_size(size: Option<DataSize>) -> String {
    if let Some(size) = size {
        format!("{size}")
    } else {
        "".into()
    }
}
