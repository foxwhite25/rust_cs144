use cs144::sequence::{AbsoluteSequence, RelativeSequence};
use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};

#[test]
fn wrapping_cmp() {
    assert!(RelativeSequence(3) != RelativeSequence(1));
    assert!(RelativeSequence(3) != RelativeSequence(1));

    let n_reps = 32768;
    let mut rng = rand::thread_rng();

    for _ in 0..n_reps {
        let n = rng.gen::<u32>();
        let diff = rng.gen::<u8>();
        let m = n.wrapping_add(diff as u32);
        assert_eq!(RelativeSequence(n) == RelativeSequence(m), n == m);
        assert_eq!(RelativeSequence(n) != RelativeSequence(m), n != m);
    }
}

#[test]
fn test_relative_sequence_unwrap() {
    assert_eq!(
        RelativeSequence(1).unwrap(RelativeSequence(0), AbsoluteSequence(0)),
        AbsoluteSequence(1)
    );
    assert_eq!(
        RelativeSequence(1).unwrap(RelativeSequence(0), AbsoluteSequence(u32::MAX as u64)),
        AbsoluteSequence((1u64 << 32) + 1)
    );
    assert_eq!(
        RelativeSequence(u32::MAX - 1).unwrap(RelativeSequence(0), AbsoluteSequence(3 * (1 << 32))),
        AbsoluteSequence(3 * (1 << 32) - 2)
    );
    assert_eq!(
        RelativeSequence(u32::MAX - 10)
            .unwrap(RelativeSequence(0), AbsoluteSequence(3 * (1u64 << 32))),
        AbsoluteSequence(3 * (1u64 << 32) - 11)
    );
    assert_eq!(
        RelativeSequence(u32::MAX).unwrap(RelativeSequence(10), AbsoluteSequence(3 * (1u64 << 32))),
        AbsoluteSequence(3 * (1u64 << 32) - 11)
    );
    assert_eq!(
        RelativeSequence(u32::MAX).unwrap(RelativeSequence(0), AbsoluteSequence(0)),
        AbsoluteSequence(u32::MAX as u64)
    );
    assert_eq!(
        RelativeSequence(16).unwrap(RelativeSequence(16), AbsoluteSequence(0)),
        AbsoluteSequence(0)
    );
    assert_eq!(
        RelativeSequence(15).unwrap(RelativeSequence(16), AbsoluteSequence(0)),
        AbsoluteSequence(u32::MAX as u64)
    );
    assert_eq!(
        RelativeSequence(0).unwrap(RelativeSequence(i32::MAX as u32), AbsoluteSequence(0)),
        AbsoluteSequence((i32::MAX as u64) + 2)
    );
    assert_eq!(
        RelativeSequence(u32::MAX).unwrap(RelativeSequence(i32::MAX as u32), AbsoluteSequence(0)),
        AbsoluteSequence(1u64 << 31)
    );
    assert_eq!(
        RelativeSequence(u32::MAX)
            .unwrap(RelativeSequence((1u64 << 31) as u32), AbsoluteSequence(0)),
        AbsoluteSequence((u32::MAX as u64) >> 1)
    );
}

#[test]
fn test_wrap() {
    assert_eq!(
        AbsoluteSequence(3 * (1u64 << 32)).wrap(RelativeSequence(0)),
        RelativeSequence(0)
    );
    assert_eq!(
        AbsoluteSequence(3 * (1u64 << 32) + 17).wrap(RelativeSequence(15)),
        RelativeSequence(32)
    );
    assert_eq!(
        AbsoluteSequence(7 * (1u64 << 32) - 2).wrap(RelativeSequence(15)),
        RelativeSequence(13)
    );
}

#[test]
fn test_roundtrip() {
    fn check_roundtrip(
        isn: RelativeSequence,
        value: AbsoluteSequence,
        checkpoint: AbsoluteSequence,
    ) {
        let abs_seq = value;
        let rel_seq = abs_seq.wrap(isn);

        // Assuming you have implemented `unwrap` function in `RelativeSequence` struct
        let unwrapped_value = rel_seq.unwrap(isn, checkpoint);

        if unwrapped_value != value {
            panic!(
                "Expected unwrap(wrap()) to recover same value, and it didn't! \
                unwrap(wrap(value, isn), isn, checkpoint) did not equal value \
                where value = {:?}, isn = {}, and checkpoint = {:?} \
                (Difference between value and checkpoint is {:?}.)",
                value,
                isn.0,
                checkpoint,
                value - checkpoint
            );
        }
    }

    let mut rng = rand::thread_rng();

    let dist31minus1 = Uniform::from(0..(1 << 31));
    let dist32 = Uniform::from(0..=u32::MAX);
    let dist63 = Uniform::from(0..(1 << 63));

    let big_offset = AbsoluteSequence((1 << 31) - 1);

    for _ in 0..1_000_000 {
        let isn = RelativeSequence(dist32.sample(&mut rng));
        let val = AbsoluteSequence(dist63.sample(&mut rng));
        let offset = AbsoluteSequence(dist31minus1.sample(&mut rng));

        check_roundtrip(isn, val, val);
        check_roundtrip(isn, val + AbsoluteSequence(1), val);
        check_roundtrip(isn, val - AbsoluteSequence(1), val);
        check_roundtrip(isn, val + offset, val);
        check_roundtrip(isn, val - offset, val);
        check_roundtrip(isn, val + big_offset, val);
        check_roundtrip(isn, val - big_offset, val);
    }
}
