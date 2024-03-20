use crate::command::Command;
use chksum_sha1 as sha1;
use clap::Parser;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs::{read_to_string, File};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::rc::Rc;
use std::{fs, io};

#[derive(Parser, Debug, Clone)]
pub struct CommitArgs {
    pub(crate) message: String,
}

#[derive(Debug)]
pub struct CommitCommand {
    arguments: CommitArgs,
}

struct BlobInfo {
    pub path: String,
    _type_of_file: String,
    pub hash: String,
}
type TreeNodeRef = Rc<RefCell<TreeInfo>>;
#[derive(Debug, PartialEq, Clone, Default)]
struct TreeInfo {
    path: String,
    type_of_file: String,
    hash: String,
    pit_path: String,
    parent_commit_hash: String,
    parent: Option<TreeNodeRef>,
    name: String,
    children: Vec<TreeNodeRef>,
}

impl TreeInfo {
    fn create_tree_info(path: String, type_of_file: String, parent: Option<TreeNodeRef>) -> Self {
        TreeInfo {
            children: Vec::new(),
            type_of_file,
            path,
            parent_commit_hash: Default::default(),
            pit_path: Default::default(),
            name: Default::default(),
            parent,
            hash: "".to_string(),
        }
    }
}

impl CommitCommand {
    pub fn new(args: CommitArgs) -> Self {
        CommitCommand { arguments: args }
    }
}

