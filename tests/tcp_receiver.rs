use cs144::{
    byte_stream::ByteStream, reassembler::Reassembler, sequence::RelativeSequence,
    tcp_receiver::TcpReceiver, TcpSenderMessage,
};
use rand::Rng;

#[test]
fn connect_1() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(4000);
    let mut reassembler = Reassembler::new(4000);
    assert_eq!(receiver.send(&mut writer).window_size, 4000);
    assert_eq!(receiver.send(&mut writer).ack_no, None);
    assert_eq!(writer.pushed(), 0);
    assert_eq!(reassembler.pending(), 0);

    let message = TcpSenderMessage::new().with_syn().with_seq(0);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(receiver.send(&mut writer).ack_no, Some(RelativeSequence(1)));
    assert_eq!(writer.pushed(), 0);
    assert_eq!(reassembler.pending(), 0);
}

#[test]
fn connect_2() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(5435);
    let mut reassembler = Reassembler::new(5435);
    assert_eq!(receiver.send(&mut writer).ack_no, None);
    assert_eq!(writer.pushed(), 0);
    assert_eq!(reassembler.pending(), 0);

    let message = TcpSenderMessage::new().with_syn().with_seq(89347598);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(89347599))
    );
    assert_eq!(writer.pushed(), 0);
    assert_eq!(reassembler.pending(), 0);
}

#[test]
fn connect_3() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(5435);
    let mut reassembler = Reassembler::new(5435);
    assert_eq!(receiver.send(&mut writer).ack_no, None);
    assert_eq!(writer.pushed(), 0);
    assert_eq!(reassembler.pending(), 0);

    let message = TcpSenderMessage::new().with_seq(893475);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(receiver.send(&mut writer).ack_no, None);
    assert_eq!(writer.pushed(), 0);
    assert_eq!(reassembler.pending(), 0);
}

#[test]
fn connect_4() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(5435);
    let mut reassembler = Reassembler::new(5435);
    assert_eq!(receiver.send(&mut writer).ack_no, None);
    assert_eq!(writer.pushed(), 0);
    assert_eq!(reassembler.pending(), 0);

    let message = TcpSenderMessage::new().with_fin().with_seq(893475);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(receiver.send(&mut writer).ack_no, None);
    assert_eq!(writer.pushed(), 0);
    assert_eq!(reassembler.pending(), 0);
}

#[test]
fn connect_5() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(5435);
    let mut reassembler = Reassembler::new(5435);
    assert_eq!(receiver.send(&mut writer).ack_no, None);
    assert_eq!(writer.pushed(), 0);
    assert_eq!(reassembler.pending(), 0);

    let message = TcpSenderMessage::new().with_fin().with_seq(893475);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(receiver.send(&mut writer).ack_no, None);
    assert_eq!(writer.pushed(), 0);
    assert_eq!(reassembler.pending(), 0);

    let message = TcpSenderMessage::new().with_syn().with_seq(89347598);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(89347599))
    );
    assert_eq!(writer.pushed(), 0);
    assert_eq!(reassembler.pending(), 0);
}

#[test]
fn connect_6() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(4000);
    let mut reassembler = Reassembler::new(4000);

    let message = TcpSenderMessage::new().with_syn().with_seq(5).with_fin();
    receiver.receive(message, &mut reassembler, &mut writer);
    assert!(writer.closed());
    assert_eq!(receiver.send(&mut writer).ack_no, Some(RelativeSequence(7)));
    assert_eq!(writer.pushed(), 0);
    assert_eq!(reassembler.pending(), 0);
}

#[test]
fn in_window_last_segment() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(2358);
    let mut reassembler = Reassembler::new(2358);
    let mut rng = rand::thread_rng();

    let isn = rng.gen();
    let message = TcpSenderMessage::new().with_syn().with_seq(isn);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 1))
    );

    let message = TcpSenderMessage::new().with_seq(isn + 10).with_str("abcd");
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 1))
    );
    assert_eq!(writer.read_all(), "");
    assert_eq!(reassembler.pending(), 4);
    assert_eq!(writer.pushed(), 0);
}

