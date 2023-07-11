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
    let commit = format!("tree {}\n\n{}", tree_id, message);
    let oid = data::hash_object(commit.as_bytes().to_vec(), "commit")?;
    Ok(oid)
}

fn is_ignored(path: &str) -> bool {
    return path.split("/").any(|x| x == ".grit");
}
