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
    type Item = (u16, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.msg_len {
            return None;
        }

        let tag_start = self.i;
        let tag_len = self.msg[tag_start..].iter().position(|&b| b == b'=')?;
        self.i = tag_start + tag_len;

        let tag = self.msg[tag_start..self.i]
            .iter()
            .fold(0u16, |n, &b| n * 10 + (b.wrapping_sub(b'0')) as u16);

        self.i += 1;

        let value_start = self.i;
        while self.i < self.msg_len && self.msg[self.i] != b'\x01' {
            self.i += 1;
        }

        let value = &self.msg[value_start..self.i];

        self.i += 1;

        Some((tag, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_iterator_basic_fields() {
        let msg = b"35=D\x0134=1\x0149=str\x01";
        let mut it = FixIterator::new(msg);

        let f1 = it.next().expect("err");
        assert_eq!(f1.0, 35);
        assert_eq!(f1.1, b"D");

        let f2 = it.next().expect("err");
        assert_eq!(f2.0, 34);
        assert_eq!(f2.1, b"1");

        let p3 = it.next().expect("err");
        assert_eq!(p3.0, 49);
        assert_eq!(p3.1, b"str");

        assert!(it.next().is_none());
    }

    #[test]
    fn test_fix_iterator_full_message() {
        let msg = b"8=FIX.4.2\x019=177\x0135=D\x0134=1\x0149=CLIENT01\x0152=20260223-16:56:36.513\x0156=ENGINE01\x0111=1\x0121=1\x0138=10\x0140=2\x0144=666\x0154=1\x0155=OSISTRING\x0160=20260223-16:56:36.510\x0177=0\x0177=0\x01200=202602\x01201=1\x01202=10\x01204=0\x01205=10\x0110=092\x01";

        let mut it = FixIterator::new(msg);
        let mut v = Vec::new();

        while let Some(f1) = it.next() {
            v.push(f1);
        }

        assert!(v.len() > 10);

        assert_eq!(v[0].0, 8);
        assert_eq!(v[0].1, b"FIX.4.2");

        assert_eq!(v[2].0, 35);
        assert_eq!(v[2].1, b"D");

        let last = v.last().expect("err");
        assert_eq!(last.0, 10);
        assert_eq!(last.1, b"092");
    }

    #[test]
    fn test_fix_iterator_duplicate_tags() {
        let msg = b"77=0\x0177=1\x01";
        let mut it = FixIterator::new(msg);

        let f1 = it.next().expect("err");
        assert_eq!(f1.0, 77);
        assert_eq!(f1.1, b"0");

        let f2 = it.next().expect("err");
        assert_eq!(f2.0, 77);
        assert_eq!(f2.1, b"1");

        assert!(it.next().is_none());
    }

    #[test]
    fn test_fix_iterator_empty_input() {
        let msg = b"";
        let mut it = FixIterator::new(msg);

        assert!(it.next().is_none());
    }

    #[test]
    fn test_fix_iterator_missing_delimiter() {
        let msg = b"35=D";
        let mut it = FixIterator::new(msg);

        assert!(it.next().is_none());
    }
}
