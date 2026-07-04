use std::collections::HashMap;
use crate::runner::{Runner, Value, Stored, Function};
use crate::modules::Module;

pub struct Math;

impl Math {
    pub fn add(_r: &mut Runner, args: &Vec<Value>, _: Option<Value>) -> Value {
        if args.len() < 2 {
            println!("Invalid number of arguments");
            return Value::Nil;
        }

        if let Value::Number(x1) = args[0] && let Value::Number(x2) = args[1] {
            Value::Number(x1 + x2)
        } else {
            println!("Invalide argument types");
            Value::Nil
        }
    }

    pub fn equ(_r: &mut Runner, args: &Vec<Value>, _: Option<Value>) -> Value {
        if args.len() < 2 {
            println!("Invalid number of arguments");
            return Value::Nil;
        }

        if let Value::Number(x1) = args[0] && let Value::Number(x2) = args[1] {
            Value::Bool(x1 == x2)
        } else {
            println!("Invalide argument types");
            Value::Nil
        }
    }
}

impl Module for Math {
    fn get_load_name() -> String {
        "math".to_string()
    }

    fn load(r: &mut Runner) -> Value {
        let res = Stored::Object(HashMap::from([
            ("add".to_string(), r.register(Stored::Func(Function::Native(Math::add)))),
            ("equ".to_string(), r.register(Stored::Func(Function::Native(Math::equ)))),
        ]));
        r.register(res)
    }
}
