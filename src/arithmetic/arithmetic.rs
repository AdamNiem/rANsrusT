
//This is a rust port of the C++ implementation of Arithmetic Encoding
//As provided by Nayuki on github
//Modified a bit to fit this library
//Ref: https://github.com/nayuki/Reference-arithmetic-coding/tree/master/cpp
//

///Equivalent to Frequency Table used in RANS implementation
#[derive(Debug, Clone)]
pub struct SymbolStats {
    frequencies: Vec<u32>,
    cumulative: Vec<u32>,
    ///Records total symbols
    pub total: u32,  
}

///Implementation for symbol stats TODO: Look into integrating into one common frequency table
///implementation across encoders
impl SymbolStats {
    ///Initialize Symbol Stats with empty values
    pub fn new() -> Self {
        Self {
            frequencies: vec![],
            cumulative: vec![],
            total: 0,
        }
    }
    ///Fill frequency table with values
    pub fn new_static(probs: &[u32]) -> Self {
        let mut cumulative = Vec::with_capacity(probs.len() + 1);
        let mut sum = 0;
        cumulative.push(sum);

        for &freq in probs {
            sum += freq;
            cumulative.push(sum);
        }

        Self {
            frequencies: probs.to_vec(),
            cumulative,
            total: sum,
        }
    }

    ///Get low value
    pub fn get_low(&self, symbol: u32) -> u32 {
        self.cumulative[symbol as usize]
    }

    ///Get high value
    pub fn get_high(&self, symbol: u32) -> u32 {
        self.cumulative[(symbol + 1) as usize]
    }

    ///Get total
    pub fn get_total(&self) -> u32 {
        self.total
    }

    ///Get symbol limit
    pub fn get_symbol_limit(&self) -> u32 {
        self.frequencies.len() as u32
    }


}

///Provides basic fields and methods shared between encoding and decoding
#[derive(Debug)]
struct ArithmeticCoderBase {
    /*---- Configuration fields ----*/
    // Number of bits for the 'low' and 'high' state variables. Must be in the range [1, 63].
    // - For state sizes less than the midpoint of around 32, larger values are generally better -
    //   they allow a larger maximum frequency total (maximumTotal), and they reduce the approximation
    //   error inherent in adapting fractions to integers: u64, both effects reduce the data encoding loss
    //   and asymptotically approach the efficiency of arithmetic coding using exact fractions.
    // - But for state sizes greater than the midpoint, because intermediate computations are limited
    //   to the long integer type's 63-bit unsigned precision, larger state sizes will decrease the
    //   maximum frequency total, which might constrain the user-supplied probability model.
    // - Therefore numStateBits=32 is recommended as the most versatile setting
    //   because it maximizes maximumTotal (which ends up being slightly over 2^30).
    // - Note that numStateBits=63 is legal but useless because it implies maximumTotal=1,
    //   which means a frequency table can only support one symbol with non-zero frequency.
    pub num_state_bits: u32,
    
    // Maximum range (high+1-low) during coding (trivial), which is 2^numStateBits = 1000...000.
    pub full_range: u64,
    
    // The top bit at width numStateBits, which is 0100...000.
    pub half_range: u64,
    
    // The second highest bit at width numStateBits, which is 0010...000. This is zero when numStateBits=1.
    pub quarter_range: u64,
    
    // Minimum range (high+1-low) during coding (non-trivial), which is 0010...010.
    pub minimum_range: u64,
    
    // Maximum allowed total from a frequency table at all times during coding.
    pub maximum_total: u64,
    
    // Bit mask of numStateBits ones, which is 0111...111.
    pub state_mask: u64,
    
    
    /*---- State fields ----*/
    
    // Low end of this arithmetic coder's current range. Conceptually has an infinite number of trailing 0s.
    pub low: u64,
    
    // High end of this arithmetic coder's current range. Conceptually has an infinite number of trailing 1s.
    pub high: u64,

}

impl ArithmeticCoderBase {
    pub fn new(num_bits: u32) -> Self {
        assert!(num_bits >= 1 && num_bits <= 63, "State size out of range");
        let full_range = 1u64 << num_bits;
        let half_range = full_range >> 1;
        let quarter_range = half_range >> 1;
        let minimum_range = quarter_range + 2;
        let maximum_total = std::cmp::min(u64::MAX / full_range, minimum_range);
        let state_mask = full_range - 1;

        Self {
            num_state_bits: num_bits,
            full_range,
            half_range,
            quarter_range,
            minimum_range,
            maximum_total,
            state_mask,
            low: 0,
            high: state_mask,
        }
    }
}

