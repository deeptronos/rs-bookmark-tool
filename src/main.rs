use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use unidecode::unidecode;

use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

/// A link to an educational resource.
/// # Fields
/// * `title` - The title of the resource.
/// * `link` - The URL of the resource.
/// * `desc` - A description of the resource.
/// * `added` - The date (in `YYYY-MM-DD`) the resource was added to the database.
/// * `accessed` - The date (in `YYYY-MM-DD`) the resource was last accessed.
/// * `tags` - A set of tags associated with the resource.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Link {
    pub title: String,
    pub link: String,
    pub desc: String,
    pub added: NaiveDate,
    pub accessed: NaiveDate,

    // #[serde(with = "ts_seconds_option")]
    // pub added: toml_datetime::Date,
    // #[serde(with = "ts_seconds_option")]
    // pub accessed: toml_datetime::Date,
    pub tags: Option<HashSet<String>>,
}

impl Link {
    pub fn new(
        title: &str,
        link: &str,
        desc: &str,
        added: &str,
        accessed: &str,
        tags: &Option<HashSet<String>>,
    ) -> Link {
        let title = title.into();
        let link = link.into();
        let desc = desc.into();
        let added_str: String = added.into();
        let accessed_str: String = accessed.into();
        let tags = tags.clone();

        // Parse the added date string into a NaiveDate object
        let added = if added_str.is_empty() || added_str.trim().to_lowercase() == "x" {
            chrono::Local::now().date_naive()
        } else {
            match chrono::NaiveDate::parse_from_str(&added_str, "%Y-%m-%d") {
                Ok(date) => date,
                Err(_) => {
                    eprintln!("Invalid date format for 'added' field. Using current date instead.");
                    chrono::Local::now().date_naive()
                }
            }
        };

        let accessed = if accessed_str.is_empty() || added_str.trim().to_lowercase() == "x" {
            chrono::Local::now().date_naive()
        } else {
            match chrono::NaiveDate::parse_from_str(&accessed_str, "%Y-%m-%d") {
                Ok(date) => date,
                Err(_) => {
                    eprintln!(
                        "Invalid date format for 'accessed' field. Using current date instead."
                    );
                    chrono::Local::now().date_naive()
                }
            }
        };

        Link {
            title,
            link,
            desc,
            added,
            accessed,
            tags,
        }
    }
}

pub fn format_tags(tags: &HashSet<String>) -> String {
    let taglist: Vec<&str> = tags.iter().map(String::as_ref).collect();
    let quoted_taglist: Vec<String> = taglist.iter().map(|tag| format!("\"{}\"", tag)).collect();
    quoted_taglist.join(", ")
}

/// Prompt the user to input a new link's info once.
fn prompt() -> Link {
    let title = inquire::Text::new("Title: ")
        .prompt()
        .expect("An error happened while asking you for a title");
    let link = inquire::Text::new("URL: ")
        .prompt()
        .expect("An error happened while asking you for a URL");
    let desc = inquire::Text::new("Description: ")
        .prompt()
        .expect("An error happened while asking you for a description");
    let added = inquire::Text::new("Date (YYYY-MM-DD format) the resource was added to bookmarks (Leave empty to autofill today)")
        .prompt()
        .expect("An error happened while asking you for the date the resource was added");
    let accessed = inquire::Text::new(
        "Date (YYYY-MM-DD format) the resource was last accessed (Leave empty to autofill today)",
    )
    .prompt()
    .expect("An error happened while asking you for the date the resource was last accessed");
    let tags_str =
        inquire::Text::new("Enter tags (seperated by only a comma \",\") or leave blank:")
            .prompt()
            .expect("An error happened while asking you for tags");
    let tags: Option<HashSet<String>> = if tags_str.is_empty() {
        None
    } else {
        Some(
            tags_str
                .split(',')
                .map(str::trim)
                .map(str::to_string)
                .collect(),
        )
    };

    Link::new(&title, &link, &desc, &added, &accessed, &tags)
}

/// Output the link's info to a TOML file.
fn output(lnk: Link, dir: &str) {
    let safe_title = unidecode(&lnk.title);
    let safe_title = safe_title.replace('/', "_");
    let safe_title = safe_title.replace('\\', "_");
    let safe_title = safe_title.replace(':', "_");
    let safe_title = safe_title.replace('*', "_");
    let safe_title = safe_title.replace('?', "_");
    let safe_title = safe_title.replace('"', "_");
    let safe_title = safe_title.replace('<', "_");
    let safe_title = safe_title.replace('>', "_");
    let safe_title = safe_title.replace('|', "_");
    let safe_title = safe_title.replace(' ', "_");
    let safe_title = safe_title.replace('(', "_");
    let safe_title = safe_title.replace(')', "_");
    let safe_title = safe_title.to_lowercase();

    let mut text: String = format!(
        "[{safe_title}]
title = \"{title}\"
link = \"{link}\"
desc = \"{desc}\"
added = \"{added}\"
accessed = \"{accessed}\"
",
        safe_title = safe_title,
        title = lnk.title,
        link = lnk.link,
        desc = lnk.desc,
        added = lnk.added,
        accessed = lnk.accessed,
    );
    if let Some(tags) = &lnk.tags {
        text += &format!("tags = [{}]", format_tags(tags));
    }
    // print!("Got: {}", text);
    fs::write(format!("{}/{}.toml", dir, safe_title), text).expect("Unable to write file");
}

/// Fix an issue where many link TOMLs had headers with invalid characters (mostly, spaces)
fn fix_headers(dir: &str) -> std::io::Result<()> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(OsStr::to_str) == Some("toml") {
            let content = fs::read_to_string(&path)?;
            let mut new_content = String::new();
            for line in content.lines() {
                if line.starts_with('[') && line.ends_with(']') {
                    let header = &line[1..line.len() - 1];
                    let safe_header = unidecode(header);
                    let safe_header = safe_header.replace(' ', "_");
                    let safe_header = safe_header.replace("'", "_");
                    let safe_header = safe_header.replace('"', "_");
                    let safe_header = safe_header.replace('(', "");
                    let safe_header = safe_header.replace(')', "");
                    new_content.push_str(&format!("[{}]\n", safe_header));
                } else {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }
            fs::write(&path, new_content)?;
    
    
            }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let toml_directory = "/toml";
    let cwd: PathBuf = env::current_dir()?;
    let cwd: String = cwd
        .to_str()
        .expect("Unable to convert working path's PathBuf to &str.")
        .to_string();
    let toml_path = format!("{}{}", cwd, toml_directory);
    if fs::metadata(toml_path.clone()).is_err() {
        fs::create_dir(toml_path.clone()).expect("Unable to create directory");
    } else {
        println!("Found existing directory at {}.", &toml_path)
    }


    fix_headers(&toml_path).expect("Unable to fix headers");
    Ok(())
    // loop {
    //     let lnk = prompt();
    //     output(lnk, &toml_path);
    //     // print!();
    //     // let ans =
    //     let ans = inquire::Text::new(
    //         "Would you like to add another link? ((N)o/(y)es or any other input): ",
    //     )
    //     .prompt()
    //     .expect("An error happened when asking if you'd like to continue");
    //     if ans.to_lowercase() == "n" {
    //         break;
    //     }
    // }

    // Ok(())
}
