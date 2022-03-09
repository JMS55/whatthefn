use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=data");

    let mut input_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    input_dir.push("data");
    let mut output_dir = input_dir.clone();
    output_dir.push("ui");
    input_dir.push("blp");

    let blueprint_files = fs::read_dir(&input_dir)
        .unwrap()
        .map(|file| file.unwrap().path())
        .collect::<Vec<_>>();

    assert!(Command::new("blueprint-compiler")
        .arg("batch-compile")
        .arg(&output_dir)
        .arg(input_dir)
        .args(&blueprint_files)
        .status()
        .unwrap()
        .success());

    gio::compile_resources(
        "data",
        "data/com.github.jms55.WhatTheFn.gresource.xml",
        "com.github.jms55.WhatTheFn.gresource",
    );

    fs::remove_dir_all(&output_dir).unwrap();
}
