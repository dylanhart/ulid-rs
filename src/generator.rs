use std::time::{Duration, SystemTime};

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
    pub fn generate(&mut self) -> Result<Ulid, MonotonicError> {
        self.generate_from_datetime(crate::time_utils::now())
    }

    /// Generate a new Ulid. Each call is guaranteed to provide a Ulid with a larger value than the
    /// last call. The time part of the returned [`Ulid`] can be up to one millisecond in the future,
    /// under reasonable assumptions about processor speeds.
    ///
    /// # Panics
    ///
    /// This function panics if the previously returned value was already the highest possible ulid.
    /// This should not happen before year 91000 of the gregorian calendar at least.
    ///
    /// ```rust
    /// use ulid::Generator;
    /// let mut gen = Generator::new();
    ///
    /// let ulid1 = gen.generate_overflowing();
    /// let ulid2 = gen.generate_overflowing();
    ///
    /// assert!(ulid1 < ulid2);
    /// ```
    pub fn generate_overflowing(&mut self) -> Ulid {
        let next = Ulid::new();
        if next > self.previous {
            self.previous = next;
            next
        } else {
            self.previous = self.previous.increment_overflowing();
            self.previous
        }
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
    pub fn generate_from_datetime(&mut self, datetime: SystemTime) -> Result<Ulid, MonotonicError> {
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
    /// let mut rng = StdRng::from_os_rng();
    /// let mut gen = Generator::new();
    ///
    /// let ulid1 = gen.generate_with_source(&mut rng).unwrap();
    /// let ulid2 = gen.generate_with_source(&mut rng).unwrap();
    ///
    /// assert!(ulid1 < ulid2);
    /// ```
    pub fn generate_with_source<R>(&mut self, source: &mut R) -> Result<Ulid, MonotonicError>
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
    /// let mut rng = StdRng::from_os_rng();
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
    ) -> Result<Ulid, MonotonicError>
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
