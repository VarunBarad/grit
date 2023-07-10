use crate::data;
use std::fs;

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

fn is_ignored(path: &str) -> bool {
    return path.split("/").any(|x| x == ".grit");
}
