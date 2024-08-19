use chrono::Datelike;
use chrono::NaiveDate;
use scraper::Html;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use unidecode::unidecode;

use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

mod link;
use link::JsonLink;
use link::Link;
pub mod validate;

/// Format hashset of strings representing a collection of tags associated with a link into a string containing a TOML-format array of tags.
pub fn format_tags(tags: &HashSet<String>) -> String {
    let taglist: Vec<&str> = tags.iter().map(String::as_ref).collect();
    let quoted_taglist: Vec<String> = taglist.iter().map(|tag| format!("\"{}\"", tag)).collect();
    quoted_taglist.join(", ")
}

/// Uses scraper and reqwuest to browse a webpage and extract the meta description.
fn browse_meta_description(url: &str) -> Option<String> {
    let response = reqwest::blocking::get(url).ok()?;
    let body = response.text().ok()?;
    let document = Html::parse_document(&body);
    let selector = scraper::Selector::parse("meta[name='description']").ok()?;
    let meta_description = document.select(&selector).next()?;
    let content = meta_description.value().attr("content")?;
    Some(content.to_string())
}

/// Prompt the user to input a new link's info once.
fn prompt() -> Link {
    let title = inquire::Text::new("Title: ")
        .prompt()
        .expect("An error happened while asking you for a title");
    let link = inquire::Text::new("URL: ")
        .prompt()
        .expect("An error happened while asking you for a URL");

    let auto_desc = browse_meta_description(&link);
    let desc = inquire::Text::new("(Press enter to use default) Description: ") // Autofill with meta description
        .with_default(&auto_desc.unwrap_or_default()) // also use with_placeholder? or does this show the default to the user?
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
    // TODO ROFL - condense by using iterator with a list of invalid chars.
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
    let safe_title = safe_title.replace('+', "_");
    let safe_title = safe_title.replace('&', "_");
    let safe_title = safe_title.replace('^', "_");
    let safe_title = safe_title.replace('#', "_");
    let safe_title = safe_title.replace('@', "_");
    let safe_title = safe_title.replace('$', "_");
    let safe_title = safe_title.replace('%', "_");
    let safe_title = safe_title.replace('=', "_");
    let safe_title = safe_title.replace("'", "_");
    let safe_title = safe_title.replace(".", "_");
    let safe_title = safe_title.replace("__", "_");
    let safe_title = safe_title.to_lowercase();

    let mut text: String = format!(
        "title = \"{title}\"
link = \"{link}\"
desc = \"{desc}\"
added = \"{added}\"
accessed = \"{accessed}\"
",
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

/// Take a JSON file of links w/ data attributes (name, description, etc.) and turn them into Link objects to output.
fn read_links_from_json(file_path: &str) -> Result<Vec<JsonLink>> {
    let file = File::open(file_path).expect("Unable to open file_path JSON");
    let reader = BufReader::new(file);
    let links: Vec<JsonLink> = serde_json::from_reader(reader).expect("from_reader failed.");
    print!("{} links found.", links.len());
    Ok(links)
}

fn output_from_json(links: Vec<JsonLink>, dir: &str) {
    for link in links {
        let title = link.title;
        let lnk = link.url;
        let desc = link.description;
        let added = chrono::Local::now().date_naive().year();
        let accessed = chrono::Local::now().date_naive().year();
        let tags = link.tags;
        let lnk = Link::new(
            &title,
            &lnk,
            &desc,
            &added.to_string(),
            &accessed.to_string(),
            &tags,
        );
        output(lnk, dir);
    }
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

    let result = validate::validate_entries(&toml_path);
    match (result) {
        Ok(()) => println!("No errors found."),
        _ => println!("{:#?} errors found.", result),
    }
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
