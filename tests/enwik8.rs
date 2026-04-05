use std::fs::File;
use std::io::Read;
use std::time::Instant;

use std::collections::HashMap;

use ransrust::{codebook, ANSCoder, ANSDecoder, FastBook, BitReader};

fn vec_compare<N: std::cmp::PartialEq + Copy>(va: &[N], vb: &[N]) -> bool {
    va.iter().zip(vb).all(|(&a, &b)| a == b)
}

fn rans_benchmark_test(path: &str) {
    let path = "enwik8";
    println!("Looking for file at: {:?}",
             std::fs::canonicalize(".").unwrap()
    );
    if !std::path::Path::new(path).exists() {
        eprintln!("Skpping enwiki8 test - run data.sh first");
        return;
    }

    let mut data: Vec<u8> = vec![];

    // Read test file
    let mut f = File::open(path).unwrap();
    f.read_to_end(&mut data).unwrap();

    // Compute probabilities for every token in document
    let mut probs = vec![0; 256];
    for c in data.iter() {
        probs[*c as usize] += 1;
    }

    println!("Total tokens: {}",
             probs.iter().sum::<u32>()
    );

    let mut ans = ANSCoder::new_static(&probs);

    println!("Normal:");
    for _ in 0..5 {
        ans = ANSCoder::new_static(&probs);
        let now = Instant::now();
        for symbol in data.iter() {
            ans.encode_symbol(*symbol);
        }
        let dur = now.elapsed();
        println!(
            "\t{:.3} seconds elapsed, {:.3}MiB/sec",
            dur.as_millis() as f64 / 1000.,
            data.len() as f64 / (2_f64.powf(20.) * dur.as_nanos() as f64 / 1e9)
        );
    }

    // println!("Update probs every 10000 tokens:");
    // for _ in 0..5 {
    //     ans = ANSCoder::new();
    //     let now = Instant::now();
    //     for (i, symbol) in data.iter().enumerate() {
    //         if i % 10000 == 0 {
    //             ans.stats.update_probs(&probs);
    //         }
    //         ans.encode_symbol(*symbol);
    //     }
    //     let dur = now.elapsed();
    //     println!(
    //         "\t{:.3} seconds elapsed, {:.3}MiB/sec",
    //         dur.as_millis() as f64 / 1000.,
    //         data.len() as f64 / (2_f64.powf(20.) * dur.as_nanos() as f64 / 1e9)
    //     );
    // }

    println!("Optimized:");
    for _ in 0..5 {
        ans = ANSCoder::new_precomp(&probs);
        let now = Instant::now();
        for symbol in data.iter() {
            ans.encode_symbol_precomp(*symbol);
        }
        let dur = now.elapsed();
        println!(
            "\t{:.3} seconds elapsed, {:.3}MiB/sec",
            dur.as_millis() as f64 / 1000.,
            data.len() as f64 / (2_f64.powf(20.) * dur.as_nanos() as f64 / 1e9)
        );
    }

    let encoded = ans.get_encoded();
    println!("Encoded data size {}", encoded.len() * 4);
    println!("Compression ratio: {:.3}",
             (data.len()) as f64 / (encoded.len()*4) as f64
    );

    // Decode data
    let mut decoder = ANSDecoder::new(encoded);
    decoder.stats = ans.stats;
    let mut decoded_data = vec![];
    let length_decoded = data.len();

    for _ in 0..length_decoded {
        decoded_data.push(decoder.decode_symbol().unwrap())
    }
    decoded_data = decoded_data.into_iter().rev().collect();

    assert!(vec_compare(&data, &decoded_data));
    println!("Decoding ok!");
}

fn huffman_benchmark_test(path: &str) {
    if !std::path::Path::new(path).exists() {
        eprintln!("Skipping enwiki8 huffman test - run data.sh first");
        return;
    }

    let mut data: Vec<u8> = vec![];
    let mut f = File::open(path).unwrap();
    f.read_to_end(&mut data).unwrap();

    // Build weight table (same as probs in ANS test)
    let mut weights = vec![0u32; 256];
    for c in data.iter() {
        weights[*c as usize] += 1;
    }
    println!("Total tokens: {}", data.len());

    // Build huffman book and tree from weights
    let weight_map: HashMap<u8, u32> = weights
        .iter()
        .enumerate()
        .filter(|&(_, &w)| w > 0)  // huffman can't handle 0-weight symbols
        .map(|(i, &w)| (i as u8, w))
        .collect();

    let (book, tree) = codebook(weight_map.iter());
    let fast_book = FastBook::from_book(&book);

    // Benchmark encoding
    let mut encoded = Vec::new();
    println!("Huffman fast encoding:");
    for _ in 0..5 {
        let now = Instant::now();
        encoded = fast_book.encode(&data);
        let dur = now.elapsed();
        println!(
            "\t{:.3} seconds elapsed, {:.3}MiB/sec",
            dur.as_millis() as f64 / 1000.,
            data.len() as f64 / (2_f64.powf(20.) * dur.as_nanos() as f64 / 1e9)
        );
    }

    println!("Encoded size: {} bytes ({} bits)", encoded.len(), encoded.len());
    println!("Compression ratio: {:.3}",
             data.len() as f64 / encoded.len() as f64
    );

    // Decode and verify
    let decoded: Vec<u8> = tree.decoder(BitReader::new(&encoded), data.len()).collect();

    assert!(vec_compare(&data, &decoded));
    println!("Decoding ok!");
}

#[test]
fn test_rans_enwik8() {
   rans_benchmark_test("enwik8");
}

#[test]
fn test_huffman_enwik8() {
    huffman_benchmark_test("enwik8");
}

#[test]
fn test_rans_nyx() {
   rans_benchmark_test("SDRBENCH-EXASKY-NYX-512x512x512/temperature.f32");
}

#[test]
fn test_huffman_nyx() {
    huffman_benchmark_test("SDRBENCH-EXASKY-NYX-512x512x512/temperature.f32");
}
