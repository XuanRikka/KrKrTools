use std::error::Error;

pub const MODE0: [u8; 5] = [0xFE, 0xFE, 0x00, 0xFF, 0xFE];
pub const MODE1: [u8; 5] = [0xFE, 0xFE, 0x01, 0xFF, 0xFE];
pub const MODE2: [u8; 5] = [0xFE, 0xFE, 0x02, 0xFF, 0xFE];


pub enum Mode {
    MODE0,
    MODE1,
    MODE2,
}


pub fn file_mode(header: [u8; 5]) -> Result<Mode, Box<dyn Error>>
{
    match header {
        MODE0 => Ok(Mode::MODE0),
        MODE1 => Ok(Mode::MODE1),
        MODE2 => Ok(Mode::MODE2),
        _ => Err("输入文件未知".into())
    }
}

