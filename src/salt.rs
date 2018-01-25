pub struct StaticSalt(pub &'static [u8]);

pub trait SaltProvider {
    type Salt: Salt;
    fn get(&self) -> Self::Salt;
}

impl SaltProvider for StaticSalt {
    type Salt = StaticSource;

    fn get(&self) -> Self::Salt {
        StaticSource {
            idx: 0,
            data: self.0,
        }
    }
}

pub struct StaticSource {
    idx: usize,
    data: &'static [u8],
}

pub trait Salt {
    fn next(&mut self) -> u8;

    fn shift(&mut self, u: u8) -> u8 {
        let shift_by = self.next();
        u ^ (shift_by % 32)
    }
}

impl Salt for StaticSource {
    fn next(&mut self) -> u8 {
        if self.idx >= self.data.len() {
            self.idx = 1;
            self.data[0]
        } else {
            let ret = self.data[self.idx];
            self.idx += 1;
            ret
        }
    }
}
