use crate::data;
use std::fs;

pub(crate) fn write_tree(directory: &str) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let full_path = format!("{}/{}", directory, file_name);
        if is_ignored(&full_path) {
            continue;
        }

        if fs::symlink_metadata(&path).unwrap().is_file() {
            match data::hash_object(fs::read(&full_path).unwrap(), "blob") {
                Ok(oid) => println!("{}", oid),
                Err(e) => eprintln!("Failed to hash {}. Reason: {:?}", full_path, e),
            }
        } else if fs::symlink_metadata(&path).unwrap().is_dir() {
            write_tree(&full_path);
        }
    }

    // ToDo: Actually create the tree object
}

fn is_ignored(path: &str) -> bool {
    return path.split("/").any(|x| x == ".grit");
}
