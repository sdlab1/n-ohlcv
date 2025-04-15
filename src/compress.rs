/*use flate2::bufread::{ZlibDecoder, ZlibEncoder};
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
    println!("comp: {}, data: {}, ratio: {:.2}",data.len(),output.len(), output.len() as f64/data.len() as f64);
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
}*/



use bincode;
use std::io::{self, Read, Write};
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;
use xz2::stream::{Check, Filters, LzmaOptions, Stream};
use crate::fetch::KLine;

// Конфигурация bincode
fn bincode_config() -> impl bincode::config::Config {
    bincode::config::standard()
        .with_variable_int_encoding()
        .with_little_endian()
}

pub fn compress_klines(klines: &[KLine]) -> Result<Vec<u8>, io::Error> {
    let serialized = bincode::encode_to_vec(klines, bincode_config())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let compressed = compress_lzma2_max(&serialized)?;
    
    println!(
        "Compressed from {} to {} bytes (ratio: {:.2})",
        serialized.len(),
        compressed.len(),
        serialized.len() as f32 / compressed.len() as f32
    );
    
    Ok(compressed)
}

pub fn decompress_klines(data: &[u8]) -> Result<Vec<KLine>, io::Error> {
    let decompressed = decompress_lzma2(data)?;
    let (result, _) = bincode::decode_from_slice(&decompressed, bincode_config())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(result)
}

fn compress_lzma2_max(data: &[u8]) -> Result<Vec<u8>, io::Error> {
    let mut opts = LzmaOptions::new_preset(9)?; // Максимальный уровень сжатия
    
    // Оптимальные параметры для финансовых данных
    opts.dict_size(1 << 20); // 1MB словарь
    opts.nice_len(128);      // Длина совпадений
    
    let mut filters = Filters::new();
    filters.lzma2(&opts);
    
    let stream = Stream::new_stream_encoder(&filters, Check::Crc64)?;
    let mut encoder = XzEncoder::new_stream(Vec::new(), stream);
    encoder.write_all(data)?;
    encoder.finish()
}

fn decompress_lzma2(data: &[u8]) -> Result<Vec<u8>, io::Error> {
    let mut decoder = XzDecoder::new(data);
    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;
    Ok(buf)
}