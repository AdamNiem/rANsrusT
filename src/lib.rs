#![deny(missing_docs,
        missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications)]


//! This crate implements common Asymmetric numeral systems coding algorithms
//!
//!
//!
//! # Quickstart
//! ## Encoding
//! ```ignore
//! use ransrust::ANSCoder;
//! use std::fs::File;
//! use std::io::Read;
//!
//! // Read test file
//! let mut f = File::open("book1").unwrap();
//! f.read_to_end(&mut data).unwrap();
//!
//! // Compute probablities for every token in the document
//! let mut probs = vec![0; 256];
//! for c in data.iter() {
//!     probs[*c as usize] += 1;
//! }
//! let mut ans = ANSCoder::new_static(&probs);
//!
//! for symbol in data.iter() {
//!   ans.encode_symbol(*symbol);
//! }
//! let encoded = ans.get_encoded();
//! ```
//! ## Decoding
//!
//! ```ignore
//! # use ransrust::ANSCoder;
//! # use std::fs::File;
//! # use std::io::Read;
//! # let mut data: Vec<u8> = vec![];
//! # // Read test file
//! # let mut f = File::open("book1").unwrap();
//! # f.read_to_end(&mut data).unwrap();
//! # // Compute probablities for every token in the document
//! # let mut probs = vec![0; 256];
//! # for c in data.iter() {
//! #     probs[*c as usize] += 1;
//! # }
//! # let mut ans = ANSCoder::new_static(&probs);
//! # for symbol in data.iter() {
//! #   ans.encode_symbol(*symbol);
//! # }
//! # let encoded = ans.get_encoded();
//! use ransrust::ANSDecoder;
//! // Construct decoder with the same stats object as the encoder
//! let mut decoder = ANSDecoder::new(encoded);
//! decoder.stats = ans.stats;
//!
//! let mut decoded_data = vec![];
//! let length_decoded = data.len();
//! for _ in 0..length_decoded {
//!   decoded_data.push(decoder.decode_symbol().unwrap())
//! }
//! decoded_data = decoded_data.into_iter().rev().collect();
//! ```


//! [Huffman compression](https://en.wikipedia.org/wiki/Huffman_coding)
//! given a probability distribution over arbitrary symbols.
//!
//! # Examples
//!
//! ```ignore
//! extern crate bit_vec;
//! extern crate huffman_compress;
//!
//! # use std::error::Error;
//! #
//! # fn try_main() -> Result<(), Box<dyn Error>> {
//! use std::iter::FromIterator;
//! use std::collections::HashMap;
//! use bit_vec::BitVec;
//! use huffman_compress::{CodeBuilder, Book, Tree};
//!
//! let mut weights = HashMap::new();
//! weights.insert("CG", 293);
//! weights.insert("AG", 34);
//! weights.insert("AT", 4);
//! weights.insert("CT", 4);
//! weights.insert("TG", 1);
//!
//! // Construct a Huffman code based on the weights (e.g. counts or relative
//! // frequencies).
//! let (book, tree) = CodeBuilder::from_iter(weights).finish();
//!
//! // More frequent symbols will be encoded with fewer bits.
//! assert!(book.get("CG").map_or(0, |cg| cg.len()) <
//!         book.get("AG").map_or(0, |ag| ag.len()));
//!
//! // Encode some symbols using the book.
//! let mut buffer = BitVec::new();
//! let example = vec!["AT", "CG", "AT", "TG", "AG", "CT", "CT", "AG", "CG"];
//! for symbol in &example {
//!     book.encode(&mut buffer, symbol);
//! }
//!
//! // Decode the symbols using the tree.
//! let decoded: Vec<&str> = tree.decoder(&buffer, example.len()).collect();
//! assert_eq!(decoded, example);
//! #     Ok(())
//! # }
//! #
//! # fn main() {
//! #     try_main().unwrap();
//! # }
//! ```

/// The coder module with an encoder and a decoder
mod rans;
mod huffman;

pub use crate::rans::{ANSCoder, ANSDecoder};
pub use crate::huffman::{Book, Tree, Decoder, CodeBuilder, EncodeError, codebook, FastBook, BitReader};
