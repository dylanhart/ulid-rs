use std::fmt;
use std::time::{Duration, SystemTime};

use crate::Ulid;

/// A Ulid generator that provides monotonically increasing Ulids. This is implemented to match the
/// reference generator's algorithm and it's [issues].
///
/// [issues]: https://github.com/dylanhart/ulid-rs/issues/80
///
/// # Example
/// ```rust
/// use ulid::Generator;
///
/// let mut gen = Generator::new();
///
/// let ulid1 = match gen.generate() {
///     Ok(ulid) => ulid,
///     // In the unlikely case of overflow in the random bits, the overflow behavior
///     // may be selected.
///     // * increment: ulid.random() == 0
///     Err(overflow) => overflow.commit_overflow_increment(),
/// };
/// let ulid2 = match gen.generate() {
///     Ok(ulid) => ulid,
///     // * random: ulid.random() == <random value>
///     Err(overflow) => overflow.commit_overflow_random(),
/// };
///
/// // Outputs will always be in order
/// assert!(ulid1 < ulid2);
/// ```
#[derive(Debug, Clone)]
pub struct Generator {
    previous: Ulid,
}

impl Generator {
    /// Create a new ulid generator for monotonically ordered ulids
    pub const fn new() -> Generator {
        Generator {
            previous: Ulid::nil(),
        }
    }

    /// Generate a new Ulid. Each call is guaranteed to provide a Ulid with a larger value than the
    /// last call. If the random bits would overflow, this method will return an error.
    ///
    /// ```rust
    /// use ulid::Generator;
    /// let mut gen = Generator::new();
    ///
    /// let ulid1 = gen.generate().unwrap();
    /// let ulid2 = gen.generate().unwrap();
    ///
    /// assert!(ulid1 < ulid2);
    /// ```
    pub fn generate(&mut self) -> Result<Ulid, Overflow<'_>> {
        self.generate_from_datetime(crate::time_utils::now())
    }

    /// Generate a new Ulid matching the given DateTime.
    /// Each call is guaranteed to provide a Ulid with a larger value than the last call.
    /// If the random bits would overflow, this method will return an error.
    ///
    /// # Example
    /// ```rust
    /// use ulid::Generator;
    /// use std::time::SystemTime;
    ///
    /// let dt = SystemTime::now();
    /// let mut gen = Generator::new();
    ///
    /// let ulid1 = gen.generate_from_datetime(dt).unwrap();
    /// let ulid2 = gen.generate_from_datetime(dt).unwrap();
    ///
    /// assert_eq!(ulid1.datetime(), ulid2.datetime());
    /// assert!(ulid1 < ulid2);
    /// ```
    pub fn generate_from_datetime(&mut self, datetime: SystemTime) -> Result<Ulid, Overflow<'_>> {
        self.generate_from_datetime_with_source(datetime, &mut rand::rng())
    }

    /// Generate a new monotonic increasing Ulid with the given source
    /// Each call is guaranteed to provide a Ulid with a larger value than the last call.
    /// If the random bits would overflow, this method will return an error.
    ///
    /// # Example
    /// ```rust
    /// use ulid::Generator;
    /// use ulid::Ulid;
    /// use std::time::SystemTime;
    /// use rand::prelude::*;
    ///
    /// let mut rng: StdRng = rand::make_rng();
    /// let mut gen = Generator::new();
    ///
    /// let ulid1 = gen.generate_with_source(&mut rng).unwrap();
    /// let ulid2 = gen.generate_with_source(&mut rng).unwrap();
    ///
    /// assert!(ulid1 < ulid2);
    /// ```
    pub fn generate_with_source<R>(&mut self, source: &mut R) -> Result<Ulid, Overflow<'_>>
    where
        R: rand::Rng + ?Sized,
    {
        self.generate_from_datetime_with_source(crate::time_utils::now(), source)
    }

    /// Generate a new monotonic increasing Ulid with the given source matching the given DateTime
    /// Each call is guaranteed to provide a Ulid with a larger value than the last call.
    /// If the random bits would overflow, this method will return an error.
    ///
    /// # Example
    /// ```rust
    /// use ulid::Generator;
    /// use std::time::SystemTime;
    /// use rand::prelude::*;
    ///
    /// let dt = SystemTime::now();
    /// let mut rng: StdRng = rand::make_rng();
    /// let mut gen = Generator::new();
    ///
    /// let ulid1 = gen.generate_from_datetime_with_source(dt, &mut rng).unwrap();
    /// let ulid2 = gen.generate_from_datetime_with_source(dt, &mut rng).unwrap();
    ///
    /// assert_eq!(ulid1.datetime(), ulid2.datetime());
    /// assert!(ulid1 < ulid2);
    /// ```
    pub fn generate_from_datetime_with_source<R>(
        &mut self,
        datetime: SystemTime,
        source: &mut R,
    ) -> Result<Ulid, Overflow<'_>>
    where
        R: rand::Rng + ?Sized,
    {
        let last_ms = self.previous.timestamp_ms();
        // maybe time went backward, or it is the same ms.
        // increment instead of generating a new random so that it is monotonic
        if datetime
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis()
            <= u128::from(last_ms)
        {
            if let Ok(next) = self.previous.increment() {
                self.previous = next;
                return Ok(next);
            } else {
                return Err(Overflow { generator: self });
            }
        }
        let next = Ulid::from_datetime_with_source(datetime, source);
        self.previous = next;
        Ok(next)
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

/// Would overflow into the next millisecond
#[derive(Debug)]
pub struct Overflow<'a> {
    generator: &'a mut Generator,
}

