use cs144::{
    byte_stream::ByteStream,
    sequence::{AbsoluteSequence, RelativeSequence},
    tcp_sender::TcpSender,
    TcpConfig, TcpReceiverMessage, DEFAULT_TIMEOUT_RT, MAX_PAYLOAD_SIZE, MAX_RETRY_ATTEMPT,
};
use rand::{thread_rng, Rng};

const DEFAULT_TEST_WINDOW: u16 = 137;

#[derive(Debug)]
struct Message {
    syn: Option<bool>,
    fin: Option<bool>,
    seq_no: Option<RelativeSequence>,
    data: Option<String>,
    payload_size: Option<usize>,
}

impl Message {
    fn new() -> Message {
        Message {
            syn: None,
            fin: None,
            seq_no: None,
            data: None,
            payload_size: None,
        }
    }

    fn syn(mut self, syn: bool) -> Message {
        self.syn = Some(syn);
        self
    }

    fn fin(mut self, fin: bool) -> Message {
        self.fin = Some(fin);
        self
    }

    fn no_flag(mut self) -> Message {
        self.syn = Some(false);
        self.fin = Some(false);
        self
    }

    fn seq(mut self, seq: u32) -> Message {
        self.seq_no = Some(RelativeSequence(seq));
        self
    }

    fn data(mut self, data: &str) -> Message {
        self.data = Some(data.to_string());
        self
    }

    fn payload_size(mut self, size: usize) -> Message {
        self.payload_size = Some(size);
        self
    }
}

struct SenderTester {
    sender: TcpSender,
    stream: ByteStream,
}

impl From<TcpConfig> for SenderTester {
    fn from(value: TcpConfig) -> Self {
        let (sender, stream) = value.generate_parts();
        SenderTester { sender, stream }
    }
}

impl SenderTester {
    fn push(mut self, data: &str) -> Self {
        if !data.is_empty() {
            self.stream.push_str(data);
        }
        self.sender.push(&mut self.stream);
        self
    }

    fn push_and_close(mut self, data: &str) -> Self {
        if !data.is_empty() {
            self.stream.push_str(data);
        }
        self.stream.close();
        self.sender.push(&mut self.stream);
        self
    }

    fn close(self) -> Self {
        self.push_and_close("")
    }

    fn expect_message(mut self, message: Message) -> Self {
        dbg!(&message);
        dbg!(&self.sender, &self.stream);
        if let (Some(msg), Some(payload)) = (&message.data, message.payload_size) {
            assert_eq!(msg.len(), payload)
        };
        let Some(seg) = self.sender.try_send() else {
            panic!("Expect message but none was sent!");
        };
        dbg!(&seg);
        if let Some(syn) = message.syn {
            assert_eq!(syn, seg.syn);
        };
        if let Some(fin) = message.fin {
            assert_eq!(fin, seg.fin);
        };
        if let Some(seq_no) = message.seq_no {
            assert_eq!(seq_no, seg.seq_no);
        };
        if let Some(payload_size) = message.payload_size {
            assert_eq!(payload_size, seg.payload.len());
        };
        if seg.payload.len() > MAX_PAYLOAD_SIZE {
            panic!(
                "payload has length {} which is larger than MAX_PAYLOAD_SIZE",
                seg.payload.len()
            );
        }
        if let Some(data) = message.data {
            assert_eq!(
                data,
                seg.payload.into_iter().map(char::from).collect::<String>()
            )
        }
        self
    }

    fn expect_no_segment(mut self) -> Self {
        assert!(self.sender.try_send().is_none());
        self
    }

    fn receive(mut self, msg: TcpReceiverMessage, push: bool) -> Self {
        self.sender.receive(msg);
        if push {
            self.sender.push(&mut self.stream);
        }
        self
    }

    fn receive_ackno(self, ack_no: u32) -> Self {
        self.receive(
            TcpReceiverMessage::new()
                .with_ack(ack_no)
                .with_window_size(DEFAULT_TEST_WINDOW),
            true,
        )
    }

    fn receive_ackno_with_window(self, ack_no: u32, window: u16) -> Self {
        self.receive(
            TcpReceiverMessage::new()
                .with_ack(ack_no)
                .with_window_size(window),
            true,
        )
    }

    fn expect_seq_in_flight(self, seq: u64) -> Self {
        assert_eq!(self.sender.seq_in_flight(), AbsoluteSequence(seq));
        self
    }

    fn expect_seq_no(mut self, seq: u32) -> Self {
        assert_eq!(
            self.sender.send_empty_message().seq_no,
            RelativeSequence(seq)
        );
        self
    }