#[test]
fn in_window_later_segment_then_hole_filled() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(2358);
    let mut reassembler = Reassembler::new(2358);
    let mut rng = rand::thread_rng();

    let isn = rng.gen();
    let message = TcpSenderMessage::new().with_syn().with_seq(isn);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 1))
    );

    let message = TcpSenderMessage::new().with_seq(isn + 5).with_str("efgh");
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 1))
    );
    assert_eq!(writer.read_all(), "");
    assert_eq!(reassembler.pending(), 4);
    assert_eq!(writer.pushed(), 0);

    let message = TcpSenderMessage::new().with_seq(isn + 1).with_str("abcd");
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 9))
    );
    assert_eq!(writer.read_all(), "abcdefgh");
    assert_eq!(reassembler.pending(), 0);
    assert_eq!(writer.pushed(), 8);
}

#[test]
fn hole_filled_bit_by_bit() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(2358);
    let mut reassembler = Reassembler::new(2358);
    let mut rng = rand::thread_rng();

    let isn = rng.gen();
    let message = TcpSenderMessage::new().with_syn().with_seq(isn);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 1))
    );

    let messages = [(isn + 5, "efgh"), (isn + 1, "ab"), (isn + 3, "cd")];
    let expected_values = [(1, "", 4, 0), (3, "ab", 4, 2), (9, "cdefgh", 0, 8)];

    for ((seq, data), (ack_no, read_all, pending, pushed)) in
        messages.iter().zip(expected_values.iter())
    {
        let message = TcpSenderMessage::new().with_seq(*seq).with_str(data);
        receiver.receive(message, &mut reassembler, &mut writer);
        assert_eq!(
            receiver.send(&mut writer).ack_no,
            Some(RelativeSequence(isn + ack_no))
        );
        assert_eq!(writer.read_all(), *read_all);
        assert_eq!(reassembler.pending(), *pending);
        assert_eq!(writer.pushed(), *pushed);
    }
}

#[test]
fn many_gaps_filled_bit_by_bit() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(2358);
    let mut reassembler = Reassembler::new(2358);
    let mut rng = rand::thread_rng();

    let isn = rng.gen();
    let message = TcpSenderMessage::new().with_syn().with_seq(isn);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 1))
    );

    let messages = [
        (isn + 5, "e"),
        (isn + 7, "g"),
        (isn + 3, "c"),
        (isn + 1, "ab"),
        (isn + 6, "f"),
        (isn + 4, "d"),
    ];

    let expected_values = [
        (1, "", 1, 0),
        (1, "", 2, 0),
        (1, "", 3, 0),
        (4, "abc", 2, 3),
        (4, "", 3, 3),
        (8, "defg", 0, 7),
    ];

    for ((seq, data), (ack_no, read_all, pending, pushed)) in
        messages.iter().zip(expected_values.iter())
    {
        let message = TcpSenderMessage::new().with_seq(*seq).with_str(data);
        receiver.receive(message, &mut reassembler, &mut writer);
        assert_eq!(
            receiver.send(&mut writer).ack_no,
            Some(RelativeSequence(isn + ack_no))
        );
        assert_eq!(writer.read_all(), *read_all);
        assert_eq!(reassembler.pending(), *pending);
        assert_eq!(writer.pushed(), *pushed);
    }
}

