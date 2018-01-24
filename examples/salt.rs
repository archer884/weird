extern crate weird;

use weird::*;

static SALT_PROVIDER: StaticSalt = StaticSalt(b"In the beginning, God created the heavens and the earth.");

fn main() {
    let mut a = SALT_PROVIDER.get();
    let mut b = SALT_PROVIDER.get();

    for n in 0..256 {
        let n = n as u8;
        let shifted = a.shift(n);
        let unshifted = b.shift(shifted);

        if n == unshifted {
            println!("{} -> {} -> {}", n, shifted, unshifted);
        } else {
            println!("{} -> {} -> {} FUCK", n, shifted, unshifted);
        }
    }
}
