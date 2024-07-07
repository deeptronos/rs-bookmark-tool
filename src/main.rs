use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use text_io::read;

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
    pub added: String,
    pub accessed: String,
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
        let added = added.into();
        let accessed = accessed.into();
        let tags = tags.clone();
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

/// Prompt the user to input a new link's info once.
fn prompt() -> Link {
    print!("Title: ");
    let title: String = read!();
    print!("URL: ");
    let link: String = read!();
    print!("Description: ");
    let desc: String = read!();
    print!("Date the resource was added (YYYY-MM-DD): ");
    let added: String = read!();
    print!("Date the resource was last accessed (YYYY-MM-DD): ");
    let accessed: String = read!();
    print!("Enter tags (seperated by comma \",\") or leave blank:");
    let tags_str: String = read!();
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
fn output(lnk: Link) {
    let str: String = format!(
        "
[{title}]
link = \"{link}\"
desc = \"{desc}\"
added = \"{added}\"
accessed = \"{accessed}\"
tags = {tags:?}
    ",
        title = lnk.title,
        link = lnk.link,
        desc = lnk.desc,
        added = lnk.added,
        accessed = lnk.accessed,
        tags = lnk.tags.unwrap()
    );
    print!("Got: {}", str);
    fs::write(format!("/toml/{}.toml", lnk.title), str).expect("Unable to write file");
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
        fs::create_dir(toml_path).expect("Unable to create directory");
    } else {
        print!("Directory already exists. Moving on...");
    }

    loop {
        let lnk = prompt();
        output(lnk);
        print!("Would you like to add another link? ((N)o/(y)es or any other input): ");
        let ans: String = read!();
        if ans.to_lowercase() == "n" {
            break;
        }
    }

    Ok(())
}
