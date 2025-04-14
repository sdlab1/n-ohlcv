use flate2::bufread::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use std::io::{Cursor, Error, ErrorKind, Read};
use crate::fetch::KLine;
use bincode;

pub fn compress_data(data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut encoder = ZlibEncoder::new(Cursor::new(data), Compression::best());
    let mut output = Vec::with_capacity(data.len() / 2);
    encoder.read_to_end(&mut output)?;
    Ok(output)
}

pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut decoder = ZlibDecoder::new(Cursor::new(data));
    let mut output = Vec::new();
    decoder.read_to_end(&mut output)?;
    println!("comp: {}, data: {}, ratio: {:.2}",data.len(),output.len(), output.len()/data.len());
    Ok(output)
}

pub fn compress_klines(klines: &[KLine]) -> Result<Vec<u8>, Error> {
    let serialized = bincode::encode_to_vec(klines, bincode::config::standard())
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Serialization error: {}", e)))?;
    compress_data(&serialized)
}

pub fn decompress_klines(data: &[u8]) -> Result<Vec<KLine>, Error> {
    let decompressed = decompress_data(data)?;
    let (deserialized, _): (Vec<KLine>, _) = bincode::decode_from_slice(&decompressed, bincode::config::standard())
        .map_err(|e| Error::new(ErrorKind::InvalidData, format!("Deserialization error: {}", e)))?;
    Ok(deserialized)
}