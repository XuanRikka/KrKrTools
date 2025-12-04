fn transform_mode1(data: &mut [u8])
{
    for chunk in data.chunks_exact_mut(2) {
        let c = u16::from(chunk[0]) | (u16::from(chunk[1]) << 8);

        let scrambled = ((c & 0xAAAA) >> 1) | ((c & 0x5555) << 1);

        chunk[0] = scrambled as u8;
        chunk[1] = (scrambled >> 8) as u8;
    }
}


pub fn scramble_mode1(data: &mut [u8]) {
    transform_mode1(data);
}

pub fn descramble_mode1(data: &mut [u8])
{
    transform_mode1(data);
}
