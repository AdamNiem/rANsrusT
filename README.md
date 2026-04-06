
Citations:

``` text
Hugo Cisneros, *rust-rans: A Rust implementation of range Asymmetric Numeral Systems (rANS)*, GitHub, 2022. https://github.com/hugcis/rust-rans

Fiekas, Niklas. rust-huffman-compress. GitHub, https://github.com/niklasf/rust-huffman-compress.

https://sdrbench.github.io/

https://mattmahoney.net/dc/textdata.html

For the rust port of the C++ implementation of Arithmetic Encoding : 
As provided by Nayuki (with modifications) on github https://github.com/nayuki/Reference-arithmetic-coding/tree/master/cpp

```

Remember:

```text
Challenges to talk about:
- Learning Rust
- Uniform Random Data
  - Huffman CR 1
  - ANS CR Slightly <1. Normalization code has rounding logic that cna slightly misrepersent true uniform distributions. 
- rANS initally used a BTree (O(log n)) for encoding. Replaced with FastBook (O(1)) and BitReader
```