#[test]
fn many_gaps_then_subsumed() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(2358);
    let mut reassembler = Reassembler::new(2358);
    let mut rng = rand::thread_rng();

    let isn = rng.gen();
    let message = TcpSenderMessage::new().with_syn().with_seq(isn);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 1))
    );

    let messages = [
        (isn + 5, "e"),
        (isn + 7, "g"),
        (isn + 3, "c"),
        (isn + 1, "abcdefgh"),
    ];

    let expected_values = [
        (1, "", 1, 0),
        (1, "", 2, 0),
        (1, "", 3, 0),
        (9, "abcdefgh", 0, 8),
    ];

    for ((seq, data), (ack_no, read_all, pending, pushed)) in
        messages.iter().zip(expected_values.iter())
    {
        let message = TcpSenderMessage::new().with_seq(*seq).with_str(data);
        receiver.receive(message, &mut reassembler, &mut writer);
        assert_eq!(
            receiver.send(&mut writer).ack_no,
            Some(RelativeSequence(isn + ack_no))
        );
        assert_eq!(writer.read_all(), *read_all);
        assert_eq!(reassembler.pending(), *pending);
        assert_eq!(writer.pushed(), *pushed);
    }
}

#[test]
fn transmit_2() {
    let isn = 384678;
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(4000);
    let mut reassembler = Reassembler::new(4000);

    let message = TcpSenderMessage::new().with_syn().with_seq(isn);
    receiver.receive(message, &mut reassembler, &mut writer);

    let message = TcpSenderMessage::new().with_seq(isn + 1).with_str("abcd");
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 5))
    );
    assert_eq!(reassembler.pending(), 0);
    assert_eq!(writer.pushed(), 4);
    assert_eq!(writer.read_all(), "abcd");

    let message = TcpSenderMessage::new().with_seq(isn + 5).with_str("efgh");
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 9))
    );
    assert_eq!(reassembler.pending(), 0);
    assert_eq!(writer.pushed(), 8);
    assert_eq!(writer.read_all(), "efgh");
}

#[test]
fn transmit_3() {
    let isn = 5;
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(4000);
    let mut reassembler = Reassembler::new(4000);

    let message = TcpSenderMessage::new().with_syn().with_seq(isn);
    receiver.receive(message, &mut reassembler, &mut writer);

    let message = TcpSenderMessage::new().with_seq(isn + 1).with_str("abcd");
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 5))
    );
    assert_eq!(reassembler.pending(), 0);
    assert_eq!(writer.pushed(), 4);

    let message = TcpSenderMessage::new().with_seq(isn + 5).with_str("efgh");
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 9))
    );
    assert_eq!(reassembler.pending(), 0);
    assert_eq!(writer.pushed(), 8);
    assert_eq!(writer.read_all(), "abcdefgh");
}

#[test]
fn transmit_4() {
    let max_block_size = 10;
    let n_rounds = 10000;
    let isn = 893472;
    let mut bytes_sent = 0;
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(4000);
    let mut reassembler = Reassembler::new(4000);
    let mut rng = rand::thread_rng();

    let message = TcpSenderMessage::new().with_syn().with_seq(isn);
    receiver.receive(message, &mut reassembler, &mut writer);

    for _ in 0..n_rounds {
        let block_size: u32 = rng.gen_range(1..=max_block_size);
        let data: String = (0..block_size)
            .map(|j| {
                let c = 'a' as u8 + ((bytes_sent + j as u32) % 26) as u8;
                c as char
            })
            .collect();

        assert_eq!(
            receiver.send(&mut writer).ack_no,
            Some(RelativeSequence(isn + bytes_sent + 1))
        );
        assert_eq!(writer.pushed(), bytes_sent as usize);

        let message = TcpSenderMessage::new()
            .with_seq(isn + bytes_sent + 1)
            .with_str(&data);
        receiver.receive(message, &mut reassembler, &mut writer);

        bytes_sent += block_size as u32;
        assert_eq!(writer.read_all(), data);
    }
}

