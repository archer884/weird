//! # weird
//!
//! > Salted Base32 encoding for 64-bit values.
//!
//! [Crockford Base32 Encoding](https://www.crockford.com/wrmg/base32.html) is most commonly used to make numeric identifiers slightly more user-resistant. Similar to [Hashids](http://hashids.org/), the purpose here is to make the identifiers shorter and less confusing. Unlike Hashids, Crockford Base32 does nothing to conceal the real value of the number (beyond the actual encoding, anyway) and the fact that they are sequential is still pretty obvious when you see consecutive identifiers side by side. **This is where Weird differs from Crockford.**
//!
//! Rather than directly encoding values as Base32, weird employs a shuffled alphabet and applies an arbitrary salt to the encoded values. This is to give any observer the appearance of non-sequential data and to make it more difficult to derive the original identifiers based on the encoded identifiers.
//!
//! This library does not support encoding and decoding of arbitrary data; there is [another library for that](https://crates.io/crates/base32). Additionally, the spec supports the idea of check digits, but this library currently does not.
//!
//! **The primary purpose of this library is to provide high performance, user-resistant encoding of numeric identifiers.** To that end, both encoding and decoding are, in fact, pretty darn fast. How fast? According to my testing, `crockford` (on which this library is based) decodes **fifty times faster** and encodes **twenty-seven times faster** than `harsh`. This library is roughly half as fast, given that it is applying an additional operation.
//!
//! ## Usage
//!
//! Unlike the original `crockford` library, an instance of `weird::Weird` must be configured before use. The user may create an instance based on a salt or based on a salt and alphabet. (See documentation for more about the alphabet.)
//!
//! The alphabet itself is generated based on an implementation of `rand::Rng` that is unlikely to change (read: I'm much too lazy to change it) and which, therefore, should provide stable functionality for this constructor. If you prefer not to trust me on that (understandable!), implement your own rng and pass that to the constructor of `Alphabet`.
//!
//! ### Encoding
//!
//! ```rust
//! # use weird::Weird;
//! // A salt can be anything that translates into a slice of bytes.
//! let weird = Weird::from_salt("Salt goes here");
//! let encoded = weird.encode(13); // I have no idea what this would look like.
//!                                 // It's random, remember?
//! ```
//!
//! #### Plan B (faster encoding)
//!
//! The `Weird::encode_into()` method permits encoding directly to `std::fmt::Write`. This allows reuse of a buffer or... you know, whatever you wanna do. The standard encoding mechanism is, in fact, implemented in terms of this; it simply unwraps the result since, of course, encoding into a string can't really fail.
//!
//! ### Decoding
//!
//! Decoding can fail (since the decoder can accept arbitrary strings), so it returns a result instead.
//!
//! ```rust
//! # use weird::Weird;
//! let weird = Weird::from_salt("Salt goes here.");
//! let decoded = weird.decode("Hello, world!"); // I bet this isn't valid.
//!
//! let id = match decoded {
//!     Ok(id) => id,
//!     Err(_) => {
//!         println!("I knew that wasn't valid!");
//!         return;
//!     }
//! };
//! ```

use std::{
    borrow::Borrow,
    fmt::{self, Write},
};

mod error;

pub use error::*;
use rand::{prelude::SliceRandom, Rng};
use squirrel::SquirrelRng;

static CANONICAL_MAPPING: &[i8; 256] = &include!("../resources/u8-mapping.txt");
static UPPERCASE_ENCODING: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub trait Salt {
    fn byte_source(&self) -> ByteSource;
    fn bytes(&self) -> &[u8];
}

impl<T: Borrow<str>> Salt for T {
    fn byte_source(&self) -> ByteSource {
        ByteSource {
            idx: 0,
            data: self.borrow().as_bytes(),
        }
    }

    fn bytes(&self) -> &[u8] {
        self.borrow().as_bytes()
    }
}

pub struct ByteSource<'a> {
    idx: usize,
    data: &'a [u8],
}

impl<'a> ByteSource<'a> {
    #[inline]
    pub fn next(&mut self) -> u8 {
        if self.idx == self.data.len() {
            self.idx = 0;
            self.next();
        }

        let ret = self.data[self.idx];
        self.idx += 1;
        ret
    }

    #[inline]
    pub fn apply(&mut self, u: u8) -> u8 {
        let x = self.next() % 32;
        u ^ x
    }
}

#[derive(Clone)]
pub struct Alphabet {
    alpha: [u8; 32],
    mapping: [i8; 256],
}

impl Alphabet {
    /// Shuffle an `Alphabet` using the provided salt.
    pub fn from_salt(salt: impl AsRef<[u8]>) -> Self {
        let hash = fnv_hash(salt.as_ref());
        let mut rng = SquirrelRng::with_seed(hash);
        let mut alphabet = UPPERCASE_ENCODING.clone();
        alphabet.shuffle(&mut rng);
        Self::from_checked_alphabet(alphabet)
    }

    /// Shuffle an `Alphabet` using the provided seed.
    pub fn from_seed(seed: u32) -> Self {
        let mut rng = SquirrelRng::with_seed(seed);
        let mut alphabet = UPPERCASE_ENCODING.clone();
        alphabet.shuffle(&mut rng);
        Self::from_checked_alphabet(alphabet)
    }

    /// Shuffle an `Alphabet` using the provided rng.
    ///
    /// Note that the provided implementation should produce the same result
    /// each time the alphabet is initialized, otherwise your identifiers will
    /// change each time you run your application!
    pub fn from_rng(rng: &mut impl Rng) -> Self {
        let mut alphabet = UPPERCASE_ENCODING.clone();
        alphabet.shuffle(rng);
        Self::from_checked_alphabet(alphabet)
    }

