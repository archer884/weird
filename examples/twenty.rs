extern crate weird;

use weird::*;

fn main() {
    let salt_a = "In the beginning, God created the heavens and the earth.";

    // Via constructor.
    let weird_a = Weird::with_salt(salt_a);

    // Statically.
    static WEIRD_B: &Weird<&[u8]> = &Weird {
        salt: Salt(b"And the earth was without form, and void; and darkness was upon the face of the waters.")
    };

    for n in 10..21 {
        println!("{}  {}", weird_a.encode(n), WEIRD_B.encode(n));
    }
}
