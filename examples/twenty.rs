use weird::Weird;

// This works now, but I'm not convinced that the randomized alphabet
// works any better than the salt alone, in the grand scheme of things.

fn main() {
    let salt = "In the beginning God created the heaven and the earth.";
    let weird = Weird::from_salt(salt);

    for identifier in (1..=200).map(|n| n + 1_000_000) {
        let encoded = weird.encode(identifier);
        assert_eq!(identifier, weird.decode(&encoded).unwrap());
        println!("{:>6}: {:>6}", identifier, encoded);
    }
}
