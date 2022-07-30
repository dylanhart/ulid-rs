use crate::{bitmask, Ulid};
use std::time::{Duration, SystemTime};

impl Ulid {
    /// Creates a new Ulid with the current time (UTC)
    ///
    /// # Example
    /// ```rust
    /// use ulid::Ulid;
    ///
    /// let my_ulid = Ulid::new();
    /// ```
    pub fn new() -> Ulid {
        Ulid::from_datetime(SystemTime::now())
    }

    /// Creates a new Ulid using data from the given random number generator
    ///
    /// # Example
    /// ```rust
    /// use rand::prelude::*;
    /// use ulid::Ulid;
    ///
    /// let mut rng = StdRng::from_entropy();
    /// let ulid = Ulid::with_source(&mut rng);
    /// ```
    pub fn with_source<R: rand::Rng>(source: &mut R) -> Ulid {
        Ulid::from_datetime_with_source(SystemTime::now(), source)
    }

    /// Creates a new Ulid with the given datetime
    ///
    /// This can be useful when migrating data to use Ulid identifiers.
    ///
    /// This will take the maximum of the `[SystemTime]` argument and `[SystemTime::UNIX_EPOCH]`
    /// as earlier times are not valid for a Ulid timestamp
    ///
    /// # Example
    /// ```rust
    /// use std::time::{SystemTime, Duration};
    /// use ulid::Ulid;
    ///
    /// let ulid = Ulid::from_datetime(SystemTime::now());
    /// ```
    pub fn from_datetime(datetime: SystemTime) -> Ulid {
        Ulid::from_datetime_with_source(datetime, &mut rand::thread_rng())
    }

    /// Creates a new Ulid with the given datetime and random number generator
    ///
    /// This will take the maximum of the `[SystemTime]` argument and `[SystemTime::UNIX_EPOCH]`
    /// as earlier times are not valid for a Ulid timestamp
    ///
    /// # Example
    /// ```rust
    /// use std::time::{SystemTime, Duration};
    /// use rand::prelude::*;
    /// use ulid::Ulid;
    ///
    /// let mut rng = StdRng::from_entropy();
    /// let ulid = Ulid::from_datetime_with_source(SystemTime::now(), &mut rng);
    /// ```
    pub fn from_datetime_with_source<R>(datetime: SystemTime, source: &mut R) -> Ulid
    where
        R: rand::Rng,
    {
        let timestamp = datetime
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis();
        let timebits = (timestamp & bitmask!(Self::TIME_BITS)) as u64;

        let msb = timebits << 16 | u64::from(source.gen::<u16>());
        let lsb = source.gen::<u64>();
        Ulid::from((msb, lsb))
    }

    /// Gets the datetime of when this Ulid was created accurate to 1ms
    ///
    /// # Example
    /// ```rust
    /// use std::time::{SystemTime, Duration};
    /// use ulid::Ulid;
    ///
    /// let dt = SystemTime::now();
    /// let ulid = Ulid::from_datetime(dt);
    ///
    /// assert!(
    ///     dt + Duration::from_millis(1) >= ulid.datetime()
    ///     && dt - Duration::from_millis(1) <= ulid.datetime()
    /// );
    /// ```
    pub fn datetime(&self) -> SystemTime {
        let stamp = self.timestamp_ms();
        SystemTime::UNIX_EPOCH + Duration::from_millis(stamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic() {
        let ulid = Ulid::new();
        let encoded = ulid.to_string();
        let ulid2 = Ulid::from_string(&encoded).expect("failed to deserialize");

        println!("{}", encoded);
        println!("{:?}", ulid);
        println!("{:?}", ulid2);
        assert_eq!(ulid, ulid2);
    }

    #[test]
    fn test_source() {
        use rand::rngs::mock::StepRng;
        let mut source = StepRng::new(123, 0);

        let u1 = Ulid::with_source(&mut source);
        let dt = SystemTime::now() + Duration::from_millis(1);
        let u2 = Ulid::from_datetime_with_source(dt, &mut source);
        let u3 = Ulid::from_datetime_with_source(dt, &mut source);

        assert!(u1 < u2);
        assert_eq!(u2, u3);
    }

    #[test]
    fn test_order() {
        let dt = SystemTime::now();
        let ulid1 = Ulid::from_datetime(dt);
        let ulid2 = Ulid::from_datetime(dt + Duration::from_millis(1));
        assert!(ulid1 < ulid2);
    }

    #[test]
    fn test_datetime() {
        let dt = SystemTime::now();
        let ulid = Ulid::from_datetime(dt);

        println!("{:?}, {:?}", dt, ulid.datetime());
        assert!(ulid.datetime() <= dt);
        assert!(ulid.datetime() + Duration::from_millis(1) >= dt);
    }

    #[test]
    fn test_timestamp() {
        let dt = SystemTime::now();
        let ulid = Ulid::from_datetime(dt);
        let ts = dt
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        assert_eq!(u128::from(ulid.timestamp_ms()), ts);
    }

    #[test]
    fn default_is_nil() {
        assert_eq!(Ulid::default(), Ulid::nil());
    }

    #[test]
    fn nil_is_at_unix_epoch() {
        assert_eq!(Ulid::nil().datetime(), SystemTime::UNIX_EPOCH);
    }

    #[test]
    fn truncates_at_unix_epoch() {
        let before_epoch = SystemTime::UNIX_EPOCH - Duration::from_secs(100);
        assert!(before_epoch < SystemTime::UNIX_EPOCH);
        assert_eq!(
            Ulid::from_datetime(before_epoch).datetime(),
            SystemTime::UNIX_EPOCH
        );
    }
}
