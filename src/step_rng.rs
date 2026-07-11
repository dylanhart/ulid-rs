use core::convert::Infallible;

use rand::{rand_core::utils, TryRng};

/// Mock Rng implementation that returns a predictable sequence of values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRng {
    value: u64,
    increment: u64,
}

impl StepRng {
    /// Create a new `StepRng` returning `value` on the first call,
    /// and adding `increment` on each subsequent call.
    pub fn new(value: u64, increment: u64) -> Self {
        StepRng { value, increment }
    }
}

impl TryRng for StepRng {
    type Error = Infallible;

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.try_next_u64()? as u32)
    }

    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let result = self.value;
        self.value = self.value.wrapping_add(self.increment);
        Ok(result)
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        utils::fill_bytes_via_next_word(dest, || self.try_next_u64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_next_u64_sequence() {
        let mut rng = StepRng::new(10, 5);
        assert_eq!(rng.next_u64(), 10);
        assert_eq!(rng.next_u64(), 15);
        assert_eq!(rng.next_u64(), 20);
        assert_eq!(rng.next_u64(), 25);
    }

    #[test]
    fn test_next_u64_sequence_inc_zero() {
        let mut rng = StepRng::new(10, 0);
        assert_eq!(rng.next_u64(), 10);
        assert_eq!(rng.next_u64(), 10);
        assert_eq!(rng.next_u64(), 10);
        assert_eq!(rng.next_u64(), 10);
    }

    #[test]
    fn test_next_u64_overflow() {
        let mut rng = StepRng::new(u64::MAX, 2);
        assert_eq!(rng.next_u64(), u64::MAX);
        assert_eq!(rng.next_u64(), 1);
        assert_eq!(rng.next_u64(), 3);
    }

    #[test]
    fn test_next_u32_sequence() {
        let mut rng = StepRng::new(100, 10);
        assert_eq!(rng.next_u32(), 100);
        assert_eq!(rng.next_u32(), 110);
        assert_eq!(rng.next_u32(), 120);
    }

    #[test]
    fn test_next_u32_value_truncated() {
        let mut rng = StepRng::new(0xFFFF_FFFF_0000_0000, 1);
        assert_eq!(rng.next_u32(), 0);
        assert_eq!(rng.next_u32(), 1);
        assert_eq!(rng.next_u32(), 2);
    }

    #[test]
    fn test_fill_bytes() {
        let mut rng = StepRng::new(0xAAAA_AAAA_AAAA_AAAA, 0);
        let mut buf = [0u8; 16];
        rng.fill_bytes(&mut buf);
        assert_eq!(buf, [0xAAu8; 16]);
    }
}
