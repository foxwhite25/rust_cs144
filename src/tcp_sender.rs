

use crate::{
    byte_stream::ByteStream,
    sequence::{AbsoluteSequence, RelativeSequence},
    TcpReceiverMessage, TcpSenderMessage, MAX_PAYLOAD_SIZE,
};

#[derive(Debug)]
pub struct TcpSender {
    isn: RelativeSequence,
    initial_rto: u64,
    rto_timeout: u64,
    timer: u64,

    syn: bool,
    fin: bool,

    next_abs_seq: AbsoluteSequence,

    windows: u16,
    outstanding_seq: AbsoluteSequence,
    outstanding_segment: Vec<(AbsoluteSequence, TcpSenderMessage)>,
    segment_out: Vec<TcpSenderMessage>,

    retries_times: u64,
}

impl TcpSender {
    /// Creates a new [`TcpSender`].
    pub fn new(isn: RelativeSequence, initial_rto: u64) -> Self {
        Self {
            isn,
            initial_rto,
            rto_timeout: 0,
            timer: 0,
            syn: false,
            fin: false,
            next_abs_seq: AbsoluteSequence(0),
            windows: 1,
            outstanding_seq: AbsoluteSequence(0),
            outstanding_segment: Vec::new(),
            segment_out: Vec::new(),
            retries_times: 0,
        }
    }

    pub fn next_abs_seq(&self) -> AbsoluteSequence {
        self.next_abs_seq
    }

    pub fn next_relative_seq(&self) -> RelativeSequence {
        self.next_abs_seq.wrap(self.isn)
    }

    pub fn seq_in_flight(&self) -> AbsoluteSequence {
        self.outstanding_seq
    }

    pub fn consecutive_retransmissions(&self) -> u64 {
        self.retries_times
    }

    pub fn try_send(&mut self) -> Option<TcpSenderMessage> {
        if !self.syn {
            return None;
        }

        self.segment_out.pop()
    }

    pub fn push(&mut self, reader: &mut ByteStream) {
        let window_size = self.windows.max(1) as usize;

        while window_size > self.outstanding_seq.0 as usize {
            let outstanding_seq = self.outstanding_seq.0 as usize;
            let mut message = TcpSenderMessage::new().with_seq(self.next_relative_seq().0);

            if !self.syn {
                self.syn = true;
                message.syn = true;
            }

            let payload_size = MAX_PAYLOAD_SIZE
                .min(window_size - outstanding_seq - message.syn as usize);
            let payload = reader.read(payload_size);
            let size = payload.len() + outstanding_seq + message.syn as usize;

            if !self.fin && reader.closed() && reader.is_empty() && size < window_size {
                self.fin = true;
                message.fin = true;
            }

            message.payload = payload.chars().map(|x| x as u8).collect();
            if message.sequence_length() == 0 {
                break;
            }

            if self.outstanding_segment.is_empty() {
                self.rto_timeout = self.initial_rto;
                self.timer = 0
            }

            self.outstanding_seq += message.sequence_length() as u64;
            self.outstanding_segment
                .push((self.next_abs_seq(), message.clone()));
            self.next_abs_seq += message.sequence_length() as u64;
            let fin = message.fin;
            self.segment_out.push(message);

            if fin {
                break;
            }
        }
    }

    pub fn send_empty_message(&mut self) -> TcpSenderMessage {
        TcpSenderMessage::new().with_seq(self.next_relative_seq().0)
    }

    pub fn receive(&mut self, message: TcpReceiverMessage) {
        if let Some(ref ack_no) = message.ack_no {
            let recv_abs_seq = ack_no.unwrap(self.isn, self.next_abs_seq);
            if recv_abs_seq > self.next_abs_seq() {
                return;
            }

            let outdated_pos = self
                .outstanding_segment
                .iter()
                .position(|(abs_seq, segment)| {
                    abs_seq.0 + segment.sequence_length() as u64 > recv_abs_seq.0
                });

            let pos = match outdated_pos {
                Some(pos) => pos,
                None => self.outstanding_segment.len(),
            };
            if pos != 0 {
                self.outstanding_seq -= self
                    .outstanding_segment
                    .drain(..pos)
                    .map(|(_, x)| x.sequence_length() as u64)
                    .sum::<u64>();
                self.rto_timeout = self.initial_rto;
                if !self.outstanding_segment.is_empty() {
                    self.timer = 0
                }
            }
        }
        self.windows = message.window_size;
        self.retries_times = 0;
    }

    pub fn tick(&mut self, ms_since: u64) {
        self.timer += ms_since;

        for (_, segment) in self.outstanding_segment.iter() {
            if self.timer < self.rto_timeout {
                return;
            }
            if self.windows > 0 {
                self.rto_timeout *= 2;
            }
            self.timer = 0;
            self.retries_times += 1;
            self.segment_out.push(segment.clone());
        }
    }
}
