use std::env;
use std::process;

use dotenv::dotenv;
use serde::Deserialize;

mod accesslog;
mod geodata;
mod report;

// Shortcut for println + exit(1).
#[macro_export]
macro_rules! fatal {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ({
        print!(concat!($fmt, "\n"), $($arg)*);
        process::exit(1)
    });
}

// Main application config.
#[derive(Deserialize)]
struct Config {
    api_addr: String,
    api_key: String,
    skip_invalid: bool,
}

fn main() {
    // Read config from env
    dotenv().ok();
    let conf = match envy::from_env::<Config>() {
        Ok(value) => value,
        Err(err) => {
            fatal!("Failed to parse config: {}", err);
        }
    };

    // Get input logfile
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        fatal!("No input file provided");
    }
    let logfile = &args[1];

    // Parse logfile
    let log = accesslog::parse(logfile, conf.skip_invalid).unwrap_or_else(|err| {
        fatal!("Failed to read input file: {}", err);
    });

    // Get geodata from remote API or cache
    let geo =
        geodata::get_geodata(&conf.api_addr, &conf.api_key, log.get_ips()).unwrap_or_else(|err| {
            fatal!("Failed to get geodata: {}", err);
        });

    // Save parsed and resolved data in a human readable format
    report::generate(log, geo).unwrap_or_else(|err| {
        fatal!("Failed to build report: {}", err);
    });
    println!("Done");
}
