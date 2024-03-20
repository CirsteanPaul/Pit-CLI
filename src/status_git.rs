use crate::command::Command;
use chksum_sha1 as sha1;
use clap::Parser;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::Path;
use std::process::exit;
use std::rc::Rc;
use std::string::String;
use std::{fs, io};

#[derive(Parser, Debug, Clone)]
pub struct StatusArgs {}

#[derive(Debug)]
pub struct StatusCommand {
    _arguments: StatusArgs,
}
type TreeNodeRef = Rc<RefCell<TreeInfo>>;
#[derive(Debug, PartialEq, Clone, Default)]
struct TreeInfo {
    path: String,
    hash: String,
    pit_path: String,
    parent: Option<TreeNodeRef>,
    name: String,
    type_of_file: String,
    children: Vec<TreeNodeRef>,
}
#[derive(Debug)]
struct Message {
    modified_files: Vec<String>,
    added_files: Vec<String>,
    untracked_changes: Vec<String>,
    untracked_added: Vec<String>,
}
impl TreeInfo {
    fn new(path: String, name: String, parent: Option<TreeNodeRef>) -> Self {
        TreeInfo {
            children: Vec::new(),
            path,
            pit_path: Default::default(),
            parent,
            name,
            type_of_file: Default::default(),
            hash: "".to_string(),
        }
    }
}

impl StatusCommand {
    pub fn new(args: StatusArgs) -> Self {
        StatusCommand { _arguments: args }
    }
}

impl Command for StatusCommand {
    fn execute(&mut self) {
        let mut root = TreeNodeRef::new(RefCell::from(TreeInfo::new(
            "./".to_string(),
            "./".to_string(),
            None,
        )));
        let current_branch = take_current_branch();
        let result_current_commit =
            fs::read_to_string("./.pit/".to_string() + current_branch.as_str());

        if result_current_commit.is_ok() {
            root = construct_tree(result_current_commit.ok());
        }
        // root will contain the committed tree already/
        let mut message = Message {
            modified_files: vec![],
            added_files: vec![],
            untracked_changes: vec![],
            untracked_added: vec![],
        };
        root = add_tracked_files(root, &mut message);

        add_untracked_files(root.clone(), "./".to_string(), &mut message);
        println!("Tracked files: ");
        for mes in message.added_files {
            println!("{} added", mes);
        }

        for mes in message.modified_files {
            println!("{} modified", mes);
        }
        println!("\nUntracked files: ");
        for mes in message.untracked_added {
            println!("{} added", mes);
        }

        for mes in message.untracked_changes {
            println!("{} modified", mes);
        }
    }
}

fn take_current_branch() -> String {
    let base_path = String::from("./");
    let head_path = Path::new(&base_path).join(".pit/HEAD");
    let head = fs::read_to_string(head_path.clone());
    if head.is_err() {
        println!("Head file is not present. Fatal error");
        exit(1);
    }

    head.unwrap()
}

fn construct_tree(commit: Option<String>) -> TreeNodeRef {
    let objects_file_path = String::from("./.pit/objects/");
    let root = TreeNodeRef::new(RefCell::from(TreeInfo::new(
        "./".to_string(),
        "./".to_string(),
        None,
    )));

    if commit.is_none() {
        return root;
    }

    let commit_content_result =
        fs::read_to_string(objects_file_path.clone() + commit.unwrap().as_str());
    if commit_content_result.is_err() {
        return root;
    }

    let commit_content = commit_content_result.unwrap();
    let tree_line: Vec<&str> = commit_content.lines().next().unwrap().split(' ').collect();
    root.borrow_mut().pit_path = objects_file_path.clone() + tree_line[1].clone();
    root.borrow_mut().hash = tree_line[1].clone().to_string();
    root.borrow_mut().type_of_file = "tree".to_string();
    // we consider the first tree file as the root of our changes.
    let mut deque: VecDeque<TreeNodeRef> = VecDeque::new();
    deque.push_back(root.clone());
    while !deque.is_empty() {
        let current = deque.pop_front().unwrap();
        let current_content_result = fs::read_to_string(current.borrow().pit_path.clone());
        if current_content_result.is_err() {
            continue;
        }

        let current_content = current_content_result.unwrap();
        let lines = current_content.lines();

        if current.borrow().type_of_file == "tree" {
            for line in lines {
                if line.is_empty() {
                    continue;
                }
                let data_line: Vec<&str> = line.split(' ').collect();

                if data_line.len() != 3 {
                    continue;
                }

                let new_node = TreeNodeRef::new(RefCell::from(TreeInfo::new(
                    data_line[2].to_string(),
                    data_line[2].to_string(),
                    Some(current.clone()),
                )));
                new_node.borrow_mut().hash = data_line[1].to_string();
                new_node.borrow_mut().pit_path = objects_file_path.clone() + data_line[1];
                new_node.borrow_mut().type_of_file = data_line[0].to_string();
                current.borrow_mut().children.push(new_node.clone());
                deque.push_back(new_node);
            }
        }
    }

    root
}

