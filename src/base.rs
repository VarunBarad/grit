use std::fs;

pub(crate) fn write_tree(directory: &str) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let full_path = format!("{}/{}", directory, file_name);

        if fs::symlink_metadata(&path).unwrap().is_file() {
            // ToDo: Write the file to object-store
            println!("{}", full_path);
        } else if fs::symlink_metadata(&path).unwrap().is_dir() {
            write_tree(&full_path);
        }
    }

    // ToDo: Actually create the tree object
}
