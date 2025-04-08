use lzf;

pub fn compress(data: &[u8]) -> Vec<u8> {
    let compressed = lzf::compress(data).expect("Compression failed");
    println!("Compressed data from {} to {} bytes", data.len(), compressed.len());
    compressed
}

pub fn decompress(data: &[u8], max_size: usize) -> Vec<u8> {
    let decompressed = lzf::decompress(data, max_size).expect("Decompression failed");
    println!("Decompressed data to {} bytes", decompressed.len());
    decompressed
}