use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Error as IOError};
use std::path::Path;

pub fn convert_perf_json_to_wtf<P: AsRef<Path>>(path: P) -> Result<Profile, IOError> {
    let reader = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader::<_, Profile>(reader)?)
}

#[derive(Deserialize)]
pub struct Profile {
    pub headers: Headers,
    pub samples: Vec<Sample>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Headers {
    pub captured_on: String,
    pub hostname: String,
    pub os_release: String,
    pub arch: String,
    pub cpu_desc: String,
    pub cpuid: String,
    pub nrcpus_online: u16,
    pub nrcpus_avail: u16,
    pub perf_version: String,
    pub cmdline: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct Sample {
    pub timestamp: u64,
    pub tid: u32,
    pub callchain: Vec<Symbol>,
}

#[derive(Deserialize, Clone)]
pub struct Symbol {
    pub ip: String,
    pub symbol: Option<String>,
    pub dso: Option<String>,
}
