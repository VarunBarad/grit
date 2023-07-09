use clap::Command;

fn main() {
    let program_arguments = Command::new("grit")
        .version("0.1.0")
        .author("Varun Barad <contact@varunbarad.com>")
        .about("A tiny model of git built using Rust to learn git")
        .args_override_self(true)
        .subcommand_required(true)
        .subcommand(Command::new("greet").about("Say hi to the world"))
        .get_matches();

    match program_arguments.subcommand() {
        Some(("greet", _arguments)) => greet(),
        _ => eprintln!("No known pattern found"),
    }
}

fn greet() {
    println!("Hi, world!");
}