///Defines some methods both encoding and decoding share
trait ArithmeticCoder {
    fn base_mut(&mut self) -> &mut ArithmeticCoderBase;
    fn shift(&mut self);
    fn underflow(&mut self);

    fn update(&mut self, freqs: &SymbolStats, symbol: u32) -> Result<(), String> {
        let total = freqs.get_total();
        let sym_low = freqs.get_low(symbol);
        let sym_high = freqs.get_high(symbol);
        if sym_low == sym_high {
            return Err("Symbol has zero frequency".to_string());
        }
        let range = {
            let b = self.base_mut();
            if b.low >= b.high || (b.low & b.state_mask) != b.low || (b.high & b.state_mask) != b.high {
                return Err("Assertion Error: Low or high out of range".to_string());
            }
            let range = b.high - b.low + 1;
            if !(b.minimum_range <= range && range <= b.full_range) {
                return Err("Assertion Error: Range out of range".to_string());
            }

            if total as u64 > b.maximum_total {
                return Err("Cannot code symbol because total is too large".to_string());
            }


            range
        };
        
        //Update Range
        {
            let b = self.base_mut();
            let new_low = b.low + (sym_low as u64 * range) / (total as u64);
            let new_high = b.low + (sym_high as u64 * range) / (total as u64) - 1;
            b.low = new_low;
            b.high = new_high;
        }

        // While low and high have the same top bit value, shift them out
        loop {
            let matches = {
                let b = self.base_mut();
                ((b.low ^ b.high) & b.half_range) == 0
            };
            if !matches { break; }
            
            self.shift();
            let b = self.base_mut();
            b.low = (b.low << 1) & b.state_mask;
            b.high = ((b.high << 1) & b.state_mask) | 1;
        }

        // While low's top two bits are 01 and high's are 10, delete the second highest bit
        loop {
            let matches = {
                let b = self.base_mut();
                (b.low & !b.high & b.quarter_range) != 0
            };
            if !matches { break; }

            self.underflow();
            let b = self.base_mut();
            b.low = (b.low << 1) ^ b.half_range;
            b.high = ((b.high ^ b.half_range) << 1) | b.half_range | 1;
        }
        
        Ok(())
    }
}

///ArithmeticDecoder 
#[derive(Debug)]
pub struct ArithmeticDecoder {
    /*---- Fields ----*/
    base: ArithmeticCoderBase,

    ///freq table for decoding
    pub stats: SymbolStats,

    encoded_data: Vec<u8>,

    // Number of saved underflow bits. This value can grow without bound,
    // so a truly correct implementation would use a bigint.
    code: u64,

    // Bit Reader State Trackers 
    byte_index: usize,
    bit_index: u8,

}

impl ArithmeticCoder for ArithmeticDecoder {
    fn base_mut(&mut self) -> &mut ArithmeticCoderBase {
        &mut self.base       
    }
    fn shift(&mut self) {
        self.code = ((self.code << 1) & self.base.state_mask) | self.read_code_bit();
    }

    fn underflow(&mut self) {
        self.code = (self.code & self.base.half_range)
            | ((self.code << 1) & (self.base.state_mask >> 1))
            | self.read_code_bit();
    }

}


impl ArithmeticDecoder {
    /// initialize decoder
    pub fn new(encoded: Vec<u8>) -> Self {
        let mut decoder = Self {
            base: ArithmeticCoderBase::new(32),
            stats: SymbolStats::new(), // To be set manually, matching Quickstart
            encoded_data: encoded,
            code: 0,
            byte_index: 0,
            bit_index: 0,
        };

        // Initialize the code by reading the first num_state_bits
        for _ in 0..decoder.base.num_state_bits {
            let bit = decoder.read_code_bit();
            decoder.code = (decoder.code << 1) | bit;
        }
        decoder
    }

