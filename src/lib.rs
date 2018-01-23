mod error;

use error::*;

pub use error::Error;

static UPPERCASE_ENCODING: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub struct Weird<A = CrockfordAlphabet, S = Salt> {
    alphabet: A,
    salt: S,
}

pub struct CrockfordAlphabet;

pub trait AlphabetProvider {
    fn encode<T: SaltSource>(&self, u: u8, s: &mut T) -> u8;
    fn decode<T: SaltSource>(&self, u: u8, idx: usize, s: &mut T) -> Result<u8>;
}

impl AlphabetProvider for CrockfordAlphabet {
    fn encode<T: SaltSource>(&self, u: u8, s: &mut T) -> u8 {
        let idx = (u as i32 + s.next()) % 32;
        unsafe {
            *UPPERCASE_ENCODING.get_unchecked(idx as usize)
        }
    }

    fn decode<T: SaltSource>(&self, u: u8, idx: usize, s: &mut T) -> Result<u8> {
        static VALUE_MAPPING: [i8; 256] = include!("../resources/u8-mapping.txt");

        unsafe {
            match *VALUE_MAPPING.get_unchecked(u as usize) {
                -1 => Err(Error::new(Kind::InvalidDigit(idx, u), "Invalid encoded digit.")),
                -2 => Err(Error::new(Kind::CheckDigitUnsupported(idx, u), "Check digits currently unsupported.")),
                result => {
                    let offset = (s.next() % 32) as i8;
                    let result = if result > offset {
                        result - offset
                    } else {
                        offset - result
                    };

                    Ok(result as u8)
                }
            }
        }
    }
}

pub struct Salt { pub data: &'static [u8] }

pub struct SaltInstance {
    idx: usize,
    data: &'static [u8],
}

pub trait SaltProvider {
    type SaltSource: SaltSource;
    fn get_instance(&self) -> Self::SaltSource;
}

impl SaltProvider for Salt {
    type SaltSource = SaltInstance;

    fn get_instance(&self) -> SaltInstance {
        SaltInstance { idx: 0, data: self.data }
    }
}

pub trait SaltSource {
    fn next(&mut self) -> i32;
}

impl SaltSource for SaltInstance {
    fn next(&mut self) -> i32 {
        let result = self.data[self.idx];
        self.idx += 1;
        if self.idx >= self.data.len() {
            self.idx = 0;
        }
        result as i32
    }
}

impl<A, S> Weird<A, S>
where
    A: AlphabetProvider,
    S: SaltProvider,
{
    pub fn new(alphabet: A, salt: S) -> Self {
        Self { alphabet, salt }
    }

    pub fn with_salt(salt: &'static [u8]) -> Weird<CrockfordAlphabet, Salt> {
        Weird {
            alphabet: CrockfordAlphabet,
            salt: Salt { data: salt },
        }
    }

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

        let mut salt_instance = self.salt.get_instance();

        if n == 0 {
            target.push(self.alphabet.encode(0, &mut salt_instance) as char);
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
                target.push(self.alphabet.encode(i as u8, &mut salt_instance) as char);
            }
        }

        // From now until we reach the stop bit, take the five most significant bits and then shift
        // left by five bits.
        while n != STOP_BIT {
            target.push(self.alphabet.encode((n >> FIVE_SHIFT) as u8, &mut salt_instance) as char);
            n <<= FIVE_RESET;
        }
    }

    pub fn decode<T: AsRef<str>>(&self, input: T) -> Result<u64> {
        const BASE: u64 = 0x20;

        let input = input.as_ref();
        match input.len() {
            0 => Err(Error::new(Kind::EmptyString, "Encoded input string is empty.")),
            n if n > 13 => Err(Error::new(Kind::OutOfRange, "Encoded value is too large.")),
            _ => {
                let mut salt_instance = self.salt.get_instance();
                let mut place = BASE.pow(input.len() as u32 - 1);
                let mut n = 0;

                for (idx, u) in input.bytes().enumerate() {
                    let digit = self.alphabet.decode(u, idx, &mut salt_instance)?;
                    n += u64::from(digit).wrapping_mul(place);
                    place /= BASE;
                }

                Ok(n)
            }
        }
    }
}

#[cfg(test)]
mod encoding {
    use super::*;

    #[test]
    fn zero_returns_zero() {
        let weird = get_weird();
        assert_eq!("0", &*weird.encode(0));
    }

    #[test]
    fn large_value_returns_correct_large_value() {
        let weird = get_weird();
        assert_eq!("1ZZZ", &*weird.encode(65535));
    }

