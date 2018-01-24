mod error;

use error::*;

pub use error::Error;

static UPPERCASE_ENCODING: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub struct Weird<T = StaticSalt> { pub salt: T }

impl Weird {
    pub fn with_salt(salt: &'static [u8]) -> Self {
        Weird { salt: StaticSalt(salt) }
    }
}

pub struct StaticSalt(pub &'static [u8]);

pub trait SaltProvider {
    type Salt: Salt;
    fn get(&self) -> Self::Salt;
}

impl SaltProvider for StaticSalt {
    type Salt = StaticSource;

    fn get(&self) -> Self::Salt {
        StaticSource {
            idx: 0,
            data: self.0,
        }
    }
}

pub struct StaticSource {
    idx: usize,
    data: &'static [u8],
}

pub trait Salt {
    fn next(&mut self) -> u8;

    fn shift(&mut self, u: u8) -> u8 {
        let shift_by = self.next();
        u ^ (shift_by % 32)
    }
}

impl Salt for StaticSource {
    fn next(&mut self) -> u8 {
        if self.idx >= self.data.len() {
            self.idx = 1;
            self.data[0]
        } else {
            let ret = self.data[self.idx];
            self.idx += 1;
            ret
        }
    }
}

impl<T: SaltProvider> Weird<T> {
    pub fn encode(&self, n: u64) -> String {
        let mut buf = String::new();
        self.encode_to(n, &mut buf);
        buf
    }

    pub fn encode_to(&self, mut n: u64, target: &mut String) {
        // After we clear the four most significant bits, the four least significant bits will be
        // replaced with 0001. We can then know to stop once the four most significant bits are,
        // likewise, 0001.
        const STOP_BIT: u64 = 1 << QUAD_SHIFT;

        const QUAD_SHIFT: usize = 60;
        const QUAD_RESET: usize = 4;

        const FIVE_SHIFT: usize = 59;
        const FIVE_RESET: usize = 5;

        let mut salt = self.salt.get();
        let mut xor = |u| UPPERCASE_ENCODING[(u ^ (salt.next() % 32)) as usize] as char;

        if n == 0 {
            target.push(xor(0));
            return;
        }

        // Start by getting the most significant four bits. We get four here because these would be
        // leftovers when starting from the least significant bits. In either case, tag the four least
        // significant bits with our stop bit.
        match (n >> QUAD_SHIFT) as usize {
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
                target.push(xor(i as u8));
            }
        }

        // From now until we reach the stop bit, take the five most significant bits and then shift
        // left by five bits.
        while n != STOP_BIT {
            target.push(xor((n >> FIVE_SHIFT) as u8));
            n <<= FIVE_RESET;
        }
    }

    pub fn decode<S: AsRef<str>>(&self, input: S) -> Result<u64> {
        const BASE: u64 = 0x20;

        let input = input.as_ref();
        match input.len() {
            0 => Err(Error::new(Kind::EmptyString, "Encoded input string is empty.")),
            n if n > 13 => Err(Error::new(Kind::OutOfRange, "Encoded value is too large.")),
            _ => {
                let mut salt = self.salt.get();
                let mut xor = |u| (u ^ (salt.next() % 32));

                let mut place = BASE.pow(input.len() as u32 - 1);
                let mut n = 0;

                for (idx, u) in input.bytes().enumerate() {
                    let digit = to_normal_digit(idx, u, &mut xor)?;
                    n += u64::from(digit).wrapping_mul(place);
                    place /= BASE;
                }

                Ok(n)
            }
        }
    }
}

/// Attempts to convert an ascii digit to a normalized form.
fn to_normal_digit<F: FnMut(u8) -> u8>(idx: usize, u: u8, xor: &mut F) -> Result<u8> {
    static VALUE_MAPPING: [i8; 256] = include!("../resources/u8-mapping.txt");

    unsafe {
        match *VALUE_MAPPING.get_unchecked(xor(u) as usize) {
            -1 => Err(Error::new(Kind::InvalidDigit(idx, u), "Invalid encoded digit.")),
            -2 => Err(Error::new(Kind::CheckDigitUnsupported(idx, u), "Check digits not currently supported.")),
            result => Ok(result as u8),
        }
    }
}