    fn tick(mut self, ms: u64) -> Self {
        self.sender.tick(ms);
        self
    }

    fn expect_max_retx_exceeded(self, to_be: bool) -> Self {
        assert_eq!(
            to_be,
            self.sender.consecutive_retransmissions() > MAX_RETRY_ATTEMPT
        );
        self
    }
}

#[test]
fn repeat_ack_is_ignored() {
    let isn = thread_rng().gen();
    let tester: SenderTester = TcpConfig::new().fixed_isn(RelativeSequence(isn)).into();
    tester
        .push("")
        .expect_message(Message::new().fin(false).syn(true).payload_size(0).seq(isn))
        .expect_no_segment()
        .receive_ackno(isn + 1)
        .push("a")
        .expect_message(Message::new().no_flag().data("a"))
        .expect_no_segment()
        .receive_ackno(isn + 1)
        .expect_no_segment();
}

#[test]
fn old_ack_is_ignored() {
    let isn = thread_rng().gen();
    let tester: SenderTester = TcpConfig::new().fixed_isn(RelativeSequence(isn)).into();
    tester
        .push("")
        .expect_message(Message::new().fin(false).syn(true).payload_size(0).seq(isn))
        .expect_no_segment()
        .receive_ackno(isn + 1)
        .push("a")
        .expect_message(Message::new().no_flag().data("a"))
        .expect_no_segment()
        .receive_ackno(isn + 2)
        .expect_no_segment()
        .push("b")
        .expect_message(Message::new().no_flag().data("b"))
        .expect_no_segment()
        .receive_ackno(isn + 1)
        .expect_no_segment();
}

#[test]
fn impossible_ackno_is_ignored() {
    let isn = thread_rng().gen();
    let tester: SenderTester = TcpConfig::new().fixed_isn(RelativeSequence(isn)).into();
    tester
        .push("")
        .expect_message(Message::new().fin(false).syn(true).payload_size(0).seq(isn))
        .expect_seq_in_flight(1)
        .receive_ackno_with_window(isn + 2, 1000)
        .expect_seq_in_flight(1);
}

#[test]
fn fin_sent_test() {
    let isn = thread_rng().gen();
    let tester: SenderTester = TcpConfig::new().fixed_isn(RelativeSequence(isn)).into();
    tester
        .push("")
        .expect_message(Message::new().fin(false).syn(true).payload_size(0).seq(isn))
        .receive_ackno(isn + 1)
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(0)
        .close()
        .expect_message(Message::new().fin(true).seq(isn + 1))
        .expect_seq_in_flight(1)
        .expect_no_segment();
}

#[test]
fn fin_with_data() {
    let isn = thread_rng().gen();
    let tester: SenderTester = TcpConfig::new().fixed_isn(RelativeSequence(isn)).into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .receive_ackno(isn + 1)
        .expect_seq_in_flight(0)
        .push_and_close("hello")
        .expect_message(Message::new().fin(true).data("hello").seq(isn + 1))
        .expect_seq_in_flight(6)
        .expect_no_segment();
}

#[test]
fn syn_plus_fin() {
    let isn = thread_rng().gen();
    let tester: SenderTester = TcpConfig::new().fixed_isn(RelativeSequence(isn)).into();
    tester
        .receive(TcpReceiverMessage::new().with_window_size(1024), false)
        .close()
        .expect_message(Message::new().syn(true).fin(true).payload_size(0).seq(isn))
        .expect_seq_in_flight(2)
        .expect_no_segment();
}

#[test]
fn fin_acked_test() {
    let isn = thread_rng().gen();
    let tester: SenderTester = TcpConfig::new().fixed_isn(RelativeSequence(isn)).into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).fin(false).payload_size(0).seq(isn))
        .receive_ackno(isn + 1)
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(0)
        .close()
        .expect_message(Message::new().fin(true).seq(isn + 1))
        .expect_seq_in_flight(1)
        .receive_ackno(isn + 2)
        .expect_seq_no(isn + 2)
        .expect_seq_in_flight(0)
        .expect_no_segment();
}

#[test]
fn fin_not_acked_test() {
    let isn = thread_rng().gen();
    let tester: SenderTester = TcpConfig::new().fixed_isn(RelativeSequence(isn)).into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).fin(false).payload_size(0).seq(isn))
        .receive_ackno(isn + 1)
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(0)
        .close()
        .expect_message(Message::new().fin(true).seq(isn + 1))
        .expect_seq_no(isn + 2)
        .expect_seq_in_flight(1)
        .receive_ackno(isn + 1)
        .expect_seq_no(isn + 2)
        .expect_seq_in_flight(1)
        .expect_no_segment();
}

