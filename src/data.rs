use sha1::{Digest, Sha1};
use std::fs;

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

pub(crate) fn hash_object(data: Vec<u8>) -> std::io::Result<String> {
    let oid = calculate_hash(&data);
    return match fs::write(format!("{}/objects/{}", GIT_DIR, oid), data) {
        Ok(_) => Ok(oid),
        Err(e) => Err(e),
    };
}

pub(crate) fn get_object(oid: &String) -> std::io::Result<Vec<u8>> {
    return fs::read(format!("{}/objects/{}", GIT_DIR, oid));
}

fn calculate_hash(data: &Vec<u8>) -> String {
    let hash = Sha1::new().chain_update(data).finalize();
    return base16ct::lower::encode_string(&hash);
}
