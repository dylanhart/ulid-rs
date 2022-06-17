use time::OffsetDateTime;

use std::fmt;

use crate::Ulid;

/// A Ulid generator that provides monotonically increasing Ulids
pub struct Generator {
    previous: Ulid,
}

impl Generator {
    /// Create a new ulid generator for monotonically ordered ulids
    ///
    /// # Example
    /// ```rust
    /// use ulid::Generator;
    ///
    /// let mut gen = Generator::new();
    ///
    /// let ulid1 = gen.generate().unwrap();
    /// let ulid2 = gen.generate().unwrap();
    ///
    /// assert!(ulid1 < ulid2);
    /// ```
    pub fn new() -> Generator {
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
    pub fn generate(&mut self) -> Result<Ulid, MonotonicError> {
        let now = OffsetDateTime::now_utc();
        self.generate_from_datetime(now)
    }

    /// Generate a new Ulid matching the given DateTime.
    /// Each call is guaranteed to provide a Ulid with a larger value than the last call.
    /// If the random bits would overflow, this method will return an error.
    ///
    /// # Example
    /// ```rust
    /// use ulid::Generator;
    /// use time::OffsetDateTime;
    ///
    /// let dt = OffsetDateTime::now_utc();
    /// let mut gen = Generator::new();
    ///
    /// let ulid1 = gen.generate_from_datetime(dt).unwrap();
    /// let ulid2 = gen.generate_from_datetime(dt).unwrap();
    ///
    /// assert_eq!(ulid1.datetime(), ulid2.datetime());
    /// assert!(ulid1 < ulid2);
    /// ```
    pub fn generate_from_datetime(
        &mut self,
        datetime: OffsetDateTime,
    ) -> Result<Ulid, MonotonicError> {
        self.generate_from_datetime_with_source(datetime, &mut rand::thread_rng())
    }

    /// Generate a new monotonic increasing Ulid with the given source
    /// Each call is guaranteed to provide a Ulid with a larger value than the last call.
    /// If the random bits would overflow, this method will return an error.
    ///
    /// # Example
    /// ```rust
    /// use ulid::Generator;
    /// use ulid::Ulid;
    /// use time::OffsetDateTime;
    /// use rand::prelude::*;
    ///
    /// let mut rng = StdRng::from_entropy();
    /// let mut gen = Generator::new();
    ///
    /// let ulid1 = gen.generate_with_source(&mut rng).unwrap();
    /// let ulid2 = gen.generate_with_source(&mut rng).unwrap();
    ///
    /// assert!(ulid1 < ulid2);
    /// ```
    pub fn generate_with_source<R>(&mut self, source: &mut R) -> Result<Ulid, MonotonicError>
    where
        R: rand::Rng,
    {
        let now = OffsetDateTime::now_utc();
        self.generate_from_datetime_with_source(now, source)
    }

    /// Generate a new monotonic increasing Ulid with the given source matching the given DateTime
    /// Each call is guaranteed to provide a Ulid with a larger value than the last call.
    /// If the random bits would overflow, this method will return an error.
    ///
    /// # Example
    /// ```rust
    /// use ulid::Generator;
    /// use time::OffsetDateTime;
    /// use rand::prelude::*;
    ///
    /// let dt = OffsetDateTime::now_utc();
    /// let mut rng = StdRng::from_entropy();
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
        datetime: OffsetDateTime,
        source: &mut R,
    ) -> Result<Ulid, MonotonicError>
    where
        R: rand::Rng,
    {
        let last_ms = self.previous.timestamp_ms() as i128;
        // maybe time went backward, or it is the same ms.
        // increment instead of generating a new random so that it is monotonic
        if (datetime.unix_timestamp_nanos() / 1_000_000) <= last_ms {
            if let Some(next) = self.previous.increment() {
                self.previous = next;
                return Ok(next);
            } else {
                return Err(MonotonicError::Overflow);
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

/// Error while trying to generate a monotonic increment in the same millisecond
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum MonotonicError {
    /// Would overflow into the next millisecond
    Overflow,
}

impl std::error::Error for MonotonicError {}

impl fmt::Display for MonotonicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let text = match *self {
            MonotonicError::Overflow => "Ulid random bits would overflow",
        };
        write!(f, "{}", text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::time::Duration;

    #[test]
    fn test_order_monotonic() {
        let dt = OffsetDateTime::now_utc();
        let mut gen = Generator::new();
        let ulid1 = gen.generate_from_datetime(dt).unwrap();
        let ulid2 = gen.generate_from_datetime(dt).unwrap();
        let ulid3 = Ulid::from_datetime(dt + Duration::milliseconds(1));
        assert_eq!(ulid1.0 + 1, ulid2.0);
        assert!(ulid2 < ulid3);
        assert!(ulid2.timestamp_ms() < ulid3.timestamp_ms())
    }

    #[test]
    fn test_order_monotonic_with_source() {
        use rand::rngs::mock::StepRng;
        let mut source = StepRng::new(123, 0);
        let mut gen = Generator::new();

        let _has_default = Generator::default();

        let ulid1 = gen.generate_with_source(&mut source).unwrap();
        let ulid2 = gen.generate_with_source(&mut source).unwrap();
        assert!(ulid1 < ulid2);
    }

    #[test]
    fn can_display_things() {
        println!("{}", MonotonicError::Overflow);
    }
}
