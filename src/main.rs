mod base;
mod data;

use clap::{arg, Arg, ArgMatches, Command};
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
        .subcommand(Command::new("cat-file").arg(Arg::new("oid").required(true)))
        .subcommand(Command::new("write-tree"))
        .subcommand(Command::new("read-tree").arg(Arg::new("tree").required(true)))
        .subcommand(Command::new("commit").arg(arg!(--message <VALUE>).required(true)))
        .subcommand(Command::new("log"))
        .get_matches();

    match program_arguments.subcommand() {
        Some(("greet", _arguments)) => greet(),
        Some(("init", _arguments)) => init(),
        Some(("hash-object", arguments)) => hash_object(arguments),
        Some(("cat-file", arguments)) => cat_file(arguments),
        Some(("write-tree", _arguments)) => write_tree(),
        Some(("read-tree", arguments)) => read_tree(arguments),
        Some(("commit", arguments)) => commit(arguments),
        Some(("log", _arguments)) => log(),
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

    match data::hash_object(file_contents, "blob") {
        Ok(oid) => println!("{}", oid),
        Err(e) => eprintln!("Failed to hash {}. Reason: {:?}", file_path.display(), e),
    }
}

fn cat_file(arguments: &ArgMatches) {
    let oid = arguments.get_one("oid").unwrap() as &String;

    match data::get_object(oid, None) {
        Ok(object) => print!("{}", String::from_utf8(object).unwrap()),
        Err(e) => eprintln!("Failed to read object {}. Reason: {:?}", oid, e),
    }
}

fn write_tree() {
    match base::write_tree(".") {
        Ok(oid) => println!("{}", oid),
        Err(e) => eprintln!("Failed to write the whole tree. Reason: {:?}", e),
    }
}

fn read_tree(arguments: &ArgMatches) {
    let tree_oid = arguments.get_one("tree").unwrap() as &String;

    match base::read_tree(tree_oid) {
        Ok(_) => println!("Successfully read tree {}", &tree_oid),
        Err(e) => eprintln!("Failed to read tree {}. Reason: {:?}", &tree_oid, e),
    }
}

fn commit(arguments: &ArgMatches) {
    let message = arguments.get_one("message").unwrap() as &String;

    match base::commit(message) {
        Ok(oid) => println!("{}", oid),
        Err(e) => eprintln!("Failed to commit. Reason: {:?}", e),
    }
}

fn log() {
    let head = data::get_HEAD();

    match head {
        Ok(mut current_oid) => {
            if current_oid.is_none() {
                println!("There are no commits yet.");
                return;
            }

            loop {
                match current_oid {
                    None => break,
                    Some(ref commit_id) => match base::get_commit(commit_id) {
                        Ok(commit) => {
                            println!("commit {}", commit_id);
                            let indented_commit_message = commit
                                .message
                                .lines()
                                .map(|line| format!("\t{}", line))
                                .collect::<Vec<String>>()
                                .join("\n");
                            println!("{}", indented_commit_message);
                            println!();
                            current_oid = commit.parent;
                        }
                        Err(e) => {
                            eprintln!(
                                "Failed to display commit log. At commit {}. Reason: {:?}",
                                commit_id, e
                            );
                            break;
                        }
                    },
                }
            }
        }
        Err(e) => eprintln!("Failed to display commit log. Reason: {:?}", e),
    }
}
