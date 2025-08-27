pub const MAGIC: &[u8] = b"FTRACE\x00\x01";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RawFtrace(u64);

impl RawFtrace {
    pub fn depth(&self) -> u8 {
        (self.0 >> 56) as u8
    }

    pub fn data(&self) -> u64 {
        self.0 & 0x00FF_FFFF_FFFF_FFFF
    }

    pub fn is_start(&self) -> bool {
        self.depth() != 0
    }

    pub fn is_end(&self) -> bool {
        self.depth() == 0
    }
}

impl From<u64> for RawFtrace {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<RawFtrace> for u64 {
    fn from(value: RawFtrace) -> Self {
        value.0
    }
}
