use crate::command::Command;
use crate::Parser;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

#[derive(Debug)]
enum Errors {
    Error,
}

#[derive(Parser, Debug, Clone)]
pub struct MergeArgs {
    branch: String,
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
pub struct MergeCommand {
    arguments: MergeArgs,
}

impl MergeCommand {
    pub fn new(args: MergeArgs) -> Self {
        MergeCommand { arguments: args }
    }
}

impl Command for MergeCommand {
    fn execute(&mut self) {
        let head_file_path = "./.pit/HEAD";
        let head_result = fs::read_to_string(head_file_path);
        if head_result.is_err() {
            println!("Fatal error. Head file not present.");
            return;
        }
        let refs_path = "./.pit/";
        let main_path = head_result.unwrap();
        let head_commit_result =
            fs::read_to_string(refs_path.to_string() + main_path.clone().as_str());
        if head_commit_result.is_err() {
            println!(
                "Commit for current branch not found {:?}",
                head_commit_result
            );
            return;
        }
        let head_commit = head_commit_result.unwrap();

        let branch_to_be_merged = self.arguments.branch.clone();
        let branch_commit_result =
            fs::read_to_string("./.pit/refs/".to_string() + branch_to_be_merged.as_str());
        if branch_commit_result.is_err() {
            println!("Branch to be merged not found");
            return;
        }
        let branch_to_commit = branch_commit_result.unwrap();
        if branch_to_commit.is_empty() || head_commit.is_empty() {
            println!("There are no commit on one branch");
            return;
        }
        if branch_to_commit == head_commit {
            println!("Nothing to be merged");
            return;
        }
        let head_root = TreeNodeRef::new(RefCell::from(TreeInfo::new(
            "./".to_string(),
            "./".to_string(),
            None,
        )));
        head_root.borrow_mut().type_of_file = "commit".to_string();
        head_root.borrow_mut().pit_path =
            "./.pit/objects/".to_string() + head_commit.clone().as_str();
        head_root.borrow_mut().hash = head_commit.clone();
        let branch_root = TreeNodeRef::new(RefCell::from(TreeInfo::new(
            "./".to_string(),
            "./".to_string(),
            None,
        )));
        branch_root.borrow_mut().type_of_file = "commit".to_string();
        branch_root.borrow_mut().pit_path =
            "./.pit/objects/".to_string() + branch_to_commit.clone().as_str();
        branch_root.borrow_mut().hash = branch_to_commit.clone();
        let lca = find_lca_node(head_root, branch_root);
        if let Ok(hash) = lca {
            println!("{}", hash);
            let _ = fs::write(refs_path.to_string() + main_path.as_str(), hash);
            println!("Merge success");
        }
    }
}

fn find_lca_node(
    mut head_root: TreeNodeRef,
    mut branch_root: TreeNodeRef,
) -> Result<String, Errors> {
    let result = populate_all_commits(head_root.clone());
    let mut mutable: TreeNodeRef;
    match result {
        Err(e) => {
            return Err(e);
        }
        Ok(node) => {
            mutable = node.clone();
            head_root = get_root_node(mutable);
        }
    }

    let result_branch_root = populate_all_commits(branch_root.clone());
    match result_branch_root {
        Err(e) => {
            return Err(e);
        }
        Ok(node) => {
            mutable = node.clone();
            branch_root = get_root_node(mutable);
        }
    }
    while branch_root.borrow().hash == head_root.borrow().hash {
        let branch_children = branch_root.borrow().children.clone();
        let next_branch_node = branch_children
            .iter()
            .find(|x| *x.borrow().type_of_file == *"commit");
        let head_children = head_root.borrow().children.clone();
        let next_head_node = head_children
            .iter()
            .find(|x| *x.borrow().type_of_file == *"commit");
        if next_head_node.is_none() || next_branch_node.is_none() {
            if next_branch_node.is_none() {
                println!("No changes to be merged");
                return Err(Errors::Error);
            } else {
                loop {
                    let branch_children = branch_root.borrow().children.clone();
                    let next_branch_node = branch_children
                        .iter()
                        .find(|x| *x.borrow().type_of_file == *"commit");
                    if next_branch_node.is_none() {
                        // create simple merge
                        return Ok(branch_root.borrow().hash.clone());
                    }
                    let next = next_branch_node.unwrap();
                    branch_root = next.clone();
                }
            }
        }
        let next_node = next_head_node.unwrap();
        let next_node_branch = next_branch_node.unwrap();
        head_root = next_node.clone();
        branch_root = next_node_branch.clone();
    }
    if branch_root.borrow().hash == head_root.borrow().hash {
        return Err(Errors::Error);
    }

    println!("No simple merge can be done");
    Err(Errors::Error)
}

fn populate_all_commits(root: TreeNodeRef) -> Result<TreeNodeRef, Errors> {
    let head_file_result = fs::read_to_string(root.borrow().pit_path.clone());
    if head_file_result.is_err() {
        println!("Can't read from commit file {:?}", head_file_result);
        return Err(Errors::Error);
    }
    let head_file = head_file_result.unwrap();
    let mut lines = head_file.lines(); // first line tree second line parent.
    let tree_line = lines.next();
    if let Some(tree) = tree_line {
        let commit = tree.split(' ').last().unwrap();
        let head_tree = TreeNodeRef::new(RefCell::from(TreeInfo::new(
            "./".to_string(),
            "./".to_string(),
            Some(root.clone()),
        )));
        head_tree.borrow_mut().hash = commit.clone().to_string();
        head_tree.borrow_mut().pit_path = "./.pit/objects/".to_string() + commit.clone();
        head_tree.borrow_mut().type_of_file = "tree".to_string();
        root.borrow_mut().children.push(head_tree.clone());
    } else {
        println!(
            "Commit file {} doesn't have a tree changes",
            root.borrow().hash
        );
        return Err(Errors::Error);
    }
    let parent_line = lines.next();
    if let Some(parent) = parent_line {
        let is_parent = parent.split(' ').next();
        if let Some(text) = is_parent {
            if text != "parent" {
                println!("Commit {} formatted wrong", root.borrow().hash);
                return Err(Errors::Error);
            }
        } else {
            println!("Commit {} formatted wrong", root.borrow().hash);
            return Err(Errors::Error);
        }
        let commit = parent.split(' ').last().unwrap();
        if commit.trim().is_empty() {
            return Ok(root);
        }
        let head_parent = TreeNodeRef::new(RefCell::from(TreeInfo::new(
            "./".to_string(),
            "./".to_string(),
            None,
        )));
        head_parent.borrow_mut().hash = commit.clone().to_string();
        head_parent.borrow_mut().pit_path = "./.pit/objects/".to_string() + commit.clone();
        head_parent.borrow_mut().type_of_file = "commit".to_string();
        head_parent.borrow_mut().children.push(root.clone());
        root.borrow_mut().parent = Some(head_parent.clone());
        let result = populate_all_commits(head_parent.clone());
        if result.is_err() {
            return Err(Errors::Error);
        }
    }
    Ok(root)
}

fn get_root_node(mut node: TreeNodeRef) -> TreeNodeRef {
    while node.borrow().parent.is_some() {
        let aux_node = node.clone();
        node = aux_node.clone().borrow_mut().parent.clone().unwrap();
    }

    node
}
