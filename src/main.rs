use std::env;

mod parser;
mod runner;
mod modules;

mod common;

fn main() {
    let mut args = env::args();

    match args.nth(1) {
        Some(command) if command == "run".to_string() => {
            if args.len() == 0 {
                println!("No input file given");
            } else {
                common::run_file(&args.next().unwrap());
            }
        },
        Some(command) if command == "repl".to_string() => common::repl(),
        Some(command) => {
            println!("Unrecognized command: {command}");
            common::usage();
        },
        None => {
            println!("No command given");
            common::usage();
        }
    }
}
