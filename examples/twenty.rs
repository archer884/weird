use weird::{Weird, Salt};

fn main() {
    let salt_a = "In the beginning God created the heaven and the earth.";

    // Via constructor.
    let weird_a = Weird::new(salt_a);

    // Statically.
    static WEIRD_B: &Weird<&[u8]> = &Weird {
        salt: Salt(b"And the earth was without form, and void; and darkness was upon the face of the deep.")
    };

    for n in 10..21 {
        let a = weird_a.encode(n);
        let b = WEIRD_B.encode(n);

        assert_eq!(n, weird_a.decode(&a).unwrap());
        assert_eq!(n, WEIRD_B.decode(&b).unwrap());
        
        println!("A: {} B: {}", a, b);
    }
}
