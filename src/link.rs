use std::collections::HashSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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

/// Link struct specified to resource.json format.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonLink {
    pub title: String,
    pub url: String,
    pub description: String,
    pub category: String,
    pub year: i32,
    pub tags: Option<HashSet<String>>,
    pub free: bool,
}

impl JsonLink {
    pub fn new(
        title: &str,
        url: &str,
        description: &str,
        category: &str,
        year: i32,
        tags: &Option<HashSet<String>>,
        free: bool,
    ) -> JsonLink {
        let title = title.into();
        let url = url.into();
        let description = description.into();
        let category = category.into();
        let tags = tags.clone();

        JsonLink {
            title,
            url,
            description,
            category,
            year,
            tags,
            free,
        }
    }
}
