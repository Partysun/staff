use dirs::config_dir;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

const APPLICATION_NAME: &str = "staff";

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
        None => grimoires_path.push(Path::new("basic.md")),
    };
    if Path::new(&grimoires_path).exists() {
        fs::read_to_string(grimoires_path).expect("Something went wrong reading the file")
    } else {
        println!("Spell not found!");
        "".to_string()
    }
}

#[derive(PartialEq, Eq, Hash)]
struct Metadata<'a> {
    author: &'a str,
    title: &'a str,
    tags: Vec<String>,
}

pub fn get_meta(content: String) {
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
    // meta_ast is a hashMap
    let mut title: String = "".to_string();
    let mut author: String = "".to_string();
    let mut tags: Vec<String> = vec![];

    for (els, _) in meta_ast.iter() {
        println!("Els: {:?}", els);
        println!("{:?}", els.get("title"));
        match els.get("title") {
            Some(t) => title = t.clone().as_string().unwrap(),
            None => (),
        }
        match els.get("author") {
            Some(a) => author = a.clone().as_string().unwrap(),
            None => (),
        }
        match els.get("tags") {
            Some(t) => tags = t.clone().as_array().unwrap(),
            None => (),
        }
    }

    println!("{}", title);
    println!("{}", author);
    println!("{:?}", tags);
    println!("{:#?}", meta);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_meta() {
        let content = include_str!("../grimoires/ask_zelda.md").to_string();
        get_meta(content);
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
