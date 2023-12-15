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
