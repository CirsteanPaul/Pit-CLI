use crate::command::Command;
use crate::Parser;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(Parser, Debug, Clone)]
pub struct InitArgs {
    directory: Option<String>,
}

#[derive(Debug)]
pub struct InitializeCommand {
    arguments: InitArgs,
}

impl InitializeCommand {
    pub fn new(args: InitArgs) -> Self {
        InitializeCommand { arguments: args }
    }
}

impl Command for InitializeCommand {
    fn execute(&mut self) {
        let mut path_string: String = String::from("./");

        if self.arguments.directory.is_some() {
            path_string.push_str(&self.arguments.directory.clone().unwrap());
        }

        let directory_path = Path::new(&path_string);

        let path = Path::new(&path_string).join(".pit");
        if !directory_path.is_dir() {
            println!("The path provided for the pit file is not a folder");
            return;
        }
        if path.exists() {
            println!("Pit file already exist. No action done.");
            return;
        }

        create_pid_folder(path);

        println!("Pit file was successfully created!");
    }
}

fn create_pid_folder(path: PathBuf) {
    let pit_folder_result = fs::create_dir(path.clone());

    if pit_folder_result.is_err() {
        println!("Pit file cannot be created: {:?}", pit_folder_result.err());
        exit(1);
    }

    let pit_objects_path = path.to_str().unwrap().to_owned() + "/objects";
    let pit_objects_folder = Path::new(&pit_objects_path);
    let pit_objects_folder_result = fs::create_dir(pit_objects_folder);
    if pit_objects_folder_result.is_err() {
        println!(
            "Pit object file cannot be created: {:?}",
            pit_objects_folder_result.err()
        );
        exit(1);
    }
    let add_folder_path = pit_objects_path + "/info";
    let add_folder_result = File::create(add_folder_path);
    if add_folder_result.is_err() {
        println!(
            "Cache file cannot be created: {:?}",
            add_folder_result.err()
        );
        exit(1);
    }

    let pit_refs_path = path.to_str().unwrap().to_owned() + "/refs";

    let pit_refs_folder = Path::new(&pit_refs_path);

    let pit_refs_folder_result = fs::create_dir(pit_refs_folder);

    if pit_refs_folder_result.is_err() {
        println!(
            "Pit refs file cannot be created: {:?}",
            pit_refs_folder_result.err()
        );
        exit(1);
    }

    let pit_head_path = path.to_str().unwrap().to_owned() + "/HEAD";

    let pit_head_file = Path::new(&pit_head_path);
    let file_result = File::create(pit_head_file);

    if file_result.is_err() {
        println!("Pit head file cannot be created: {:?}", file_result.err());
        exit(1);
    }

    file_result
        .unwrap()
        .write_all(b"refs/main")
        .expect("No permissions");
}
