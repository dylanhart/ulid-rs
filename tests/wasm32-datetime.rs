#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]

use ulid::Ulid;

use wasm_bindgen_test::*;
use web_time::web::SystemTimeExt;

use std::time::{Duration, SystemTime};

fn now() -> std::time::SystemTime {
    return web_time::SystemTime::now().to_std();
}

#[wasm_bindgen_test]
fn test_dynamic() {
    let ulid = Ulid::new();
    let encoded = ulid.to_string();
    let ulid2 = Ulid::from_string(&encoded).expect("failed to deserialize");

    println!("{}", encoded);
    println!("{:?}", ulid);
    println!("{:?}", ulid2);
    assert_eq!(ulid, ulid2);
}

#[wasm_bindgen_test]
fn test_source() {
    use rand::rngs::mock::StepRng;
    let mut source = StepRng::new(123, 0);

    let u1 = Ulid::with_source(&mut source);
    let dt = now() + Duration::from_millis(1);
    let u2 = Ulid::from_datetime_with_source(dt, &mut source);
    let u3 = Ulid::from_datetime_with_source(dt, &mut source);

    assert!(u1 < u2);
    assert_eq!(u2, u3);
}

#[wasm_bindgen_test]
fn test_order() {
    let dt = now();
    let ulid1 = Ulid::from_datetime(dt);
    let ulid2 = Ulid::from_datetime(dt + Duration::from_millis(1));
    assert!(ulid1 < ulid2);
}

#[wasm_bindgen_test]
fn test_datetime() {
    let dt = now();
    let ulid = Ulid::from_datetime(dt);

    println!("{:?}, {:?}", dt, ulid.datetime());
    assert!(ulid.datetime() <= dt);
    assert!(ulid.datetime() + Duration::from_millis(1) >= dt);
}

#[wasm_bindgen_test]
fn test_timestamp() {
    let dt = now();
    let ulid = Ulid::from_datetime(dt);
    let ts = dt
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    assert_eq!(u128::from(ulid.timestamp_ms()), ts);
}

#[wasm_bindgen_test]
fn default_is_nil() {
    assert_eq!(Ulid::default(), Ulid::nil());
}

#[wasm_bindgen_test]
fn nil_is_at_unix_epoch() {
    assert_eq!(Ulid::nil().datetime(), SystemTime::UNIX_EPOCH);
}
