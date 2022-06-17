use ::time::OffsetDateTime;

use crate::{bitmask, Ulid};

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
        Ulid::from_datetime(OffsetDateTime::now_utc())
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
        Ulid::from_datetime_with_source(OffsetDateTime::now_utc(), source)
    }

    /// Creates a new Ulid with the given datetime
    ///
    /// This can be useful when migrating data to use Ulid identifiers
    ///
    /// # Example
    /// ```rust
    /// use time::OffsetDateTime;
    /// use ulid::Ulid;
    ///
    /// let ulid = Ulid::from_datetime(OffsetDateTime::now_utc());
    /// ```
    pub fn from_datetime(datetime: OffsetDateTime) -> Ulid {
        Ulid::from_datetime_with_source(datetime, &mut rand::thread_rng())
    }

    /// Creates a new Ulid with the given datetime and random number generator
    ///
    /// # Example
    /// ```rust
    /// use time::OffsetDateTime;
    /// use rand::prelude::*;
    /// use ulid::Ulid;
    ///
    /// let mut rng = StdRng::from_entropy();
    /// let ulid = Ulid::from_datetime_with_source(OffsetDateTime::now_utc(), &mut rng);
    /// ```
    pub fn from_datetime_with_source<R>(datetime: OffsetDateTime, source: &mut R) -> Ulid
    where
        R: rand::Rng,
    {
        let timestamp = datetime.unix_timestamp_nanos() / 1_000_000;
        let timebits = (timestamp & bitmask!(Self::TIME_BITS)) as u64;

        let msb = timebits << 16 | u64::from(source.gen::<u16>());
        let lsb = source.gen::<u64>();
        Ulid::from((msb, lsb))
    }

    /// Gets the datetime of when this Ulid was created accurate to 1ms
    ///
    /// # Example
    /// ```rust
    /// use time::Duration;
    /// use time::OffsetDateTime;
    /// use ulid::Ulid;
    ///
    /// let dt = OffsetDateTime::now_utc();
    /// let ulid = Ulid::from_datetime(dt);
    ///
    /// assert!((dt - ulid.datetime()) < Duration::milliseconds(1));
    /// ```
    pub fn datetime(&self) -> OffsetDateTime {
        let stamp = self.timestamp_ms();
        let secs = stamp / 1000;
        let millis = stamp % 1000;
        OffsetDateTime::from_unix_timestamp_nanos(
            ((secs * 1_000_000_000) + (millis * 1_000_000)) as i128,
        )
        .expect("Seconds and Milliseconds are out of range")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::time::Duration;

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
        let dt = OffsetDateTime::now_utc() + Duration::milliseconds(1);
        let u2 = Ulid::from_datetime_with_source(dt, &mut source);
        let u3 = Ulid::from_datetime_with_source(dt, &mut source);

        assert!(u1 < u2);
        assert_eq!(u2, u3);
    }

    #[test]
    fn test_order() {
        let dt = OffsetDateTime::now_utc();
        let ulid1 = Ulid::from_datetime(dt);
        let ulid2 = Ulid::from_datetime(dt + Duration::milliseconds(1));
        assert!(ulid1 < ulid2);
    }

    #[test]
    fn test_datetime() {
        let dt = OffsetDateTime::now_utc();
        let ulid = Ulid::from_datetime(dt);

        println!("{:?}, {:?}", dt, ulid.datetime());
        assert!(ulid.datetime() <= dt);
        assert!(ulid.datetime() + Duration::milliseconds(1) >= dt);
    }

    #[test]
    fn test_timestamp() {
        let dt = OffsetDateTime::now_utc();
        let ulid = Ulid::from_datetime(dt);
        let ts = dt.unix_timestamp() as u64 * 1000 + dt.millisecond() as u64;

        assert_eq!(ulid.timestamp_ms(), ts);
    }

    #[test]
    fn default_is_nil() {
        assert_eq!(Ulid::default(), Ulid::nil());
    }
}
