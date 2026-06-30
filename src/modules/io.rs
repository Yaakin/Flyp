use std::collections::HashMap;
use crate::runner::{Runner, Value, Stored};
use crate::modules::Module;

pub struct Io;

impl Io {
    pub fn print(r: &mut Runner, args: &Vec<Value>) -> Value {
        for a in args {
            print!("{} ", a.repr(r));
        }
        println!();
        Value::Nil
    }
}

impl Module for Io {
    fn get_load_name() -> String {
        "io".to_string()
    }

    fn load(r: &mut Runner) -> Value {
        let res = Stored::Object(HashMap::from([
            ("print".to_string(), r.register(Stored::Native(Io::print))),
        ]));
        r.register(res)
    }
}
