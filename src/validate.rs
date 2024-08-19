use core::panic;
use std::{ffi::OsStr, fs};

use crate::link::Link;

fn validate_entry(content: &str) -> Result<(), String> {
    print!("Content: {}", content);
    let entry: Link = toml::from_str(content).expect("Unable to parse toml");
    if entry.title.is_empty() {
        return Err("Title is empty".to_string());
    }
    if entry.link.is_empty() {
        return Err("URL is empty".to_string());
    }
    if entry.desc.is_empty() {
        return Err("Description is empty".to_string());
    }
    Ok(())
}

pub fn validate_entries(dir: &str) -> std::io::Result<()> {
    let mut entries = fs::read_dir(dir)?;
    let i = 1;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(OsStr::to_str) == Some("toml") {
            print!("{}: Validating {}... ", i, path.display());
            let content = fs::read_to_string(&path)?;
            let result = validate_entry(&content);
            match (result) {
                Ok(()) => println!("{} is valid", path.display()),
                Err(err) => panic!("{} is invalid: {}", path.display(), err),
            }
        }
    }
    Ok(())
}
