use flate2::{Compress, Compression, FlushCompress};
use crate::fetch::KLine;

pub fn compress_klines(klines: &[KLine]) -> Vec<u8> {
    // Сериализуем сразу в массив без ключей
    let json_array: Vec<_> = klines.iter()
        .map(|k| k.to_json_array())
        .collect();
    
    let serialized = serde_json::to_vec(&json_array).unwrap();
    
    // Сжимаем только если выгодно
    if serialized.len() > 1024 {
        compress_data(&serialized)
    } else {
        serialized
    }
}

fn compress_data(data: &[u8]) -> Vec<u8> {
    let mut compressor = Compress::new(Compression::best(), true);
    let mut output = Vec::with_capacity(data.len() / 2);
    
    compressor.compress_vec(data, &mut output, FlushCompress::Finish)
        .unwrap();

    if output.len() < data.len() {
        output
    } else {
        data.to_vec()
    }
}