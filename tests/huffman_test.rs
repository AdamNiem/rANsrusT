extern crate quickcheck;

use bit_vec::BitVec;

use std::collections::HashMap;

use ransrust::{Decoder, encode}
use ransrust::huffman::HuffmanCoder

#[test]
fn test_uniform() {
    let mut sample = HashMap::new();
    sample.insert(1, 1);
    sample.insert(2, 1);
    sample.insert(3, 1);
    sample.insert(4, 1);
    sample.insert(5, 1);
    let (book, tree) = CodeBuilder::from_iter(sample).finish();

    let mut buffer = BitVec::new();
    book.encode(&mut buffer, &1).unwrap();
    book.encode(&mut buffer, &2).unwrap();
    book.encode(&mut buffer, &3).unwrap();
    book.encode(&mut buffer, &4).unwrap();
    book.encode(&mut buffer, &5).unwrap();

    let mut decoder = tree.unbounded_decoder(buffer);
    assert_eq!(decoder.next(), Some(1));
    assert_eq!(decoder.next(), Some(2));
    assert_eq!(decoder.next(), Some(3));
    assert_eq!(decoder.next(), Some(4));
    assert_eq!(decoder.next(), Some(5));
    assert_eq!(decoder.next(), None);
}

#[test]
fn test_uniform_from_static() {
    const WEIGHTS: &[(&char, &usize)] = &[(&'a', &1), (&'b', &1), (&'c', &1), (&'d', &1)];
    let (book, tree) = codebook(WEIGHTS.iter().cloned());

    let mut buffer = BitVec::new();
    book.encode(&mut buffer, &'a').unwrap();
    book.encode(&mut buffer, &'b').unwrap();
    book.encode(&mut buffer, &'c').unwrap();
    book.encode(&mut buffer, &'d').unwrap();

    let mut decoder = tree.unbounded_decoder(buffer);
    assert_eq!(decoder.next(), Some('a'));
    assert_eq!(decoder.next(), Some('b'));
    assert_eq!(decoder.next(), Some('c'));
    assert_eq!(decoder.next(), Some('d'));
    assert_eq!(decoder.next(), None);
}

#[test]
fn test_empty() {
    let (book, tree) = CodeBuilder::<&str, i32>::new().finish();

    let mut buffer = BitVec::new();
    assert!(book.encode(&mut buffer, "hello").is_err());

    let mut decoder = tree.unbounded_decoder(buffer);
    assert_eq!(decoder.next(), None);
}

#[test]
fn test_single() {
    let mut builder = CodeBuilder::new();
    builder.push("hello", 1);
    let (book, tree) = builder.finish();

    let mut buffer = BitVec::new();
    book.encode(&mut buffer, "hello").unwrap();

    let mut decoder = tree.unbounded_decoder(buffer);
    assert_eq!(decoder.next(), Some("hello"));
    assert_eq!(decoder.next(), Some("hello")); // repeats
}

quickcheck! {
    fn efficient_order(ag: u32, at: u32, cg: u32, ct: u32, tg: u32) -> bool {
        let mut builder = CodeBuilder::new();
        builder.push("CG", cg);
        builder.push("AG", ag);
        builder.push("AT", at);
        builder.push("CT", ct);
        builder.push("TG", tg);
        let (book, _) = builder.finish();

        let len = |symbol| {
            book.get(symbol).map_or(0, |code| code.len())
        };

        at >= ct || len("CT") <= len("AT") ||
        ag.saturating_add(at).saturating_add(cg).saturating_add(ct).saturating_add(tg) == u32::MAX
    }

    fn encode_decode_bytes(symbols: Vec<u8>) -> bool {
        let mut counts = [0; 256];
        for symbol in &symbols {
            counts[usize::from(*symbol)] += 1;
        }

        let (book, tree) = counts.iter()
            .enumerate()
            .map(|(k, v)| (k as u8, *v))
            .collect::<CodeBuilder<_, _>>()
            .finish();

        let mut buffer = BitVec::new();
        for symbol in &symbols {
            book.encode(&mut buffer, symbol).unwrap();
        }

        tree.unbounded_decoder(&buffer).eq(symbols)
    }
}
