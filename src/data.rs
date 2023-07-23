use sha1::{Digest, Sha1};
use std::fs;
use std::path::Path;

pub(crate) const GIT_DIR: &str = ".grit";

pub(crate) fn init() -> std::io::Result<()> {
    match fs::create_dir(GIT_DIR) {
        Ok(_) => {}
        Err(e) => return Err(e),
    }
    match fs::create_dir(format!("{}/objects", GIT_DIR)) {
        Ok(_) => Result::Ok(()),
        Err(e) => return Err(e),
    }
}

pub(crate) fn hash_object(data: Vec<u8>, object_type: &str) -> std::io::Result<String> {
    let mut object: Vec<u8> = Vec::new();
    object.extend(object_type.as_bytes());
    object.push(b'\x00');
    object.extend(data);
    let oid = calculate_hash(&object);
    return match fs::write(format!("{}/objects/{}", GIT_DIR, oid), object) {
        Ok(_) => Ok(oid),
        Err(e) => Err(e),
    };
}

pub(crate) fn get_object(oid: &str, expected_type: Option<&str>) -> std::io::Result<Vec<u8>> {
    match fs::read(format!("{}/objects/{}", GIT_DIR, oid)) {
        Ok(object) => {
            let mut x = object.split(|byte| byte == &b'\x00');
            let object_type = String::from_utf8(x.next().unwrap().to_vec()).unwrap();
            let contents = x.next().unwrap().to_vec();

            match expected_type {
                None => {}
                Some(expected_type) => assert_eq!(expected_type, object_type),
            }

            Ok(contents)
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn update_ref(ref_name: &str, oid: &str) -> std::io::Result<()> {
    let ref_path = format!("{}/{}", GIT_DIR, ref_name);
    let ref_path = Path::new(&ref_path);
    fs::create_dir_all(ref_path.parent().unwrap())?;
    fs::write(format!("{}/{}", GIT_DIR, ref_name), oid)
}

pub(crate) fn get_ref(ref_name: &str) -> std::io::Result<Option<String>> {
    let path = format!("{}/{}", GIT_DIR, ref_name);
    let path = Path::new(&path);
    if path.exists() {
        match fs::read_to_string(format!("{}/{}", GIT_DIR, ref_name)) {
            Ok(oid) => Ok(Some(oid)),
            Err(e) => Err(e),
        }
    } else {
        Ok(None)
    }
}

fn calculate_hash(data: &Vec<u8>) -> String {
    let hash = Sha1::new().chain_update(data).finalize();
    return base16ct::lower::encode_string(&hash);
}
