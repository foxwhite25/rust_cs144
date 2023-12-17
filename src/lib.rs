use core::time;

use byte_stream::ByteStream;
use rand::Rng;
use sequence::RelativeSequence;
use tcp_sender::TcpSender;

pub mod byte_stream;
pub mod reassembler;
pub mod sequence;
pub mod tcp_receiver;
pub mod tcp_sender;

#[derive(Default, Clone, Debug)]
pub struct TcpSenderMessage {
    pub seq_no: RelativeSequence,
    pub syn: bool,
    pub payload: Vec<u8>,
    pub fin: bool,
}

impl TcpSenderMessage {
    pub fn new() -> TcpSenderMessage {
        TcpSenderMessage {
            seq_no: RelativeSequence(0),
            syn: false,
            payload: Default::default(),
            fin: false,
        }
    }

    pub fn with_seq(mut self, no: u32) -> TcpSenderMessage {
        self.seq_no = RelativeSequence(no);
        self
    }

    pub fn with_syn(mut self) -> TcpSenderMessage {
        self.syn = true;
        self
    }

    pub fn with_fin(mut self) -> TcpSenderMessage {
        self.fin = true;
        self
    }

    pub fn with_payload(mut self, payload: &[u8]) -> TcpSenderMessage {
        self.payload = payload.to_vec();
        self
    }

    pub fn with_str(self, content: &str) -> TcpSenderMessage {
        self.with_payload(content.as_bytes())
    }

    pub fn sequence_length(&self) -> usize {
        self.payload.len() + self.fin as usize + self.syn as usize
    }
}

#[derive(Default)]
pub struct TcpReceiverMessage {
    pub ack_no: Option<RelativeSequence>,
    pub window_size: u16,
}

impl TcpReceiverMessage {
    pub fn new() -> TcpReceiverMessage {
        TcpReceiverMessage {
            ack_no: None,
            window_size: 0,
        }
    }

    pub fn with_ack(mut self, no: u32) -> TcpReceiverMessage {
        self.ack_no = Some(RelativeSequence(no));
        self
    }

    pub fn with_window_size(mut self, size: u16) -> TcpReceiverMessage {
        self.window_size = size;
        self
    }
}

pub const DEFAULT_CAPACITY: usize = 64000;
pub const MAX_PAYLOAD_SIZE: usize = 1000;
pub const DEFAULT_TIMEOUT_RT: u64 = 1000;
pub const MAX_RETRY_ATTEMPT: u64 = 8;

pub struct TcpConfig {
    rt_timeout: u64,
    recv_capacity: usize,
    send_capacity: usize,
    fixed_isn: Option<RelativeSequence>,
}

impl TcpConfig {
    pub fn new() -> TcpConfig {
        TcpConfig {
            rt_timeout: DEFAULT_TIMEOUT_RT,
            recv_capacity: DEFAULT_CAPACITY,
            send_capacity: DEFAULT_CAPACITY,
            fixed_isn: None,
        }
    }

    pub fn fixed_isn(mut self, isn: RelativeSequence) -> Self {
        self.fixed_isn = Some(isn);
        self
    }

    pub fn rt_timeout(mut self, timeout: u64) -> Self {
        self.rt_timeout = timeout;
        self
    }

    pub fn recv_capacity(mut self, capacity: usize) -> Self {
        self.recv_capacity = capacity;
        self
    }

    pub fn send_capacity(mut self, capacity: usize) -> Self {
        self.send_capacity = capacity;
        self
    }

    pub fn generate_parts(self) -> (TcpSender, ByteStream) {
        let isn = if let Some(isn) = self.fixed_isn {
            isn
        } else {
            let isn = rand::thread_rng().gen();
            RelativeSequence(isn)
        };
        let byte_stream = ByteStream::new(self.send_capacity);
        let sender = TcpSender::new(isn, self.rt_timeout);
        (sender, byte_stream)
    }
}
