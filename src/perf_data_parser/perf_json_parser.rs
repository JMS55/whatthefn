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
    headers: Headers,
    samples: Vec<Sample>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Headers {
    captured_on: String,
    hostname: String,
    os_release: String,
    arch: String,
    cpu_desc: String,
    cpuid: String,
    nrcpus_online: u16,
    nrcpus_avail: u16,
    perf_version: String,
    cmdline: Vec<String>,
}

#[derive(Deserialize)]
pub struct Sample {
    timestamp: u64,
    tid: u32,
    callchain: Vec<Symbol>,
}

#[derive(Deserialize)]
pub struct Symbol {
    ip: String,
    symbol: Option<String>,
    dso: Option<String>,
}