impl<'a> Overflow<'a> {
    /// Commit an overflow value into the generator via increment. The generator will be incremented
    /// into the next millisecond with the random field starting at zero.
    pub fn commit_overflow_increment(self) -> Ulid {
        let next = match self.generator.previous.increment() {
            Ok(next) => next,
            Err(next) => next,
        };
        self.generator.previous = next;
        next
    }

    /// Commit an overflow value into the generator via a random value. The generator will be
    /// incremented into the next millisecond with the random field starting at a random value.
    pub fn commit_overflow_random(self) -> Ulid {
        self.commit_overflow_random_with_source(&mut rand::rng())
    }

    /// Commit an overflow value into the generator via a random value. The generator will be
    /// incremented into the next millisecond with the random field starting at a random value. The
    /// random value will be generated from the given source.
    pub fn commit_overflow_random_with_source<R>(self, source: &mut R) -> Ulid
    where
        R: rand::Rng,
    {
        let inc = match self.generator.previous.increment() {
            Ok(inc) => inc,
            Err(inc) => inc,
        };
        let next = Ulid::from_datetime_with_source(inc.datetime(), source);
        self.generator.previous = next;
        next
    }
}

impl std::error::Error for Overflow<'_> {}

impl fmt::Display for Overflow<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "Ulid random bits would overflow")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_order_monotonic() {
        let dt = SystemTime::now();
        let mut gen = Generator::new();
        let ulid1 = gen.generate_from_datetime(dt).unwrap();
        let ulid2 = gen.generate_from_datetime(dt).unwrap();
        let ulid3 = Ulid::from_datetime(dt + Duration::from_millis(1));
        assert_eq!(ulid1.0 + 1, ulid2.0);
        assert!(ulid2 < ulid3);
        assert!(ulid2.timestamp_ms() < ulid3.timestamp_ms())
    }

    #[test]
    fn test_order_monotonic_with_source() {
        let mut source = crate::StepRng::new(123, 0);
        let mut gen = Generator::new();

        let _has_default = Generator::default();

        let ulid1 = gen.generate_with_source(&mut source).unwrap();
        let ulid2 = gen.generate_with_source(&mut source).unwrap();
        assert!(ulid1 < ulid2);
    }

    #[test]
    fn test_can_display_overflow() {
        let generator = &mut Generator::new();
        println!("{}", Overflow { generator });
    }

    #[test]
    fn test_overflow_commit_increment() {
        let maxed_random = {
            let ulid = Ulid::gen();
            Ulid::from_parts(
                ulid.timestamp_ms(),
                u128::MAX & crate::bitmask!(Ulid::RAND_BITS),
            )
        };

        let mut gen = Generator {
            previous: maxed_random,
        };
        let err_overflow = gen
            .generate_from_datetime(maxed_random.datetime())
            .unwrap_err();

        let next_ulid = err_overflow.commit_overflow_increment();

        assert_eq!(next_ulid.timestamp_ms(), maxed_random.timestamp_ms() + 1);
        assert_eq!(next_ulid.random(), 0);
        assert_eq!(next_ulid.0, maxed_random.0 + 1);
        assert_eq!(gen.previous, next_ulid);
    }

    #[test]
    fn test_overflow_commit_random() {
        let maxed_random = {
            let ulid = Ulid::gen();
            Ulid::from_parts(
                ulid.timestamp_ms(),
                u128::MAX & crate::bitmask!(Ulid::RAND_BITS),
            )
        };

        let mut gen = Generator {
            previous: maxed_random,
        };
        let err_overflow = gen
            .generate_from_datetime(maxed_random.datetime())
            .unwrap_err();

        let next_ulid = err_overflow.commit_overflow_random();

        assert_eq!(next_ulid.timestamp_ms(), maxed_random.timestamp_ms() + 1);
        assert_ne!(next_ulid.random(), 0);
        assert!(next_ulid > maxed_random);
        assert_eq!(gen.previous, next_ulid);
    }

    #[test]
    fn test_overflow_commit_random_with_source() {
        let maxed_random = {
            let ulid = Ulid::gen();
            Ulid::from_parts(
                ulid.timestamp_ms(),
                u128::MAX & crate::bitmask!(Ulid::RAND_BITS),
            )
        };

        let mut gen = Generator {
            previous: maxed_random,
        };
        let err_overflow = gen
            .generate_from_datetime(maxed_random.datetime())
            .unwrap_err();

        let mut source = crate::StepRng::new(42, 0);
        let next_ulid = err_overflow.commit_overflow_random_with_source(&mut source);

        assert_eq!(next_ulid.timestamp_ms(), maxed_random.timestamp_ms() + 1);
        assert_eq!(next_ulid.random(), 42 << 64 | 42);
        assert!(next_ulid > maxed_random);
        assert_eq!(gen.previous, next_ulid);
    }
}
