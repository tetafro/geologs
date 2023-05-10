use std::collections::{HashMap};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::accesslog;
use crate::geodata;

// Report-file related files and parameters.
const TEMPLATE_FILE: &str = "index.html.j2";
const TEMPLATE_NAME: &str = "index";
const REPORT_FILE: &str = "index.html";

// ReportLine is a single entry of a report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportLine {
    pub date: String,
    pub ip: String,
    pub country: String,
    pub country_code: String,
    pub city: String,
    pub lat: f32,
    pub lon: f32,
}

// Report combines all data for further processing by frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Report {
    lines: Vec<ReportLine>,
}

// Build a report and save it to a file.
pub fn generate(
    log: accesslog::AccessLog,
    geo: HashMap<String, geodata::GeoData>,
) -> Result<(), Box<dyn Error>> {
    let mut rep: Report = Report{lines: Vec::new()};

    for line in log.lines {
        if let Some(geodata) = geo.get(&line.ip) {
            rep.lines.push(ReportLine{
                date: line.date,
                ip: line.ip,
                country: geodata.country.clone(),
                country_code: geodata.country_code.clone(),
                city: geodata.city.clone(),
                lat: geodata.lat,
                lon: geodata.lon,
            });
        }
    }

    // Init and render template with report data
    let mut tera = Tera::default();
    match tera.add_template_file(TEMPLATE_FILE, Some(TEMPLATE_NAME)) {
        Ok(_) => (),
        Err(err) => return Err(format!("parse template: {}", err).into()),
    };
    let ctx = match Context::from_serialize(rep) {
        Ok(ctx) => ctx,
        Err(err) => return Err(format!("serialize context: {}", err).into()),
    };
    let data = match tera.render(TEMPLATE_NAME, &ctx) {
        Ok(data) => data,
        Err(err) => return Err(format!("render template: {}", err).into()),
    };

    // Save result to a file
    let path = Path::new(REPORT_FILE);
    match OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(path)
    {
        Ok(mut f) => match f.write_all(data.as_bytes()) {
            Ok(_) => Ok(()),
            Err(err) => return Err(format!("save file: {}", err).into()),
        },
        Err(err) => return Err(format!("open file: {}", err).into()),
    }
}