#[test]
fn fin_retx_test() {
    let isn = thread_rng().gen();
    let tester: SenderTester = TcpConfig::new().fixed_isn(RelativeSequence(isn)).into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).fin(false).payload_size(0).seq(isn))
        .receive_ackno(isn + 1)
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(0)
        .close()
        .expect_message(Message::new().fin(true).seq(isn + 1))
        .expect_seq_no(isn + 2)
        .expect_seq_in_flight(1)
        .receive_ackno(isn + 1)
        .expect_seq_no(isn + 2)
        .expect_seq_in_flight(1)
        .expect_no_segment()
        .tick(DEFAULT_TIMEOUT_RT - 1)
        .expect_seq_no(isn + 2)
        .expect_seq_in_flight(1)
        .expect_no_segment()
        .tick(1)
        .expect_message(Message::new().fin(true).seq(isn + 1))
        .expect_seq_no(isn + 2)
        .expect_seq_in_flight(1)
        .expect_no_segment()
        .tick(1)
        .expect_seq_no(isn + 2)
        .expect_seq_in_flight(1)
        .expect_no_segment()
        .receive_ackno(isn + 2)
        .expect_seq_in_flight(0)
        .expect_seq_no(isn + 2)
        .expect_no_segment();
}

#[test]
fn syn_sent_after_first_push() {
    let isn = thread_rng().gen();
    let tester: SenderTester = TcpConfig::new().fixed_isn(RelativeSequence(isn)).into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(1);
}

#[test]
fn syn_acked_test() {
    let isn = thread_rng().gen();
    let tester: SenderTester = TcpConfig::new().fixed_isn(RelativeSequence(isn)).into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(1)
        .receive_ackno(isn + 1)
        .expect_no_segment()
        .expect_seq_in_flight(0);
}

#[test]
fn syn_wrong_ack_test() {
    let isn = thread_rng().gen();
    let tester: SenderTester = TcpConfig::new().fixed_isn(RelativeSequence(isn)).into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(1)
        .receive_ackno(isn)
        .expect_seq_no(isn + 1)
        .expect_no_segment()
        .expect_seq_in_flight(1);
}

#[test]
fn syn_acked_data() {
    let isn = thread_rng().gen();
    let tester: SenderTester = TcpConfig::new().fixed_isn(RelativeSequence(isn)).into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(1)
        .receive_ackno(isn + 1)
        .expect_no_segment()
        .expect_seq_in_flight(0)
        .push("abcdefgh")
        .tick(1)
        .expect_message(Message::new().data("abcdefgh").seq(isn + 1))
        .expect_seq_no(isn + 9)
        .expect_seq_in_flight(8)
        .receive_ackno(isn + 9)
        .expect_no_segment()
        .expect_seq_in_flight(0)
        .expect_seq_no(isn + 9);
}

#[test]
fn timer_stays_running_when_new_segment_sent() {
    let isn = thread_rng().gen();
    let rto = thread_rng().gen_range(30..10000);
    let tester: SenderTester = TcpConfig::new()
        .fixed_isn(RelativeSequence(isn))
        .rt_timeout(rto)
        .into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .receive_ackno_with_window(isn + 1, 1000)
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(0)
        .push("abc")
        .expect_message(Message::new().data("abc").seq(isn + 1).payload_size(3))
        .tick(rto - 5)
        .expect_no_segment()
        .push("def")
        .expect_message(Message::new().data("def").payload_size(3))
        .tick(6)
        .expect_message(Message::new().data("abc").seq(isn + 1))
        .expect_no_segment();
}

#[test]
fn retransmission_still_happens_when_expiration_time_not_hit_exactly() {
    let isn = thread_rng().gen();
    let rto = thread_rng().gen_range(30..10000);
    let tester: SenderTester = TcpConfig::new()
        .fixed_isn(RelativeSequence(isn))
        .rt_timeout(rto)
        .into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .receive_ackno_with_window(isn + 1, 1000)
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(0)
        .push("abc")
        .expect_message(Message::new().data("abc").seq(isn + 1).payload_size(3))
        .tick(rto - 5)
        .expect_no_segment()
        .push("def")
        .expect_message(Message::new().data("def").payload_size(3))
        .tick(200)
        .expect_message(Message::new().data("abc").seq(isn + 1).payload_size(3))
        .expect_no_segment();
}

