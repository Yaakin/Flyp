use std::collections::HashMap;
use crate::runner::{Runner, Value};
use crate::modules::Module;

pub struct Obj;

impl Obj {
    pub fn has(_r: &mut Runner, args: &Vec<Value>, _reflection: Option<Value>) -> Value {
        if args.len() != 2 {
            println!("Invalid Obj.has arguments");
            return Value::Nil;
        }
        if let Value::Object(o) = &args[0] && let Value::Str(field) = &args[1] {
            Value::Bool(o.contains_key(field))
        } else {
            println!("Invalid Obj.has arguments");
            Value::Nil
        }
    }
}

impl Module for Obj {
    fn get_load_name() -> String {
        "obj".to_string()
    }

    fn load(_r: &mut Runner) -> Value {
        Value::Object(HashMap::from([
            ("has".to_string(), Value::Native(Obj::has)),
        ]))
    }
}
