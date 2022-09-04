use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json;

// Cache for geodata from remote API.
const CACHE_FILE: &str = "cache.json";

// Geodata for an IP address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoData {
    pub country: String,
    pub country_code: String,
    pub city: String,
    pub lat: f32,
    pub lon: f32,
}

// Implementation of hashing for using in hashmaps.
impl Hash for GeoData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        format!("{}|{}", self.country, self.city).hash(state);
    }
}

// Implementation of partitial equality for using in hashmaps.
impl PartialEq for GeoData {
    fn eq(&self, other: &Self) -> bool {
        return self.country == other.country && self.city == other.city;
    }
}

// Implementation of full equality. Same as PartialEq.
impl Eq for GeoData {}

// Named point on ф map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub name: String,
    pub lat: f32,
    pub lon: f32,
}

// Implementation of hashing for using in hashmaps.
impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

// Implementation of partitial equality for using in hashmaps.
impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        return self.name == other.name;
    }
}

// Implementation of full equality. Same as PartialEq.
impl Eq for Point {}

// Response from geodata remote API.
#[derive(Debug, Serialize, Deserialize)]
struct Response {
    country_name: String,
    country_code2: String,
    state_prov: String,
    city: String,
    latitude: String,
    longitude: String,
}

// Get geodata from a remote HTTP API or cache.
// Single-threaded non-concurrent process, IPs are resolved one by one.
// TODO: Make it concurrent.
pub fn get_geodata(
    api_addr: &str,
    api_key: &str,
    ips: Vec<String>,
) -> Result<HashMap<String, GeoData>, Box<dyn Error>> {
    let mut geo = read_cache(CACHE_FILE)?;
    let mut cached: u32 = 0;

    println!("IP addresses to resolve: {}", ips.len());
    for (i, ip) in ips.iter().enumerate() {
        // Track progress - print each 100th line
        if i > 0 && i % 100 == 0 {
            println!("Progress: {} done ({} cached)", i, cached);
        }
        // Skip IP if it's in cache, and count cache hits
        if geo.contains_key(ip) {
            cached += 1;
            continue;
        }
        // Make a call to remote API
        let geodata = match resolve(api_addr, api_key, &ip) {
            Ok(v) => v,
            Err(e) => {
                // Save current cache to not loose progress on next run
                save_cache(&geo, CACHE_FILE)?;
                return Err(e);
            }
        };
        geo.insert(ip.clone(), geodata);
    }

    save_cache(&geo, CACHE_FILE)?;

    Ok(geo)
}

// Resolve a single IP address to its geodata.
// Example: https://api.ipgeolocation.io/ipgeo?apiKey=token&ip=1.1.1.1
fn resolve(api_addr: &str, api_key: &str, ip: &String) -> Result<GeoData, Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let request = client
        .get(api_addr)
        .query(&[("apiKey", api_key), ("ip", &ip)]);
    let resp = request.send()?;

    let code = resp.status();
    if !code.is_success() {
        return Err(format!("invalid response code: {}", code.as_str()).into());
    }

    let json = resp.text()?;
    let r: Response = serde_json::from_str(&json)?;

    Ok(GeoData {
        country: r.country_name,
        country_code: r.country_code2,
        city: r.city,
        lat: r.latitude.parse()?,
        lon: r.longitude.parse()?,
    })
}

// Saves input cache data in a given file.
fn save_cache(cache: &HashMap<String, GeoData>, file: &str) -> Result<(), Box<dyn Error>> {
    serde_json::to_writer(&File::create(file)?, cache)?;
    Ok(())
}

// Reads cache data from a given file.
fn read_cache(file: &str) -> Result<HashMap<String, GeoData>, Box<dyn Error>> {
    let path = Path::new(file);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = File::open(path)?;
    let reader = BufReader::new(content);
    let cache: HashMap<String, GeoData> = serde_json::from_reader(reader)?;
    Ok(cache)
}
