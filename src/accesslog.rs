use std::error::Error;
use std::fs;

use chrono::NaiveDate;
use regex::Regex;

// Parsed access logs line by line.
#[derive(Debug)]
pub struct AccessLog {
    pub lines: Vec<AccessLogLine>,
}

impl AccessLog {
    // Get all IPs from access log as a vector of strings.
    pub fn get_ips(&self) -> Vec<String> {
        self.lines.iter().map(|line| line.ip.clone()).collect()
    }
}

// Single line of access log. All insignificant fields are omitted
// from this representation.
#[derive(Debug)]
pub struct AccessLogLine {
    pub date: String,
    pub ip: String,
}

// PrintStandard printing format for access log line.
impl std::fmt::Display for AccessLogLine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{} {}]", self.date, self.ip)
    }
}

// Parse and filter input file into an AccessLog struct.
// TODO: Pass filtering funtion as an argument.
pub fn parse(file: &str) -> Result<AccessLog, Box<dyn Error>> {
    let content = fs::read_to_string(file)?;

    // Regexp for parsing each line
    let re = Regex::new(concat!(
        r#"^\[(?P<date>[0-9]{2}/[A-Za-z]{3}/[0-9]{4}).+\] "#,
        r#"(?P<ip>[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+).* "#,
        r#"(?P<path>/.*) "#,
        r#"(?P<resp_code>[0-9]{3}) "#,
        r#"(?P<referrer>".*") "#,
        r#""(?P<user_agent>.*)"$"#,
    ))?;

    let mut loglines: Vec<AccessLogLine> = Vec::new();
    let mut i = 0;
    for line in content.split("\n") {
        i += 1;
        if line.len() == 0 {
            continue;
        }

        // Parse line with regexp
        let groups = match re.captures(line) {
            Some(res) => res,
            None => return Err(format!("invalid line: {}", i).into()),
        };
        if groups.len() != 7 {
            return Err(format!("invalid line: {}", i).into());
        }
        let date = match groups.name("date") {
            Some(res) => res.as_str(),
            None => return Err(format!("invalid line, missing date: {}", i).into()),
        };
        let ip = match groups.name("ip") {
            Some(res) => res.as_str(),
            None => return Err(format!("invalid line, missing ip: {}", i).into()),
        };
        let path = match groups.name("path") {
            Some(res) => res.as_str(),
            None => return Err(format!("invalid line, missing path: {}", i).into()),
        };
        let user_agent = match groups.name("user_agent") {
            Some(res) => res.as_str(),
            None => return Err(format!("invalid line, missing user_agent: {}", i).into()),
        };

        // Filter out bots, and paths other than root
        if path != "/" || user_agent.to_lowercase().contains("bot") {
            continue;
        }

        // Change date's format
        let dt = match NaiveDate::parse_from_str(date, "%d/%b/%Y") {
            Ok(parsed) => parsed.format("%Y-%m-%d"),
            Err(_) => return Err(format!("invalid time format on line: {}", i).into()),
        };

        // Save the line along with the other
        loglines.push(AccessLogLine {
            date: dt.to_string(),
            ip: ip.to_string(),
        });
    }

    Ok(AccessLog { lines: loglines })
}
