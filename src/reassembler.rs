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

        if data.is_empty() {
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