fn add_tracked_files(root: TreeNodeRef, message: &mut Message) -> TreeNodeRef {
    let objects_file_path = String::from("./.pit/objects/");

    let cached_files_result = fs::read_to_string(objects_file_path.clone() + "info");
    if cached_files_result.is_err() {
        return root;
    }
    let cached_files = cached_files_result.unwrap();
    for cached_file in cached_files.lines() {
        if cached_file.is_empty() {
            continue;
        }
        let cached_file_content_result =
            fs::read_to_string(objects_file_path.clone() + cached_file);
        if cached_file_content_result.is_err() {
            continue;
        }
        let content = cached_file_content_result.unwrap();
        let lines: Vec<&str> = content.lines().collect();
        // last 3 lines in order file_path "empty" file_type
        let file_name = lines[lines.len() - 3];
        let mut node = root.clone();
        let mut is_added = false;
        let file_paths: Vec<String> = file_name.split('/').map(|x| x.to_string()).collect();
        let mut idx = 0;
        loop {
            if idx == file_paths.len() {
                break;
            }
            if file_paths[idx] == "." {
                idx += 1;
                continue;
            }
            let mut next_node: Option<TreeNodeRef> = None;
            for child in &node.borrow().children {
                let child_data = child.borrow();
                let corresponding_name = child_data.name.split('/').take(idx + 2).last();
                if corresponding_name.is_none() {
                    continue;
                }
                if file_paths[idx] == corresponding_name.unwrap() {
                    next_node = Some(child.clone());
                }
            }
            if next_node.is_none() {
                is_added = true;
                let path = node.borrow().path.clone().trim_matches('/').to_string()
                    + "/"
                    + file_paths[idx].as_str();
                let added_node = TreeNodeRef::new(RefCell::from(TreeInfo::new(
                    path.clone(),
                    path,
                    Some(node.clone()),
                )));
                if idx == file_paths.len() - 1 {
                    added_node.borrow_mut().hash = cached_file.to_string();
                    added_node.borrow_mut().pit_path = objects_file_path.clone() + cached_file;
                    added_node.borrow_mut().type_of_file = "blob".to_string();
                } else {
                    added_node.borrow_mut().type_of_file = "tree".to_string();
                }
                node.borrow_mut().children.push(added_node.clone());
                node = added_node;
                idx += 1;
                continue;
            }
            node = next_node.unwrap();
            idx += 1;
        }
        if is_added {
            message.added_files.push(file_name.to_string());
        } else {
            node.borrow_mut().hash = cached_file.to_string();
            node.borrow_mut().pit_path = objects_file_path.clone() + cached_file;
            message.modified_files.push(file_name.to_string());
        }
    }

    root
}

fn add_untracked_files(root: TreeNodeRef, path: String, message: &mut Message) {
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
        let entry_path = String::from(entry.to_str().unwrap());
        if entry_path == "./.pit" || entry_path == "." {
            continue;
        }
        if entry.is_dir() {
            let root_values = root.borrow();
            let node = root_values
                .children
                .iter()
                .find(|x| *x.borrow().path.clone() == entry_path);
            match node {
                Some(x) => {
                    add_untracked_files(x.clone(), entry_path.clone(), message);
                }
                None => {
                    message.untracked_added.push(entry_path);
                }
            }
        } else {
            let root_values = root.borrow();
            let node = root_values
                .children
                .iter()
                .find(|x| *x.borrow().path.clone() == entry_path);
            match node {
                Some(x) => {
                    let content_result = fs::read_to_string(entry_path.clone());
                    if content_result.is_err() {
                        return;
                    }
                    let mut content = content_result.unwrap();
                    let result = fs::read_to_string(x.borrow().pit_path.clone()).unwrap();
                    let file: Vec<&str> = result.lines().collect();
                    let name_of_file = file[file.len() - 3];
                    content.push_str("\n\n");
                    content.push_str(name_of_file);
                    content.push_str("\n\nblob");
                    let digest = sha1::chksum(content.clone()).unwrap();
                    if digest.to_hex_lowercase() != x.borrow().hash.clone() {
                        message.untracked_changes.push(x.borrow().path.clone());
                    }
                }
                None => {
                    message.untracked_added.push(entry_path);
                }
            }
        }
    }
}
