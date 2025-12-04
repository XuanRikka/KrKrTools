pub fn utf16le_to_utf8(input: &[u8]) -> Vec<u8> {
    let u16_values: Vec<u16> = input
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    let string: String = char::decode_utf16(u16_values.into_iter())
        .collect::<Result<String, _>>().unwrap();

    string.into_bytes()
}


pub fn utf8_to_utf16le(input: &[u8]) -> Vec<u8> {
    let string = String::from_utf8(input.to_vec()).expect("输入的文本不是UTF-8");

    let mut result = Vec::new();
    for ch in string.encode_utf16() {
        result.extend_from_slice(&ch.to_le_bytes());
    }

    result
}
