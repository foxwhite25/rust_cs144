use cs144::byte_stream::ByteStream;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

#[test]
fn overwrite() {
    let mut stream = ByteStream::new(2);
    stream.push_str("cat");
    assert_eq!(stream.avalible_capacity(), 0);
    assert!(!stream.is_empty());
    assert!(!stream.finished());
    assert!(!stream.closed());
    assert_eq!(stream.len(), 2);
    assert_eq!(stream.peek(), "ca");

    stream.push_str("t");
    assert_eq!(stream.avalible_capacity(), 0);
    assert!(!stream.is_empty());
    assert!(!stream.finished());
    assert!(!stream.closed());
    assert_eq!(stream.len(), 2);
    assert_eq!(stream.peek(), "ca");
}

#[test]
fn clear_overwrite() {
    let mut stream = ByteStream::new(2);
    stream.push_str("cat");
    assert_eq!(stream.len(), 2);

    stream.pop(2);
    stream.push_str("tac");
    assert!(!stream.is_empty());
    assert!(!stream.finished());
    assert!(!stream.closed());
    assert_eq!(stream.pushed(), 4);
    assert_eq!(stream.poped(), 2);
    assert_eq!(stream.len(), 2);
    assert_eq!(stream.avalible_capacity(), 0);
    assert_eq!(stream.peek(), "ta")
}

#[test]
fn pop_overwrite() {
    let mut stream = ByteStream::new(2);
    stream.push_str("cat");
    assert_eq!(stream.len(), 2);

    stream.pop(1);
    stream.push_str("tac");
    assert!(!stream.is_empty());
    assert!(!stream.finished());
    assert!(!stream.closed());
    assert_eq!(stream.pushed(), 3);
    assert_eq!(stream.poped(), 1);
    assert_eq!(stream.len(), 2);
    assert_eq!(stream.avalible_capacity(), 0);
    assert_eq!(stream.peek(), "at")
}

#[test]
fn peek() {
    let mut stream = ByteStream::new(2);
    stream.push_str("");
    stream.push_str("");
    stream.push_str("");
    stream.push_str("");
    stream.push_str("");
    stream.push_str("cat");
    stream.push_str("");
    stream.push_str("");
    stream.push_str("");
    stream.push_str("");
    assert_eq!(stream.peek(), "ca");
    assert_eq!(stream.peek(), "ca");
    assert_eq!(stream.len(), 2);
    assert_eq!(stream.peek(), "ca");
    assert_eq!(stream.peek(), "ca");

    stream.pop(1);
    stream.push_str("");
    stream.push_str("");
    stream.push_str("");
    assert_eq!(stream.peek(), "a");
    assert_eq!(stream.peek(), "a");
    assert_eq!(stream.len(), 1);
}

#[test]
fn many_writes() {
    let iter = 1000;
    let max_write = 200;
    let min_write = 10;
    let capacity = max_write * iter;

    let mut stream = ByteStream::new(capacity);
    let mut acc = 0;
    for _ in 0..iter {
        let mut rng = thread_rng();
        let size = rng.gen_range(min_write..max_write);
        let random_str = rng
            .sample_iter(&Alphanumeric)
            .take(size)
            .map(char::from)
            .collect::<String>();
        stream.push_str(&random_str);
        acc += size;

        assert_eq!(stream.pushed(), acc);
        assert_eq!(stream.poped(), 0);
        assert_eq!(stream.avalible_capacity(), capacity - acc);
        assert_eq!(stream.len(), acc);
    }
}

#[test]
fn write_end_pop() {
    let mut stream = ByteStream::new(10);

    stream.push_str("hello");
    assert_eq!(stream.pushed(), 5);
    assert_eq!(stream.poped(), 0);
    assert_eq!(stream.avalible_capacity(), 5);
    assert_eq!(stream.len(), 5);
    assert_eq!(stream.peek(), "hello");

    stream.close();
    assert!(stream.closed());
    assert!(!stream.finished());

    stream.pop(5);
    assert_eq!(stream.pushed(), 5);
    assert_eq!(stream.poped(), 5);
    assert_eq!(stream.avalible_capacity(), 10);
    assert_eq!(stream.len(), 0);
    assert_eq!(stream.peek(), "");

    assert!(stream.finished());
}

#[test]
fn write_pop_end() {
    let mut stream = ByteStream::new(10);

    stream.push_str("hello");
    assert_eq!(stream.pushed(), 5);
    assert_eq!(stream.poped(), 0);
    assert_eq!(stream.avalible_capacity(), 5);
    assert_eq!(stream.len(), 5);
    assert_eq!(stream.peek(), "hello");

    stream.pop(5);
    assert_eq!(stream.pushed(), 5);
    assert_eq!(stream.poped(), 5);
    assert_eq!(stream.avalible_capacity(), 10);
    assert_eq!(stream.len(), 0);
    assert_eq!(stream.peek(), "");

    stream.close();
    assert!(stream.closed());
    assert!(stream.finished());
}

#[test]
fn write_pop2_end() {
    let mut stream = ByteStream::new(10);

    stream.push_str("hello");
    assert_eq!(stream.pushed(), 5);
    assert_eq!(stream.poped(), 0);
    assert_eq!(stream.avalible_capacity(), 5);
    assert_eq!(stream.len(), 5);
    assert_eq!(stream.peek(), "hello");

    stream.pop(2);
    assert_eq!(stream.pushed(), 5);
    assert_eq!(stream.poped(), 2);
    assert_eq!(stream.avalible_capacity(), 7);
    assert_eq!(stream.len(), 3);
    assert_eq!(stream.peek(), "llo");

    stream.push_str("world");
    assert_eq!(stream.pushed(), 10);
    assert_eq!(stream.poped(), 2);
    assert_eq!(stream.avalible_capacity(), 2);
    assert_eq!(stream.len(), 8);
    assert_eq!(stream.peek(), "lloworld");

    stream.pop(8);
    assert_eq!(stream.pushed(), 10);
    assert_eq!(stream.poped(), 10);
    assert_eq!(stream.avalible_capacity(), 10);
    assert_eq!(stream.len(), 0);
    assert_eq!(stream.peek(), "");

    stream.close();
    assert!(stream.closed());
    assert!(stream.finished());
}
