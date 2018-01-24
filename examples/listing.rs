extern crate weird;

use weird::*;

fn main() {
    let salt = b"In the beginning, God created the heavens and the earth. \
        And the earth was without form, and void; and darkness was upon the face of the waters.";
    let weird = Weird::with_salt(salt);
    
    for n in 1..11 {
        let encoded = weird.encode(n);
        let decoded = weird.decode(&encoded).unwrap();
        // assert_eq!(n, decoded);
        println!("{} -> {} -> {}", n, encoded, decoded);
    }
}