    #[test]
    fn x5111_is_4zq() {
        let weird = get_weird();
        assert_eq!("4ZQ", &*weird.encode(5111));
    }

    #[test]
    fn x18446744073709551615_is_fzzzzzzzzzzzz() {
        let weird = get_weird();
        assert_eq!("FZZZZZZZZZZZZ", &*weird.encode(18446744073709551615));
    }

    struct NullSaltProvider;

    impl SaltProvider for NullSaltProvider {
        type SaltSource = NullSaltInstance;

        fn get_instance(&self) -> Self::SaltSource {
            NullSaltInstance
        }
    }

    struct NullSaltInstance;

    impl SaltSource for NullSaltInstance {
        fn next(&mut self) -> i32 { 0 }
    }

    // Weird with nullsaltprovider is 100% compatible with Crockford Base32.
    fn get_weird() -> Weird<CrockfordAlphabet, NullSaltProvider> {
        Weird {
            alphabet: CrockfordAlphabet,
            salt: NullSaltProvider,
        }
    }
}

#[cfg(test)]
mod decoding {
    use super::*;

    #[test]
    fn zero_length_strings_fail() {
        let weird = get_weird();
        let input = "";
        let expected = Err(Error::new(Kind::EmptyString, "Don't care"));
        let actual = weird.decode(input);

        assert_eq!(expected, actual);
    }

    #[test]
    fn long_strings_fail() {
        let weird = get_weird();
        let input = "12345678910121";
        let expected = Err(Error::new(Kind::OutOfRange, "Don't care"));
        let actual = weird.decode(input);

        assert_eq!(expected, actual);
    }

    #[test]
    fn invalid_bytes_fail() {
        let weird = get_weird();
        let input = "fZZ!2";
        let expected = Err(Error::new(Kind::InvalidDigit(3, 33), "Don't care"));
        let actual = weird.decode(input);

        assert_eq!(expected, actual);
    }

    #[test]
    fn zero_becomes_zero() {
        let weird = get_weird();
        let input = "0";
        let expected = Ok(0);
        let actual = weird.decode(input);

        assert_eq!(expected, actual);
    }

    #[test]
    fn large_values_become_large_values() {
        let weird = get_weird();
        assert_eq!(Ok(65535), weird.decode("1zzz"));
        assert_eq!(Ok(65535), weird.decode("1ZZZ"));
    }

    #[test]
    fn map_to_0() {
        let weird = get_weird();
        assert_eq!(Ok(0), weird.decode("O"));
        assert_eq!(Ok(0), weird.decode("o"));
    }

    #[test]
    fn map_to_1() {
        let weird = get_weird();
        assert_eq!(Ok(1), weird.decode("I"));
        assert_eq!(Ok(1), weird.decode("i"));
        assert_eq!(Ok(1), weird.decode("L"));
        assert_eq!(Ok(1), weird.decode("l"));
    }

    #[test]
    fn z_equals_31() {
        let weird = get_weird();
        assert_eq!(Ok(31), weird.decode("z"));
        assert_eq!(Ok(31), weird.decode("Z"));
    }

    #[test]
    fn q_equals_23() {
        let weird = get_weird();
        assert_eq!(Ok(23), weird.decode("q"));
        assert_eq!(Ok(23), weird.decode("Q"));
    }

    #[test]
    fn four_z_q_works() {
        let weird = get_weird();
        assert_eq!(Ok(5111), weird.decode("4zq"));
        assert_eq!(Ok(5111), weird.decode("4ZQ"));
    }

    #[test]
    fn max_value_works() {
        let weird = get_weird();
        assert_eq!(Ok(18446744073709551615), weird.decode("fzzzzzzzzzzzz"));
    }

    #[test]
    fn u_produces_an_error_instead_of_a_crash() {
        let weird = get_weird();
        assert!(weird.decode("iVuv").is_err());
        assert!(weird.decode("iVUv").is_err());
    }

    struct NullSaltProvider;

    impl SaltProvider for NullSaltProvider {
        type SaltSource = NullSaltInstance;

        fn get_instance(&self) -> Self::SaltSource {
            NullSaltInstance
        }
    }

    struct NullSaltInstance;

    impl SaltSource for NullSaltInstance {
        fn next(&mut self) -> i32 { 0 }
    }

    // Weird with nullsaltprovider is 100% compatible with Crockford Base32.
    fn get_weird() -> Weird<CrockfordAlphabet, NullSaltProvider> {
        Weird {
            alphabet: CrockfordAlphabet,
            salt: NullSaltProvider,
        }
    }
}