    ///decode symbol
    pub fn decode_symbol(&mut self) -> Result<u8, String> {
        let total = self.stats.get_total();
        if total as u64 > self.base.maximum_total {
            return Err("Cannot decode symbol because total is too large".to_string());
        }

        let range = self.base.high - self.base.low + 1;
        let offset = self.code - self.base.low;
        let value = ((offset + 1) * total as u64 - 1) / range;

        if value * range / total as u64 > offset || value >= total as u64 {
            return Err("Assertion Error".to_string());
        }

        // Binary search to find the highest symbol such that get_low(symbol) <= value
        let mut start = 0;
        let mut end = self.stats.get_symbol_limit();
        
        while end - start > 1 {
            let middle = (start + end) >> 1;
            if self.stats.get_low(middle) as u64 > value {
                end = middle;
            } else {
                start = middle;
            }
        }

        if start + 1 != end {
            return Err("Assertion error in binary search".to_string());
        }

        let symbol = start;
        let stats_clone = self.stats.clone(); // Clone to prevent borrow overlap
        
        self.update(&stats_clone, symbol)?;

        if !(self.base.low <= self.code && self.code <= self.base.high) {
            return Err("Assertion error: Code out of range".to_string());
        }

        Ok(symbol as u8)
    }

    /// Internal Bit Reader
    fn read_code_bit(&mut self) -> u64 {
        if self.byte_index >= self.encoded_data.len() {
            return 0; // Padding bits at EOF
        }
        let bit = (self.encoded_data[self.byte_index] >> (7 - self.bit_index)) & 1;
        self.bit_index += 1;
        if self.bit_index == 8 {
            self.bit_index = 0;
            self.byte_index += 1;
        }
        bit as u64
    }
}



///ArithmeticEncoder
#[derive(Debug)]
pub struct ArithmeticEncoder {
    /*---- Fields ----*/
    ///base
    base: ArithmeticCoderBase,

    ///freq table for encoding
    pub stats: SymbolStats,
    
    /// The underlying bit output stream.
    encoded_data: Vec<u8>,

    /// Number of saved underflow bits. This value can grow without bound,
    /// so a truly correct implementation would use a bigint.
    num_underflow: u64,

    ///Bit Writer State
    current_byte: u8,
    num_bits_filled: u8,
}

impl ArithmeticCoder for ArithmeticEncoder {
    ///return mutable reference to base fields
    fn base_mut(&mut self) -> &mut ArithmeticCoderBase {
        &mut self.base
    }
    ///shift bits and write them
    fn shift(&mut self) {
        let bit = (self.base.low >> (self.base.num_state_bits - 1)) as u8;
        self.write_bit(bit);

        //Write out the saved underflow bits
        for _ in 0..self.num_underflow {
            self.write_bit(bit ^ 1);
        }
        self.num_underflow = 0;
    }

    ///check if underflow
    fn underflow(&mut self) {
        if self.num_underflow == u64::MAX {
            panic!("Maximum underflow reached");
        }
        self.num_underflow += 1;
    }
}

impl ArithmeticEncoder {
    ///initialize
    pub fn new_static(probs: &[u32]) -> Self {
        Self {
            base: ArithmeticCoderBase::new(32), //32 bits
            stats: SymbolStats::new_static(probs),
            encoded_data: Vec::new(),
            num_underflow: 0,
            current_byte: 0,
            num_bits_filled: 0,
        }
    }
    ///encode symbol
    pub fn encode_symbol(&mut self, symbol:u8) {
        //clone stats ref to avoid conflicts with self.update
        let stats_clone = self.stats.clone(); // Clone to prevent borrow overlap
        self.update(&stats_clone, symbol as u32).unwrap();
    }

    ///finish writing and do padding if needed
    pub fn finish(&mut self) {
        self.write_bit(1); //Flush bit
        if self.num_bits_filled > 0 {
            //pad rest of bytes with zeroes
            self.current_byte <<= 8 - self.num_bits_filled;
            self.encoded_data.push(self.current_byte);
        }
    }

    ///get encoded data
    pub fn get_encoded(self) -> Vec<u8> {
        self.encoded_data
    }

    ///Internal bit writer for encoding
    fn write_bit(&mut self, bit: u8) {
        self.current_byte = (self.current_byte << 1) | bit;
        self.num_bits_filled += 1;
        if self.num_bits_filled == 8 {
            self.encoded_data.push(self.current_byte);
            self.current_byte = 0;
            self.num_bits_filled = 0;
        }
    }

}

