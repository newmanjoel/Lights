#[derive(Debug)]
pub struct ByteRGB {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl ByteRGB {
    pub fn from_u32(value: u32) -> ByteRGB {
        ByteRGB {
            red: ((value >> 16) & 0xFF) as u8,
            green: ((value >> 8) & 0xFF) as u8,
            blue: (value & 0xFF) as u8,
        }
    }
}
