// tests/arithmetic_tests.rs
use  ransrust::arithmetic::{ArithmeticDecoder, ArithmeticEncoder};

#[test]
fn test_arithmetic_roundtrip() {
    let data = b"Hello Arithmetic Coding World!".to_vec();

    let mut probs = vec![0; 256];
    for c in data.iter() {
        probs[*c as usize] += 1;
    }

    // Encode
    let mut encoder = ArithmeticEncoder::new_static(&probs);
    for symbol in data.iter() {
        encoder.encode_symbol(*symbol);
    }
    encoder.finish(); // CRUCIAL: Flush the last bits
    let encoder_stats = encoder.stats.clone();
    let encoded = encoder.get_encoded();

    // Decode
    let mut decoder = ArithmeticDecoder::new(encoded);
    decoder.stats = encoder_stats; // Pass the stats over
    let mut decoded_data = vec![];

    for _ in 0..data.len() {
        decoded_data.push(decoder.decode_symbol().unwrap());
    }

    assert_eq!(data, decoded_data);
}
