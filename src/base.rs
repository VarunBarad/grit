use crate::data;
use std::collections::HashMap;
use std::fs;
use std::ops::Not;
use std::path::Path;

struct TreeEntry {
    object_type: String,
    object_id: String,
    file_name: String,
}

pub(crate) struct Commit {
    tree: String,
    pub(crate) parent: Option<String>,
    pub(crate) message: String,
}

pub(crate) fn write_tree(directory: &str) -> std::io::Result<String> {
    let mut entries: Vec<TreeEntry> = Vec::new();

    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let full_path = format!("{}/{}", directory, file_name);
        if is_ignored(&full_path) {
            continue;
        }

        if fs::symlink_metadata(&path).unwrap().is_file() {
            let object_type = "blob";
            match data::hash_object(fs::read(&full_path).unwrap(), object_type) {
                Ok(oid) => {
                    entries.push(TreeEntry {
                        object_type: object_type.to_string(),
                        object_id: oid,
                        file_name: file_name.to_string(),
                    });
                }
                Err(e) => eprintln!("Failed to hash {}. Reason: {:?}", full_path, e),
            }
        } else if fs::symlink_metadata(&path).unwrap().is_dir() {
            let object_type = "tree";
            match write_tree(&full_path) {
                Ok(oid) => {
                    entries.push(TreeEntry {
                        object_type: object_type.to_string(),
                        object_id: oid,
                        file_name: file_name.to_string(),
                    });
                }
                Err(e) => eprintln!("Failed to write_tree {}. Reason: {:?}", full_path, e),
            }
        }
    }

    let tree = entries
        .iter()
        .map(|entry| {
            format!(
                "{} {} {}\n",
                entry.object_type, entry.object_id, entry.file_name
            )
        })
        .collect::<String>();

    return data::hash_object(tree.as_bytes().to_vec(), "tree");
}

fn tree_entries_iterator(tree_id: Option<&str>) -> Option<Vec<TreeEntry>> {
    match tree_id {
        None => None,
        Some(tree_id) => match data::get_object(tree_id, Some("tree")) {
            Err(e) => {
                eprintln!("Failed to get tree {}. Reason: {:?}", tree_id, e);
                None
            }
            Ok(tree) => {
                return Some(
                    String::from_utf8(tree)
                        .unwrap()
                        .lines()
                        .filter_map(|line| {
                            let line = line.trim();
                            if line.is_empty() {
                                return None;
                            }

                            let mut parts = line.splitn(3, ' ');
                            Some(TreeEntry {
                                object_type: parts.next().unwrap().to_string(),
                                object_id: parts.next().unwrap().to_string(),
                                file_name: parts.next().unwrap().to_string(),
                            })
                        })
                        .collect::<Vec<TreeEntry>>(),
                );
            }
        },
    }
}

fn get_tree(tree_id: &str, base_path: &str) -> HashMap<String, String> {
    let mut tree: HashMap<String, String> = HashMap::new();

    for entry in tree_entries_iterator(Some(tree_id)).unwrap() {
        assert!(entry.file_name.contains('/').not());
        assert!(["..", "."].contains(&entry.file_name.as_str()).not());

        let path = format!("{}{}", base_path, entry.file_name);
        if entry.object_type == "blob" {
            tree.insert(path, entry.object_id);
        } else if entry.object_type == "tree" {
            tree.extend(get_tree(&entry.object_id, &format!("{}/", path)));
        }
    }

    tree
}

fn empty_directory<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if is_ignored(path.to_str().unwrap()) {
            continue;
        }

        if fs::symlink_metadata(&path)?.is_file() {
            fs::remove_file(&path)?;
        } else if fs::symlink_metadata(&path)?.is_dir() {
            empty_directory(&path)?;
            fs::remove_dir(&path)?;
        }
    }
    Ok(())
}

pub(crate) fn read_tree(tree_id: &str) -> std::io::Result<()> {
    empty_directory("./")?;

    for (path, oid) in get_tree(tree_id, "./") {
        let actual_path = Path::new(&path);
        match fs::create_dir_all(actual_path.parent().unwrap()) {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        match fs::write(actual_path, data::get_object(&oid, Some("blob")).unwrap()) {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

pub(crate) fn commit(message: &str) -> std::io::Result<String> {
    let tree_id = write_tree(".")?;
    let tree = format!("tree {}\n", &tree_id);

    let parent = match data::get_ref("HEAD")? {
        None => "".to_string(),
        Some(parent) => format!("parent {}\n", parent),
    };

    let commit = format!("{}{}\n{}", tree, parent, message);
    let oid = data::hash_object(commit.as_bytes().to_vec(), "commit")?;
    data::update_ref("HEAD", &oid)?;
    Ok(oid)
}

pub(crate) fn get_commit(commit_id: &str) -> std::io::Result<Commit> {
    let commit_contents = match data::get_object(commit_id, Some("commit")) {
        Ok(commit) => String::from_utf8(commit).unwrap(),
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No commit found for Commit ID {}", commit_id),
            ));
        }
    };
    let mut commit_lines = commit_contents.lines();

    let tree = match commit_lines.next() {
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Invalid commit contents. Commit ID {}. Commit Contents {}",
                    commit_id, commit_contents,
                ),
            ))
        }
        Some(tree) => {
            let (tree_type, tree_id) = tree.split_once(' ').unwrap();
            if tree_type != "tree" {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Invalid commit contents. Commit ID {}. Commit Contents {}",
                        commit_id, commit_contents,
                    ),
                ));
            }
            tree_id
        }
    };

    let parent_line = commit_lines.next();
    let parent = match parent_line {
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Invalid commit contents. Commit ID {}. Commit Contents {}",
                    commit_id, commit_contents,
                ),
            ))
        }
        Some(parent_line) => {
            if parent_line.trim().is_empty() {
                None
            } else {
                let (parent_type, parent_id) = parent_line.split_once(' ').unwrap();
                if parent_type != "parent" {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Invalid commit contents. Commit ID {}. Commit Contents {}",
                            commit_id, commit_contents,
                        ),
                    ));
                }
                Some(parent_id.to_string())
            }
        }
    };

    if parent.is_some() {
        commit_lines.next();
    }

    let message = commit_lines.collect::<Vec<&str>>().join("\n");

    Ok(Commit {
        tree: tree.to_string(),
        parent,
        message,
    })
}

pub(crate) fn checkout(commit_id: &str) -> std::io::Result<()> {
    let commit = get_commit(commit_id)?;
    read_tree(&commit.tree)?;
    data::update_ref("HEAD", commit_id)
}

pub(crate) fn create_tag(name: &str, oid: &str) -> std::io::Result<()> {
    data::update_ref(&format!("refs/tags/{}", name), oid)
}

pub(crate) fn get_oid(name: &str) -> std::io::Result<String> {
    let name = if name == "@" { "HEAD" } else { name };

    // name is a ref
    let refs_to_try = vec![
        name.to_string(),
        format!("refs/{}", name),
        format!("refs/tags/{}", name),
        format!("refs/heads/{}", name),
    ];
    for reference in refs_to_try {
        if let Some(oid) = data::get_ref(&reference).unwrap() {
            return Ok(oid);
        }
    }

    // name is SHA1
    let name_is_hex = name.chars().all(|c| c.is_ascii_hexdigit());
    if name.len() == 40 && name_is_hex {
        return Ok(name.to_string());
    }

    return Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("No reference found for name {}", name),
    ));
}

fn is_ignored(path: &str) -> bool {
    return path.split("/").any(|x| x == ".grit");
}
