extern crate weird;

use weird::*;

fn main() {
    let salt_a = b"In the beginning, God created the heavens and the earth.";
    let salt_b = b"And the earth was without form, and void; and darkness was upon the face of the waters.";

    let weird_a = Weird::with_salt(salt_a);
    let weird_b = Weird::with_salt(salt_b);

    for n in 10..21 {
        println!("{}  {}", weird_a.encode(n), weird_b.encode(n));
    }
}
