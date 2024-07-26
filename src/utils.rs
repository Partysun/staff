use dirs::config_dir;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::path::Path;
use std::path::PathBuf;

const APPLICATION_NAME: &str = env!("CARGO_PKG_NAME");

// Usage:
//
// let mut grimoires_path = get_or_create_config(Some("grimoires")).unwrap();
// grimoires_path.push(Path::new("zelda_talk.md"));
// match download_file(
//     "http link to your grimoire".to_string(),
//     grimoires_path,
// )
// .await
// {
//     Ok(()) => println!("Downloaded new Grimoire"),
//     Err(_) => eprintln!("Failed downloading a grimoire"),
// }
pub async fn download_file(url: String, file_name: PathBuf) -> Result<(), reqwest::Error> {
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(file_name).unwrap();
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file).unwrap();
    Ok(())
}

fn create_initial_grimoires() {
    let mut path = config_dir().map(|d| d.join(APPLICATION_NAME)).unwrap();
    path.push("grimoires");
    path.push("zelda_talk.md");
    let mut initial_grimoire =
        File::create(&path).expect("Error encountered while creating grimoire!");
    initial_grimoire
        .write_all(
            b"---
title: Zelda Talk
author: Zatsepin <zatsepin.dev>
tags: [fun]
description: This spell makes your AI as a Hero from Hyrule. Pff. Tales are coming...
---

You are Link from Zelda, say answer as this hero

INPUT:",
        )
        .expect("Error while writing to file");

    let content = fs::read_to_string(path).unwrap();
    let meta = get_meta(content);
    println!("Metadata: {} \n", meta)
}

pub fn has_any_grimoires() -> bool {
    let mut path = config_dir().map(|d| d.join(APPLICATION_NAME)).unwrap();
    path.push("grimoires");
    match fs::metadata(&path) {
        Ok(_) => true, // Path exists and it is a directory
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false, // Directory does not exist
        Err(e) => panic!("An error occurred: {}", e), // Another problem occurred (i.e., file/disk may be full etc.)
    }
}

pub fn get_or_create_config(folder: Option<&str>) -> Option<PathBuf> {
    let config_dir = config_dir().unwrap();

    match dirs::config_dir() {
        Some(mut path) => {
            path.push(APPLICATION_NAME);
            match folder {
                None => println!("Nothing to do"),
                Some(f) => path.push(f),
            }
            if !path.exists() || !path.is_dir() {
                fs::create_dir_all(&path).expect("Failed to create directory");
                // If we are creating first time the grimoires
                // folder, we should to create a first grimoire
                if (folder.unwrap() == "grimoires") {
                    create_initial_grimoires();
                }
            }
            Some(path)
        }
        _ => None,
    }
}

pub fn read_spell(name: &Option<String>) -> String {
    let mut grimoires_path = get_or_create_config(Some("grimoires")).unwrap();
    match name {
        Some(n) => grimoires_path.push(Path::new(&n).with_extension("md")),
        None => grimoires_path.push(Path::new("zelda_talk.md")),
    };
    if Path::new(&grimoires_path).exists() {
        fs::read_to_string(grimoires_path).expect("Something went wrong reading the file")
    } else {
        println!("Spell not found!");
        "".to_string()
    }
}

#[derive(Debug)]
pub struct GrimoireMetadata {
    author: String,
    title: String,
    tags: Vec<String>,
    description: String,
}

impl fmt::Display for GrimoireMetadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\n Author: {} \n Title: {} \n Tags: {:?} \n Description: {}",
            self.author, self.title, self.tags, self.description
        )
    }
}

pub fn get_meta(content: String) -> GrimoireMetadata {
    let mut type_mark = HashMap::new();

    type_mark.insert("tags".into(), "array");
    type_mark.insert("released".into(), "bool");
    type_mark.insert("author".into(), "string");
    type_mark.insert("title".into(), "string");

    let meta = markdown_meta_parser::MetaData {
        content,
        required: vec!["title".into()],
        type_mark,
    };

    let meta_ast = meta.parse();
    let mut title: String = "Untitled".to_string();
    let mut author: String = "Who'knows guy".to_string();
    let mut description: String = "No description is avaible".to_string();
    let mut tags: Vec<String> = vec![];

    for (els, _) in meta_ast.iter() {
        title = els.get("title").unwrap().clone().as_string().unwrap();
        author = els.get("author").unwrap().clone().as_string().unwrap();
        description = els.get("description").unwrap().clone().as_string().unwrap();
        tags = els.get("tags").unwrap().clone().as_array().unwrap();
    }

    GrimoireMetadata {
        title,
        author,
        tags,
        description,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_meta() {
        let content = include_str!("../grimoires/ask_zelda.md").to_string();
        let meta = get_meta(content);
        assert_eq!(meta.title, "Zelda Voice");
    }

    #[test]
    fn test_get_or_create_config_folder() {
        let folder_path = "grimoires_spec";
        // how to send folder_path expected Option<&str>
        let path = get_or_create_config(Some(folder_path));
        let path = match path {
            Some(path) => path,
            None => panic!("Couldn't locate the path."),
        };

        match std::fs::metadata(&path) {
            Ok(data) => {
                assert!(data.is_dir());

                // Remove the folder
                match std::fs::remove_dir(&path) {
                    Ok(_) => println!("Directory removed successfully"),
                    Err(e) => panic!("Failed to remove directory: {:?}", e),
                }
            }

            Err(e) => panic!("Folder does not exist, error message: {:?}", e),
        };

        // Verify the folder no longer exists
        match std::fs::metadata(&path) {
            Ok(_data) => panic!("Directory still exists"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("Directory was not found, assuming it was removed successfully")
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
