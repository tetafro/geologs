use std::process;

use clap::Parser;

mod accesslog;
mod geodata;
mod report;

// Output report file.
const REPORT_FILE: &str = "index.html";

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
#[derive(Parser, Debug)]
#[command(about = "Parse HTTP server access logs and save an HTML report with statistics.")]
struct Args {
    /// Geodata API address
    #[arg(short = 'a', long = "api-addr", default_value_t = String::from("https://api.ipgeolocation.io/ipgeo"))]
    api_addr: String,

    /// Geodata API authentication key
    #[arg(short = 'k', long = "api-key", default_value_t = String::from(""))]
    api_key: String,

    /// Fail on invalid lines in logs instead skipping them
    #[arg(short = 'i', long = "fail-invalid", default_value_t = false)]
    fail_invalid: bool,

    /// Access log file path
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();

    // Parse logfiles
    let log = accesslog::parse(args.files, args.fail_invalid).unwrap_or_else(|err| {
        fatal!("Failed to read input file: {}", err);
    });

    // Get geodata from remote API or cache
    let geo =
        geodata::get_geodata(&args.api_addr, &args.api_key, log.get_ips()).unwrap_or_else(|err| {
            fatal!("Failed to get geodata: {}", err);
        });

    // Save parsed and resolved data in a human readable format
    report::generate(log, geo, REPORT_FILE).unwrap_or_else(|err| {
        fatal!("Failed to build report: {}", err);
    });
    println!("Done");
}
