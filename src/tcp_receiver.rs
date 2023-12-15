use crate::{
    byte_stream::ByteStream,
    reassembler::Reassembler,
    sequence::{AbsoluteSequence, RelativeSequence},
    TcpReceiverMessage, TcpSenderMessage,
};

#[derive(Default)]
pub struct TcpReceiver {
    isn: Option<RelativeSequence>,
}

impl TcpReceiver {
    pub fn new() -> Self {
        TcpReceiver { isn: None }
    }

    pub fn receive(
        &mut self,
        message: TcpSenderMessage,
        reassembler: &mut Reassembler,
        writer: &mut ByteStream,
    ) {
        let isn = match self.isn {
            Some(isn) => isn,
            None => {
                if !message.syn {
                    return;
                }

                self.isn = Some(message.seq_no);
                message.seq_no
            }
        };

        let checkpoint = writer.pushed() as u64 + 1;
        let abs_seq = message.seq_no.unwrap(isn, AbsoluteSequence(checkpoint));
        let stream_index = abs_seq.0 - 1 + message.syn as u64;

        dbg!(checkpoint, abs_seq, stream_index);
        reassembler.push(stream_index as usize, &message.payload, message.fin, writer);
    }

    pub fn send(&mut self, inbound: &mut ByteStream) -> TcpReceiverMessage {
        let window = if inbound.avalible_capacity() > u16::MAX as usize {
            u16::MAX
        } else {
            inbound.avalible_capacity() as u16
        };
        match self.isn {
            Some(isn) => {
                let abs_ackno = inbound.pushed() + inbound.closed() as usize + 1;

                TcpReceiverMessage::new()
                    .with_ack(isn.0 + abs_ackno as u32)
                    .with_window_size(window)
            }
            None => TcpReceiverMessage::new().with_window_size(window),
        }
    }
}
