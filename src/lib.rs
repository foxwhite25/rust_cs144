use sequence::RelativeSequence;

pub mod byte_stream;
pub mod reassembler;
pub mod sequence;
pub mod tcp_receiver;

#[derive(Default)]
pub struct TcpSenderMessage {
    seq_no: RelativeSequence,
    syn: bool,
    payload: Vec<u8>,
    fin: bool,
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
