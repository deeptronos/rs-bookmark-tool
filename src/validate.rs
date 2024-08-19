use core::panic;
use std::{ffi::OsStr, fs};

use crate::link::Link;

fn validate_entry(content: &str) -> Result<(), String> {
    let entry: Link = toml::from_str(content).expect("Unable to parse toml");
    if entry.title.is_empty() {
        return Err("Title is empty".to_string());
    }
    if entry.link.is_empty() {
        return Err("URL is empty".to_string());
    }
    if entry.desc.is_empty() {
        // Don't throw error, but alert.
        print!("Contains no description.");
    }
    Ok(())
}

/// Validates all entries in the given directory, in relation to schema defined by Link struct.
pub fn validate_entries(dir: &str) -> std::io::Result<()> {
    let mut entries = fs::read_dir(dir)?;
    let mut i = 1;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(OsStr::to_str) == Some("toml") {
            println!("File {}: validating {}... ", i, path.display());
            let content = fs::read_to_string(&path)?;
            let result = validate_entry(&content);
            match (result) {
                Ok(()) => println!("\tIt's valid"),
                Err(err) => panic!("{} is invalid: {}", path.display(), err),
            }
            i += 1;
        }
    }
    Ok(())
}
