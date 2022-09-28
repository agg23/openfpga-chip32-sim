pub struct Memory {
    ram: [u8; 8 * 1024],
}

impl Memory {
    pub fn mem_read_byte(&self, address: u16) -> u8 {
        let address = address as usize;

        self.ram[address]
    }

    pub fn mem_read_word(&self, address: u16) -> u16 {
        let address = address as usize;

        u16::from_le_bytes(
            self.ram[address..address + 2]
                .try_into()
                .expect("Slice wasn't of length 2"),
        )
    }

    pub fn mem_read_long(&self, address: u16) -> u32 {
        let address = address as usize;

        u32::from_le_bytes(
            self.ram[address..address + 4]
                .try_into()
                .expect("Slice wasn't of length 4"),
        )
    }
}
