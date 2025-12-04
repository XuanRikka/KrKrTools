pub fn scramble_mode0(data: &mut [u8])
{
    for chunk in data.chunks_mut(2)
    {
        if chunk[1] == 0 && chunk[0] < 0x20 {
            continue;
        }
        chunk[0] ^= 1;
        chunk[1] ^= chunk[0] & 0xFE;
    }
}

pub fn descramble_mode0(data: &mut [u8])
{
    for chunk in data.chunks_exact_mut(2) {
        if chunk[1] == 0 && chunk[0] < 0x20 {
            continue;
        }
        chunk[1] ^= chunk[0] & 0xFE;
        chunk[0] ^= 1;
    }
}
