use weird::{Alphabet, Weird, WeirdWithAlphabet};

// This works now, but I'm not convinced that the randomized alphabet
// works any better than the salt alone, in the grand scheme of things.

fn main() {
    let salt = "In the beginning God created the heaven and the earth.";
    let weird = Weird::new(salt);
    let weirder = WeirdWithAlphabet::new(
        salt,
        Alphabet::new("4836Q1XBVM59THGRS7Y2ADW0NFZJKPEC").unwrap(),
    );

    for n in 1..=200 {
        let a = weird.encode(n);
        let b = weirder.encode(n);

        assert_eq!(weird.decode(&a).unwrap(), n);
        assert_eq!(weirder.decode(&b).unwrap(), n);

        println!("{:>3}: {:>2} / {:>2}", n, a, b);
    }
}
