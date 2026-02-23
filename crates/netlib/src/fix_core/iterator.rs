pub struct FixIterator<'a> {
    i: usize,
    msg: &'a [u8],
    msg_len: usize,
}

impl<'a> FixIterator<'a> {
    pub fn new(msg: &'a [u8]) -> Self {
        Self {
            i: 0,
            msg,
            msg_len: msg.len(),
        }
    }
}

impl<'a> Iterator for FixIterator<'a> {
    type Item = (&'a [u8], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.msg_len {
            return None;
        }

        let tag_start = self.i;
        while self.i < self.msg_len && self.msg[self.i] != b'=' {
            self.i += 1;
        }
        if self.i >= self.msg_len {
            return None;
        }
        let tag = &self.msg[tag_start..self.i];

        self.i += 1;

        let value_start = self.i;
        while self.i < self.msg_len && self.msg[self.i] != b'\x01' {
            self.i += 1;
        }
        if self.i >= self.msg_len {
            return None;
        }

        let value = &self.msg[value_start..self.i];

        self.i += 1;

        Some((tag, value))
    }
}
