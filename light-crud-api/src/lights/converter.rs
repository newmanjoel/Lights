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

    pub fn from_hex(value: &str) -> Option<ByteRGB> {
        let color = parse_color(value);
        return match color {
            None => None,
            Some(colors) => Some(ByteRGB {
                red: colors.0,
                green: colors.1,
                blue: colors.2,
            }),
        };
    }

    pub fn as_u32(&self) -> u32 {
        return ((self.red as u32) << 16) | ((self.green as u32) << 8) | (self.blue as u32);
    }
}

fn parse_color(hex: &str) -> Option<(u8, u8, u8)> {
    // Ensure the string starts with '#' and has the correct length
    if hex.len() == 7 && hex.starts_with('#') {
        // Extract substrings for r, g, b
        let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
        let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
        let b = u8::from_str_radix(&hex[5..7], 16).ok()?;
        return Some((r, g, b));
    }
    None
}
