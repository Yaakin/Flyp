use std::rc::Rc;
use std::cell::RefCell;
use crate::runner::{NativeObject, Value, Runner};
use crate::modules::Module;

pub struct Io {}

impl Io {
    pub fn new() -> Self {
        Self {}
    }

    pub fn print(r: &mut Runner, args: &[Value]) -> Value {
        for a in args {
            print!("{} ", a.repr(r));
        }
        println!();
        Value::Nil
    }
}

impl NativeObject for Io {
    fn get(&self, field: &str) -> Value {
        match field {
            "print" => Value::NativeFunction(Rc::new(RefCell::new(Io::print))),
            _ => {
                println!("Field {field} not found");
                Value::Nil
            }
        }
    }
}

impl Module for Io {
    fn name() -> String {
        "io".to_string()
    }

    fn load(_r: &mut Runner) -> Value {
        Value::NativeObject(Rc::new(RefCell::new(Io::new())))
    }
}