    // This function is private because calling it with an unchecked alphabet
    // is wildly unsafe.
    fn from_checked_alphabet(alpha: [u8; 32]) -> Self {
        let mut mapping = CANONICAL_MAPPING.clone();

        for (&canonical, &scrambled) in UPPERCASE_ENCODING.iter().zip(alpha.iter()) {
            match scrambled {
                // When mapping 0, we must also map Oo
                b'0' => {
                    let value = CANONICAL_MAPPING[canonical as usize];
                    mapping[b'0' as usize] = value;
                    mapping[b'O' as usize] = value;
                    mapping[b'o' as usize] = value;
                }

                // When mapping 1, we must also map IiLl
                b'1' => {
                    let value = CANONICAL_MAPPING[canonical as usize];
                    mapping[b'1' as usize] = value;
                    mapping[b'I' as usize] = value;
                    mapping[b'i' as usize] = value;
                    mapping[b'L' as usize] = value;
                    mapping[b'l' as usize] = value;
                }

                u => {
                    let value = CANONICAL_MAPPING[canonical as usize];
                    mapping[u as usize] = value;
                    mapping[u.to_ascii_lowercase() as usize] = value;
                }
            }
        }

        Self { alpha, mapping }
    }
}

impl fmt::Debug for Alphabet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let alphabet: String = self.alpha.iter().copied().map(|u| u as char).collect();
        write!(f, "Alphabet({:?})", alphabet)
    }
}

pub struct Weird<T> {
    alphabet: Alphabet,
    salt: T,
}

impl<T: Salt> Weird<T> {
    pub fn from_salt(salt: T) -> Self {
        Self {
            alphabet: Alphabet::from_salt(salt.bytes()),
            salt,
        }
    }

    pub fn from_salt_with_rng(salt: T, rng: &mut impl Rng) -> Self {
        Self {
            alphabet: Alphabet::from_rng(rng),
            salt,
        }
    }

    pub fn new(salt: T, alphabet: Alphabet) -> Self {
        Self { alphabet, salt }
    }

    pub fn encode(&self, n: u64) -> String {
        let mut buf = String::with_capacity(13);
        self.encode_into(n, &mut buf).expect("Infallible");
        buf
    }

    pub fn encode_into<W: Write>(&self, mut n: u64, w: &mut W) -> fmt::Result {
        // After we clear the four most significant bits, the four least significant bits will be
        // replaced with 0001. We can then know to stop once the four most significant bits are,
        // likewise, 0001.
        const STOP_BIT: u64 = 1 << QUAD_SHIFT;

        const QUAD_SHIFT: usize = 60;
        const QUAD_RESET: usize = 4;

        const FIVE_SHIFT: usize = 59;
        const FIVE_RESET: usize = 5;

        if n == 0 {
            w.write_char('0')?;
            return Ok(());
        }

        let mut salt = self.salt.byte_source();

        // Start by getting the most significant four bits. We get four here because these would be
        // leftovers when starting from the least significant bits. In either case, tag the four least
        // significant bits with our stop bit.
        match (n >> QUAD_SHIFT) as u8 {
            // Eat leading zero-bits. This should not be done if the first four bits were non-zero.
            // Additionally, we *must* do this in increments of five bits.
            0 => {
                n <<= QUAD_RESET;
                n |= 1;
                n <<= n.leading_zeros() / 5 * 5;
            }

            // Write value of first four bytes.
            i => {
                n <<= QUAD_RESET;
                n |= 1;
                w.write_char(self.alphabet.alpha[salt.apply(i) as usize] as char)?;
            }
        }

        // From now until we reach the stop bit, take the five most significant bits and then shift
        // left by five bits.
        while n != STOP_BIT {
            w.write_char(
                self.alphabet.alpha[salt.apply((n >> FIVE_SHIFT) as u8) as usize] as char,
            )?;
            n <<= FIVE_RESET;
        }

        Ok(())
    }

    pub fn decode(&self, input: impl AsRef<str>) -> Result<u64> {
        fn get_mapping(idx: usize, u: u8, mapping: &[i8]) -> Result<u8> {
            unsafe {
                match *mapping.get_unchecked(u as usize) {
                    -1 => Err(Error::new(
                        Kind::InvalidDigit(idx, u),
                        "Invalid encoded digit.",
                    )),
                    -2 => Err(Error::new(
                        Kind::CheckDigitUnsupported(idx, u),
                        "Check digits not currently supported.",
                    )),
                    result => Ok(result as u8),
                }
            }
        }

        const BASE: u64 = 0x20;

        let input = input.as_ref();
        match input.len() {
            0 => Err(Error::new(
                Kind::EmptyString,
                "Encoded input string is empty.",
            )),
            n if n > 13 => Err(Error::new(Kind::OutOfRange, "Encoded value is too large")),
            _ => {
                let mut salt = self.salt.byte_source();
                let mut place = BASE.pow(input.len() as u32 - 1);
                let mut n = 0;

                for (idx, u) in input.bytes().enumerate() {
                    let digit = get_mapping(idx, u, &self.alphabet.mapping)?;
                    let digit = salt.apply(digit as u8);
                    n += u64::from(digit).wrapping_mul(place);

                    // This compiles to >>= 5
                    place /= BASE;
                }

                Ok(n)
            }
        }
    }
}

// FIXME: write a test for this?
fn fnv_hash(bytes: &[u8]) -> u32 {
    const P: u32 = 16777619;
    const SEED_HASH: u32 = 2166136261;

    let mut hash = bytes
        .iter()
        .copied()
        .fold(SEED_HASH, |a, b| (a ^ b as u32).wrapping_mul(P));

    hash = hash.wrapping_add(hash << 13);
    hash ^= hash >> 7;
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 17;
    hash.wrapping_add(hash << 5)
}
