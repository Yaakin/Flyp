use std::fs;

use crate::parser::Parser;
use crate::runner::{Runner, Value};

pub fn usage() {
    println!("Usage: flyp run FILE_PATH");
    println!("       flyp repl");
}

pub fn run_file(filename: &str) -> Option<Value> {
    let mut r = Runner::new();
    if let Ok(src) = fs::read_to_string(filename) {
        let mut p = Parser::new(&src);
        let e = p.expr();
        Some(r.eval(&e))
    } else {
        println!("Input file {filename} not found");
        None
    }
}
