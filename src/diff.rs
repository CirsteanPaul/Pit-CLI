use crate::command::Command;
use chksum_sha1 as sha1;
use clap::Parser;
use color_print::cprint;
use similar::{ChangeTag, TextDiff};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::rc::Rc;
#[derive(Parser, Debug, Clone)]
pub struct DiffArgs {
    commit: Option<String>,
    file: Option<String>,
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

#[derive(Debug)]
pub struct DiffCommand {
    arguments: DiffArgs,
}

impl DiffCommand {
    pub fn new(args: DiffArgs) -> Self {
        DiffCommand { arguments: args }
    }
}

impl Command for DiffCommand {
    fn execute(&mut self) {
        let mut commit_code: String = get_commit_code(self.arguments.commit.clone());

        let head_result = fs::read_to_string("./.pit/HEAD");
        if head_result.is_err() {
            println!("No head file. Fatal error");
            return;
        }
        let head = head_result.unwrap();
        let result = fs::read_to_string("./.pit/".to_string() + head.as_str());
        if result.is_err() {
            println!("Branch is corrupted!");
            return;
        }
        let current_commit = result.unwrap().clone();

        if commit_code.is_empty() {
            commit_code = current_commit.clone();
        }
        let mut root = construct_tree(current_commit.clone());
        if current_commit == commit_code {
            root = add_tracked_files(root.clone());
            get_diff_file_tree(root.clone());
        } else {
            let second_tree = construct_tree(commit_code.clone());
            get_diff_between_trees(root, second_tree);
        }
    }
}

fn get_commit_code(head: Option<String>) -> String {
    if head.is_none() {
        return Default::default();
    }
    let head_string = head.unwrap().trim().to_string();
    let heads_file_path = "./.pit/refs/".to_string() + head_string.clone().as_str();
    let result = fs::read_to_string(heads_file_path);
    if result.is_err() {
        let commit_hash = File::open("./.pit/objects/".to_string() + head_string.clone().as_str());
        if commit_hash.is_err() {
            return Default::default();
        } else {
            return head_string;
        }
    }

    result.unwrap().trim().to_string()
}

fn construct_tree(commit: String) -> TreeNodeRef {
    let objects_file_path = String::from("./.pit/objects/");
    let root = TreeNodeRef::new(RefCell::from(TreeInfo::new(
        "./".to_string(),
        "./".to_string(),
        None,
    )));

    let commit_content_result = fs::read_to_string(objects_file_path.clone() + commit.as_str());
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

fn add_tracked_files(root: TreeNodeRef) -> TreeNodeRef {
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
        } else {
            node.borrow_mut().hash = cached_file.to_string();
            node.borrow_mut().pit_path = objects_file_path.clone() + cached_file;
        }
    }

    root
}

fn get_diff_file_tree(root: TreeNodeRef) {
    if root.borrow().type_of_file == "tree" {
        for child in &root.borrow().children {
            get_diff_file_tree(child.clone());
        }
    } else {
        let path_borrowed = root.borrow().path.clone();
        let content_result = fs::read_to_string(path_borrowed.clone());
        if content_result.is_err() {
            println!(" {} was deleted", path_borrowed);
            return;
        }
        let pit_content_result = fs::read_to_string(root.borrow().pit_path.clone());
        if pit_content_result.is_err() {
            return;
        }
        let pit_content_string = pit_content_result.unwrap();
        let pit_content: Vec<&str> = pit_content_string.lines().collect();
        let name_of_file = pit_content[pit_content.len() - 3];
        let mut content = content_result.unwrap();
        let content_copy = content.clone();
        let pit_content_file = &pit_content[..pit_content.len() - 3];
        content.push_str("\n\n");
        content.push_str(name_of_file);
        content.push_str("\n\nblob");
        let digest = sha1::chksum(content).unwrap().to_hex_lowercase();
        if root.borrow().hash.clone() != digest {
            let pit_text_content = pit_content_file.join("\n\n");
            let content1 = content_copy.as_str();
            let content2 = pit_text_content.as_str();
            let diff = TextDiff::from_lines(content2, content1);

            for change in diff.iter_all_changes() {
                match change.tag() {
                    ChangeTag::Delete => {
                        cprint!("<red>-{}</red>", change);
                    }
                    ChangeTag::Insert => {
                        cprint!("<green>-{}</green>", change);
                    }
                    ChangeTag::Equal => {
                        cprint!("{}", change);
                    }
                };
            }
        }
    }
}

fn get_diff_between_trees(root: TreeNodeRef, second_tree: TreeNodeRef) {
    if root.borrow().type_of_file == "tree" {
        for child in &root.borrow().children {
            let children = second_tree.borrow().children.clone();
            let matched = children
                .iter()
                .find(|x| x.borrow().name == child.borrow().name);
            match matched {
                Some(node) => {
                    get_diff_between_trees(child.clone(), node.clone());
                }
                None => {
                    if child.borrow().type_of_file != "tree" {
                        println!("{} was added", child.borrow().path);
                    }
                }
            }
        }
    } else if root.borrow().hash != second_tree.borrow().hash {
        let root_result = fs::read_to_string(root.borrow().pit_path.clone());
        let second_result = fs::read_to_string(second_tree.borrow().pit_path.clone());
        if root_result.is_err() {
            println!("{} was modified", root.borrow().path);
            return;
        }
        if second_result.is_err() {
            println!("{} was modified", root.borrow().path);
            return;
        }
        let content_root = root_result.unwrap();
        let content_second = second_result.unwrap();
        let segments_content_root: Vec<&str> = content_root.split("\n\n").collect();
        let segments_content_second: Vec<&str> = content_second.split("\n\n").collect();
        let text_root = &segments_content_root[..segments_content_root.len() - 3];
        let text_second = &segments_content_second[..segments_content_second.len() - 3];
        let text_file_root = text_root.join("\n\n");
        let text_file_second = text_second.join("\n\n");
        let diff = TextDiff::from_lines(text_file_second.as_str(), text_file_root.as_str());

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Delete => {
                    cprint!("<red>-{}</red>", change);
                }
                ChangeTag::Insert => {
                    cprint!("<green>-{}</green>", change);
                }
                ChangeTag::Equal => {
                    cprint!("{}", change);
                }
            };
        }
    }
}
