use ::chrono::prelude::{DateTime, TimeZone, Utc};

use crate::{Ulid, bitmask};

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
        Ulid::from_datetime(Utc::now())
    }

    /// Creates a new Ulid using data from the given random number generator
    ///
    /// # Example
    /// ```rust
    /// use rand::FromEntropy;
    /// use rand::rngs::SmallRng;
    /// use ulid::Ulid;
    ///
    /// let mut rng = SmallRng::from_entropy();
    /// let ulid = Ulid::with_source(&mut rng);
    /// ```
    pub fn with_source<R: rand::Rng>(source: &mut R) -> Ulid {
        Ulid::from_datetime_with_source(Utc::now(), source)
    }

    /// Creates a new Ulid with the given datetime
    ///
    /// This can be useful when migrating data to use Ulid identifiers
    ///
    /// # Example
    /// ```rust
    /// use chrono::offset::Utc;
    /// use ulid::Ulid;
    ///
    /// let ulid = Ulid::from_datetime(Utc::now());
    /// ```
    pub fn from_datetime<T: TimeZone>(datetime: DateTime<T>) -> Ulid {
        Ulid::from_datetime_with_source(datetime, &mut rand::thread_rng())
    }

    /// Creates a new Ulid with the given datetime and random number generator
    ///
    /// # Example
    /// ```rust
    /// use chrono::offset::Utc;
    /// use rand::FromEntropy;
    /// use rand::rngs::SmallRng;
    /// use ulid::Ulid;
    ///
    /// let mut rng = SmallRng::from_entropy();
    /// let ulid = Ulid::from_datetime_with_source(Utc::now(), &mut rng);
    /// ```
    pub fn from_datetime_with_source<T, R>(datetime: DateTime<T>, source: &mut R) -> Ulid
    where
        T: TimeZone,
        R: rand::Rng,
    {
        let timestamp = datetime.timestamp_millis();
        let timebits = (timestamp & bitmask!(Self::TIME_BITS)) as u64;

        let msb = timebits << 16 | u64::from(source.gen::<u16>());
        let lsb = source.gen::<u64>();
        Ulid::from((msb, lsb))
    }

    /// Gets the datetime of when this Ulid was created accurate to 1ms
    ///
    /// # Example
    /// ```rust
    /// use chrono::Duration;
    /// use chrono::offset::Utc;
    /// use ulid::Ulid;
    ///
    /// let dt = Utc::now();
    /// let ulid = Ulid::from_datetime(dt);
    ///
    /// assert!((dt - ulid.datetime()) < Duration::milliseconds(1));
    /// ```
    pub fn datetime(&self) -> DateTime<Utc> {
        let stamp = self.timestamp_ms();
        let secs = stamp / 1000;
        let millis = stamp % 1000;
        Utc.timestamp(secs as i64, (millis * 1_000_000) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::chrono::Duration;

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
        let dt = Utc::now() + Duration::milliseconds(1);
        let u2 = Ulid::from_datetime_with_source(dt, &mut source);
        let u3 = Ulid::from_datetime_with_source(dt, &mut source);

        assert!(u1 < u2);
        assert_eq!(u2, u3);
    }

    #[test]
    fn test_order() {
        let dt = Utc::now();
        let ulid1 = Ulid::from_datetime(dt);
        let ulid2 = Ulid::from_datetime(dt + Duration::milliseconds(1));
        assert!(ulid1 < ulid2);
    }

    #[test]
    fn test_datetime() {
        let dt = Utc::now();
        let ulid = Ulid::from_datetime(dt);

        println!("{:?}, {:?}", dt, ulid.datetime());
        assert!(ulid.datetime() <= dt);
        assert!(ulid.datetime() + Duration::milliseconds(1) >= dt);
    }

    #[test]
    fn test_timestamp() {
        let dt = Utc::now();
        let ulid = Ulid::from_datetime(dt);
        let ts = dt.timestamp() as u64 * 1000 + dt.timestamp_subsec_millis() as u64;

        assert_eq!(ulid.timestamp_ms(), ts);
    }

    #[test]
    fn default_is_nil() {
        assert_eq!(Ulid::default(), Ulid::nil());
    }
}
