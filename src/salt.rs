pub struct Salt<T>(pub T);

impl<T: AsRef<[u8]>> Salt<T> {
    pub(crate) fn get(&self) -> ByteSource {
        ByteSource {
            idx: 0,
            data: self.0.as_ref(),
        }
    }
}

pub struct ByteSource<'a> {
    idx: usize,
    data: &'a [u8],
}

impl<'a> ByteSource<'a> {
    pub(crate) fn next(&mut self) -> u8 {
        if self.idx == self.data.len() {
            self.idx = 0;
            self.next();
        }

        let ret = self.data[self.idx];
        self.idx += 1;
        ret
    }

    pub(crate) fn apply(&mut self, u: u8) -> u8 {
        let x = self.next() % 32;
        u ^ x
    }
}
