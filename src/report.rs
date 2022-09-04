use std::cmp;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::iter::FromIterator;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::accesslog;
use crate::geodata;

// Report-file related files and parameters.
const TEMPLATE_FILE: &str = "index.html.j2";
const TEMPLATE_NAME: &str = "index";
const REPORT_FILE: &str = "index.html";

// Report contains ata for producing a human-readable representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Report {
    days: Vec<ReportCounter>,
    countries: Vec<ReportCounter>,
    cities: Vec<ReportCounter>,
    points: Vec<geodata::Point>,
}

// Named counter for representing report stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReportCounter {
    name: String,
    count: u32,
}

// Build a report and save it to a file.
pub fn generate(
    log: accesslog::AccessLog,
    geo: HashMap<String, geodata::GeoData>,
) -> Result<(), Box<dyn Error>> {
    let rep = match build(log, geo) {
        Ok(val) => val,
        Err(err) => return Err(format!("build report: {}", err).into()),
    };

    match save(&rep) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("save report: {}", err).into()),
    }
}

// Build a report from input data.
fn build(
    log: accesslog::AccessLog,
    geo: HashMap<String, geodata::GeoData>,
) -> Result<Report, Box<dyn Error>> {
    // Get lists of unique ips by different entities
    let mut days: HashMap<String, Vec<String>> = HashMap::new();
    let mut countries: HashMap<String, Vec<String>> = HashMap::new();
    let mut cities: HashMap<String, Vec<String>> = HashMap::new();
    let mut points: HashSet<geodata::Point> = HashSet::new();
    for line in log.lines {
        let g = match geo.get(&line.ip) {
            Some(res) => res,
            None => return Err(format!("no matching geodata for ip '{}'", line.ip).into()),
        };
        days.entry(line.date.clone())
            .or_insert(Vec::new())
            .push(line.ip.clone());
        countries
            .entry(g.country_code.clone())
            .or_insert(Vec::new())
            .push(line.ip.clone());
        cities
            .entry(g.city.clone())
            .or_insert(Vec::new())
            .push(line.ip.clone());
        points.insert(geodata::Point {
            name: g.city.clone(),
            lat: g.lat,
            lon: g.lon,
        });
    }

    // Get the number of elements in each ips hashset
    let mut rep: Report = Report {
        days: Vec::new(),
        countries: Vec::new(),
        cities: Vec::new(),
        points: points.into_iter().collect(),
    };
    for (day, ips) in days.iter() {
        rep.days.push(ReportCounter {
            name: day.clone(),
            count: HashSet::<String>::from_iter(ips.clone()).len() as u32,
        });
    }
    for (country, ips) in countries.iter() {
        rep.countries.push(ReportCounter {
            name: country.clone(),
            count: HashSet::<String>::from_iter(ips.clone()).len() as u32,
        });
    }
    for (city, ips) in cities.iter().filter(|(k, _)| !k.is_empty()) {
        rep.cities.push(ReportCounter {
            name: city.clone(),
            count: HashSet::<String>::from_iter(ips.clone()).len() as u32,
        });
    }

    rep.days.sort_by(|a, b| a.name.cmp(&b.name));
    rep.countries.sort_by(|a, b| a.name.cmp(&b.name));
    rep.cities.sort_by(|a, b| b.count.cmp(&a.count));
    rep.cities = rep.cities[..cmp::min(rep.cities.len(), 8)].to_vec();

    Ok(rep)
}

// Render report data and save it to a file.
fn save(rep: &Report) -> Result<(), Box<dyn Error>> {
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
