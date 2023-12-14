use cs144::{byte_stream::ByteStream, reassembler::Reassembler};

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
#[test]
fn overlapping_assembled_unread_section() {
    let mut reassembler = Reassembler::new(1000);
    let mut buf = ByteStream::new(1000);

    reassembler.push_str(0, "a", false, &mut buf);
    reassembler.push_str(0, "ab", false, &mut buf);

    assert_eq!(buf.pushed(), 2);
    assert_eq!(buf.read_all(), "ab");
}

#[test]
fn overlapping_assembled_read_section() {
    let mut reassembler = Reassembler::new(1000);
    let mut buf = ByteStream::new(1000);

    reassembler.push_str(0, "a", false, &mut buf);
    assert_eq!(buf.read_all(), "a");

    reassembler.push_str(0, "ab", false, &mut buf);
    assert_eq!(buf.read_all(), "b");
    assert_eq!(buf.pushed(), 2);
}

#[test]
fn overlapping_unassembled_section_to_fill_hole() {
    let mut reassembler = Reassembler::new(1000);
    let mut buf = ByteStream::new(1000);

    reassembler.push_str(1, "b", false, &mut buf);
    assert_eq!(buf.read_all(), "");

    reassembler.push_str(0, "ab", false, &mut buf);
    assert_eq!(buf.read_all(), "ab");
    assert_eq!(reassembler.pending(), 0);
    assert_eq!(buf.pushed(), 2);
}

#[test]
fn overlapping_unassembled_section() {
    let mut reassembler = Reassembler::new(1000);
    let mut buf = ByteStream::new(1000);

    reassembler.push_str(1, "b", false, &mut buf);
    assert_eq!(buf.read_all(), "");

    reassembler.push_str(1, "bc", false, &mut buf);
    assert_eq!(buf.read_all(), "");
    assert_eq!(reassembler.pending(), 2);
    assert_eq!(buf.pushed(), 0);
}

#[test]
fn overlapping_unassembled_section_2() {
    let mut reassembler = Reassembler::new(1000);
    let mut buf = ByteStream::new(1000);

    reassembler.push_str(2, "c", false, &mut buf);
    assert_eq!(buf.read_all(), "");

    reassembler.push_str(1, "bcd", false, &mut buf);
    assert_eq!(buf.read_all(), "");
    assert_eq!(reassembler.pending(), 3);
    assert_eq!(buf.pushed(), 0);
}

#[test]
fn overlapping_multiple_unassembled_sections() {
    let mut reassembler = Reassembler::new(1000);
    let mut buf = ByteStream::new(1000);

    reassembler.push_str(1, "b", false, &mut buf);
    reassembler.push_str(3, "d", false, &mut buf);
    assert_eq!(buf.read_all(), "");

    reassembler.push_str(1, "bcde", false, &mut buf);
    assert_eq!(buf.read_all(), "");
    assert_eq!(buf.pushed(), 0);
    assert_eq!(reassembler.pending(), 4);
}

#[test]
fn insert_over_existing_section() {
    let mut reassembler = Reassembler::new(1000);
    let mut buf = ByteStream::new(1000);

    reassembler.push_str(2, "c", false, &mut buf);
    reassembler.push_str(1, "bcd", false, &mut buf);

    assert_eq!(buf.read_all(), "");
    assert_eq!(buf.pushed(), 0);
    assert_eq!(reassembler.pending(), 3);

    reassembler.push_str(0, "a", false, &mut buf);
    assert_eq!(buf.read_all(), "abcd");
    assert_eq!(buf.pushed(), 4);
    assert_eq!(reassembler.pending(), 0);
}

#[test]
fn insert_within_existing_section() {
    let mut reassembler = Reassembler::new(1000);
    let mut buf = ByteStream::new(1000);

    reassembler.push_str(1, "bcd", false, &mut buf);
    reassembler.push_str(2, "c", false, &mut buf);

    assert_eq!(buf.read_all(), "");
    assert_eq!(buf.pushed(), 0);
    assert_eq!(reassembler.pending(), 3);

    reassembler.push_str(0, "a", false, &mut buf);
    assert_eq!(buf.read_all(), "abcd");
    assert_eq!(buf.pushed(), 4);
    assert_eq!(reassembler.pending(), 0);
}

#[test]
fn hole_filled_with_overlap() {
    let mut reassembler = Reassembler::new(20);
    let mut buf = ByteStream::new(20);

    reassembler.push_str(5, "fgh", false, &mut buf);
    assert_eq!(buf.pushed(), 0);
    assert_eq!(buf.read_all(), "");
    assert!(!buf.finished());

    reassembler.push_str(0, "abc", false, &mut buf);
    assert_eq!(buf.pushed(), 3);

    reassembler.push_str(0, "abcdef", false, &mut buf);
    assert_eq!(buf.pushed(), 8);
    assert_eq!(reassembler.pending(), 0);
    assert_eq!(buf.read_all(), "abcdefgh");
}
