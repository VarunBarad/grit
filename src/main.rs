mod data;

use clap::{Arg, ArgMatches, Command};
use std::fs;
use std::path::Path;

fn main() {
    let program_arguments = Command::new("grit")
        .version("0.1.0")
        .author("Varun Barad <contact@varunbarad.com>")
        .about("A tiny model of git built using Rust to learn git")
        .args_override_self(true)
        .subcommand_required(true)
        .subcommand(Command::new("greet").about("Say hi to the world"))
        .subcommand(Command::new("init").about("Initialize a new repository"))
        .subcommand(Command::new("hash-object").arg(Arg::new("file").required(true)))
        .get_matches();

    match program_arguments.subcommand() {
        Some(("greet", _arguments)) => greet(),
        Some(("init", _arguments)) => init(),
        Some(("hash-object", arguments)) => hash_object(arguments),
        _ => eprintln!("No known pattern found"),
    }
}

fn greet() {
    println!("Hi, world!");
}

fn init() {
    match data::init() {
        Ok(_) => println!(
            "Initialized empty grit repository in {}/{}",
            std::env::current_dir().unwrap().display(),
            data::GIT_DIR
        ),
        Err(e) => eprintln!("Failed to initialize grit repository. Reason: {:?}", e),
    };
}

fn hash_object(arguments: &ArgMatches) {
    let file_path = Path::new(arguments.get_one("file").unwrap() as &String)
        .canonicalize()
        .unwrap();

    let file_contents = fs::read(file_path.clone()).unwrap();

    match data::hash_object(file_contents) {
        Ok(oid) => println!("{}", oid),
        Err(e) => eprintln!("Failed to hash {}. Reason: {:?}", file_path.display(), e),
    }
}
