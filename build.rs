use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    compile_blueprints("res/blp", "res/ui");

    // Copy res folder to OUT_DIR
    let out_dir = env::var("OUT_DIR").unwrap();
    for entry in fs::read_dir("res").unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() != "blp" {
            fs::copy(
                entry.path(),
                [&out_dir, "res", entry.file_name().to_str().unwrap()]
                    .iter()
                    .collect::<PathBuf>(),
            )
            .unwrap();
        }
    }

    gio::compile_resources(
        format!("{out_dir}/res"),
        &format!("{out_dir}/res/com.github.jms55.WhatTheFn.gresource.xml"),
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
