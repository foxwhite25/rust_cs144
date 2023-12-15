#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy)]
pub struct AbsoluteSequence(pub u64);

impl std::ops::Sub for AbsoluteSequence {
    type Output = AbsoluteSequence;

    fn sub(self, rhs: Self) -> Self::Output {
        AbsoluteSequence(self.0 - rhs.0)
    }
}

impl std::ops::Add for AbsoluteSequence {
    type Output = AbsoluteSequence;

    fn add(self, rhs: Self) -> Self::Output {
        AbsoluteSequence(self.0 + rhs.0)
    }
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Debug, Clone, Copy)]
pub struct RelativeSequence(pub u32);

impl AbsoluteSequence {
    pub fn new(k: u64) -> Self {
        AbsoluteSequence(k)
    }

    pub fn wrap(self, isn: RelativeSequence) -> RelativeSequence {
        RelativeSequence(isn.0.wrapping_add(self.0 as u32))
    }
}

impl RelativeSequence {
    pub fn new(k: u32) -> Self {
        RelativeSequence(k)
    }

    pub fn unwrap(self, isn: RelativeSequence, checkpoint: AbsoluteSequence) -> AbsoluteSequence {
        const UINT32_SIZE: u64 = 1 << 32;
        let seqno_offset = self.0.wrapping_sub(isn.0);
        if checkpoint.0 > seqno_offset as u64 {
            let abs_seqno_extra_part_offset = checkpoint
                .0
                .wrapping_sub(seqno_offset as u64)
                .wrapping_add(UINT32_SIZE >> 1);
            let uint32_size_num = abs_seqno_extra_part_offset / UINT32_SIZE;
            AbsoluteSequence(uint32_size_num * UINT32_SIZE + seqno_offset as u64)
        } else {
            AbsoluteSequence(seqno_offset as u64)
        }
    }
}
