use std::collections::HashMap;
use crate::runner::{Runner, Value};
use crate::modules::Module;

pub struct Io;

impl Io {
    pub fn print(_r: &mut Runner, args: &Vec<Value>, _reflection: Option<Value>) -> Value {
        for a in args {
            print!("{} ", a.repr());
        }
        println!();
        Value::Nil
    }
}

impl Module for Io {
    fn get_load_name() -> String {
        "io".to_string()
    }

    fn load(_r: &mut Runner) -> Value {
        Value::Object(HashMap::from([
            ("print".to_string(), Value::Native(Io::print))
        ]))
    }
}