impl Command for CommitCommand {
    fn execute(&mut self) {
        let message = self.arguments.message.clone();
        if message.is_empty() {
            println!("No message provided!");
            return;
        }
        let last_commit = take_current_branch();
        let mut commit_message: String;

        let commit = last_commit;
        commit_message = create_first_commit(commit.clone());

        if commit_message.is_empty() {
            println!("No changes to commit");
            return;
        }
        commit_message.push_str(message.as_str());
        commit_message.push_str("\n\ncommit");
        let digest = sha1::chksum(commit_message.clone()).unwrap();
        create_tree(digest.to_hex_lowercase(), commit_message);

        let _ = fs::write("./.pit/objects/info", "");

        let _ = fs::write(
            "./.pit/".to_string() + &commit,
            digest.to_hex_lowercase().as_bytes(),
        );
        println!("Committed with hash: {}", digest.to_hex_lowercase());
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

fn create_first_commit(branch: String) -> String {
    let base_path = String::from("./");
    let objects_path = Path::new(&base_path).join(".pit/objects");
    let mut tree_objects: TreeNodeRef = Rc::new(RefCell::new(TreeInfo::create_tree_info(
        ".".to_string(),
        "tree".to_string(),
        None,
    )));
    let mut blob_list: Vec<BlobInfo> = Vec::new();

    let entries_result = fs::read_dir(objects_path.clone());
    if entries_result.is_err() {
        println!("{:?} Couldn't read the folder", entries_result);
        return Default::default();
    }

    let entries = entries_result
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>();

    match entries {
        Err(err) => {
            println!("{:?} Couldn't read the files", err);
            return Default::default();
        }
        Ok(ref entries) => {
            if entries.is_empty() {
                println!("Pit doesn't track any files");
                return Default::default();
            }
        }
    }

    tree_objects = get_last_commit_tree(tree_objects.clone(), branch);
    tree_objects = get_root_node(tree_objects.clone());
    let cache = read_to_string("./.pit/objects/info").unwrap();
    let cache_items: Vec<&str> = cache.lines().collect();
    if cache.is_empty() {
        return Default::default();
    }
    let cloned = entries.as_ref().unwrap().clone();
    let mut entries_filters: Vec<&PathBuf> = cloned
        .iter()
        .filter(|&x| {
            cache_items
                .iter()
                .any(|&y| y == x.file_name().unwrap().to_str().unwrap())
        })
        .collect();
    entries_filters.sort();
    for entry in entries_filters {
        let file = File::open(entry.to_str().unwrap());
        if file.is_err() {
            println!("Couldn't read file {}", entry.to_str().unwrap());
            continue;
        }
        let mut content: String = Default::default();
        let result = file.unwrap().read_to_string(&mut content);
        if result.is_err() {
            println!("Cannot read from file {}", entry.to_str().unwrap());
            continue;
        }
        let blocks: Vec<&str> = content.split("\n\n").collect();
        // last 2 blocks are the: file path and type of file.
        let type_of_file = blocks[blocks.len() - 1];
        if type_of_file != "blob" {
            continue;
        }

        let file_path = blocks[blocks.len() - 2];
        let blob_info = BlobInfo {
            path: file_path.to_string(),
            _type_of_file: type_of_file.to_string(),
            hash: String::from(entry.to_str().unwrap().split('/').last().unwrap()),
        };

        blob_list.push(blob_info);
    }
    let mut changed = false;
    for blob in blob_list {
        let folders: Vec<&str> = blob.path.split('/').collect();
        tree_objects = get_root_node(tree_objects);

        for (idx, &folder) in folders.iter().enumerate() {
            if folder == "." {
                continue;
            }

            let children = tree_objects.borrow_mut().children.clone();

            let tree = children
                .iter()
                .find(|&x| folder == x.borrow().path.split('/').take(idx + 2).last().unwrap());
            if let Some(node) = tree {
                if idx == folders.len() - 1 && node.borrow().hash != blob.hash {
                    let index = children
                        .iter()
                        .position(|x| x.borrow().name == node.borrow().name);
                    tree_objects.borrow_mut().children.remove(index.unwrap());
                    let new_tree_node =
                        TreeNodeRef::new(RefCell::from(TreeInfo::create_tree_info(
                            tree_objects.borrow().path.to_string() + "/" + folder,
                            "blob".to_string(),
                            Some(tree_objects.clone()),
                        )));
                    new_tree_node.borrow_mut().hash = blob.hash.clone();
                    tree_objects
                        .borrow_mut()
                        .children
                        .push(new_tree_node.clone());
                    changed = true;
                }
                tree_objects = node.clone();
            } else if idx == folders.len() - 1 {
                let new_tree_node = TreeNodeRef::new(RefCell::from(TreeInfo::create_tree_info(
                    tree_objects.borrow().path.to_string() + "/" + folder,
                    "blob".to_string(),
                    Some(tree_objects.clone()),
                )));
                new_tree_node.borrow_mut().hash = blob.hash.clone();
                tree_objects
                    .borrow_mut()
                    .children
                    .push(new_tree_node.clone());
                changed = true;
            } else {
                let new_tree_node = TreeNodeRef::new(RefCell::from(TreeInfo::create_tree_info(
                    tree_objects.borrow().path.to_string() + "/" + folder,
                    "tree".to_string(),
                    Some(tree_objects.clone()),
                )));
                tree_objects
                    .borrow_mut()
                    .children
                    .push(new_tree_node.clone());
                tree_objects = new_tree_node;
                changed = true;
            }
        }
    }

    if !changed {
        return Default::default();
    }

    tree_objects = get_root_node(tree_objects);

    let last_tree_hash = complete_hash(tree_objects.clone());
    let mut content: String = Default::default();

    content.push_str(("tree".to_owned() + " " + last_tree_hash.as_str() + "\n").as_str());
    content.push_str(
        ("parent ".to_string() + tree_objects.borrow().parent_commit_hash.as_str() + "\n\n")
            .as_str(),
    );

    content
}

fn get_root_node(mut node: TreeNodeRef) -> TreeNodeRef {
    while node.borrow().parent.is_some() {
        let aux_node = node.clone();
        node = aux_node.clone().borrow_mut().parent.clone().unwrap();
    }

    node
}

fn complete_hash(node: TreeNodeRef) -> String {
    if node.borrow().type_of_file == "blob" {
        return node.borrow().hash.clone();
    }

    if node.borrow().type_of_file != "blob" {
        let mut content: String = Default::default();
        for i in &node.borrow().children {
            let hash = complete_hash(i.clone());
            content.push_str(
                (i.borrow().type_of_file.clone()
                    + " "
                    + hash.as_str()
                    + " "
                    + i.borrow().path.clone().as_str())
                .as_str(),
            );
            content.push('\n');
            content.push('\n');
        }
        content.push_str(node.borrow().path.clone().as_str());
        content.push_str("\n\ntree");

        let digest = sha1::chksum(content.clone()).unwrap();

        create_tree(digest.to_hex_lowercase(), content);
        return digest.to_string();
    }

    Default::default()
}

fn create_tree(hash: String, content: String) {
    let objects_folder = String::from("./.pit/objects/");
    let object_path = objects_folder + &*hash;
    let new_file_path = Path::new(&object_path);
    let file_result = File::create(new_file_path);

    if file_result.is_err() {
        return;
    }

    let writing_result = file_result.unwrap().write_all(content.as_ref());
    if writing_result.is_err() {
        println!(
            "Error writing in file {}: Error {:?}",
            object_path, writing_result
        );
    }
}

fn get_last_commit_tree(mut root: TreeNodeRef, branch: String) -> TreeNodeRef {
    let path = "./.pit/".to_owned() + branch.as_str();

    let last_commit = fs::read_to_string(path);
    if last_commit.is_err() {
        if last_commit.as_ref().err().unwrap().kind() == ErrorKind::NotFound {
            return root;
        }
        println!("{:?}", last_commit.err().unwrap());
        exit(2);
    }
    let all_commit = last_commit.unwrap();
    let commit = all_commit.trim();
    if commit.is_empty() {
        return root;
    }
    let tree_objects: TreeNodeRef = Rc::new(RefCell::new(TreeInfo::create_tree_info(
        ".".to_string(),
        "commit".to_string(),
        None,
    )));
    tree_objects.borrow_mut().pit_path = "./.pit/objects/".to_string() + commit;
    tree_objects.borrow_mut().name = ".".to_string();
    tree_objects.borrow_mut().hash = commit.to_string();
    root = tree_objects.clone();

    let mut queue: VecDeque<TreeNodeRef> = VecDeque::new();
    queue.push_back(tree_objects);
    while !queue.is_empty() {
        let node = queue.pop_front().unwrap();
        let content = fs::read_to_string(node.borrow().pit_path.clone()).unwrap();
        let type_of_file = content.lines().last().unwrap();

        if type_of_file == "commit" {
            let tree = content.lines().next().unwrap();
            let commit_hash = tree.split(' ').last().unwrap();
            let tree_node: TreeNodeRef = Rc::new(RefCell::new(TreeInfo::create_tree_info(
                ".".to_string(),
                "tree".to_string(),
                None,
            )));
            tree_node.borrow_mut().pit_path = "./.pit/objects/".to_string() + commit_hash.clone();
            tree_node.borrow_mut().name = ".".to_string();
            tree_node.borrow_mut().parent_commit_hash = node.borrow().hash.clone();
            root = tree_node.clone();
            queue.push_back(tree_node);

            continue;
        }
        if type_of_file == "tree" {
            for line in content.lines() {
                if line.is_empty() {
                    continue;
                }
                let data: Vec<&str> = line.split(' ').collect();

                if data.len() != 3 {
                    continue;
                }
                let tree_node: TreeNodeRef = Rc::new(RefCell::new(TreeInfo::create_tree_info(
                    data[2].to_string(),
                    data[0].to_string(),
                    Some(node.clone()),
                )));
                tree_node.borrow_mut().hash = data[1].to_string();
                tree_node.borrow_mut().name = data[2].to_string();
                tree_node.borrow_mut().pit_path =
                    "./.pit/objects/".to_string() + data[1].to_string().as_str();
                node.borrow_mut().children.push(tree_node.clone());
                queue.push_back(tree_node);
            }
        }
    }
    root.clone()
}
