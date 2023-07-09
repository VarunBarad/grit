use std::fs;

pub(crate) const GIT_DIR: &str = ".grit";

pub(crate) fn init() -> std::io::Result<()> {
    return fs::create_dir(GIT_DIR);
}
