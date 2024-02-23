use std::fs;
use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::accesslog;
use crate::geodata;

// NOTE: `include_str!` macro is not cross-platform for paths.
// https://github.com/rust-lang/rust/issues/75075

// Report-file related files and parameters.
const TEMPLATE: &str = include_str!("../index.html.j2");
const TEMPLATE_NAME: &str = "index";
const HTML_DEPS_DIR: &str = "static";

// HtmlDep describes a file that is required by the HTML template
// (e.g. images, CSS, JS files).
struct HtmlDep {
    content: &'static [u8],
    file: &'static str,
}

// A list of all dependencies with their content.
const HTML_DEPS: [HtmlDep; 7] = [
    HtmlDep{
        content: include_bytes!("../static/favicon.png"),
        file: "favicon.png",
    },
    HtmlDep{
        content: include_bytes!("../static/apexcharts.js"),
        file: "apexcharts.js",
    },
    HtmlDep{
        content: include_bytes!("../static/jsvectormap.js"),
        file: "jsvectormap.js",
    },
    HtmlDep{
        content: include_bytes!("../static/jsvectormap.min.css"),
        file: "jsvectormap.min.css",
    },
    HtmlDep{
        content: include_bytes!("../static/tabler.js"),
        file: "tabler.js",
    },
    HtmlDep{
        content: include_bytes!("../static/tabler.min.css"),
        file: "tabler.min.css",
    },
    HtmlDep{
        content: include_bytes!("../static/world.js"),
        file: "world.js",
    },
];

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
    file: &str,
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
    match tera.add_raw_template(TEMPLATE_NAME, TEMPLATE) {
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

    match html_deps() {
        Ok(_) => (),
        Err(err) => return Err(format!("generate HTML dependencies: {}", err).into()),
    };

    // Save result to a file
    let path = Path::new(file);
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

// Create HTML dependencies files and directories.
fn html_deps() -> Result<(), Box<dyn Error>> {
    match fs::create_dir_all(HTML_DEPS_DIR) {
        Ok(_) => (),
        Err(err) => return Err(format!("create directory {}: {}", HTML_DEPS_DIR, err).into()),
    };
    for dep in HTML_DEPS {
        let p = Path::new(HTML_DEPS_DIR).join(dep.file);
        match fs::write(&p, dep.content) {
            Ok(_) => (),
            Err(err) => return Err(format!("save file {}: {}", p.display(), err).into()),
        };
    }
    Ok(())
}
