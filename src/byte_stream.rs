use std::collections::VecDeque;

pub struct ByteStream {
    inner: VecDeque<u8>,
    capacity: usize,
    closed: bool,
    poped: usize,
    pushed: usize,
}

impl ByteStream {
    pub fn new(capacity: usize) -> Self {
        ByteStream {
            inner: VecDeque::with_capacity(capacity),
            capacity,
            closed: false,
            poped: 0,
            pushed: 0,
        }
    }

    pub fn push(&mut self, buf: &[u8]) {
        if self.closed() {
            return;
        }

        let ac = self.avalible_capacity().min(buf.len());
        let bytes = &buf[0..ac];
        self.pushed += ac;
        self.inner.extend(bytes.iter())
    }

    pub fn push_str(&mut self, content: &str) {
        self.push(content.as_bytes())
    }

    pub fn close(&mut self) {
        self.closed = true
    }

    pub fn closed(&self) -> bool {
        self.closed
    }

    pub fn finished(&self) -> bool {
        self.empty() && self.closed()
    }

    pub fn avalible_capacity(&self) -> usize {
        self.capacity - self.inner.len()
    }

    pub fn empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn peek(&self) -> String {
        self.inner.iter().map(|c| *c as char).collect()
    }

    pub fn pop(&mut self, mut count: usize) {
        if self.len() < count {
            count = self.len()
        }
        self.poped += count;
        self.inner.drain(0..count);
    }

    pub fn poped(&self) -> usize {
        self.poped
    }

    pub fn pushed(&self) -> usize {
        self.pushed
    }
}
