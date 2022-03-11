use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    compile_blueprints("data/blp", "data/ui");

    // Copy data folder to OUT_DIR
    let out_dir = env::var("OUT_DIR").unwrap();
    for entry in fs::read_dir("data").unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() != "blp" {
            fs::copy(
                entry.path(),
                [&out_dir, "data", entry.file_name().to_str().unwrap()]
                    .iter()
                    .collect::<PathBuf>(),
            )
            .unwrap();
        }
    }

    gio::compile_resources(
        format!("{out_dir}/data"),
        &format!("{out_dir}/data/com.github.jms55.WhatTheFn.gresource.xml"),
        "com.github.jms55.WhatTheFn.gresource",
    );
}

pub fn compile_blueprints(source_dir: &str, target_dir: &str) {
    println!("cargo:rerun-if-changed={}", source_dir);

    let blueprint_files = fs::read_dir(source_dir)
        .unwrap()
        .map(|file| file.unwrap().path())
        .collect::<Vec<_>>();

    let out_dir = env::var("OUT_DIR").unwrap();
    let status = Command::new("blueprint-compiler")
        .arg("batch-compile")
        .arg(format!("{out_dir}/{target_dir}"))
        .arg(source_dir)
        .args(blueprint_files)
        .status()
        .unwrap();

    assert!(
        status.success(),
        "blueprint-compiler failed with exit status {status}",
    );
}
