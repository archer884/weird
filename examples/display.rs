use std::fmt::{self, Display};

use weird::Weird;

struct DisplayAdapter<'w, S> {
    value: u64,
    weird: &'w Weird<S>,
}

impl<'w, S> DisplayAdapter<'w, S> {
    fn new(value: u64, weird: &'w Weird<S>) -> Self {
        Self { value, weird }
    }
}

impl<'w, S: AsRef<[u8]>> Display for DisplayAdapter<'w, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        self.weird.encode_into(self.value, f)
    }
}

fn main() {
    let weird = Weird::new("<insert funny salt here>");
    let id = 1337331;
    let encoded = weird.encode(id);
    let adapter = DisplayAdapter::new(id, &weird);
    let other = format!("{}", adapter);
    println!("{} / {}", encoded, other);
}
