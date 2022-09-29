pub trait LowerWord {
    fn to_lower_word(self) -> u16;
}

impl LowerWord for u32 {
    fn to_lower_word(self) -> u16 {
        let bytes = self.to_le_bytes();

        (bytes[1] as u16) << 8 | (bytes[0] as u16)
    }
}
