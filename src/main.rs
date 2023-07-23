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
        .subcommand(Command::new("log").arg(Arg::new("commit_id").required(false)))
        .subcommand(Command::new("checkout").arg(Arg::new("commit_id").required(true)))
        .subcommand(
            Command::new("tag")
                .arg(Arg::new("tag_name").required(true))
                .arg(Arg::new("commit_id").required(false)),
        )
        .get_matches();

    match program_arguments.subcommand() {
        Some(("greet", _arguments)) => greet(),
        Some(("init", _arguments)) => init(),
        Some(("hash-object", arguments)) => hash_object(arguments),
        Some(("cat-file", arguments)) => cat_file(arguments),
        Some(("write-tree", _arguments)) => write_tree(),
        Some(("read-tree", arguments)) => read_tree(arguments),
        Some(("commit", arguments)) => commit(arguments),
        Some(("log", arguments)) => log(arguments),
        Some(("checkout", arguments)) => checkout(arguments),
        Some(("tag", arguments)) => tag(arguments),
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
    let resolved_oid = base::get_oid(oid);

    match data::get_object(&resolved_oid, None) {
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
    let resolved_oid = base::get_oid(tree_oid);

    match base::read_tree(&resolved_oid) {
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

fn log(arguments: &ArgMatches) {
    let starting_commit_id = match arguments.get_one("commit_id") as Option<&String> {
        None => {
            let head = data::get_ref("HEAD");
            match head {
                Ok(head_commit_id) => match head_commit_id {
                    None => {
                        println!("There are no commits yet.");
                        return;
                    }
                    Some(head_commit_id) => head_commit_id,
                },
                Err(e) => {
                    eprintln!("Failed to display commit log. Reason: {:?}", e);
                    return;
                }
            }
        }
        Some(commit_id) => commit_id.to_string(),
    };
    let resolved_starting_commit_id = base::get_oid(&starting_commit_id);

    let mut current_commit_id = Some(resolved_starting_commit_id);
    loop {
        match current_commit_id {
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
                    current_commit_id = commit.parent;
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

fn checkout(arguments: &ArgMatches) {
    let commit_id = arguments.get_one("commit_id").unwrap() as &String;
    let resolved_commit_id = base::get_oid(commit_id);
    match base::checkout(&resolved_commit_id) {
        Ok(_) => println!("Checked out commit {}", commit_id),
        Err(e) => eprintln!("Failed to checkout commit {}. Reason: {:?}", commit_id, e),
    }
}

fn tag(arguments: &ArgMatches) {
    let tag_name = arguments.get_one("tag_name").unwrap() as &String;
    let commit_id = match arguments.get_one("commit_id") as Option<&String> {
        None => data::get_ref("HEAD").unwrap().unwrap(),
        Some(commit_id) => commit_id.to_string(),
    };
    let resolved_commit_id = base::get_oid(&commit_id);

    match base::create_tag(tag_name, &resolved_commit_id) {
        Ok(_) => println!("Tagged commit {} as {}", commit_id, tag_name),
        Err(e) => eprintln!(
            "Failed to tag commit {} as {}. Reason: {:?}",
            commit_id, tag_name, e,
        ),
    }
}
