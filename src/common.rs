use std::fs;
use std::io;
use std::io::Write;

use crate::parser::Parser;
use crate::runner::Runner;

pub fn usage() {
    println!("Usage: flyp run FILE_PATH");
    println!("       flyp repl");
}

pub fn repl() {
    const PROMPT : &str = "$ ";
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut r = Runner::new();

    while buffer != ":q" {
        buffer.clear();
        print!("{PROMPT}");
        io::stdout().flush().unwrap();

        let Ok(_) = stdin.read_line(&mut buffer) else {
            println!("Error reading input");
            return;
        };
        buffer = buffer.trim().to_string();
        
        if buffer != ":q" {
            let mut p = Parser::new(&buffer);
            let res = r.eval(&p.expr());
            println!("{}", res.repr(&r));
        }
    }
}

pub fn run_file(filename: &str) {
    let mut r = Runner::new();
    if let Ok(src) = fs::read_to_string(filename) {
        let mut p = Parser::new(&src);
        r.eval(&p.expr());
    } else {
        println!("Input file {filename} not found");
    }
}
