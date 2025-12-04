use flate2::{Decompress, Compress, FlushDecompress, FlushCompress};


pub fn decompress(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut d = Decompress::new(false);
    d.decompress_vec(input, &mut out, FlushDecompress::Finish).unwrap();
    out
}


pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut c = Compress::new(flate2::Compression::default(), false);
    c.compress_vec(input, &mut out, FlushCompress::Finish).unwrap();
    out
}
