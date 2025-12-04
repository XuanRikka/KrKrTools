use std::io::Write;
use flate2::write::{ZlibEncoder, ZlibDecoder};
use flate2::Compression;

pub fn compress_zlib(data: &[u8]) -> Vec<u8> {
    let raw_len = data.len() as u64;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(data).unwrap();
    let compress_data = encoder.finish().unwrap();
    let compress_len = (compress_data.len()+16) as u64;

    let mut header = Vec::with_capacity(compress_len as usize);

    header.extend_from_slice(&compress_len.to_be_bytes());
    header.extend_from_slice(&raw_len.to_be_bytes());
    header.extend_from_slice(&compress_data);
    header
}

pub fn decompress_zlib(data: &[u8]) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(Vec::new());
    decoder.write_all(data).unwrap();
    decoder.finish().unwrap()
}
