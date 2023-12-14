use crate::byte_stream::ByteStream;

pub struct Reassembler {
    buffer: Vec<Option<u8>>,
    current_index: usize,
}

impl Reassembler {
    pub fn new(capacity: usize) -> Reassembler {
        Reassembler {
            buffer: vec![None; capacity],
            current_index: 0,
        }
    }

    pub fn push(&mut self, first_index: usize, data: &[u8], last: bool, writer: &mut ByteStream) {
        if last {
            writer.close();
        }

        let (data, first_index) = if first_index <= self.current_index {
            let diff = self.current_index - first_index;
            if data.len() < diff {
                return;
            }
            let data = &data[diff..];
            (data, 0)
        } else {
            (data, first_index - self.current_index)
        };

        if data.len() == 0 {
            return;
        }

        if first_index >= self.buffer.len() {
            return;
        }

        let last_index = data.len().min(writer.avalible_capacity()) + first_index;

        self.buffer[first_index..last_index]
            .iter_mut()
            .zip(data.iter())
            .for_each(|(buf, d)| *buf = Some(*d));

        let buf = self
            .buffer
            .iter_mut()
            .map_while(|x| x.take())
            .collect::<Vec<_>>();

        for i in 0..(self.buffer.len() - buf.len()) {
            self.buffer[i] = self.buffer[i + buf.len()]
        }
        for i in (self.buffer.len() - buf.len())..self.buffer.len() {
            self.buffer[i] = None
        }

        if buf.is_empty() {
            return;
        }
        self.current_index += writer.push(&buf);
    }

    pub fn push_str(
        &mut self,
        first_index: usize,
        data: &str,
        last: bool,
        writer: &mut ByteStream,
    ) {
        self.push(first_index, data.as_bytes(), last, writer)
    }

    pub fn pending(&self) -> usize {
        self.buffer.iter().filter(|x| x.is_some()).count()
    }
}

#[cfg(test)]
mod test {
    use crate::byte_stream::ByteStream;

    use super::Reassembler;

    #[test]
    fn all_within_capacity() {
        let mut reassembler = Reassembler::new(2);
        let mut buf = ByteStream::new(2);
        reassembler.push_str(0, "ab", false, &mut buf);
        assert_eq!(buf.pushed(), 2);
        assert_eq!(reassembler.pending(), 0);
        assert_eq!(buf.read_all(), "ab");

        reassembler.push_str(2, "cd", false, &mut buf);
        assert_eq!(buf.pushed(), 4);
        assert_eq!(reassembler.pending(), 0);
        assert_eq!(buf.read_all(), "cd");

        reassembler.push_str(4, "ef", false, &mut buf);
        assert_eq!(buf.pushed(), 6);
        assert_eq!(reassembler.pending(), 0);
        assert_eq!(buf.read_all(), "ef");
    }

    #[test]
    fn insert_beyond_capacity() {
        let mut reassembler = Reassembler::new(2);
        let mut buf = ByteStream::new(2);

        reassembler.push_str(0, "ab", false, &mut buf);
        assert_eq!(buf.pushed(), 2);
        assert_eq!(reassembler.pending(), 0);

        reassembler.push_str(2, "cd", false, &mut buf);
        assert_eq!(buf.pushed(), 2);
        assert_eq!(reassembler.pending(), 0);

        assert_eq!(buf.read_all(), "ab");
        assert_eq!(buf.pushed(), 2);
        assert_eq!(reassembler.pending(), 0);

        reassembler.push_str(2, "cd", false, &mut buf);
        assert_eq!(buf.pushed(), 4);
        assert_eq!(reassembler.pending(), 0);

        assert_eq!(buf.read_all(), "cd");
    }

    #[test]
    fn overlapping_inserts() {
        let mut reassembler = Reassembler::new(1);
        let mut buf = ByteStream::new(1);

        reassembler.push_str(0, "ab", false, &mut buf);
        assert_eq!(buf.pushed(), 1);
        assert_eq!(reassembler.pending(), 0);

        reassembler.push_str(0, "ab", false, &mut buf);
        assert_eq!(buf.pushed(), 1);
        assert_eq!(reassembler.pending(), 0);

        assert_eq!(buf.read_all(), "a");
        assert_eq!(buf.pushed(), 1);
        assert_eq!(reassembler.pending(), 0);

        reassembler.push_str(0, "abc", false, &mut buf);
        assert_eq!(buf.pushed(), 2);
        assert_eq!(reassembler.pending(), 0);

        assert_eq!(buf.read_all(), "b");
        assert_eq!(buf.pushed(), 2);
        assert_eq!(reassembler.pending(), 0);
    }

