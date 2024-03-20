use crate::command::Command;
use chksum_sha1 as sha1;
use clap::Parser;
use std::fs::{read_to_string, File};
use std::io::{Read, Write};
use std::path::Path;
use std::process::exit;
use std::{fs, io};

#[derive(Parser, Debug, Clone)]
pub struct AddArgs {
    directory: Vec<String>,
}

#[derive(Debug)]
pub struct AddCommand {
    arguments: AddArgs,
}

impl AddCommand {
    pub fn new(args: AddArgs) -> Self {
        AddCommand { arguments: args }
    }
}

impl Command for AddCommand {
    fn execute(&mut self) {
        let base_path = String::from("./");

        let pit_path = Path::new(&base_path).join(".pit");

        if !pit_path.exists() {
            println!("Pit file not present. Run pit init.");
            return;
        }

        let mut ignored_files: Vec<String> = Vec::new();
        get_gitignore_files(base_path.clone(), &mut ignored_files);

        let ignored: Vec<String> = ignored_files.iter().map(|r| r.to_string()).collect();
        let mut hashes: Vec<String> = vec![];

        for path in self.arguments.directory.clone() {
            let file_path = Path::new(&base_path).join(&path);

            if !file_path.exists() {
                println!("{} does not exist", file_path.to_str().unwrap());
                continue;
            }
            let index = ignored
                .iter()
                .position(|r| *r == path || *r == "./".to_owned() + &*path);
            if index.is_some() {
                continue;
            }

            if file_path.is_dir() {
                select_all_files(String::from(file_path.to_str().unwrap()), &ignored_files);
                continue;
            }

            let hash = create_blob_file(path);
            if !hash.is_empty() {
                hashes.push(hash);
            }
        }
        let cache_result = read_to_string("./.pit/objects/info");
        let cache = cache_result.unwrap();
        let cached = cache.clone();
        let mut cache_items: Vec<&str> = cached.lines().collect();
        for hash in &hashes {
            let same_file_hash = is_file_modified(cache_items.clone(), hash.clone());
            if !same_file_hash.is_empty() {
                let index = cache_items
                    .iter()
                    .position(|x| *x == same_file_hash)
                    .unwrap();
                cache_items.remove(index);
            }
            if !cache_items.contains(&hash.as_str()) {
                cache_items.push(hash.as_str());
            }
        }

        let _ = fs::write("./.pit/objects/info", cache_items.join("\n"));
    }
}

fn select_all_files(path: String, ignored: &Vec<String>) {
    let entries_result = fs::read_dir(path.clone());
    if entries_result.is_err() {
        println!("{:?} Couldn't read the files", entries_result);
        return;
    }
    let entries = entries_result
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>();

    if entries.is_err() {
        println!("{:?} Couldn't read the files", entries);
        return;
    }
    for entry in entries.unwrap() {
        if entry.is_dir() {
            select_all_files(String::from(entry.to_str().unwrap()), ignored);
        } else {
            let index = ignored.iter().position(|r| {
                *r == entry.to_str().unwrap() || *r == "./".to_owned() + entry.to_str().unwrap()
            });
            if index.is_some() {
                continue;
            }
            let cache_result = read_to_string("./.pit/objects/info");
            let cache = cache_result.unwrap();
            let mut cache_items: Vec<&str> = cache.lines().collect();
            let hash = create_blob_file(String::from(entry.to_str().unwrap()));
            let same_file_hash = is_file_modified(cache_items.clone(), hash.clone());
            if !same_file_hash.is_empty() {
                let index = cache_items
                    .iter()
                    .position(|x| *x == same_file_hash)
                    .unwrap();
                cache_items.remove(index);
            }
            if !cache_items.contains(&hash.as_str()) {
                cache_items.push(hash.as_str());
            }
            let _ = fs::write("./.pit/objects/info", cache_items.join("\n"));
        }
    }
}

fn get_gitignore_files(path: String, ignored_files: &mut Vec<String>) {
    let entries_result = fs::read_dir(path.clone());
    if entries_result.is_err() {
        println!("{:?} Couldn't read the folder", entries_result);
    }
    let entries = entries_result
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>();

    if entries.is_err() {
        println!("{:?} Couldn't read the files", entries);
    }

    for entry in entries.unwrap() {
        if entry.is_dir() {
            if entry.to_str().unwrap() == "./" || entry.to_str().unwrap() == "../" {
                continue;
            }

            get_gitignore_files(String::from(entry.to_str().unwrap()), ignored_files);
        } else if entry.file_name().unwrap() == ".pitignore" {
            for line in read_to_string(entry.clone()).unwrap().lines() {
                if path == "./" {
                    ignored_files.push(path.clone() + line);
                } else {
                    ignored_files.push(path.clone() + "/" + line);
                }
            }
        }
    }
}

fn create_blob_file(path: String) -> String {
    let mut content: String = String::from("");
    let mut file = File::open(&path).unwrap();

    let result = file.read_to_string(&mut content);
    if result.is_err() {
        return Default::default();
    }
    content.push_str("\n\n");
    content.push_str(&path);
    content.push_str("\n\nblob");
    let digest = sha1::chksum(content.clone()).unwrap();

    let objects_folder = String::from("./.pit/objects/");

    let object_path = objects_folder + &*digest.to_hex_lowercase();
    let new_file_path = Path::new(&object_path);
    let file_result = File::create(new_file_path);

    if file_result.is_err() {
        println!("File cannot be created: {:?}", file_result.err());
        exit(1);
    }

    let writing_result = file_result.unwrap().write_all(content.as_ref());
    if writing_result.is_err() {
        println!(
            "Error writing in file {}: Error {:?}",
            object_path, writing_result
        );
    }

    digest.to_hex_lowercase()
}

fn is_file_modified(items: Vec<&str>, hash: String) -> String {
    let result = fs::read_to_string("./.pit/objects/".to_string() + &hash);
    if result.is_err() {
        return Default::default();
    }
    let content = result.unwrap();
    let lines = content.lines().collect::<Vec<&str>>();
    let file_name_hash = lines[0..lines.len() - 2].last().unwrap();
    for item in items {
        if item.is_empty() {
            continue;
        }

        let result = fs::read_to_string("./.pit/objects/".to_string() + item);
        if result.is_err() {
            continue;
        }
        let content = result.unwrap();
        let lines = content.lines().collect::<Vec<&str>>();
        let file_name = lines[0..lines.len() - 2].last().unwrap();
        if file_name == file_name_hash {
            return item.to_string();
        }
    }

    Default::default()
}