#[test]
fn timer_restarts_on_ack_of_new_data() {
    let isn = thread_rng().gen();
    let rto = thread_rng().gen_range(30..10000);
    let tester: SenderTester = TcpConfig::new()
        .fixed_isn(RelativeSequence(isn))
        .rt_timeout(rto)
        .into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .receive_ackno_with_window(isn + 1, 1000)
        .expect_seq_in_flight(0)
        .push("abc")
        .expect_message(Message::new().data("abc").seq(isn + 1))
        .tick(rto - 5)
        .push("def")
        .expect_message(Message::new().data("def").seq(isn + 4))
        .receive_ackno_with_window(isn + 4, 1000)
        .tick(rto - 1)
        .expect_no_segment()
        .tick(2)
        .expect_message(Message::new().data("def").seq(isn + 4));
}

#[test]
fn timer_doesnt_restart_without_ack_of_new_data() {
    let isn = thread_rng().gen();
    let rto = thread_rng().gen_range(30..10000);
    let tester: SenderTester = TcpConfig::new()
        .fixed_isn(RelativeSequence(isn))
        .rt_timeout(rto)
        .into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .receive_ackno_with_window(isn + 1, 1000)
        .expect_seq_in_flight(0)
        .push("abc")
        .expect_message(Message::new().data("abc").seq(isn + 1))
        .tick(rto - 5)
        .push("def")
        .expect_message(Message::new().data("def").seq(isn + 4))
        .receive_ackno_with_window(isn + 1, 1000)
        .tick(6)
        .expect_message(Message::new().data("abc").seq(isn + 1))
        .expect_no_segment()
        .tick(rto * 2 - 5)
        .expect_no_segment()
        .tick(8)
        .expect_message(Message::new().data("abc").seq(isn + 1))
        .expect_no_segment();
}

#[test]
fn rto_resets_on_ack_of_new_data() {
    let isn = thread_rng().gen();
    let rto = thread_rng().gen_range(30..10000);
    let tester: SenderTester = TcpConfig::new()
        .fixed_isn(RelativeSequence(isn))
        .rt_timeout(rto)
        .into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .receive_ackno_with_window(isn + 1, 1000)
        .expect_seq_in_flight(0)
        .push("abc")
        .expect_message(Message::new().data("abc").seq(isn + 1))
        .tick(rto - 5)
        .push("def")
        .expect_message(Message::new().data("def").seq(isn + 4))
        .push("ghi")
        .expect_message(Message::new().data("ghi").seq(isn + 7))
        .receive_ackno_with_window(isn + 1, 1000)
        .tick(6)
        .expect_message(Message::new().data("abc").seq(isn + 1))
        .expect_no_segment()
        .tick(rto * 2 - 5)
        .expect_no_segment()
        .tick(5)
        .expect_message(Message::new().data("abc").seq(isn + 1))
        .expect_no_segment()
        .tick(rto * 4 - 5)
        .expect_no_segment()
        .receive_ackno_with_window(isn + 4, 1000)
        .tick(rto - 1)
        .expect_no_segment()
        .tick(2)
        .expect_message(Message::new().data("def").seq(isn + 4))
        .expect_no_segment();
}

#[test]
fn retransmit_a_fin_containing_segment_same_as_any_other() {
    let isn = thread_rng().gen();
    let rto = thread_rng().gen_range(30..10000);
    let tester: SenderTester = TcpConfig::new()
        .fixed_isn(RelativeSequence(isn))
        .rt_timeout(rto)
        .into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .receive_ackno_with_window(isn + 1, 1000)
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(0)
        .push_and_close("abc")
        .expect_message(Message::new().data("abc").seq(isn + 1).fin(true))
        .tick(rto - 1)
        .expect_no_segment()
        .tick(2)
        .expect_message(Message::new().data("abc").seq(isn + 1).fin(true));
}

#[test]
fn retransmit_a_fin_only_segment_same_as_any_other() {
    let isn = thread_rng().gen();
    let rto = thread_rng().gen_range(30..10000);
    let tester: SenderTester = TcpConfig::new()
        .fixed_isn(RelativeSequence(isn))
        .rt_timeout(rto)
        .into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .receive_ackno_with_window(isn + 1, 1000)
        .expect_seq_in_flight(0)
        .push("abc")
        .expect_message(Message::new().data("abc").seq(isn + 1))
        .close()
        .expect_message(Message::new().seq(isn + 4).fin(true))
        .tick(rto - 1)
        .expect_no_segment()
        .receive_ackno_with_window(isn + 4, 1000)
        .tick(rto - 1)
        .expect_no_segment()
        .tick(2)
        .expect_message(Message::new().seq(isn + 4).fin(true))
        .tick(2 * rto - 5)
        .expect_no_segment()
        .tick(10)
        .expect_message(Message::new().seq(isn + 4).fin(true))
        .expect_seq_no(isn + 5);
}

