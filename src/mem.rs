#[derive(Clone)]
pub struct Memory {
    ram: [u8; 8 * 1024],
    rom_size: usize,
}

impl Memory {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let mut ram = [0; 8 * 1024];

        bytes
            .iter()
            .enumerate()
            .for_each(|(i, byte)| ram[i] = *byte);

        Memory {
            ram,
            rom_size: bytes.len(),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;

        self.ram[address]
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let address = address as usize;

        u16::from_le_bytes(
            self.ram[address..address + 2]
                .try_into()
                .expect("Slice wasn't of length 2"),
        )
    }

    pub fn read_long(&self, address: u16) -> u32 {
        let address = address as usize;

        u32::from_le_bytes(
            self.ram[address..address + 4]
                .try_into()
                .expect("Slice wasn't of length 4"),
        )
    }

    // TODO: Log message when you clobber the ROM data
    pub fn write_byte(&mut self, address: u16, byte: u8) {
        let address = address as usize;

        if address < self.rom_size {
            println!("ERROR: Clobbering ROM data");
        }

        self.ram[address] = byte;
    }

    pub fn write_word(&mut self, address: u16, word: u16) {
        let address = address as usize;

        if address < self.rom_size {
            println!("ERROR: Clobbering ROM data");
        }

        let [lower, upper] = word.to_le_bytes();

        self.ram[address] = lower;
        self.ram[address + 1] = upper;
    }

    pub fn write_long(&mut self, address: u16, word: u32) {
        let address = address as usize;

        if address < self.rom_size {
            println!("ERROR: Clobbering ROM data");
        }

        let [lower_a, upper_a, lower_b, upper_b] = word.to_le_bytes();

        self.ram[address] = lower_a;
        self.ram[address + 1] = upper_a;
        self.ram[address + 2] = lower_b;
        self.ram[address + 3] = upper_b;
    }
}
