pub trait BitIndex {
    fn bit_at_index(self, index: Self) -> bool;
}

impl BitIndex for u32 {
    fn bit_at_index(self, index: Self) -> bool {
        self & (1 << index) != 0
    }
}

impl BitIndex for u64 {
    fn bit_at_index(self, index: Self) -> bool {
        self & (1 << index) != 0
    }
}