#[test]
fn dont_add_fin_if_this_would_make_the_segment_exceed_the_receivers_window() {
    let isn = thread_rng().gen();
    let rto = thread_rng().gen_range(30..10000);
    let tester: SenderTester = TcpConfig::new()
        .fixed_isn(RelativeSequence(isn))
        .rt_timeout(rto)
        .into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .push_and_close("abc")
        .receive_ackno_with_window(isn + 1, 3)
        .expect_message(Message::new().data("abc").seq(isn + 1))
        .expect_seq_no(isn + 4)
        .expect_seq_in_flight(3)
        .receive_ackno_with_window(isn + 2, 2)
        .expect_no_segment()
        .receive_ackno_with_window(isn + 3, 1)
        .expect_no_segment()
        .receive_ackno_with_window(isn + 4, 1)
        .expect_message(Message::new().seq(isn + 4).fin(true));
}

#[test]
fn retx_syn_twice_at_the_right_times_then_ack() {
    let isn = thread_rng().gen();
    let retx_timeout = thread_rng().gen_range(10..10000);
    let tester: SenderTester = TcpConfig::new()
        .fixed_isn(RelativeSequence(isn))
        .rt_timeout(retx_timeout)
        .into();
    tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .expect_no_segment()
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(1)
        .tick(retx_timeout - 1)
        .expect_no_segment()
        .tick(1)
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(1)
        .receive_ackno(isn + 1)
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(0);
}

#[test]
fn retx_syn_until_too_many_retransmissions() {
    let isn = thread_rng().gen();
    let retx_timeout = thread_rng().gen_range(10..10000);
    let tester: SenderTester = TcpConfig::new()
        .fixed_isn(RelativeSequence(isn))
        .rt_timeout(retx_timeout)
        .into();
    let mut tester = tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .expect_no_segment()
        .expect_seq_no(isn + 1)
        .expect_seq_in_flight(1);
    for attempt_no in 0..MAX_RETRY_ATTEMPT {
        tester = tester
            .tick((retx_timeout << attempt_no) - 1)
            .expect_max_retx_exceeded(false)
            .expect_no_segment()
            .tick(1)
            .expect_max_retx_exceeded(false)
            .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
            .expect_seq_no(isn + 1)
            .expect_seq_in_flight(1);
    }
    tester
        .tick((retx_timeout << MAX_RETRY_ATTEMPT) - 1)
        .expect_max_retx_exceeded(false)
        .tick(1)
        .expect_max_retx_exceeded(true);
}

#[test]
fn send_some_data_the_retx_and_succeed_then_retx_till_limit() {
    let isn = thread_rng().gen();
    let retx_timeout = thread_rng().gen_range(10..10000);
    let tester: SenderTester = TcpConfig::new()
        .fixed_isn(RelativeSequence(isn))
        .rt_timeout(retx_timeout)
        .into();
    let mut tester = tester
        .push("")
        .expect_message(Message::new().syn(true).payload_size(0).seq(isn))
        .expect_no_segment()
        .receive_ackno(isn + 1)
        .push("abcd")
        .expect_message(Message::new().data("abcd").payload_size(4))
        .expect_no_segment()
        .receive_ackno(isn + 5)
        .expect_seq_in_flight(0)
        .push("efgh")
        .expect_message(Message::new().data("efgh").payload_size(4))
        .expect_no_segment()
        .tick(retx_timeout)
        .expect_max_retx_exceeded(false)
        .expect_message(Message::new().data("efgh").payload_size(4))
        .expect_no_segment()
        .receive_ackno(isn + 9)
        .expect_seq_in_flight(0)
        .push("ijkl")
        .expect_message(Message::new().data("ijkl").seq(isn + 9).payload_size(4));
    for attempt_no in 0..MAX_RETRY_ATTEMPT {
        tester = tester
            .tick((retx_timeout << attempt_no) - 1)
            .expect_max_retx_exceeded(false)
            .expect_no_segment()
            .tick(1)
            .expect_max_retx_exceeded(false)
            .expect_message(Message::new().data("ijkl").seq(isn + 9).payload_size(4))
            .expect_seq_in_flight(4);
    }
    tester
        .tick((retx_timeout << MAX_RETRY_ATTEMPT) - 1)
        .expect_max_retx_exceeded(false)
        .tick(1)
        .expect_max_retx_exceeded(true);
}
