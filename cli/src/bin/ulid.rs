extern crate structopt;

use std::io::{self, Write};
use ulid::{Generator, Ulid};

use std::{thread, time::Duration};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Number of ULIDs to generate
    #[structopt(short = "n", long = "count", default_value = "1")]
    count: u32,
    #[structopt(short = "m", long = "monotonic")]
    monotonic: bool,
    /// ULIDs for inspection
    #[structopt(conflicts_with = "count")]
    ulids: Vec<String>,
}

fn main() {
    let opt = Opt::from_args();

    if !opt.ulids.is_empty() {
        inspect(&opt.ulids);
    } else {
        generate(opt.count, opt.monotonic);
    }
}

fn generate(count: u32, monotonic: bool) {
    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut locked = stdout.lock();
    let mut err_locked = stderr.lock();
    if monotonic {
        let mut gen = Generator::new();
        let mut i = 0;
        while i < count {
            match gen.generate() {
                Ok(ulid) => {
                    writeln!(&mut locked, "{}", ulid).unwrap();
                    i += 1;
                }
                Err(_) => {
                    writeln!(
                        &mut err_locked,
                        "Failed to create new ulid due to overflow, sleeping 1 ms"
                    )
                    .unwrap();
                    thread::sleep(Duration::from_millis(1));
                    // do not increment i
                }
            }
        }
    } else {
        for _ in 0..count {
            writeln!(&mut locked, "{}", Ulid::new()).unwrap();
        }
    }
}

fn inspect(values: &[String]) {
    for val in values {
        let ulid = Ulid::from_string(&val);
        match ulid {
            Ok(ulid) => {
                let upper_hex = format!("{:X}", ulid.0);
                println!(
                    "
REPRESENTATION:

  String: {}
     Raw: {}

COMPONENTS:

       Time: {}
  Timestamp: {}
    Payload: {}
",
                    ulid.to_string(),
                    upper_hex,
                    ulid.datetime().to_rfc2822(),
                    ulid.timestamp_ms(),
                    upper_hex.chars().skip(6).collect::<String>()
                );
            }
            Err(e) => {
                println!("{} is not a valid ULID: {}", val, e);
            }
        }
    }
}