#[test]
fn transmit_5() {
    let max_block_size = 10;
    let n_rounds = 100;
    let isn = 238;
    let mut bytes_sent = 0;
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(max_block_size * n_rounds);
    let mut reassembler = Reassembler::new(max_block_size * n_rounds);
    let mut rng = rand::thread_rng();
    let mut all_data = String::new();

    let message = TcpSenderMessage::new().with_syn().with_seq(isn);
    receiver.receive(message, &mut reassembler, &mut writer);

    for _ in 0..n_rounds {
        let block_size: u32 = rng.gen_range(1..=max_block_size as u32);
        let data: String = (0..block_size)
            .map(|j| {
                let c = 'a' as u8 + ((bytes_sent + j as u32) % 26) as u8;
                all_data.push(c as char);
                c as char
            })
            .collect();

        assert_eq!(
            receiver.send(&mut writer).ack_no,
            Some(RelativeSequence(isn + bytes_sent + 1))
        );
        assert_eq!(writer.pushed(), bytes_sent as usize);

        let message = TcpSenderMessage::new()
            .with_seq(isn + bytes_sent + 1)
            .with_str(&data);
        receiver.receive(message, &mut reassembler, &mut writer);

        bytes_sent += block_size;
    }
    assert_eq!(writer.read_all(), all_data);
}

#[test]
fn segment_before_syn() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(4000);
    let mut reassembler = Reassembler::new(4000);
    let mut rng = rand::thread_rng();

    let isn = rng.gen::<u32>();
    assert_eq!(receiver.send(&mut writer).ack_no.is_none(), true);
    let message = TcpSenderMessage::new().with_seq(isn + 1).with_str("hello");
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(receiver.send(&mut writer).ack_no.is_none(), false);
    assert_eq!(reassembler.pending(), 0);
    assert_eq!(writer.read_all(), "");
    assert_eq!(writer.pushed(), 0);

    let message = TcpSenderMessage::new().with_syn().with_seq(isn);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(receiver.send(&mut writer).ack_no.is_none(), true);
    assert_eq!(writer.closed(), false);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 1))
    );
}

#[test]
fn segment_with_syn_and_data() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(4000);
    let mut reassembler = Reassembler::new(4000);
    let mut rng = rand::thread_rng();

    let isn = rng.gen::<u32>();
    assert_eq!(receiver.send(&mut writer).ack_no.is_none(), true);
    let message = TcpSenderMessage::new()
        .with_syn()
        .with_seq(isn)
        .with_str("Hello, CS144!");
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 14))
    );
    assert_eq!(reassembler.pending(), 0);
    assert_eq!(writer.read_all(), "Hello, CS144!");
    assert_eq!(writer.closed(), false);
}

#[test]
fn empty_segment() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(4000);
    let mut reassembler = Reassembler::new(4000);
    let mut rng = rand::thread_rng();

    let isn = rng.gen::<u32>();
    assert_eq!(receiver.send(&mut writer).ack_no.is_none(), true);
    let message = TcpSenderMessage::new().with_syn().with_seq(isn);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 1))
    );
    assert_eq!(reassembler.pending(), 0);

    let message = TcpSenderMessage::new().with_syn().with_seq(isn + 1);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(reassembler.pending(), 0);
    assert_eq!(writer.pushed(), 0);
    assert_eq!(writer.closed(), false);

    let message = TcpSenderMessage::new().with_syn().with_seq(isn + 5);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(reassembler.pending(), 0);
    assert_eq!(writer.pushed(), 0);
    assert_eq!(writer.closed(), false);
}

#[test]
fn segment_with_null_byte() {
    let mut receiver = TcpReceiver::new();
    let mut writer = ByteStream::new(4000);
    let mut reassembler = Reassembler::new(4000);
    let mut rng = rand::thread_rng();

    let isn = rng.gen::<u32>();
    let text = "Here's a null byte:\0and it's gone.";

    assert_eq!(receiver.send(&mut writer).ack_no.is_none(), true);
    let message = TcpSenderMessage::new().with_syn().with_seq(isn);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(reassembler.pending(), 0);
    assert_eq!(writer.pushed(), 0);

    let message = TcpSenderMessage::new().with_seq(isn + 1).with_str(text);
    receiver.receive(message, &mut reassembler, &mut writer);
    assert_eq!(writer.read_all(), text);
    assert_eq!(
        receiver.send(&mut writer).ack_no,
        Some(RelativeSequence(isn + 35))
    );
    assert_eq!(writer.closed(), false);
}
