pub trait LowerWord {
    fn to_lower_word(self) -> u16;
}

impl LowerWord for u32 {
    fn to_lower_word(self) -> u16 {
        let bytes = self.to_le_bytes();

        (bytes[1] as u16) << 8 | (bytes[0] as u16)
    }
}

pub trait LowerLong {
    fn to_lower_long(self) -> u32;
}

impl LowerLong for u64 {
    fn to_lower_long(self) -> u32 {
        let bytes = self.to_le_bytes();

        (bytes[3] as u32) << 24
            | (bytes[2] as u32) << 16
            | (bytes[1] as u32) << 8
            | (bytes[0] as u32)
    }
}