    #[test]
    fn insert_beyond_capacity_repeated_with_different_data() {
        let mut reassembler = Reassembler::new(2);
        let mut buf = ByteStream::new(2);

        reassembler.push_str(1, "b", false, &mut buf);
        assert_eq!(buf.pushed(), 0);
        assert_eq!(reassembler.pending(), 1);

        reassembler.push_str(2, "bX", false, &mut buf);
        assert_eq!(buf.pushed(), 0);
        assert_eq!(reassembler.pending(), 1);

        reassembler.push_str(0, "a", false, &mut buf);
        assert_eq!(buf.pushed(), 2);
        assert_eq!(reassembler.pending(), 0);
        assert_eq!(buf.read_all(), "ab");

        reassembler.push_str(1, "bc", false, &mut buf);
        assert_eq!(buf.pushed(), 3);
        assert_eq!(reassembler.pending(), 0);

        assert_eq!(buf.read_all(), "c");
    }

    #[test]
    fn dup_1() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);
        reassembler.push_str(0, "abcd", false, &mut buf);
        assert_eq!(buf.pushed(), 4);
        assert_eq!(buf.read_all(), "abcd");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(0, "abcd", false, &mut buf);
        assert_eq!(buf.pushed(), 4);
        assert_eq!(buf.read_all(), "");
        assert_eq!(buf.finished(), false);
    }

    #[test]
    fn dup_2() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);
        reassembler.push_str(0, "abcd", false, &mut buf);
        assert_eq!(buf.pushed(), 4);
        assert_eq!(buf.read_all(), "abcd");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(4, "abcd", false, &mut buf);
        assert_eq!(buf.pushed(), 8);
        assert_eq!(buf.read_all(), "abcd");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(0, "abcd", false, &mut buf);
        assert_eq!(buf.pushed(), 8);
        assert_eq!(buf.read_all(), "");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(4, "abcd", false, &mut buf);
        assert_eq!(buf.pushed(), 8);
        assert_eq!(buf.read_all(), "");
        assert_eq!(buf.finished(), false);
    }

    #[test]
    fn dup_3() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);
        let data = "abcdefgh";
        reassembler.push_str(0, data, false, &mut buf);
        assert_eq!(buf.pushed(), 8);
        assert_eq!(buf.read_all(), "abcdefgh");
        assert_eq!(buf.finished(), false);

        for _ in 0..1000 {
            let start_i: usize = rng.gen_range(0..9);
            let end_i: usize = rng.gen_range(start_i..9);
            let sub_data = &data[start_i..end_i];
            reassembler.push_str(start_i, sub_data, false, &mut buf);
            assert_eq!(buf.pushed(), 8);
            assert_eq!(buf.read_all(), "");
            assert_eq!(buf.finished(), false);
        }
    }

    #[test]
    fn dup_4() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);
        reassembler.push_str(0, "abcd", false, &mut buf);
        assert_eq!(buf.pushed(), 4);
        assert_eq!(buf.read_all(), "abcd");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(0, "abcdef", false, &mut buf);
        assert_eq!(buf.pushed(), 6);
        assert_eq!(buf.read_all(), "ef");
        assert_eq!(buf.finished(), false);
    }
    #[test]
    fn holes_1() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);
        reassembler.push_str(1, "b", false, &mut buf);
        assert_eq!(buf.pushed(), 0);
        assert_eq!(buf.read_all(), "");
        assert_eq!(buf.finished(), false);
    }

    #[test]
    fn holes_2() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);
        reassembler.push_str(1, "b", false, &mut buf);
        reassembler.push_str(0, "a", false, &mut buf);
        assert_eq!(buf.pushed(), 2);
        assert_eq!(buf.read_all(), "ab");
        assert_eq!(buf.finished(), false);
    }

    #[test]
    fn holes_3() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);
        reassembler.push_str(1, "b", true, &mut buf);
        assert_eq!(buf.pushed(), 0);
        assert_eq!(buf.read_all(), "");
        assert_eq!(buf.finished(), false);
        reassembler.push_str(0, "a", false, &mut buf);
        assert_eq!(buf.pushed(), 2);
        assert_eq!(buf.read_all(), "ab");
        assert_eq!(buf.finished(), true);
    }

    #[test]
    fn holes_4() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);
        reassembler.push_str(1, "b", false, &mut buf);
        reassembler.push_str(0, "ab", false, &mut buf);
        assert_eq!(buf.pushed(), 2);
        assert_eq!(buf.read_all(), "ab");
        assert_eq!(buf.finished(), false);
    }

    #[test]
    fn holes_5() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);
        reassembler.push_str(1, "b", false, &mut buf);
        assert_eq!(buf.pushed(), 0);
        assert_eq!(buf.read_all(), "");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(3, "d", false, &mut buf);
        assert_eq!(buf.pushed(), 0);
        assert_eq!(buf.read_all(), "");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(2, "c", false, &mut buf);
        assert_eq!(buf.pushed(), 0);
        assert_eq!(buf.read_all(), "");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(0, "a", false, &mut buf);
        assert_eq!(buf.pushed(), 4);
        assert_eq!(buf.read_all(), "abcd");
        assert_eq!(buf.finished(), false);
    }

    #[test]
    fn holes_6() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);
        reassembler.push_str(1, "b", false, &mut buf);
        assert_eq!(buf.pushed(), 0);
        assert_eq!(buf.read_all(), "");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(3, "d", false, &mut buf);
        assert_eq!(buf.pushed(), 0);
        assert_eq!(buf.read_all(), "");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(0, "abc", false, &mut buf);
        assert_eq!(buf.pushed(), 4);
        assert_eq!(buf.read_all(), "abcd");
        assert_eq!(buf.finished(), false);
    }

    #[test]
    fn holes_7() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);
        reassembler.push_str(1, "b", false, &mut buf);
        assert_eq!(buf.pushed(), 0);
        assert_eq!(buf.read_all(), "");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(3, "d", false, &mut buf);
        assert_eq!(buf.pushed(), 0);
        assert_eq!(buf.read_all(), "");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(0, "a", false, &mut buf);
        assert_eq!(buf.pushed(), 2);
        assert_eq!(buf.read_all(), "ab");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(2, "c", false, &mut buf);
        assert_eq!(buf.pushed(), 4);
        assert_eq!(buf.read_all(), "cd");
        assert_eq!(buf.finished(), false);

        reassembler.push_str(4, "", true, &mut buf);
        assert_eq!(buf.pushed(), 4);
        assert_eq!(buf.read_all(), "");
        assert_eq!(buf.finished(), true);
    }

    #[test]
    fn seq_1() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);

        reassembler.push_str(0, "abcd", false, &mut buf);
        assert_eq!(buf.pushed(), 4);
        assert_eq!(buf.read_all(), "abcd");
        assert!(!buf.finished());

        reassembler.push_str(4, "efgh", false, &mut buf);
        assert_eq!(buf.pushed(), 8);
        assert_eq!(buf.read_all(), "efgh");
        assert!(!buf.finished());
    }

    #[test]
    fn seq_2() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);

        reassembler.push_str(0, "abcd", false, &mut buf);
        assert_eq!(buf.pushed(), 4);
        assert!(!buf.finished());

        reassembler.push_str(4, "efgh", false, &mut buf);
        assert_eq!(buf.pushed(), 8);
        assert_eq!(buf.read_all(), "abcdefgh");
        assert!(!buf.finished());
    }

    #[test]
    fn seq_3() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);
        let mut expected_string = String::new();

        for i in 0..100 {
            reassembler.push_str(4 * i, "abcd", false, &mut buf);
            assert_eq!(buf.pushed(), 4 * (i + 1));
            assert!(!buf.finished());

            expected_string.push_str("abcd");
        }

        assert_eq!(buf.read_all(), expected_string);
        assert!(!buf.finished());
    }

    #[test]
    fn seq_4() {
        let mut reassembler = Reassembler::new(65000);
        let mut buf = ByteStream::new(65000);

        for i in 0..100 {
            reassembler.push_str(4 * i, "abcd", false, &mut buf);
            assert_eq!(buf.pushed(), 4 * (i + 1));
            assert_eq!(buf.read_all(), "abcd");
            assert!(!buf.finished());
        }
    }
}
