use crate::runner::{NativeObject, Value, Runner};
use crate::modules::Module;
use std::rc::Rc;
use std::cell::RefCell;

pub struct List {
}

impl List {
    pub fn new() -> Self {
        Self {}
    }

    pub fn push(r: &mut Runner, args: &[Value]) -> Value {
        if args.len() < 2 {
            println!("Invalid arguments");
            return Value::Nil;
        }

        if let Value::List(id) = args[0] {
            let l = r.get_list_mut(id);
            l.push(args[1].clone());
            Value::Nil
        } else {
            println!("Invalid arguments");
            Value::Nil
        }
    }

    pub fn len(r: &mut Runner, args: &[Value]) -> Value {
        if args.len() < 1 {
            println!("Invalid arguments");
            return Value::Nil;
        }

        if let Value::List(id) = args[0] {
            let l = r.get_list(id);
            Value::Number(l.len() as f64)
        } else {
            println!("Invalid arguments");
            Value::Nil
        }
    }
}

impl NativeObject for List {
    fn get(&self, field: &str) -> Value {
        match field {
            "push" => Value::NativeFunction(Rc::new(RefCell::new(List::push))),
            "len" => Value::NativeFunction(Rc::new(RefCell::new(List::len))),
            _ => {
                println!("Field {field} not found");
                Value::Nil
            }
        }
    }
}

impl Module for List {
    fn name() -> String {
        "list".to_string()
    }

    fn load(_: &mut Runner) -> Value {
        Value::NativeObject(Rc::new(RefCell::new(List::new())))
    }
}
