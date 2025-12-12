pub fn string_to_utf16le(s: String) -> Vec<u16> {
    s.encode_utf16()
        .map(|code_unit| {
            u16::from_le_bytes(code_unit.to_le_bytes())
        })
        .collect()
}


pub fn utf16le_to_string(vec: &[u16]) -> String {
    let platform_vec: Vec<u16> = vec
        .iter()
        .map(|&le_unit| {
            u16::from_le(le_unit)
        })
        .collect();

    String::from_utf16(&platform_vec).expect("解码字符串失败")
}