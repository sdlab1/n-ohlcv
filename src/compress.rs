// compress.rs
use flate2::{Compress, Decompress, Compression, FlushCompress, FlushDecompress};
use std::io::{Error, ErrorKind};
use crate::fetch::KLine;

pub fn compress_data(data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut compressor = Compress::new(Compression::best(), true);
    let mut output = Vec::with_capacity(data.len() / 2);
    compressor.compress_vec(data, &mut output, FlushCompress::Finish)?;
    Ok(output)
}

pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut decompressor = Decompress::new(true);
    let mut output = Vec::with_capacity(data.len() * 2);
    decompressor.decompress_vec(data, &mut output, FlushDecompress::Finish)?;
    Ok(output)
}

pub fn compress_klines(klines: &[KLine]) -> Result<Vec<u8>, Error> {
    let serialized = serde_json::to_vec(klines)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    compress_data(&serialized)
}

pub fn decompress_klines(data: &[u8]) -> Result<Vec<KLine>, Error> {
    let decompressed = decompress_data(data)?;
    serde_json::from_slice(&decompressed)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))
}