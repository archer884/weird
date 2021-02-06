use std::fmt::{self, Write};

mod error;
mod salt;

pub use error::*;
pub use salt::*;

pub struct Weird<T> {
    pub salt: Salt<T>,
}

impl<T: AsRef<[u8]>> Weird<T> {
    pub fn new(salt: T) -> Self {
        Weird { salt: Salt(salt) }
    }
}

impl<T: AsRef<[u8]>> Weird<T> {
    pub fn encode(&self, n: u64) -> String {
        let mut buf = String::with_capacity(13);
        self.encode_into(n, &mut buf)
            .expect("Cannot fail to encode into a string");
        buf
    }

    pub fn encode_into<W: Write>(&self, mut n: u64, w: &mut W) -> fmt::Result {
        static UPPERCASE_ENCODING: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

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

        let mut salt = self.salt.get();

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
                w.write_char(UPPERCASE_ENCODING[salt.apply(i) as usize] as char)?;
            }
        }

        // From now until we reach the stop bit, take the five most significant bits and then shift
        // left by five bits.
        while n != STOP_BIT {
            w.write_char(UPPERCASE_ENCODING[salt.apply((n >> FIVE_SHIFT) as u8) as usize] as char)?;
            n <<= FIVE_RESET;
        }

        Ok(())
    }

    pub fn decode<S: AsRef<str>>(&self, input: S) -> Result<u64> {
        const BASE: u64 = 0x20;

        let input = input.as_ref();
        match input.len() {
            0 => Err(Error::new(
                Kind::EmptyString,
                "Encoded input string is empty.",
            )),
            n if n > 13 => Err(Error::new(Kind::OutOfRange, "Encoded value is too large")),
            _ => {
                let mut salt = self.salt.get();
                let mut place = BASE.pow(input.len() as u32 - 1);
                let mut n = 0;

                for (idx, u) in input.bytes().enumerate() {
                    let digit = to_normal_digit(idx, u)?;
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

/// Attempts to convert an ascii digit to a normalized form.
fn to_normal_digit(idx: usize, u: u8) -> Result<u8> {
    static VALUE_MAPPING: [i8; 256] = include!("../resources/u8-mapping.txt");

    unsafe {
        match *VALUE_MAPPING.get_unchecked(u as usize) {
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
